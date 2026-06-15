# Task Completion

- Frontend code changes: run `bun lint` or `bun check`; run `bun build` when types/routes/components changed.
- Frontend pure utility changes with tests: run `bun run test` plus lint/build as risk warrants.
- Rust backend/IPC changes: run `cargo test --manifest-path src-tauri/Cargo.toml`; run `cargo build --manifest-path src-tauri/Cargo.toml` when tests are not sufficient or build wiring changed.
- Cross-boundary IPC changes: update both `src/ipc/v1.ts` and `src-tauri/src/ipc/v1.rs`, then run frontend build and Rust tests.
- UI/desktop integration changes: prefer `bun dev` for browser-only feedback; use `bun tauri dev` or tauri-driver validation when desktop shell behavior matters.
- Use `serena memories check` from project root after memory maintenance if needed.