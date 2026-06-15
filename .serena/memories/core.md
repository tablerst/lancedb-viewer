# Core

- Tauri v2 + Vue 3 + TypeScript desktop app for LanceDB visualization/studio workflows.
- Main frontend entry: `src/App.vue`; router in `src/router.ts`; app routes include Explorer/Search/Credentials/Capabilities and dialog views.
- Main Rust entry: `src-tauri/src/lib.rs`; Tauri command wrappers in `src-tauri/src/commands/v1.rs`; backend logic in `src-tauri/src/services/v1.rs`; IPC structs mirrored by `src/ipc/v1.ts` and `src-tauri/src/ipc/v1.rs`.
- Core user flow: connection profiles -> connect -> list/open table -> schema/data browsing -> search/index/schema/table/data operations.
- Read `mem:frontend/core` for Vue/UI layout and `mem:backend/core` for Rust/Tauri/LanceDB service shape before changing feature behavior.