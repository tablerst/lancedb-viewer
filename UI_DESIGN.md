# UI_DESIGN.md

This document is the active UI/UX baseline for LanceDB Studio. It describes the
current application shell, interaction contracts, and validation expectations.

## Product Shape

The app is an operational desktop studio, not a marketing surface. The first
screen should help users connect to a database, inspect tables, browse rows,
search, and manage table metadata.

Primary layout:

- Dark global primary navigation.
- Collapsible connection sidebar.
- Right workspace with route-specific tools.

The right workspace should remain stable while users expand, collapse, connect,
refresh, or select tables in the sidebar.

The first viewport should feel like a working surface. Avoid large centered
hero-style empty regions. When no connection or table is selected, the workspace
must still offer the next useful action instead of only explaining what is
missing.

## Navigation And Workspace

- Explorer owns table inspection and management.
- Search owns filter, vector, full-text, and hybrid retrieval.
- Credentials is the visible auth-management surface, but full `secret_ref`
  connection support is disabled for this version.
- Breadcrumb and active table controls should keep the selected connection/table
  clear without nesting multiple layers of tabs.
- Primary navigation is a global rail. It should remain visually distinct from
  the connection sidebar in both light and dark themes, expose accessible labels
  for icon/text buttons, and keep active route state obvious at a glance.
- The connection sidebar is context, not the main content. Expanded cards should
  show connection state and available tables clearly; collapsed state should
  preserve recognizable connection/table context without cryptic abbreviations.
- Sidebar table selection is contextual:
  - from Explorer, selecting a table opens the table workspace;
  - from Search, selecting a table should keep the user in the search workflow
    unless they explicitly choose an Explorer action.
- Route-level empty states should include a primary action when one exists:
  connect current profile, create a connection, pick the only available table,
  or open the relevant configuration page.
- Use Chinese primary labels for peer UI controls. Keep English for API names,
  protocol names, command names, route fragments, and transport labels when that
  avoids ambiguity. Do not mix one English tab label with Chinese sibling tabs
  unless the English label is itself the product term.

Explorer table routes:

```text
/connections/:id/table/:name/schema
/connections/:id/table/:name/data
/connections/:id/table/:name/indexes
/connections/:id/table/:name/versions
```

## Explorer Data Browsing

`DataTab` is Arrow-first:

- Request `scan_v1({ format: "arrow" })` for normal browsing.
- Decode `DataChunk.format === "arrow"` with `apache-arrow`.
- Convert decoded Arrow data into the same row/schema shape consumed by
  `DataGrid`.
- Fall back to `scan_v1({ format: "json" })` when Arrow decoding or transport
  compatibility fails.
- Show the active transport state: Arrow IPC, JSON, or JSON fallback.
- Preserve `limit`, `offset`, and `nextOffset`. Cursor pagination is not part of
  this version.

Complex Arrow values should normalize into displayable values instead of forcing
the user into a JSON-only path.

`DataGrid` is the highest-value inspection surface and should read like a dense
desktop data tool:

- Keep table content, toolbar, and pagination in one stable framed work area.
- Keep the active transport state visible but secondary.
- Group toolbar actions by job: refresh/filter, row editing, batch operations,
  import/export. High-frequency actions may be icon buttons, but destructive or
  batch actions need clear labels, grouping, or menu placement.
- Icon-only controls need stable `aria-label` and hover/focus tooltips. Keyboard
  shortcuts belong in tooltips or command help, not as always-visible clutter.
- Long vector, struct, JSON, and text values should be inspectable without
  destroying row density. Prefer truncation plus cell expansion/detail affordance
  over wrapping large content into the grid.
- Numeric, vector, score, date, and count columns should use tabular numerals
  where alignment helps comparison.

## Search Workspace

Search has four modes:

- Filter query.
- Vector search.
- Full-text search.
- Hybrid search.

Hybrid search replaces the older "combined query" wording. It calls
`combined_search_v1` and requires both query text and vector input. The backend
uses LanceDB hybrid query + `RRFReranker`; the UI should display available
metadata fields:

- `_relevance_score`: fused/reranked score.
- `_distance`: vector distance when emitted.
- `_score`: full-text score when emitted.
- `_hybrid_rank`: frontend/backend rank annotation.
- `_hybrid_source`: source label for the hybrid path.

Missing score columns are rendered as empty cells, not as errors.

Search paging uses actionable previous/next controls while preserving offset
semantics internally. Do not expose a raw `nextOffset` string as the primary UI.

Search table context:

- If exactly one connected table is available, Search may preselect it or offer
  a prominent one-click selection.
- Quick table selection should update the Search workspace context; it should
  not unexpectedly move the user to the Explorer schema tab.
- Search empty states should make the required next input explicit: connection,
  table, query text, vector input, or index/field selection.
- Search result tables should preserve score metadata columns when available,
  but missing score columns stay empty rather than becoming warnings.

## Management Tabs

Schema, Versions, and Indexes should let users understand table state without
leaving the app.

Schema:

- Show field name, type, nullability, vector dimension where available, and
  metadata.

Versions:

- Show version number, timestamp, and structured metadata entries.
- Avoid raw comma-joined metadata strings.

Indexes:

- Show index name, type, columns, status, distance type, partition/index count,
  and loss when available.
- Create index controls expose durable high-value parameters:
  - vector column or indexed columns
  - FTS columns
  - index type
  - replace/name
  - distance type
  - IVF partitions, sample rate, max iterations, target partition size
  - PQ/RQ sub-vectors and bits
  - HNSW edges and EF construction
- Do not expose every LanceDB low-level knob until the UI has a clear need.

## Auth UX

Supported connection auth modes:

- `none`
- `inline`

`secret_ref` must show a friendly unsupported/not-implemented message in this
version. The user-facing copy should make clear that the schema reserves
`secret_ref`, but current connection support is limited to `none` and `inline`.

Connection forms should keep the common path short:

- Name, URI, folder picker, and auth mode are primary.
- `storageOptions` JSON and low-level auth descriptor fields are advanced
  controls behind clear disclosure.
- Disclosures should not look disabled when they are interactive.
- Folder picker copy should explain that the database root directory is expected
  and that accidental `*.lance` table-directory picks are normalized upward.

## Visual System And Theming

The app should use one coherent token ladder across Tailwind utility styling,
Naive UI theme overrides, and custom component CSS.

- The visual target is a Linear-style operational workbench: neutral graphite
  surfaces, restrained borders, short radii, minimal shadows, dense typography,
  and one low-saturation accent. Avoid glossy gradients, cyan-heavy controls,
  oversized pills, and card stacks that make routine UI feel decorative.
- Interactive controls should share one grammar:
  - default: transparent or very soft fill with a 1px rule;
  - hover: subtle surface lift or fill change, not saturated color;
  - focus: visible accent ring;
  - active/selected: soft accent wash plus crisp text, not glow;
  - disabled: lower contrast while keeping shape stable.
- Inputs and editable fields should read as precise workbench controls: stable
  height, 1px border, quiet background, clear focus ring, and no heavy shadows.
- Light and dark themes must both cover global app surface, primary navigation,
  connection sidebar, right workspace, dialogs, data tables, popovers, empty
  states, and status/toast surfaces.
- Custom components should reference shared tokens or theme-aware classes rather
  than hard-coded light colors.
- Dark mode is not partial inversion. If a screen enters dark mode, text,
  borders, table rows, empty states, disabled controls, and hover/focus states
  must remain readable and visually intentional.
- The primary navigation may stay dark in both themes, but the selected route,
  inactive routes, and theme toggle must still pass contrast and focus checks.
- Use accent color sparingly for route state, primary actions, selected table,
  active transport, and focused controls. Avoid turning every panel edge or icon
  into an accent.
- Status colors are semantic, not decorative. Connected/dirty/warning/danger
  states should be visible but secondary to the structure and labels.
- Toasts/status messages are secondary feedback. They should not cover the page
  title, tabs, or first actionable controls, especially in narrow desktop
  windows.

## Visual And Interaction Rules

- Prefer Naive UI for complex controls and Tailwind for layout and light
  styling.
- Use `lucide-vue-next` for icon buttons where an icon exists.
- Keep data-heavy screens dense, quiet, and scan-friendly.
- Avoid nested cards. Cards are for individual panels, repeated items, and
  dialogs, not for wrapping whole page sections inside other cards.
- Keep data table motion minimal. Do not add decorative animations to grid
  interactions.
- User-visible errors should be near the operation that caused them and should
  be actionable.
- Success state can use the shared status/toast path, but visible in-place state
  changes are preferred when the result is already obvious.
- Text must fit in its parent at mobile and desktop widths.
- Design for the Tauri default window size and for narrower desktop windows.
  The app does not need to become a phone-first mobile app, but at about
  390-480 px width it should not overlap text, hide critical context, or let
  toasts obscure navigation.
- Avoid hover-only critical controls. Hover affordances must also be reachable
  through focus or visible secondary actions.
- Dangerous actions should be grouped away from routine actions and should use
  clear labels.
- Do not add decorative animation to operational surfaces. Motion should clarify
  state changes, not decorate data tables.

## Validation Checklist

For UI/data changes:

- `bun run test`
- `bun run build`
- Targeted `bunx biome ci <touched files...>`

For cross-boundary IPC changes:

- `bun run build`
- `cargo test --manifest-path src-tauri/Cargo.toml`

For native desktop behavior:

- Use a real Tauri smoke test when filesystem dialogs, plugins, WebView-specific
  behavior, or screenshots are part of the acceptance evidence.

For visual or UX shell changes, capture real Tauri screenshots at minimum for:

- initial no-connection or disconnected workspace;
- connected table workspace in Explorer;
- Search with no selected table and with a selected table;
- DataGrid in light and dark themes;
- Credentials or connection form in light and dark themes;
- collapsed sidebar;
- a narrow desktop window around 390-480 px wide.

Screenshots should be treated as evidence for hierarchy, contrast, clipping,
toast placement, and route context, not only as smoke-test artifacts.
