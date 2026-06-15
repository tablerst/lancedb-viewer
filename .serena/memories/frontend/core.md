# Frontend Core

- `src/App.vue` composes app shell, Naive config, workspace context, sidebar, primary nav, status bridge, and route view.
- `src/components/sidebar/Sidebar.vue` manages profile filtering/actions and renders `ConnectionItem.vue`; it owns dialog window opening for connection create/edit/credentials.
- `src/views/ExplorerView.vue` is the main LanceDB table workspace and currently concentrates many responsibilities: schema/data tabs plus table/schema/index/version/import/export/optimization/data mutation dialogs.
- `src/views/SearchView.vue` is the query workspace for filter/vector/FTS/combined search over the active table context.
- Profile/connection state is split across `src/composables/useProfiles.ts`, `src/composables/useConnection.ts`, and `src/composables/workspaceContext.ts`.
- `src/lib/tauriClient.ts` wraps typed IPC calls and envelope unwrapping; use it instead of raw invoke in feature UI.