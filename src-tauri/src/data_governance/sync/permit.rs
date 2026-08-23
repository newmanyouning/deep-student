//! # 云同步读写权限基础设施 (Permit)
//!
//! 同步期间需要"读写权限的分配与回收"，本模块提供三层 RAII 权限结构：
//!
//! - **读权限** [`ReadPermit`]：持有一个只读 SQLite 连接（`PRAGMA query_only=ON`）。
//!   持有期间该连接上的写操作被 SQLite 拒绝；Drop 即关闭连接 = 权限自动回收。
//! - **写权限** [`WritePermit`]：应用级同步写门。持有期间 `AppState.sync_write_guard`
//!   指向本 db，业务写命令入口调用 [`check_write_gate`] 检查写门；
//!   Drop 清空槽位 = 权限自动回收。
//! - **同步会话** [`SyncSession`]：由同步命令链创建持有；Drop 时按逆序释放：
//!   写权限 → 读连接 → 全局信号量（`BACKUP_GLOBAL_LIMITER`）。
//!
//! ## 原则
//!
//! 1. **RAII 兜底**：所有错误路径（panic 除外）经 Drop 释放，不残留权限。
//! 2. **最小持有**：读权限只在导出快照期持有；写权限只在单库应用事务期持有，
//!    每库独立 acquire/release，永不嵌套。
//! 3. **写门只挡业务写，不挡同步自身**：写门检查放在业务命令入口，
//!    同步内部写不经过命令层，不受此检查影响。
//! 4. **可重试语义**：[`SyncError::SyncInProgress`] / [`SyncError::SyncBusy`]
//!    均为可重试错误，前端提示"数据正在同步，请稍后重试"，不产生死锁。
//!
//! ## 接线方式（已接线, 2026-08-07）
//!
//! - 读权限：`session.acquire_read(db_path)?`，经 [`ReadPermit::conn`] 执行只读查询；
//!   阶段结束 `session.release_read_permits()` 回收；
//! - 写权限：apply 每库独立 `WritePermit::acquire(guard, db)` / 作用域结束即释放
//!   （由 `commands_backup::apply_downloaded_changes_to_databases` 统一接线）；
//! - 全局信号量：命令链带 60s 超时获取后 `session.attach_global_semaphore(permit)` 移交；
//! - 业务写命令入口：`check_write_gate(&state.sync_write_guard, db)?`。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rusqlite::Connection;

use super::SyncError;

/// 同步写门槽位 — `AppState.sync_write_guard` 的共享句柄。
///
/// `Some(db_name)` = 该 db 正在同步（写门被占）；`None` = 写门空闲。
/// 由调用方从 AppState 克隆传入（`app.state::<AppState>().sync_write_guard.clone()`）。
pub type SyncGuardSlot = Arc<tokio::sync::Mutex<Option<String>>>;

/// 读权限 — 持有一个只读 SQLite 连接。
///
/// 构造时执行 `PRAGMA query_only=ON` 并查询断言生效；
/// Drop 即关闭连接 = 权限自动回收（失败路径无需手动释放）。
pub struct ReadPermit {
    /// 只读连接（私有，只能经 [`ReadPermit::conn`] 读取；写操作会被 SQLite 拒绝）
    _conn: Connection,
    /// 数据库文件路径（诊断用）
    db_path: PathBuf,
}

impl ReadPermit {
    /// 打开数据库并应用只读模式。
    ///
    /// 执行 `PRAGMA query_only=ON` 后通过查询断言其生效；
    /// 断言失败返回 [`SyncError::Database`]，不产生半只读连接。
    pub fn open(db_path: impl AsRef<Path>) -> Result<Self, SyncError> {
        let db_path = db_path.as_ref();
        let conn = Connection::open(db_path).map_err(|e| {
            SyncError::Database(format!("打开只读连接失败 ({}): {}", db_path.display(), e))
        })?;
        conn.execute_batch("PRAGMA query_only = ON;").map_err(|e| {
            SyncError::Database(format!(
                "设置 PRAGMA query_only 失败 ({}): {}",
                db_path.display(),
                e
            ))
        })?;
        // 断言只读模式生效（PRAGMA query_only 返回 1 = 只读）
        let query_only: i64 = conn
            .query_row("SELECT * FROM pragma_query_only", [], |row| row.get(0))
            .map_err(|e| {
                SyncError::Database(format!(
                    "验证 PRAGMA query_only 失败 ({}): {}",
                    db_path.display(),
                    e
                ))
            })?;
        if query_only != 1 {
            return Err(SyncError::Database(format!(
                "PRAGMA query_only 未生效 (db={}, 实际值={})",
                db_path.display(),
                query_only
            )));
        }
        Ok(Self {
            _conn: conn,
            db_path: db_path.to_path_buf(),
        })
    }

    /// 只读连接引用（供同步导出阶段执行只读查询；写操作会被 SQLite 拒绝）
    pub fn conn(&self) -> &Connection {
        &self._conn
    }

    /// 数据库文件路径（诊断用）
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }
}

/// 写权限 — 应用级同步写门。
///
/// 持有期间 `AppState.sync_write_guard` 指向本 db；
/// 业务写命令入口调用 [`check_write_gate`] 命中 → `SyncInProgress`（可重试）。
/// Drop 时清空槽位（仅在槽位仍指向本 db 时清空，防嵌套错乱）= 权限自动回收。
#[derive(Debug)]
pub struct WritePermit {
    /// 持有写门的数据库名（静态字符串，来自 `DatabaseId::as_str` 等）
    db_name: &'static str,
    /// AppState.sync_write_guard 的共享句柄（由调用方从 AppState 克隆传入）
    guard: SyncGuardSlot,
}

impl WritePermit {
    /// 尝试获取写门（非阻塞 try_lock）。
    ///
    /// - 锁被瞬时占用（其他线程正在 acquire/release）→ [`SyncError::SyncBusy`]（可重试）
    /// - 槽位已被占用（其他 db 正在同步）→ [`SyncError::SyncBusy`]（可重试）
    /// - 成功 → 槽位置为 `Some(db_name)`，返回写权限
    ///
    /// 接线示例：
    /// ```rust,ignore
    /// let permit = WritePermit::acquire(
    ///     app.state::<crate::commands::AppState>().sync_write_guard.clone(),
    ///     "mistakes",
    /// )?;
    /// ```
    pub fn acquire(guard: SyncGuardSlot, db_name: &'static str) -> Result<Self, SyncError> {
        {
            let mut slot = guard
                .try_lock()
                .map_err(|_| SyncError::SyncBusy { db: db_name.to_string() })?;
            if slot.is_some() {
                return Err(SyncError::SyncBusy { db: db_name.to_string() });
            }
            *slot = Some(db_name.to_string());
        } // slot 在此释放，借用结束
        Ok(Self { db_name, guard })
    }

    /// 写门持有者的数据库名
    pub fn db_name(&self) -> &'static str {
        self.db_name
    }

    /// 槽位当前是否仍指向本 db（被嵌套覆盖时为 false）
    pub fn is_holder(&self) -> bool {
        match self.guard.try_lock() {
            Ok(slot) => slot.as_deref() == Some(self.db_name),
            Err(_) => false,
        }
    }
}

impl Drop for WritePermit {
    fn drop(&mut self) {
        // 仅在槽位仍指向本 db 时清空 — 防嵌套错乱
        // （嵌套场景下，外层 drop 不应误清内层覆盖后的槽位）
        if let Ok(mut slot) = self.guard.try_lock() {
            if slot.as_deref() == Some(self.db_name) {
                *slot = None;
            }
        }
    }
}

/// 业务写命令入口的同步写门检查。
///
/// ## 用法（接线阶段）
///
/// 所有会写库的 Tauri 命令在入口处调用：
///
/// ```rust,ignore
/// check_write_gate(&state.sync_write_guard, "mistakes")?;
/// ```
///
/// 返回语义：
/// - 写门空闲 → `Ok(())`，业务写继续；
/// - 同步进行中（写门被占）→ `SyncInProgress { db: 持有者 }` — 可重试，
///   前端提示"数据正在同步，请稍后重试"；
/// - 锁被瞬时占用 → `SyncBusy { db: 请求的 db }` — 可重试。
///
/// 原则：**写门只挡业务写，不挡同步自身** — 同步内部写不经过命令层，不受此检查影响。
pub fn check_write_gate(guard: &SyncGuardSlot, db: &str) -> Result<(), SyncError> {
    match guard.try_lock() {
        Ok(slot) => {
            if let Some(holder) = slot.as_ref() {
                Err(SyncError::SyncInProgress { db: holder.clone() })
            } else {
                Ok(())
            }
        }
        Err(_) => Err(SyncError::SyncBusy { db: db.to_string() }),
    }
}

/// 同步会话 — 由同步命令链创建持有，Drop 时按逆序释放全部权限。
///
/// 释放顺序：写权限（清 AppState 写门）→ 读连接 → 全局信号量。
/// `storage` / `device_id` / `progress` 等字段按后续接线需要扩展（本块先保持最小）。
pub struct SyncSession {
    /// 读权限集合（P1 导出阶段持有，连接关闭即回收）
    read_permits: Vec<ReadPermit>,
    /// 写权限集合（P3 单库事务期持有，每库独立 acquire/release）
    write_permits: Vec<WritePermit>,
    /// 全局信号量许可（`BACKUP_GLOBAL_LIMITER`，与备份/恢复互斥；Drop 自动释放）
    _semaphore: Option<tokio::sync::OwnedSemaphorePermit>,
    /// 全局信号量句柄（默认 `BACKUP_GLOBAL_LIMITER`；测试可注入独立信号量）
    limiter: Arc<tokio::sync::Semaphore>,
}

impl SyncSession {
    /// 创建空会话（容量 4，对应 4 个受管数据库）
    pub fn new() -> Self {
        Self::with_capacity(4)
    }

    /// 按容量创建会话
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            read_permits: Vec::with_capacity(capacity),
            write_permits: Vec::with_capacity(capacity),
            _semaphore: None,
            limiter: crate::backup_common::BACKUP_GLOBAL_LIMITER.clone(),
        }
    }

    /// 使用自定义信号量创建会话（测试注入用；默认使用 `BACKUP_GLOBAL_LIMITER`）
    pub fn with_limiter(limiter: Arc<tokio::sync::Semaphore>, capacity: usize) -> Self {
        Self {
            read_permits: Vec::with_capacity(capacity),
            write_permits: Vec::with_capacity(capacity),
            _semaphore: None,
            limiter,
        }
    }

    /// 获取读权限：打开 db 的只读连接并加入会话。
    /// 失败路径由调用方 `?` 传播，已持有的权限在会话 Drop 时回收。
    pub fn acquire_read(&mut self, db_path: impl AsRef<Path>) -> Result<(), SyncError> {
        self.read_permits.push(ReadPermit::open(db_path)?);
        Ok(())
    }

    /// 获取写权限：尝试设置 AppState 写门；被占 → [`SyncError::SyncBusy`]（可重试）。
    pub fn acquire_write(
        &mut self,
        guard: SyncGuardSlot,
        db_name: &'static str,
    ) -> Result<(), SyncError> {
        self.write_permits
            .push(WritePermit::acquire(guard, db_name)?);
        Ok(())
    }

    /// 获取全局信号量许可（非阻塞尝试；被备份/恢复占用时 → [`SyncError::SyncBusy`]，可重试）。
    /// 已持有时幂等返回 `Ok(())`；Drop 时自动释放。
    pub fn acquire_global_semaphore(&mut self) -> Result<(), SyncError> {
        if self._semaphore.is_some() {
            return Ok(());
        }
        match self.limiter.clone().try_acquire_owned() {
            Ok(permit) => {
                self._semaphore = Some(permit);
                Ok(())
            }
            Err(_) => Err(SyncError::SyncBusy {
                db: "backup_global_limiter".to_string(),
            }),
        }
    }

    /// 移交外部已获取的全局信号量许可给会话持有（幂等）。
    ///
    /// 同步命令链用「带超时」的方式获取信号量（保持 60s 超时语义不变），
    /// 然后把拿到的 `OwnedSemaphorePermit` 移交给本会话统一持有，
    /// 由会话 Drop 时自动释放 —— 会话只是"持证者"，不改变获取方式。
    ///
    /// 幂等：若会话已持有信号量许可（例如已调用过 `acquire_global_semaphore`），
    /// 新传入的许可在本方法返回时立即 Drop 自动释放，不会重复占用信号量。
    pub fn attach_global_semaphore(&mut self, permit: tokio::sync::OwnedSemaphorePermit) {
        if self._semaphore.is_none() {
            self._semaphore = Some(permit);
        }
        // 已持有时, 新传入的 permit 在此返回即释放
    }

    /// 结束只读快照阶段：回收全部读权限（关闭只读连接）。
    ///
    /// 同步命令链在 P1（读 pending changes + enrich 整行数据）阶段结束时显式调用，
    /// 避免只读连接跨阶段持有；调用后 `read_permit_count() == 0`，
    /// 之后可再次 `acquire_read` 开启新的只读快照。
    pub fn release_read_permits(&mut self) {
        self.read_permits.clear();
    }

    /// 当前持有的读权限数
    pub fn read_permit_count(&self) -> usize {
        self.read_permits.len()
    }

    /// 当前持有的写权限数
    pub fn write_permit_count(&self) -> usize {
        self.write_permits.len()
    }

    /// 是否持有指定 db 的写权限
    pub fn has_write_permit(&self, db_name: &str) -> bool {
        self.write_permits.iter().any(|p| p.db_name == db_name)
    }

    /// 读权限集合（供导出阶段经 [`ReadPermit::conn`] 执行只读查询）
    pub fn read_permits(&self) -> &[ReadPermit] {
        &self.read_permits
    }

    /// 写权限集合
    pub fn write_permits(&self) -> &[WritePermit] {
        &self.write_permits
    }
}

impl Default for SyncSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SyncSession {
    fn drop(&mut self) {
        // 1. 释放全部写权限（清 AppState.sync_write_guard 槽位）
        self.write_permits.clear();
        // 2. 关闭全部读连接（ReadPermit Drop 即关闭连接）
        self.read_permits.clear();
        // 3. 释放全局信号量许可
        self._semaphore.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;

    /// 构造一个独立的写门槽位（等价于 AppState.sync_write_guard）
    fn test_guard() -> SyncGuardSlot {
        Arc::new(Mutex::new(None))
    }

    /// 预建测试表（普通连接）
    fn setup_table(db_path: &Path) {
        let conn = Connection::open(db_path).unwrap();
        conn.execute_batch("CREATE TABLE t (x INTEGER);").unwrap();
        drop(conn);
    }

    #[test]
    fn test_read_permit_open_and_read() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test_read.db");
        setup_table(&db_path);

        let permit = ReadPermit::open(&db_path).unwrap();
        // 只读查询可用
        let count: i64 = permit
            .conn()
            .query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
        assert_eq!(permit.db_path(), db_path.as_path());
    }

    #[test]
    fn test_read_permit_rejects_writes() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test_ro.db");
        setup_table(&db_path);

        let permit = ReadPermit::open(&db_path).unwrap();
        // DML 写操作应被拒（SQLITE_READONLY）
        let err = permit
            .conn()
            .execute("INSERT INTO t (x) VALUES (1)", [])
            .unwrap_err();
        assert!(
            err.to_string().contains("readonly"),
            "期望 INSERT 被拒 (readonly)，实际: {}",
            err
        );
        // DDL 同样被拒
        let err = permit
            .conn()
            .execute_batch("CREATE TABLE t2 (y INTEGER);")
            .unwrap_err();
        assert!(
            err.to_string().contains("readonly"),
            "期望 DDL 被拒 (readonly)，实际: {}",
            err
        );
    }

    #[test]
    fn test_write_permit_sets_and_clears_guard() {
        let guard = test_guard();
        {
            let permit = WritePermit::acquire(guard.clone(), "mistakes").unwrap();
            assert_eq!(permit.db_name(), "mistakes");
            assert!(permit.is_holder());
            let slot = guard.try_lock().unwrap();
            assert_eq!(slot.as_deref(), Some("mistakes"));
        } // Drop → 槽位清空
        let slot = guard.try_lock().unwrap();
        assert_eq!(slot.as_deref(), None, "Drop 后写门应自动回收");
    }

    #[test]
    fn test_write_permit_busy_when_occupied() {
        let guard = test_guard();
        let _first = WritePermit::acquire(guard.clone(), "mistakes").unwrap();
        // 同 db 二次获取 → SyncBusy
        let err = WritePermit::acquire(guard.clone(), "mistakes").unwrap_err();
        assert!(
            matches!(err, SyncError::SyncBusy { .. }),
            "期望 SyncBusy，实际: {}",
            err
        );
        // 其他 db 同样 → SyncBusy（写门同一时刻只能被一个 db 持有）
        let err = WritePermit::acquire(guard.clone(), "vfs").unwrap_err();
        assert!(matches!(err, SyncError::SyncBusy { .. }));
    }

    #[test]
    fn test_write_permit_drop_does_not_clear_foreign() {
        // 防嵌套错乱：槽位被外部覆盖为其他持有者时，drop 不得误清
        let guard = test_guard();
        let permit = WritePermit::acquire(guard.clone(), "a").unwrap();
        *guard.try_lock().unwrap() = Some("b".to_string()); // 模拟嵌套覆盖
        assert!(!permit.is_holder(), "覆盖后本写权限不再是持有者");
        drop(permit);
        let slot = guard.try_lock().unwrap();
        assert_eq!(slot.as_deref(), Some("b"), "不应清空非本持有者槽位");
    }

    #[test]
    fn test_check_write_gate() {
        let guard = test_guard();
        // 写门空闲 → Ok
        assert!(check_write_gate(&guard, "mistakes").is_ok());
        // 写门被占 → SyncInProgress（携带持有者 db 名）
        let _p = WritePermit::acquire(guard.clone(), "mistakes").unwrap();
        match check_write_gate(&guard, "mistakes") {
            Err(SyncError::SyncInProgress { db }) => assert_eq!(db, "mistakes"),
            other => panic!("期望 SyncInProgress，实际: {:?}", other),
        }
        // 锁被瞬时占用（未写值，仅持锁）→ SyncBusy
        let _held = guard.try_lock().unwrap();
        let err = check_write_gate(&guard, "vfs").unwrap_err();
        assert!(
            matches!(err, SyncError::SyncBusy { .. }),
            "期望 SyncBusy，实际: {}",
            err
        );
    }

    #[test]
    fn test_sync_session_acquire_and_drop_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("session.db");
        setup_table(&db_path);

        let guard = test_guard();
        let limiter = Arc::new(tokio::sync::Semaphore::new(1));
        {
            let mut session = SyncSession::with_limiter(limiter.clone(), 4);
            session.acquire_global_semaphore().unwrap();
            assert_eq!(session.read_permit_count(), 0);
            session.acquire_read(&db_path).unwrap();
            session.acquire_write(guard.clone(), "mistakes").unwrap();
            assert_eq!(session.read_permit_count(), 1);
            assert_eq!(session.write_permit_count(), 1);
            assert!(session.has_write_permit("mistakes"));
            assert!(!session.has_write_permit("vfs"));
            // 会话持有期间：读连接可用、写门被占
            assert_eq!(session.read_permits().len(), 1);
            assert_eq!(session.write_permits().len(), 1);
            let slot = guard.try_lock().unwrap();
            assert_eq!(slot.as_deref(), Some("mistakes"));
        } // Drop：写 → 读 → 信号量，全部回收

        // 写门已清空
        let slot = guard.try_lock().unwrap();
        assert_eq!(slot.as_deref(), None, "会话 Drop 后写门应清空");
        // 信号量已释放 → 新会话可再次获取
        let mut s2 = SyncSession::with_limiter(limiter.clone(), 1);
        s2.acquire_global_semaphore().unwrap();
        // 读连接已关闭 → 普通连接可正常写入
        let conn = Connection::open(&db_path).unwrap();
        conn.execute("INSERT INTO t (x) VALUES (1)", []).unwrap();
    }

    #[test]
    fn test_sync_session_semaphore_busy_then_released() {
        let limiter = Arc::new(tokio::sync::Semaphore::new(1));
        let mut s1 = SyncSession::with_limiter(limiter.clone(), 1);
        s1.acquire_global_semaphore().unwrap();
        // 幂等：二次获取仍 Ok
        assert!(s1.acquire_global_semaphore().is_ok());
        // 第二个会话 → SyncBusy（可重试）
        let mut s2 = SyncSession::with_limiter(limiter.clone(), 1);
        let err = s2.acquire_global_semaphore().unwrap_err();
        assert!(matches!(err, SyncError::SyncBusy { .. }));
        drop(s1);
        // s1 释放后，s2 可再获取
        assert!(s2.acquire_global_semaphore().is_ok());
    }

    #[test]
    fn test_attach_global_semaphore_and_release_read_permits() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("attach.db");
        setup_table(&db_path);

        // 双许可信号量: 模拟命令链 60s 超时获取 + 幂等移交
        let limiter = Arc::new(tokio::sync::Semaphore::new(2));
        let p1 = limiter.clone().try_acquire_owned().unwrap();
        let p2 = limiter.clone().try_acquire_owned().unwrap();

        let mut session = SyncSession::with_limiter(limiter.clone(), 4);
        session.attach_global_semaphore(p1);
        // 幂等: 已持有时, 新传入的 p2 在方法返回时自动释放
        session.attach_global_semaphore(p2);
        assert_eq!(limiter.available_permits(), 1, "p2 应被释放, p1 由会话持有");

        // 只读快照: acquire → release 后计数归零, 连接关闭可再写
        session.acquire_read(&db_path).unwrap();
        assert_eq!(session.read_permit_count(), 1);
        session.release_read_permits();
        assert_eq!(session.read_permit_count(), 0, "release 后读权限应全部回收");

        drop(session);
        assert_eq!(limiter.available_permits(), 2, "会话 Drop 后信号量应全部释放");
    }

    #[test]
    fn test_retryable_error_display() {
        // 两个新错误变体均注明"可重试"，供前端提示"稍后重试"
        let in_progress = SyncError::SyncInProgress {
            db: "mistakes".to_string(),
        };
        let busy = SyncError::SyncBusy {
            db: "mistakes".to_string(),
        };
        assert!(in_progress.to_string().contains("可重试"));
        assert!(busy.to_string().contains("可重试"));
    }
}
