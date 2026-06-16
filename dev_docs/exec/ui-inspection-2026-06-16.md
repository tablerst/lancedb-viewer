# Tauri UI Inspection 2026-06-16

Scope: real Tauri WebView inspection through `.agents/skills/tauri-webdriver`.

Environment:

- Executable: `src-tauri/target/debug/lancedb-viewer.exe`
- Dev URL: `http://localhost:1420`
- Driver: `tauri-driver` + `msedgedriver`
- Existing profile used: `111`, local LanceDB path `D:\PyCharm\novel-summarizer\data\lancedb`
- Table used for read-only inspection: `chunks_vectors_1`
- No destructive action was submitted. Batch update dialog was opened only, then cancelled.

Screenshots:

- Initial local `tmp-ui-inspection-*.png` screenshots were used as scratch
  inspection evidence and then cleaned up.
- Closeout screenshots for the fixed paths were attached to the related Agenta
  tasks:
  - `LDBREV-08`: empty Search filter local validation and route-change cleanup.
  - `LDBREV-07`: DataGrid toolbar and column menu accessible labels.
  - `LDBREV-06`: Versions metadata structured key/value display.

## Verified Paths

- App shell loads in the real Tauri window.
- Existing local profile connects successfully.
- Table list loads and `chunks_vectors_1` opens.
- Schema, Data, Indexes, Versions, Search, and Credentials routes all render.
- DataTab loads real rows and the grid has internal scrolling.
- Batch update modal opens in the real WebView and can be cancelled.
- 800px and 1200px widths did not produce document-level horizontal overflow.

## Findings

### P1: Search empty filter can trigger a global backend error

Repro:

1. Open Search.
2. Stay on "过滤查询".
3. Leave filter expression empty.
4. Click "查询".

Observed:

- Backend error appears: `filter expression cannot be empty`.
- The query button is enabled even though the required filter is empty.

Suggested fix:

- In `src/views/SearchView.vue`, validate filter query input before calling `queryFilterV1`.
- Disable "查询" for filter mode when the expression is blank, or show an inline form validation message.

### P1: Global error banner leaks across route changes

Repro:

1. Trigger the empty search error above.
2. Navigate to "凭证库" or back to "资源".

Observed:

- The red global banner with `filter expression cannot be empty` remains visible on unrelated pages.
- It only disappeared later after table/tab interactions cleared messages.

Suggested fix:

- Clear transient global status/error messages on route change in `src/App.vue`, or require each top-level route to clear stale messages on mount.
- Keep persistent operation results only when explicitly tied to the current route/context.

### P2: Naive UI locale is not configured

Observed examples:

- Indexes empty table shows `No Data`.
- Select controls show `Please Select`.
- Input placeholders show `Please Input`.

Likely source:

- `src/App.vue` uses `NConfigProvider` without `locale` / `date-locale`.

Suggested fix:

- Import `zhCN` and `dateZhCN` from `naive-ui`.
- Pass them to `NConfigProvider`.

### P2: App metadata is still scaffold default

Observed:

- `document.title` is `Tauri + Vue + Typescript App`.
- `html lang` is `en`.
- `index.html` still references `/vite.svg`.

Suggested fix:

- Update `index.html` to the product name, proper language, and real app icon metadata.

### P2: Icon-only DataGrid buttons lack accessible names

Observed:

- DataGrid toolbar buttons have `title` but no `aria-label`.
- WebDriver sees empty button text for refresh, filter, add row, save, batch write/update/delete, export.
- Column menu trigger also has `title="列菜单"` but no `aria-label`.

Likely files:

- `src/components/datagrid/DataGridToolbar.vue`
- `src/components/datagrid/DataGrid.vue`
- `src/components/datagrid/DataGridFilterRow.vue`

Suggested fix:

- Mirror `title` into `aria-label` for icon-only buttons.
- Prefer stable labels that can also support webdriver selectors.

### P2: Batch dialog inputs need accessible labels

Observed:

- Batch update modal inputs are visually labeled or placeholder-only, but DOM inputs have no `aria-label`.
- Close button has generic `aria="close"` from Naive UI, which is acceptable but not localized.

Likely files:

- `src/components/datagrid/BatchUpdateDialog.vue`
- `src/components/datagrid/BatchDeleteDialog.vue`
- `src/components/datagrid/BatchWriteDialog.vue`

Suggested fix:

- Add explicit labels or `aria-label` for filter, column, expression, JSON rows, and mode fields.

### P2: 640px width is technically non-overflowing but not practically usable

Observed:

- At 640px window width, document-level horizontal overflow stays at `scrollW=640`.
- The fixed primary nav + connection sidebar leave the main work area around 222px wide.
- Data tab labels and grid headers collapse into short fragments; table browsing is barely usable.

Likely files:

- `src/App.vue`
- `src/components/sidebar/Sidebar.vue`

Suggested fix:

- Define a real compact breakpoint:
  - auto-collapse the connection sidebar below a threshold,
  - or turn it into an overlay/drawer,
  - or enforce a desktop minimum window size if mobile/narrow support is out of scope.

### P3: Versions metadata is raw and hard to scan

Observed:

- Version list renders long raw key/value metadata strings.
- It is usable, but visually dense and hard to compare.

Likely file:

- `src/views/explorer/VersionsTab.vue`

Suggested fix:

- Render key metrics as compact chips or a small key/value grid.
- Keep raw metadata behind an expand/details affordance.

## Suggested Repair Order

1. Fix Search empty-filter validation and route-level stale message cleanup.
2. Add Naive UI Chinese locale and fix `index.html` metadata.
3. Add `aria-label` coverage for DataGrid toolbar, column menu, and batch dialogs.
4. Decide responsive policy for sub-800px desktop windows.
5. Polish Versions metadata rendering.
