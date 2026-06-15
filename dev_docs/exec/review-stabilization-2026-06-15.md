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
  request. The Explorer UI only sends that opt-in after user confirmation.

Validation:

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `bun run test`
- `bun run build`

Known validation caveat:

- `bun run ci` still reports pre-existing repository formatting drift outside
  this focused fix. Use targeted Biome checks for touched frontend files until
  the wider formatting drift is handled separately.

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

- Start `LDBREV-06` only after the schema/search contract is stable enough to
  decide which index controls and metadata panels are durable.
- Use `LDBREV-07` as the final polish/doc/auth-contract pass, not as a blocker
  for the write safety work above.
- Do not close the Agenta version until the remaining tasks are either complete
  or intentionally moved into a newer active lane.
