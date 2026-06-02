//! MemoryStorage trait — abstracts VFS operations needed by the memory module.
//!
//! This trait decouples the memory module from VFS internals (VfsDatabase, VfsLanceStore,
//! VfsFolderRepo, VfsNoteRepo, VfsIndexStateRepo, VfsFullIndexingService, etc.).
//!
//! The memory module only depends on this trait + VFS data types (VfsNote, VfsFolder, …).
//! VFS implementation details are hidden behind the trait impl.

use std::sync::Arc;

use async_trait::async_trait;
use rusqlite::Connection;

use crate::llm_manager::LLMManager;
use crate::vfs::types::{
    ResourceLocation, VfsCreateNoteParams, VfsFolder, VfsFolderItem,
    VfsNote, VfsUpdateNoteParams,
};
use crate::vfs::VfsLanceSearchResult;

use super::error::MemoryResult;

/// Object-safe connection handle returned by [`MemoryStorage::conn`].
///
/// Auto-derefs to `&rusqlite::Connection`, so callers can use it directly
/// with `.execute()`, `.prepare()`, `.query_row()`, etc.
pub struct MemoryStorageConn(Box<dyn std::ops::Deref<Target = Connection> + Send>);

impl std::ops::Deref for MemoryStorageConn {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        self.0.deref()
    }
}

impl MemoryStorageConn {
    pub(crate) fn new(inner: Box<dyn std::ops::Deref<Target = Connection> + Send>) -> Self {
        Self(inner)
    }
}

/// Trait covering all VFS operations used by the memory module.
///
/// # Object safety
///
/// All methods are object-safe.  Use `Arc<dyn MemoryStorage>` or
/// `Box<dyn MemoryStorage>`.
///
/// # Async methods
///
/// Methods that call into LanceDB or the indexing service are `async`.
/// The `#[async_trait]` macro transforms them into `Pin<Box<dyn Future …>>`
/// so they work behind `dyn MemoryStorage`.
#[async_trait]
pub trait MemoryStorage: Send + Sync {
    // ------------------------------------------------------------------
    // Connection
    // ------------------------------------------------------------------

    /// Acquire a pooled database connection usable for direct SQL.
    fn conn(&self) -> MemoryResult<MemoryStorageConn>;

    /// Acquire a connection without the safe-guards (calls `get_conn` instead of `get_conn_safe`).
    fn conn_unchecked(&self) -> MemoryResult<MemoryStorageConn>;

    // ------------------------------------------------------------------
    // Note CRUD
    // ------------------------------------------------------------------

    /// Get a single note by ID.
    fn get_note(&self, note_id: &str) -> MemoryResult<Option<VfsNote>>;

    /// Get the text content of a note.
    fn get_note_content(&self, note_id: &str) -> MemoryResult<Option<String>>;

    /// Get the folder location of a note.
    fn get_note_location(&self, note_id: &str) -> MemoryResult<Option<ResourceLocation>>;

    /// Create a note in the given folder (None = root).
    fn create_note_in_folder(
        &self,
        params: VfsCreateNoteParams,
        folder_id: Option<&str>,
    ) -> MemoryResult<VfsNote>;

    /// Update a note's title/content/tags.
    fn update_note(
        &self,
        note_id: &str,
        params: VfsUpdateNoteParams,
    ) -> MemoryResult<VfsNote>;

    /// Update a note using an existing connection handle.
    fn update_note_with_conn(
        &self,
        conn: &Connection,
        note_id: &str,
        params: VfsUpdateNoteParams,
    ) -> MemoryResult<VfsNote>;

    /// Soft-delete a note and its folder-item association.
    fn delete_note_with_folder_item(&self, note_id: &str) -> MemoryResult<()>;

    // ------------------------------------------------------------------
    // Folder CRUD
    // ------------------------------------------------------------------

    /// Get a single folder by ID.
    fn get_folder(&self, folder_id: &str) -> MemoryResult<Option<VfsFolder>>;

    /// Check whether a folder exists by ID.
    fn folder_exists(&self, folder_id: &str) -> MemoryResult<bool>;

    /// Create a folder and return its ID.
    fn create_folder(&self, folder: &VfsFolder) -> MemoryResult<String>;

    /// List direct children of a parent folder (None = root level).
    fn list_folders_by_parent(
        &self,
        parent_id: Option<&str>,
    ) -> MemoryResult<Vec<VfsFolder>>;

    /// Like list_folders_by_parent but uses an existing connection.
    fn list_folders_by_parent_with_conn(
        &self,
        conn: &Connection,
        parent_id: Option<&str>,
    ) -> MemoryResult<Vec<VfsFolder>>;

    /// Recursively collect all folder IDs under a root.
    fn get_folder_ids_recursive(&self, root_id: &str) -> MemoryResult<Vec<String>>;

    /// List items (notes / textbooks / …) inside a folder.
    fn list_items_by_folder(
        &self,
        folder_id: Option<&str>,
    ) -> MemoryResult<Vec<VfsFolderItem>>;

    /// Like list_items_by_folder but uses an existing connection.
    fn list_items_by_folder_with_conn(
        &self,
        conn: &Connection,
        folder_id: Option<&str>,
    ) -> MemoryResult<Vec<VfsFolderItem>>;

    /// Move an item to a different folder (None = root).
    fn move_item_by_item_id(
        &self,
        item_type: &str,
        item_id: &str,
        target_folder_id: Option<&str>,
    ) -> MemoryResult<()>;

    // ------------------------------------------------------------------
    // Index-state management
    // ------------------------------------------------------------------

    fn mark_indexed(&self, resource_id: &str, version: &str) -> MemoryResult<()>;
    fn mark_pending(&self, resource_id: &str) -> MemoryResult<()>;
    fn mark_disabled_with_reason(&self, resource_id: &str, reason: &str) -> MemoryResult<()>;

    // ------------------------------------------------------------------
    // Vector store (LanceDB)
    // ------------------------------------------------------------------

    /// Hybrid full-text + vector search.
    async fn hybrid_search(
        &self,
        modality: &str,
        query_text: &str,
        query_embedding: &[f32],
        top_k: usize,
        folder_ids: Option<&[String]>,
        resource_types: Option<&[String]>,
    ) -> MemoryResult<Vec<VfsLanceSearchResult>>;

    /// Delete all vector entries for a resource.
    async fn delete_by_resource(&self, modality: &str, resource_id: &str) -> MemoryResult<usize>;

    // ------------------------------------------------------------------
    // Index-unit management (vfs_index_units table)
    // ------------------------------------------------------------------

    fn delete_index_units_by_resource(&self, resource_id: &str) -> MemoryResult<()>;

    // ------------------------------------------------------------------
    // Full indexing (embedding + FTS generation)
    // ------------------------------------------------------------------

    /// Generate embeddings and write them into the vector store + FTS index.
    async fn index_resource(&self, resource_id: &str) -> MemoryResult<(usize, usize)>;
}

// =========================================================================
// Concrete implementation: VfsMemoryStorage
// =========================================================================

use std::ops::Deref;

use crate::vfs::database::VfsDatabase;
use crate::vfs::indexing::VfsFullIndexingService;
use crate::vfs::lance_store::VfsLanceStore;
use crate::vfs::repos::embedding_repo::VfsIndexStateRepo;
use crate::vfs::repos::folder_repo::VfsFolderRepo;
use crate::vfs::repos::index_unit_repo;
use crate::vfs::repos::note_repo::VfsNoteRepo;
use crate::vfs::error::VfsError;

use super::error::MemoryError;

/// Concrete [`MemoryStorage`] implementation backed by VFS types.
///
/// Thin wrappers that delegate every call to the corresponding Vfs*Repo
/// static method or VfsLanceStore instance method.
pub struct VfsMemoryStorage {
    db: Arc<VfsDatabase>,
    lance_store: Arc<VfsLanceStore>,
    llm_manager: Arc<LLMManager>,
}

impl VfsMemoryStorage {
    pub fn new(
        db: Arc<VfsDatabase>,
        lance_store: Arc<VfsLanceStore>,
        llm_manager: Arc<LLMManager>,
    ) -> Self {
        Self {
            db,
            lance_store,
            llm_manager,
        }
    }

    /// Expose the inner database handle for external wiring (e.g. audit_log query function).
    pub fn db(&self) -> &Arc<VfsDatabase> {
        &self.db
    }

    /// Expose the inner lance store for external wiring.
    pub fn lance_store(&self) -> &Arc<VfsLanceStore> {
        &self.lance_store
    }
}

// Helper: map VfsError -> MemoryError.
fn to_memory_err(e: VfsError) -> MemoryError {
    MemoryError::Database(e.to_string())
}

#[async_trait]
impl MemoryStorage for VfsMemoryStorage {
    // ------------------------------------------------------------------
    // Connection
    // ------------------------------------------------------------------

    fn conn(&self) -> MemoryResult<MemoryStorageConn> {
        let conn = self.db.get_conn_safe().map_err(to_memory_err)?;
        Ok(MemoryStorageConn::new(Box::new(conn)))
    }

    fn conn_unchecked(&self) -> MemoryResult<MemoryStorageConn> {
        let conn = self.db.get_conn().map_err(to_memory_err)?;
        Ok(MemoryStorageConn::new(Box::new(conn)))
    }

    // ------------------------------------------------------------------
    // Note CRUD
    // ------------------------------------------------------------------

    fn get_note(&self, note_id: &str) -> MemoryResult<Option<VfsNote>> {
        VfsNoteRepo::get_note(&self.db, note_id).map_err(to_memory_err)
    }

    fn get_note_content(&self, note_id: &str) -> MemoryResult<Option<String>> {
        VfsNoteRepo::get_note_content(&self.db, note_id).map_err(to_memory_err)
    }

    fn get_note_location(&self, note_id: &str) -> MemoryResult<Option<ResourceLocation>> {
        VfsNoteRepo::get_note_location(&self.db, note_id).map_err(to_memory_err)
    }

    fn create_note_in_folder(
        &self,
        params: VfsCreateNoteParams,
        folder_id: Option<&str>,
    ) -> MemoryResult<VfsNote> {
        VfsNoteRepo::create_note_in_folder(&self.db, params, folder_id).map_err(to_memory_err)
    }

    fn update_note(
        &self,
        note_id: &str,
        params: VfsUpdateNoteParams,
    ) -> MemoryResult<VfsNote> {
        VfsNoteRepo::update_note(&self.db, note_id, params).map_err(to_memory_err)
    }

    fn update_note_with_conn(
        &self,
        conn: &Connection,
        note_id: &str,
        params: VfsUpdateNoteParams,
    ) -> MemoryResult<VfsNote> {
        VfsNoteRepo::update_note_with_conn(conn, note_id, params).map_err(to_memory_err)
    }

    fn delete_note_with_folder_item(&self, note_id: &str) -> MemoryResult<()> {
        VfsNoteRepo::delete_note_with_folder_item(&self.db, note_id).map_err(to_memory_err)
    }

    // ------------------------------------------------------------------
    // Folder CRUD
    // ------------------------------------------------------------------

    fn get_folder(&self, folder_id: &str) -> MemoryResult<Option<VfsFolder>> {
        VfsFolderRepo::get_folder(&self.db, folder_id).map_err(to_memory_err)
    }

    fn folder_exists(&self, folder_id: &str) -> MemoryResult<bool> {
        VfsFolderRepo::folder_exists(&self.db, folder_id).map_err(to_memory_err)
    }

    fn create_folder(&self, folder: &VfsFolder) -> MemoryResult<String> {
        VfsFolderRepo::create_folder(&self.db, folder).map_err(to_memory_err)?;
        Ok(folder.id.clone())
    }

    fn list_folders_by_parent(
        &self,
        parent_id: Option<&str>,
    ) -> MemoryResult<Vec<VfsFolder>> {
        VfsFolderRepo::list_folders_by_parent(&self.db, parent_id).map_err(to_memory_err)
    }

    fn list_folders_by_parent_with_conn(
        &self,
        conn: &Connection,
        parent_id: Option<&str>,
    ) -> MemoryResult<Vec<VfsFolder>> {
        VfsFolderRepo::list_folders_by_parent_with_conn(conn, parent_id).map_err(to_memory_err)
    }

    fn get_folder_ids_recursive(&self, root_id: &str) -> MemoryResult<Vec<String>> {
        VfsFolderRepo::get_folder_ids_recursive(&self.db, root_id).map_err(to_memory_err)
    }

    fn list_items_by_folder(
        &self,
        folder_id: Option<&str>,
    ) -> MemoryResult<Vec<VfsFolderItem>> {
        VfsFolderRepo::list_items_by_folder(&self.db, folder_id).map_err(to_memory_err)
    }

    fn list_items_by_folder_with_conn(
        &self,
        conn: &Connection,
        folder_id: Option<&str>,
    ) -> MemoryResult<Vec<VfsFolderItem>> {
        VfsFolderRepo::list_items_by_folder_with_conn(conn, folder_id).map_err(to_memory_err)
    }

    fn move_item_by_item_id(
        &self,
        item_type: &str,
        item_id: &str,
        target_folder_id: Option<&str>,
    ) -> MemoryResult<()> {
        VfsFolderRepo::move_item_by_item_id(&self.db, item_type, item_id, target_folder_id)
            .map_err(to_memory_err)
    }

    // ------------------------------------------------------------------
    // Index-state management
    // ------------------------------------------------------------------

    fn mark_indexed(&self, resource_id: &str, version: &str) -> MemoryResult<()> {
        VfsIndexStateRepo::mark_indexed(&self.db, resource_id, version).map_err(to_memory_err)
    }

    fn mark_pending(&self, resource_id: &str) -> MemoryResult<()> {
        VfsIndexStateRepo::mark_pending(&self.db, resource_id).map_err(to_memory_err)
    }

    fn mark_disabled_with_reason(&self, resource_id: &str, reason: &str) -> MemoryResult<()> {
        VfsIndexStateRepo::mark_disabled_with_reason(&self.db, resource_id, reason)
            .map_err(to_memory_err)
    }

    // ------------------------------------------------------------------
    // Vector store
    // ------------------------------------------------------------------

    async fn hybrid_search(
        &self,
        modality: &str,
        query_text: &str,
        query_embedding: &[f32],
        top_k: usize,
        folder_ids: Option<&[String]>,
        resource_types: Option<&[String]>,
    ) -> MemoryResult<Vec<VfsLanceSearchResult>> {
        self.lance_store
            .hybrid_search(modality, query_text, query_embedding, top_k, folder_ids, resource_types)
            .await
            .map_err(to_memory_err)
    }

    async fn delete_by_resource(&self, modality: &str, resource_id: &str) -> MemoryResult<usize> {
        self.lance_store
            .delete_by_resource(modality, resource_id)
            .await
            .map_err(to_memory_err)
    }

    // ------------------------------------------------------------------
    // Index-unit management
    // ------------------------------------------------------------------

    fn delete_index_units_by_resource(&self, resource_id: &str) -> MemoryResult<()> {
        let conn = self.db.get_conn_safe().map_err(to_memory_err)?;
        index_unit_repo::delete_by_resource(conn.deref(), resource_id).map_err(to_memory_err)?;
        Ok(())
    }

    // ------------------------------------------------------------------
    // Full indexing
    // ------------------------------------------------------------------

    async fn index_resource(&self, resource_id: &str) -> MemoryResult<(usize, usize)> {
        let svc = VfsFullIndexingService::new(
            self.db.clone(),
            self.llm_manager.clone(),
            self.lance_store.clone(),
        )
        .map_err(|e| MemoryError::Other(format!("Failed to create indexing service: {}", e)))?;

        svc.index_resource(resource_id, None, None)
            .await
            .map_err(|e| MemoryError::Database(e.to_string()))
    }
}
