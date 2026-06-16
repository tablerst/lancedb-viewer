# LanceDB Viewer / LanceDB Studio

LanceDB Viewer is a Tauri v2 desktop app for inspecting and operating LanceDB
databases. The frontend is Vue 3 + TypeScript, and the backend is Rust with
LanceDB 0.23.1, Arrow, and Tauri IPC v1.

The current product shape is a work-focused database studio: a dark global
navigation rail, a collapsible connection sidebar, and a right workspace for
Explorer, Search, Credentials, and related management views.

## Current Capabilities

- Connection profiles with `name`, `uri`, `storageOptions`, consistency options,
  and an auth descriptor.
- LanceDB connection lifecycle through `connect_v1` / `disconnect_v1`.
- Table discovery, open table, schema inspection, data scan, write, update,
  delete, import, export, optimize, and version operations.
- Explorer data browsing through `scan_v1`.
  - Default UI path requests `format: "arrow"`.
  - Frontend decodes Arrow IPC with `apache-arrow`.
  - JSON fallback is used when Arrow decoding or compatibility fails.
  - Paging remains `limit` / `offset` / `nextOffset`.
- Search workspace for filter, vector, full-text, and hybrid search.
  - `combined_search_v1` is a true hybrid path: vector query + FTS query +
    LanceDB `RRFReranker` with rank normalization.
  - Hybrid search requires both query text and vector input.
  - Result metadata may include `_relevance_score`, `_distance`, `_score`,
    `_hybrid_rank`, and `_hybrid_source`; UI displays blank cells when LanceDB
    does not emit a score column for a specific result path.
- Schema, Versions, and Indexes tabs for table management.
  - Index listing displays index stats when LanceDB reports them.
  - Index creation exposes key controls: columns, index type, name, replace,
    distance type, IVF parameters, PQ/RQ parameters, and HNSW parameters.
- Mutation guardrails for broad update/delete requests. Full-table mutation
  requires explicit confirmation and `allowFullTable: true`.

## Auth Contract

Supported auth modes in this version:

- `none`
- `inline`

`secret_ref` is present in the IPC/profile schema for forward compatibility, but
it is intentionally disabled in this version. The frontend returns a friendly
`not_implemented` envelope before invoking IPC, and the backend also rejects
`secret_ref` with `NotImplemented`. Full Stronghold-backed `secret_ref` connect
support is a later version item.

## IPC v1 Notes

- All commands return `ResultEnvelope<T>` with `apiVersion`, `ok`, `data`, and
  optional `error`.
- `DataChunk` supports `format: "json"` and `format: "arrow"`.
- `scan_v1` is the Arrow-first large-data boundary.
- Search results currently return JSON chunks.
- IPC payloads are additive where possible so frontend and backend can evolve
  without breaking existing callers.

## Development

Install dependencies:

```bash
bun install
```

Run the frontend dev server:

```bash
bun run dev
```

Run the desktop app:

```bash
bun tauri dev
```

Run frontend tests:

```bash
bun run test
bun run test:coverage
```

Build frontend assets:

```bash
bun run build
```

Run Rust tests:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Generate a sample database:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin seed_db -- sample-db
```

Optional sample DB flags include `--table <name>` and `--rows <count>`.

## Validation Guidance

- Use `bun run test` for frontend utilities, composables, and request builders.
- Use `bun run build` when Vue components, IPC types, or TypeScript contracts
  changed.
- Use targeted `bunx biome ci <files...>` for touched files while the repository
  still has unrelated formatting drift.
- Use `cargo test --manifest-path src-tauri/Cargo.toml` for backend and IPC
  command changes.

## Reference Docs

- `UI_DESIGN.md`: current UI architecture and interaction contracts.
- `dev_docs/exec/review-stabilization-2026-06-15.md`: active stabilization
  execution plan and closeout evidence.
- LanceDB docs: https://docs.lancedb.com/
- Tauri docs: https://v2.tauri.app/
