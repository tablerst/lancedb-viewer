# Tech Stack

- Frontend: Vue 3.5, Vue Router 4, TypeScript strict, Vite 6, Naive UI, Tailwind CSS 3, lucide-vue-next.
- Desktop: Tauri v2 with plugins opener/store/log/dialog/stronghold.
- Backend: Rust 2021, lancedb 0.23.1, Arrow 56 crates, serde/serde_json, futures-util, uuid, sha2.
- Package manager: Bun; `bun.lock` exists. Do not use npm/pnpm/yarn for dependency install.
- Testing: Vitest for frontend; Cargo unit/integration tests for Rust under `src-tauri`.