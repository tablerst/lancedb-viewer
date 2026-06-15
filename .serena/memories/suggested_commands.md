# Suggested Commands

- Install: `bun install`
- Frontend dev: `bun dev`
- Desktop dev: `bun tauri dev`
- Frontend tests: `bun run test`
- Frontend build/typecheck: `bun build`
- Biome CI/no-write check: `bun ci`
- Biome write checks: `bun lint` or `bun check`
- Rust build: `cargo build --manifest-path src-tauri/Cargo.toml`
- Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml`
- Single Rust test: `cargo test <test_name> --manifest-path src-tauri/Cargo.toml`
- Seed sample DB: `cargo run --manifest-path src-tauri/Cargo.toml --bin seed_db -- sample-db`
- Windows search/read preferred: `rg`, `rg --files`, `Get-Content -Raw <path>`.