# Conventions

- Follow `AGENTS.md`: small scoped changes, user-visible errors should be friendly/actionable, avoid new deps without reason.
- Vue SFCs use `<script setup lang="ts">`; Composition API; avoid `any`; keep state minimal and derive with `computed` where possible.
- Frontend imports: external, internal modules, relative files; relative imports preferred inside `src` unless aliases are introduced.
- Biome config controls TS/Vue formatting: tabs, 100 columns, double quotes, semicolons as needed; `src-tauri/` excluded.
- Tauri frontend calls use `invoke` from `@tauri-apps/api/core`; handle errors and surface them to UI.
- Rust commands return serializable envelopes/errors; keep command wrappers thin and put logic in services.
- Rust style: rustfmt defaults, `Result` + `?`, avoid `unwrap`/`expect` outside startup/tests, log via `log` macros.
- UX direction lives in `DESIGN.md`: left collapsible sidebar + right workspace; Naive UI + Tailwind + Lucide; status/error feedback consolidated in the right-side connection/status area where applicable.