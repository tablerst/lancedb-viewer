# Review Stabilization 2026-06-15

This execution note tracks the repo-side follow-up for the Agenta version
`Review Stabilization 2026-06-15`. Agenta remains the recovery ledger; this file
owns implementation sequencing and design decisions that are too detailed for
task descriptions.

## Current Pass: LDBREV-01 / LDBREV-02 / LDBREV-05

Scope:

- Restore the red Rust CRUD/vector write test.
- Support JSON row writes for LanceDB fixed-size float vectors.
- Add backend destructive mutation guardrails.
- Protect the new IPC payload contract with focused tests.

Implemented decisions:

- `write_rows_v1` still accepts JSON rows as the IPC write format.
- Tables without fixed-size float vector fields continue through Arrow's JSON
  reader.
- Tables with `FixedSizeList<Float32, N>` fields use a backend manual
  JSON-to-Arrow conversion path for the schema types currently exposed by IPC
  v1.
- Unsupported Arrow types fail with `InvalidArgument` and a field-specific
  message instead of falling through to the Arrow JSON reader.
- `update_rows_v1` and `delete_rows_v1` reject empty or trivially full-table
  filters by default.
- Full-table mutation requires explicit `allowFullTable: true` in the IPC
  request.

Merge/rebase drift and repair on 2026-06-16:

- The backend guardrail and IPC types survived the merge.
- The old root `src/views/ExplorerView.vue` confirmation path did not survive
  the later Explorer split into `src/views/explorer/ExplorerView.vue` and
  `src/views/explorer/DataTab.vue`.
- `src/views/explorer/mutationGuards.ts` now owns the frontend broad-mutation
  request decision for the current split Explorer flow.
- `DataTab.vue` now prompts before empty-filter batch update and trivially broad
  update/delete filters, and only sends `allowFullTable: true` after
  confirmation.
- `LDBREV-02` can be closed again after Agenta readback records this repair.

Validation:

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `bun run test`
- `bun run build`
- 2026-06-16 drift check: the commands above still pass, but targeted
  `bunx biome ci` on touched frontend files reports formatting drift in the
  current split Explorer/DataGrid files.
- 2026-06-16 repair check: `bun run test`, `bun run build`,
  `cargo test --manifest-path src-tauri/Cargo.toml`, and targeted
  `bunx biome ci src/views/explorer/DataTab.vue
  src/views/explorer/mutationGuards.ts
  src/views/explorer/mutationGuards.test.ts` passed.

Known validation caveat:

- `bun run ci` still reports pre-existing repository formatting drift outside
  this focused fix. Use targeted Biome checks for touched frontend files until
  the wider formatting drift is handled separately.
- `src/views/explorer/mutationGuards.test.ts` covers the current request
  decision, including blank update filters, trivially broad filters, and normal
  filtered update/delete requests. It remains a pure decision-layer test rather
  than a mounted dialog interaction test because the project does not currently
  include Vue component test utilities.

## 2026-06-16 UI Repair Pass: LDBREV-08 / LDBREV-07 / LDBREV-06

Implemented decisions:

- `LDBREV-08` now owns the Search input validation and transient error lifecycle
  repair. `src/views/search/searchRequests.ts` builds Search IPC payloads only
  after local validation succeeds, so blank filter queries, blank FTS queries,
  invalid vectors, and empty combined searches return local actionable messages
  before invoking the backend.
- `src/views/SearchView.vue` delegates Search request shaping to the helper
  module and clears stale global messages when local validation fails.
- `src/composables/statusMessagePolicy.ts` makes the route-change cleanup rule
  explicit; `src/App.vue` clears transient global messages after route changes
  so a Search error banner does not leak into unrelated workspaces.
- `LDBREV-07` received a bounded UI/a11y batch: Naive UI Chinese locale/date
  locale, corrected `index.html` language/title/favicon metadata, icon-only
  DataGrid toolbar labels, column menu labels, batch dialog input labels, and a
  compact-width rule that hides the connection sidebar below the medium
  breakpoint.
- `LDBREV-06` received the Versions metadata sub-slice: version metadata now
  renders as structured key/value entries with an empty state instead of a raw
  comma-joined timeline string.

Validation:

- `bun run test`
- `bun run test:coverage` passed with the key decision files above the configured
  80% thresholds.
- `bun run build`
- Targeted touched-surface Biome CI passed:
  `bunx biome ci .gitignore biome.json package.json vitest.config.ts index.html
  src/App.vue src/components/datagrid/BatchDeleteDialog.vue
  src/components/datagrid/BatchUpdateDialog.vue
  src/components/datagrid/BatchWriteDialog.vue
  src/components/datagrid/DataGrid.vue
  src/components/datagrid/DataGridToolbar.vue src/views/SearchView.vue
  src/views/explorer/VersionsTab.vue src/views/explorer/explorerShared.ts
  src/views/explorer/versionMetadata.ts
  src/views/explorer/versionMetadata.test.ts
  src/views/search/searchRequests.ts src/views/search/searchRequests.test.ts
  src/composables/statusMessagePolicy.ts
  src/composables/statusMessagePolicy.test.ts
  src/views/explorer/mutationGuards.ts
  src/views/explorer/mutationGuards.test.ts`
- 2026-06-16 Tauri WebDriver smoke:
  - Real Tauri WebView title is `LanceDB Studio` and document language is
    `zh-CN`.
  - Empty Search filter submit shows local validation `请输入过滤表达式`.
  - Backend text `filter expression cannot be empty` does not appear.
  - Navigating from Search to Credentials clears the transient validation state.
  - Versions metadata renders with `.version-metadata-grid` /
    `.version-metadata-entry`.
  - DataGrid toolbar and column menu controls expose explicit `aria-label`
    values in the real DOM.

Remaining boundaries:

- Full `bun run ci` still fails on unrelated pre-existing formatting drift in
  files outside this repair batch, including `src/components/DataResultTable.vue`
  and `src/ipc/v1.ts`.
- `LDBREV-08` is closed in Agenta after rendered Tauri smoke evidence.
- `LDBREV-07` remains a broader polish/doc/auth-contract task; this pass only
  closes the UI inspection issues mapped to its a11y/locale/responsive subset.
- `LDBREV-06` remains broader than the Versions metadata display slice.

## LDBREV-03 Data Browsing Slice

Do not start with a large UI rewrite. The next data browsing slice should first
settle transport and paging contracts:

- Keep `scan_v1` as the first large-data boundary.
- Preserve JSON as the ergonomic default for small pages.
- Treat Arrow IPC as the scalable transport for larger pages and complex Arrow
  types.
- Add an explicit frontend capability check for whether a result page is JSON,
  Arrow IPC, or unsupported for inline rendering.
- Add generated local fixtures for large row count and wide schema before
  changing table presentation.
- Keep current `limit` / `offset` semantics until a cursor contract is designed.

Acceptance evidence for the next slice:

- Backend test proving large scans do not require a single all-row JSON payload.
- Frontend smoke or unit coverage proving JSON and Arrow result variants route
  through separate display branches.
- UI behavior that keeps wide rows inspectable without losing table context.

## LDBREV-04 Search Semantics Slice

The current "combined" search should not be described as true hybrid ranking
unless the backend exposes enough score/source/rank metadata to justify that
term.

Next decision point:

- Option A: implement LanceDB-supported hybrid/rank-fusion semantics and expose
  normalized fields in the IPC response.
- Option B: rename the behavior to an honest union/comparison mode and display
  source attribution for vector and FTS results.

Minimum contract before UI expansion:

- Search responses must expose source, rank, and score metadata when those
  fields are available.
- Duplicate handling must be deterministic and covered by backend tests.
- README and UI copy must use the same term as the backend contract.

## Later Order

- 2026-06-16 Tauri UI inspection mapping:
  - `dev_docs/exec/ui-inspection-2026-06-16.md` records the real WebView巡检结果.
    The temporary `tmp-ui-inspection-*.png` screenshots were cleaned up after
    the closeout evidence was attached to the related Agenta tasks.
  - 新增 `LDBREV-08` 处理 Search 空过滤输入直达后端、以及全局错误 banner
    跨路由残留的问题；这不是 `LDBREV-04` 的搜索排序/语义问题。
  - Naive UI 中文 locale、`index.html` 元数据、icon-only 按钮
    `aria-label`、批量弹窗输入可访问标签、640px 窄窗口策略归入
    `LDBREV-07`.
  - Versions tab 元数据 raw key/value 展示归入 `LDBREV-06`.
- Start `LDBREV-06` only after the schema/search contract is stable enough to
  decide which index controls and metadata panels are durable.
- Use `LDBREV-07` as the final polish/doc/auth-contract pass, not as a blocker
  for the write safety work above.
- Do not close the Agenta version until the remaining tasks are either complete
  or intentionally moved into a newer active lane.
