# Backend Core

- Tauri command wrappers live in `src-tauri/src/commands/v1.rs`; most simply pass `State<AppState>` and request structs into `services::v1` functions.
- `src-tauri/src/services/v1.rs` owns LanceDB operations: connect/disconnect, table listing/open/schema, schema evolution, row CRUD, import/export, optimize, versions, clone, indexes, scan, filter/vector/FTS/combined search.
- Runtime handles are managed by `src-tauri/src/services/connection_manager.rs`; connection/table handles are keyed by generated UUIDs.
- Shared IPC enums/structs live in `src-tauri/src/ipc/v1.rs`; keep them mirrored with frontend `src/ipc/v1.ts`.
- Integration tests under `src-tauri/tests/commands_v1.rs` create/copy sample DBs and exercise service functions directly, not the full Tauri invoke boundary.