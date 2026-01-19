# AGENTS

This repository is a Tauri + Vue 3 + TypeScript desktop app with a Rust backend.

## Stack
- Frontend: Vue 3, Vite, TypeScript, Biome
- Desktop: Tauri v2 (Rust backend)
- Rust deps: tauri, lancedb, serde, log
- Package manager: use existing lockfile (`bun.lock`)

## Repository layout
- `src/` Vue app
- `src-tauri/` Tauri Rust backend
- `public/` static assets
- `biome.json` formatting/linting config
- `vite.config.ts` Vite/Tauri dev server config

## Commands
Use the project root unless noted.

### Install
- `bun install` (preferred; lockfile present)
- or `npm install` if you must, but keep lockfiles stable

### Frontend (Vite)
- `npm run dev` start Vite dev server
- `npm run build` typecheck + build (runs `vue-tsc --noEmit`)
- `npm run preview` preview production build

### Tauri (desktop)
- `npm run tauri dev` run desktop app in dev mode
- `npm run tauri build` build desktop app

### Lint/format (Biome)
- `npm run format` format all supported files (writes)
- `npm run lint` lint + auto-fix (writes)
- `npm run check` format + lint (writes)
- `npm run ci` CI check (no writes)

### Rust backend (`src-tauri`)
- `cargo build --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- Single test: `cargo test <test_name> --manifest-path src-tauri/Cargo.toml`

## Code style
### General
- Prefer small, focused commits and functions.
- Keep user-visible errors friendly and actionable.
- Avoid new dependencies without a good reason.

### TypeScript / Vue
- Use `<script setup lang="ts">` in SFCs.
- Prefer Composition API (`ref`, `computed`, `watchEffect`) over Options API.
- Keep state minimal; derive when possible.
- Avoid `any`; use `unknown` and narrow.
- `tsconfig.json` is `strict: true` — keep types explicit.
- Prefer `const` over `let` unless reassignment is required.
- Prefer template strings over string concatenation.
- Import order: external packages, then internal modules, then relative files.
- Prefer named exports for utilities; default exports for Vue components.
- Components: PascalCase file names, kebab-case for DOM components in templates.
- Keep templates readable; avoid deeply nested markup.
- When calling Tauri, use `invoke` from `@tauri-apps/api/core`.
- Always handle `invoke` errors (`try/catch`) and surface them to UI.
- For async work, prefer `await` over `.then()` chains.

### Formatting (Biome)
- Indentation: tabs.
- Line width: 100.
- Quotes: double.
- Semicolons: as needed.
- Trailing commas: ES5.
- Imports are auto-organized; do not disable.
- `src-tauri/` is excluded from Biome.

### Rust (`src-tauri`)
- Use rustfmt defaults (4-space indentation).
- Naming: `snake_case` for functions/modules, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for consts.
- Prefer `Result<T, E>` and `?` for error propagation.
- Avoid `unwrap()`/`expect()` outside of `main`/startup.
- Log with the `log` crate macros (`info!`, `warn!`, `error!`).
- Use `serde` for IPC types passed between frontend and backend.
- Keep Tauri commands thin; move logic into helpers/services.
- Tauri commands should return `Result<T, String>` or a serializable error.
- Prefer non-blocking work for long tasks; consider background threads/events.

## Testing guidance
- No JS test runner configured yet; add one only if needed.
- Rust tests live in `src-tauri/` with standard `cargo test` flow.
- Keep unit tests deterministic; avoid network access by default.

## Build notes
- Vite dev server runs on port 1420; HMR on 1421 in Tauri dev.
- Tauri build uses the Vite build output.

## Security & secrets
- Do not hardcode secrets in `src/` or `src-tauri/`.
- Prefer OS keychain/secure storage for credentials if added later.

## Rules from tooling
- No Cursor rules (`.cursor/rules/`) or `.cursorrules` found.
- No `.github/copilot-instructions.md` found.

## Suggested workflow
1. `bun install`
2. `npm run dev` for frontend-only changes.
3. `npm run tauri dev` for desktop integration.
4. `npm run lint` before committing.

## When editing
- Keep changes scoped; avoid reformatting unrelated files.
- Update types when adding new fields or IPC payloads.
- If you add new commands, document them and keep names stable.
- If you add new env vars, document them in README and sample files.

## File templates
- Vue component: `<script setup lang="ts">` + `<template>` + optional `<style scoped>`.
- Rust command: `#[tauri::command] async fn ... -> Result<_, String>`.

## Notes
- This repo uses ES modules (`"type": "module"`).
- Keep `node_modules/` out of patches.
- Keep the lockfile consistent with the package manager used.

## UI conventions
- Keep layout responsive; use CSS classes and avoid inline styles.
- Use semantic HTML elements.
- Keep forms accessible with labels and ARIA when needed.
- Prefer small reusable components for repeated UI.
- Keep assets in `src/assets/` or `public/` as appropriate.

## Imports and paths
- Prefer relative imports within `src/` unless aliases are added.
- Avoid circular dependencies between components.
- Group related utilities in dedicated modules.

## IPC/data handling
- Validate inputs on both frontend and backend.
- Avoid sending large blobs via JSON; prefer streaming/structured formats if needed.
- Keep payloads versionable (additive changes when possible).

## Rust safety
- Prefer explicit lifetimes only when required; keep signatures simple.
- Use `clippy` recommendations when available.
- Treat `panic!` as a bug; replace with errors.

## Housekeeping
- Update `README.md` when adding new setup steps.
- Keep `PLAN.md` as design doc; update when scope changes.
- Keep files encoded in UTF-8.
