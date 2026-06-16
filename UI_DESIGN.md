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

## Navigation And Workspace

- Explorer owns table inspection and management.
- Search owns filter, vector, full-text, and hybrid retrieval.
- Credentials is the visible auth-management surface, but full `secret_ref`
  connection support is disabled for this version.
- Breadcrumb and active table controls should keep the selected connection/table
  clear without nesting multiple layers of tabs.

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
- Success state can use the shared status/toast path.
- Text must fit in its parent at mobile and desktop widths.

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
