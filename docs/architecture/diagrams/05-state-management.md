# State Management Architecture - Zustand Diagrams

> **Last updated**: 2026-06-06 (derived from source code analysis)
> **Source files**: `src/stores/*`, `src/features/*/stores/*`
> **Library**: Zustand (with `persist`, `subscribeWithSelector`, `immer` middleware)

---

## a) Zustand Store Map

This diagram shows all Zustand stores, their data domains, persistence status, and inter-store dependencies.

```mermaid
flowchart TB
  subgraph GlobalStores["Global Stores (src/stores/)"]
    direction TB

    subgraph Persistent["Persistent Stores (localStorage)"]
      UI["useUIStore<br/>━━━━━━━━━━━━━━<br/>leftPanelCollapsed<br/>toggleLeftPanel()"]
    end

    subgraph NonPersistent["Non-Persistent Stores"]
      VS["useViewStore<br/>━━━━━━━━━━━━━━<br/>currentView<br/>previousView<br/>setCurrentView()"]
      SYS["useSystemStatusStore<br/>━━━━━━━━━━━━━━<br/>migrationVisible<br/>maintenanceMode<br/>showMigrationStatus()"]
      NET["useNetworkStore<br/>━━━━━━━━━━━━━━<br/>isOnline<br/>lastChangedAt"]
      SSS["useSettingsShellStore<br/>━━━━━━━━━━━━━━<br/>activeTab<br/>dataGovernanceTabTarget"]
    end

    subgraph APIBacked["API-Backed Stores (fetch → cache)"]
      QB["useQuestionBankStore<br/>━━━━━━━━━━━━━━<br/>questions, filters<br/>stats, pagination<br/>CRUD + answer tracking"]
      RP["useReviewPlanStore<br/>━━━━━━━━━━━━━━<br/>dueReviews, history<br/>review stats<br/>SM-2 algorithm"]
      RS["useResearchStore (HpiasStore)<br/>━━━━━━━━━━━━━━<br/>sessionId, round, synthesis<br/>events log, artifacts<br/>research orchestration"]
      UI2["useUnifiedIndexStore<br/>━━━━━━━━━━━━━━<br/>summary, selectedResource<br/>filters, reindex actions"]
    end

    subgraph AnkiStores["Anki Stores"]
      ANK["useAnkiUIStore<br/>━━━━━━━━━━━━━━<br/>Document, Template, Cards<br/>AnkiConnect, Import, UI<br/>6 slices via Slice Pattern"]
      ANKQ["useAnkiQueueStore"]
      TA["useTemplateAiStore"]
    end
  end

  subgraph FeatureStores["Feature Stores (src/features/*/stores/)"]
    direction TB

    subgraph LHPersist["Learning Hub (persistent)"]
      FS["useFinderStore (persist)<br/>━━━━━━━━━━━━━━<br/>currentPath, history<br/>items, search, selection<br/>navigation actions<br/>backend: dstu API"]
      DS["useDesktopStore (persist)<br/>━━━━━━━━━━━━━━<br/>shortcuts list<br/>desktopRoot config<br/>shortcut CRUD"]
      RS["useRecentStore (persist)<br/>━━━━━━━━━━━━━━<br/>recent items list<br/>maxItems: 50<br/>addRecent / removeRecent"]
    end

    subgraph PDFStores["PDF Reader"]
      PPS["usePdfProcessingStore<br/>━━━━━━━━━━━━━━<br/>statusMap: fileId→status<br/>stage, percent, readyModes<br/>auto-cleanup after 60s"]
      PSS["usePdfSettingsStore (persist)<br/>━━━━━━━━━━━━━━<br/>DPR, text layer, annotation<br/>thumbnail, default view<br/>scroll DPR downgrade"]
    end

    subgraph OtherFeature["Other Feature Stores"]
      NTS["useNotesTreeStore<br/>(immer + subscribeWithSelector)<br/>━━━━━━━━━━━━━━<br/>treeData, expandedIds<br/>drag state, filter<br/>persistence snapshot"]
      TDS["useTodoStore<br/>━━━━━━━━━━━━━━<br/>lists, items, filter<br/>CRUD + view queries<br/>backend: todo API"]
      PMS["usePomodoroStore (persist)<br/>━━━━━━━━━━━━━━<br/>mode, timeLeft<br/>currentTaskId<br/>session recording"]
      SBS["useSandboxWorkbenchStore<br/>━━━━━━━━━━━━━━<br/>workbench state"]
      TMS["template-management stores"]
      PRS["practice stores"]
    end
  end

  subgraph ChatStoreSystem["Chat V2 Store System<br/>(Per-Session)"]
    direction TB
    CHAT["ChatStore<br/>(created per session via createChatStore)<br/>━━━━━━━━━━━━━━<br/>messages, blocks, variants<br/>session state, params<br/>streaming state<br/>queued messages"]
    SM["sessionManager<br/>(manages multiple ChatStores)<br/>━━━━━━━━━━━━━━<br/>session lifecycle<br/>current-session-changed events"]
    GC["groupCache<br/>(group metadata cache)"]
  end

  %% Store-to-Store Dependencies
  VS -->|"currentView changed → re-renders"| ANK
  VS -->|"visibility check"| PPS
  VS -->|"visibility check"| SBS

  FS -->|"imports recentStore"| RS
  FS -->|"imports desktopStore (for quickAccess)"| DS
  SYS -->|"maintenance mode blocks writes"| QB

  SM -->|"creates/manages"| CHAT
  GC -->|"provides group data to"| CHAT

  %% Store → API → Backend flows
  FS -.->|"dstu.list(), dstu.search()"| DSTU_API["DSTU API"]
  FS -.->|"folderApi, trashApi"| DSTU_API
  QB -.->|"invoke question bank commands"| TAURI["Tauri IPC (invoke)"]
  RP -.->|"invoke review plan commands"| TAURI
  PPS -.->|"invoke pdf processing commands"| TAURI
  TDS -.->|"invoke todo commands"| TAURI
  ANK -.->|"invoke Anki commands"| TAURI
  UI2 -.->|"vfsUnifiedIndexApi"| TAURI

  PSS -.->|"localStorage only (no backend)"| NONE["No Backend Calls"]
  RS -.->|"localStorage only"| NONE
  DS -.->|"localStorage only"| NONE
  UI -.->|"localStorage only"| NONE

  %% Component subscriptions
  subgraph ComponentSubs["Key Component → Store Subscriptions"]
    direction LR
    APP["App.tsx"] --> VS
    APP -->|"sidebar collapse"| UI
    APP --> SYS
    SIDEBAR["ModernSidebar"] --> UI
    SIDEBAR --> VS
    CHATPG["ChatV2Page"] --> FS
    CHATPG --> SM
    LHUB_PG["LearningHubPage"] --> FS
    LHUB_PG --> DS
    LHUB_PG --> RS
    PDFRD["PdfReader"] --> PSS
    PDFRD --> PPS
    TODO_PG["TodoPage"] --> TDS
    TODO_PG --> PMS
    SETTPG["SettingsPage"] --> SSS
    SETTPG --> SYS
  end
```

---

## b) Data Flow per Store — Key Stores Detailed Analysis

### Store 1: `useFinderStore` (Learning Hub — File Browser)

**File**: `src/features/learning-hub/stores/finderStore.ts`

```mermaid
flowchart LR
  subgraph InitState["Initial State"]
    CP["currentPath: {<br/>  viewKind: 'folder',<br/>  breadcrumbs: [],<br/>  folderId: null,<br/>  typeFilter: null<br/>}"]
    HIS["history: [DEFAULT_PATH]<br/>historyIndex: 0"]
    VIEW["viewMode: 'grid'<br/>sortBy: 'updatedAt'<br/>sortOrder: 'desc'"]
    SEL["selectedIds: Set()<br/>lastSelectedId: null"]
    DATA["items: []<br/>isLoading: false<br/>error: null"]
    SEARCH["searchQuery: ''<br/>isSearching: false"]
  end

  subgraph Actions["Key Actions"]
    NAV["navigateTo(path)"]
    ENTER["enterFolder(folderId)"]
    GO_UP["goUp()"]
    GO_BACK["goBack() / goForward()"]
    LOAD["loadItems() ⟶ dstu.list()"]
    SEARCH_ACT["executeSearch() ⟶ dstu.search()"]
    SELECT["select(id, mode)"]
    REFRESH["refresh()"]
    SORT["setSorting(sortBy, sortOrder)"]
  end

  subgraph Backend["Backend Calls"]
    DL["dstu.list(options)<br/>→ 'dstu_list_folder'"]
    DS["dstu.search(query, options)<br/>→ 'dstu_search'"]
    SIF["dstu.searchInFolder()"]
    DLIST["dstu.listDeleted()"]
    TLIST["trashApi.listTrash()"]
    BRE["folderApi.getBreadcrumbs(id)"]
  end

  subgraph Persistence["localStorage Persistence (key: 'learning-hub-finder')"]
    PV["Persisted Fields:<br/>- viewMode<br/>- sortBy<br/>- sortOrder<br/>- quickAccessCollapsed<br/><br/>NOT persisted:<br/>- items, isLoading, error<br/>- selection, search<br/>- currentPath, history"]
  end

  subgraph ReRender["Trigger Re-renders"]
    COMPS["Components subscribing:<br/>- LearningHubSidebar<br/>- FinderView (Grid/List)<br/>- Breadcrumb<br/>- QuickAccess panel<br/>- Selection toolbar"]
    SELE["Fine-grained selectors:<br/>useFinderStore(s => s.items)<br/>useFinderStore(s => s.isLoading)<br/>useFinderStore(s => s.selectedIds)"]
  end

  InitState --> Actions
  Actions --> Backend
  Actions -->|"set()"| ReRender
  Backend -->|"result → set()"| ReRender
  Persistence -.->|"hydrate on init"| InitState
  Actions -.->|"partialize"| Persistence

  subgraph CacheInvalidation["Cache Invalidation"]
    REQ_ID["_currentRequestId counter<br/>Each new request increments"]
    STALE["On response: check requestId match<br/>Stale responses discarded"]
    AUTO_REFRESH["refresh() called after:<br/>- sort change<br/>- folder navigation<br/>- external mutation events"]
  end
  Actions --> CacheInvalidation
```

### Store 2: `usePdfProcessingStore` (Media Processing Progress)

**File**: `src/features/pdf/stores/pdfProcessingStore.ts`

```mermaid
flowchart LR
  subgraph InitState["Initial State"]
    MAP["statusMap: Map<fileId, ProcessingStatus>"]
  end

  subgraph ProcessingStage["Processing Pipeline"]
    PEND["pending"]
    TEX["text_extraction"]
    REND["page_rendering"]
    COMP["page_compression /<br/>image_compression"]
    OCR["ocr_processing"]
    VEC["vector_indexing"]
    DONE["completed /<br/>completed_with_issues"]
    ERR["error"]
  end

  subgraph Actions["Actions"]
    UPD["update(fileId, partialStatus)"]
    COMPL["setCompleted(fileId, readyModes)"]
    ERRS["setError(fileId, error)"]
    FULL["setFullStatus(fileId, status)"]
    REM["remove(fileId)"]
    CLR["clear()"]
  end

  subgraph DataFlow["Data Sources"]
    POLL["Polling from component<br/>(usePdfProcessingProgress hook)"]
    EVENT["Backend events<br/>(processing progress)"]
    INIT["Initial state on file upload"]
  end

  subgraph Cleanup["Auto-cleanup"]
    EVICT["enforceMaxEntries(MAX_ENTRIES=100)"]
    TIMER["setTimeout 60s<br/>→ remove terminal entries"]
    STAGE["Stage ordering guard<br/>→ reject stale updates"]
  end

  InitState --> Actions
  DataFlow -->|"invoke"| Actions
  Actions -->|"set state"| ProcessingStage
  Actions --> Cleanup
  ProcessingStage -->|"re-render"| COMPS2["Components:<br/>- ProcessingIndicator<br/>- Chat InputBar (readyModes)<br/>- LearningHub items"]
```

### Store 3: `usePdfSettingsStore` (PDF Reader Configuration)

**File**: `src/features/pdf/stores/pdfSettingsStore.ts`

```mermaid
flowchart LR
  subgraph Init["Initial State (Defaults)"]
    MDPR["maxDevicePixelRatio: 1.5"]
    SCR["enableScrollDprDowngrade: true"]
    VIRT["virtualizerOverscan: 5"]
    TXT["enableTextLayerByDefault: true"]
    ANN["enableAnnotationLayerByDefault: false"]
    THUMB["thumbnailWidth: 100"]
    SCALE["defaultScale: 1.0"]
    VIEWM["defaultViewMode: 'single'"]
  end

  subgraph Actions["Actions"]
    UPD_SET["updateSetting(key, value)<br/>with range validation"]
    UPD_BATCH["updateSettings(partial)<br/>batch with validation"]
    RESET["resetSettings()"]
    DPR["getRenderDpr(isScrolling)<br/>→ DPR = min(device, max, scroll)"]
  end

  subgraph Persist["localStorage Persistence"]
    KEY["key: 'pdf-settings'<br/>version: 1"]
    PARTIAL["partialize: settings only"]
  end

  subgraph Consumers["Component Consumers"]
    PDF_RDR["PdfReader<br/>(page rendering)"]
    PDF_SETTINGS["PdfSettingsPanel<br/>(UI controls)"]
    THUMBNAIL["ThumbnailSidebar<br/>(thumbnail DPR)"]
  end

  Init --> Actions
  Actions -->|"set() → re-render"| Consumers
  Actions -.->|"persist"| Persist
  Persist -.->|"hydrate"| Init
```

### Store 4: `useAnkiUIStore` (Anki Card Generation — Slice Pattern)

**File**: `src/stores/anki/useAnkiUIStore.ts`

```mermaid
flowchart TB
  subgraph StoreSlices["6 Feature Slices"]
    subgraph Document["Document Slice"]
      D1["documentContent<br/>currentDocumentId<br/>selectedFiles<br/>isProcessingFiles"]
      D_ACT["setDocumentContent()<br/>clearDocument()<br/>loadMaterialToDocument()"]
    end

    subgraph Template["Template Slice"]
      T1["selectedTemplateId<br/>allTemplates<br/>isLoadingTemplates<br/>showTemplatePicker"]
      T_ACT["setAllTemplates()<br/>setSelectedTemplateId()<br/>getSelectedTemplate()"]
    end

    subgraph Cards["Cards Slice"]
      C1["generatedCards[]<br/>documentTasks[]<br/>isGenerating<br/>isPaused<br/>generationError<br/>selectedCardIds"]
      C_ACT["setGeneratedCards()<br/>addGeneratedCard()<br/>removeGeneratedCard()<br/>selectAllCards()"]
    end

    subgraph AnkiConnect["AnkiConnect Slice"]
      A1["isAnkiConnectAvailable<br/>ankiDeckNames<br/>ankiModelNames<br/>connectionError"]
      A_ACT["updateConnectionStatus()<br/>setAnkiDeckNames()"]
    end

    subgraph Import["Import Slice"]
      I1["mistakeSummaries<br/>selectedMistakeIds<br/>isLoadingMistakes"]
      I_ACT["setMistakeSummaries()<br/>toggleMistakeSelection()"]
    end

    subgraph UI["UI Slice"]
      U1["dialogs, panels<br/>activeTab, error<br/>isBatchMode, previewingCard<br/>cardViewMode<br/>selectedQueueIds"]
      U_ACT["setDialogOpen()<br/>setPanelOpen()<br/>setActiveTab()"]
    end
  end

  subgraph SelectorHooks["Fine-grained Selector Hooks"]
    useShallow["useShallow selector hooks:<br/>- useDocumentState()<br/>- useTemplateState()<br/>- useCardsState()<br/>- useAnkiConnectState()<br/>- useGenerationOptions()"]
    getActions["getAnkiUIStoreActions()<br/>(without triggering subscription)"]
  end

  subgraph Backend["Backend Communication"]
    INV["invoke('anki_*' commands)<br/>via ankiWorkflowManager"]
    GEN["Card Generation<br/>(backend event stream)"]
  end

  Backend -->|"results → slice actions"| StoreSlices
  StoreSlices -->|"subscribe"| SelectorHooks
  SelectorHooks -->|"re-render"| COMPONENTS["Components:<br/>- AnkiPanelHost<br/>- TaskDashboardPage<br/>- TemplatePickerDialog<br/>- CardPreviewModal"]
```

### Store 5: Chat V2 Store System (Per-Session Store Architecture)

**Files**: `src/features/chat/core/store/*`, `src/features/chat/core/session/sessionManager.ts`

```mermaid
flowchart TB
  subgraph MultiSession["Session Manager"]
    SM["sessionManager<br/>━━━━━━━━━━━━━━<br/>- Map<sessionId, ChatStore><br/>- currentSessionId<br/>- createSession()<br/>- deleteSession()<br/>- subscribe() events:<br/>  • current-session-changed<br/>  • session-created<br/>  • session-deleted"]
  end

  subgraph SingleSession["Single ChatStore (per session)"]
    direction TB
    CS["createChatStore(initialState)<br/>━━━━━━━━━━━━━━<br/>Zustand store factory"]

    subgraph State["Core State"]
      SESS["sessionId, sessionStatus<br/>chatParams<br/>panelStates<br/>tokenUsage"]
      MSGS["messages[]<br/>blocks[]<br/>variants[]<br/>queuedMessages[]"]
      CONTEXT["contextRefs[]<br/>sharedContext<br/>attachments[]"]
      STREAM["streamingState<br/>currentStreamId<br/>streamBlocks"]
    end

    subgraph Actions["Action Modules"]
      MSG_ACT["messageActions.ts<br/>sendMessage, retry, edit<br/>delete, toggle thinking"]
      BLK_ACT["blockActions.ts<br/>upsertBlock, updateStatus"]
      STM_ACT["streamActions.ts<br/>startStream, onChunk<br/>cancelStream, complete"]
      CTX_ACT["contextActions.ts<br/>addContext, removeContext"]
      SESS_ACT["sessionActions.ts<br/>saveSession, loadSession<br/>archiveSession"]
      QUE_ACT["queueActions.ts<br/>enqueueMessage, processQueue"]
      REST_ACT["restoreActions.ts<br/>resumeFromBackend"]
      VAR_ACT["variantActions.ts<br/>create variant, switch"]
      SKL_ACT["skillActions.ts<br/>update skillets"]
    end

    subgraph Middleware["Middleware Pipeline"]
      MW_AUTO["autoSave.ts<br/>auto-save on state change"]
      MW_CHUNK["chunkBuffer.ts<br/>buffer stream chunks"]
      MW_BRIDGE["eventBridge.ts<br/>forward events ↔ components"]
    end
  end

  subgraph External["External Communication"]
    ADAPTER["TauriAdapter.ts<br/>sendMessage → invoke<br/>streamComplete → listen"]
    CHAT_API["chatV2Api.ts<br/>session CRUD commands<br/>streaming block commands<br/>tag/variant commands"]
  end

  SM -->|"create/switch"| SingleSession
  MSG_ACT -->|"calls"| ADAPTER
  BLK_ACT -->|"calls"| CHAT_API
  STM_ACT -->|"calls"| ADAPTER
  SESS_ACT -->|"calls"| CHAT_API
  State -.->|"subscribe → components"| COMPONENTS2["Components:<br/>MessageList, InputBarV2<br/>SessionBrowser, ActivityTimeline<br/>VariantSwitcher, SourcePanel"]
  Middleware -.->|"intercept state changes"| State

  subgraph DataFlow["Data Flow"]
    direction LR
    USER["User types message"] --> INPUT["InputBarV2"]
    INPUT --> SEND["sendMessage()"]
    SEND --> ADAPTER
    ADAPTER -->|"invoke('chat_v2_send_message')"| RUST["Rust Backend"]
    RUST -->|"streaming events"| ADAPTER
    ADAPTER -->|"onChunk callback"| CS
    CS -->|"immer update"| State
    State -->|"Zustand notify"| COMPONENTS2
  end
```

### Cache Invalidation Strategy Summary

| Store | Strategy | Details |
|-------|----------|---------|
| `useFinderStore` | Request-ID based | `_currentRequestId` counter; stale responses discarded on mismatch. Auto-refresh on sort/nav changes. |
| `usePdfProcessingStore` | Stage-ordering + TTL | `shouldAcceptUpdate()` checks stage precedence map. Terminal entries auto-removed after 60s. Max 100 entries. |
| `usePdfSettingsStore` | None (persisted) | No invalidation needed; settings change only on user action. Range-validated at write time. |
| `useAnkiUIStore` | Explicit refresh | Templates reloaded on mount/explicit action. Cards cleared on new generation. AnkiConnect status re-checked on demand. |
| `useQuestionBankStore` | Page + filter based | Questions reloaded on page/filter/search change. Stats recomputed on answer submission. |
| `useReviewPlanStore` | On-submit refresh | Due reviews list refreshed after each review submission. SM-2 algorithm applied client-side then synced. |
| Chat Store System | Event-driven | Streaming events from backend update store in real-time. Auto-save middleware persists to DB on state changes. Session restore from backend on load. |
| Notes Tree Store | Snapshot-based | Persistence snapshots created on tree changes. Versioned to handle migration. |
| Todo Store | Request-vs-based | `itemsRequestVersion` tracks staleness. Refresh on mutation. |

---

## Source File References

| Store | File Path | Middleware | Persistence Key |
|-------|-----------|-----------|-----------------|
| `useViewStore` | `src/stores/viewStore.ts` | — | — |
| `useUIStore` | `src/stores/uiStore.ts` | `persist` | `dstu-ui-store` |
| `useSystemStatusStore` | `src/stores/systemStatusStore.ts` | — | — |
| `useNetworkStore` | `src/stores/networkStore.ts` | — | — |
| `useSettingsShellStore` | `src/stores/settingsShellStore.ts` | — | — |
| `useQuestionBankStore` | `src/stores/questionBankStore.ts` | `subscribeWithSelector`, `devtools` | — |
| `useReviewPlanStore` | `src/stores/reviewPlanStore.ts` | `subscribeWithSelector`, `devtools` | — |
| `useResearchStore` | `src/stores/researchStore.ts` | `subscribeWithSelector`, `devtools` | — |
| `useUnifiedIndexStore` | `src/stores/unifiedIndexStore.ts` | — | — |
| `useAnkiUIStore` | `src/stores/anki/useAnkiUIStore.ts` | `subscribeWithSelector` | — |
| `useAnkiUIStore types` | `src/stores/anki/types.ts` | — | — |
| `useFinderStore` | `src/features/learning-hub/stores/finderStore.ts` | `persist` | `learning-hub-finder` |
| `useDesktopStore` | `src/features/learning-hub/stores/desktopStore.ts` | `persist` | `learning-hub-desktop` |
| `useRecentStore` | `src/features/learning-hub/stores/recentStore.ts` | `persist` | `learning-hub-recent` |
| `usePdfProcessingStore` | `src/features/pdf/stores/pdfProcessingStore.ts` | — | — |
| `usePdfSettingsStore` | `src/features/pdf/stores/pdfSettingsStore.ts` | `subscribeWithSelector`, `persist` | `pdf-settings` |
| `useNotesTreeStore` | `src/features/notes/stores/notesTreeStore.ts` | `subscribeWithSelector`, `immer` | — |
| `useTodoStore` | `src/features/todo/stores/useTodoStore.ts` | — | — |
| `usePomodoroStore` | `src/features/pomodoro/stores/usePomodoroStore.ts` | `persist` | (pomodoro) |
| `useSandboxWorkbenchStore` | `src/features/sandbox/store/useSandboxWorkbenchStore.ts` | — | — |
| Chat Store Factory | `src/features/chat/core/store/createChatStore.ts` | — | — |
| `sessionManager` | `src/features/chat/core/session/sessionManager.ts` | — | — |
| `groupCache` | `src/features/chat/core/store/groupCache.ts` | — | — |
