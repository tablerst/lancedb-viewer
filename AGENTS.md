# AGENTS

This repository is **LanceDB Viewer / LanceDB Studio**, a Tauri v2 desktop app
with a Vue 3 + TypeScript frontend and a Rust backend.

Use this file as the operating guide for agents working in this repo. Keep
changes scoped, validate the touched surface, and prefer current implementation
over stale plans.

## Project Shape

- Frontend: Vue 3.5, Vue Router, TypeScript strict, Vite 6, Naive UI, Tailwind
  CSS, `lucide-vue-next`.
- Desktop/backend: Tauri v2, Rust 2021, LanceDB, Arrow, serde, log.
- Package manager: Bun. Use the existing `bun.lock`; do not install with npm,
  pnpm, or yarn.
- App direction: see `README.md` for current capability and `UI_DESIGN.md` for UI
  information architecture, interaction rules, and visual conventions.

## Repository Layout

- `src/`: Vue application, routes, components, composables, IPC client code.
- `src-tauri/`: Tauri Rust backend, IPC command handlers, services, tests, and
  helper binaries.
- `public/`: static assets served by Vite.
- `.agents/skills/`: repository-local agent skills and references.
- `.agenta/`: local Agenta recovery ledger configuration.
- `.serena/`: Serena project configuration and local memories.
- `README.md`: product status, dev commands, IPC overview, roadmap.
- `UI_DESIGN.md`: current UI/UX baseline and checklist.
- `biome.json`, `vitest.config.ts`, `vite.config.ts`: frontend tooling.

Keep `node_modules/`, `dist/`, `target/`, generated outputs, local caches, and
personal IDE files out of patches unless the task explicitly targets them.

## Commands

Run commands from the project root unless noted.

### Install

- `bun install`

### Frontend

- `bun run dev`: start Vite dev server.
- `bun run build`: run `vue-tsc --noEmit` and build with Vite.
- `bun run preview`: preview the production build.
- `bun run test`: run Vitest. Use this form to avoid confusion with Bun's own
  `bun test` behavior.

### Tauri

- `bun tauri dev`: run the desktop app in development.
- `bun tauri build`: build the desktop app.

### Biome

- `bun run ci`: read-only CI check. Prefer this when you need diagnostics without
  modifying files.
- `bun run lint`: lint and apply safe writes.
- `bun run check`: format/lint and apply safe writes.
- `bun run format`: format supported files and write changes.

### Rust

- `cargo build --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo test <test_name> --manifest-path src-tauri/Cargo.toml`
- `cargo run --manifest-path src-tauri/Cargo.toml --bin seed_db -- sample-db`

## Static Analysis Guidelines

- Treat static analysis as evidence for the touched surface, not as a reason to
  churn unrelated files.
- Prefer no-write checks first when you only need diagnostics:
  - Frontend formatting/lint: `bun run ci`
  - Frontend type/build boundary: `bun run build`
  - Rust compile boundary: `cargo build --manifest-path src-tauri/Cargo.toml`
  - Rust tests/compile diagnostics: `cargo test --manifest-path src-tauri/Cargo.toml`
- Use write-capable Biome commands (`bun run lint`, `bun run check`, `bun run format`) only
  when you intend to accept the resulting edits. Inspect the diff afterward and
  keep unrelated formatting churn out of the final patch.
- Do not suppress diagnostics with broad rewrites that weaken runtime or IPC
  contracts. Fix the underlying type, data, or API issue whenever possible.
- TypeScript is strict. Avoid `any`; use `unknown` at external boundaries and
  narrow before use.
- Rust should stay `rustfmt`-compatible and warning-conscious. Avoid
  `unwrap()`/`expect()` outside startup code and tests.

## Testing Guidelines

- Docs-only changes usually do not need test execution; mention that validation
  was limited to document review.
- Frontend behavior or component changes:
  - Run `bun run ci` or `bun run lint` for Biome.
  - Run `bun run build` when Vue components, routes, IPC types, or TypeScript
    contracts changed.
  - Run `bun run test` when utilities, composables, data transforms, or existing
    test-covered behavior changed.
  - For visible UI, layout, responsive, interaction, or desktop-rendering
    changes, prefer the repository `tauri-webdriver` skill for real Tauri
    WebView inspection before relying on browser-only screenshots.
- Rust backend or IPC command changes:
  - Run `cargo test --manifest-path src-tauri/Cargo.toml`.
  - Run `cargo build --manifest-path src-tauri/Cargo.toml` when build wiring,
    features, binaries, or compile-only surfaces changed.
- Cross-boundary IPC changes:
  - Update frontend and backend IPC types together.
  - Run `bun run build` and `cargo test --manifest-path src-tauri/Cargo.toml`.
- Desktop shell or WebView behavior:
  - Use `bun run dev` only for quick browser-only diagnostics.
  - Use `bun tauri dev` or the repository `tauri-webdriver` skill for real
    frontend effect checks and debugging whenever native shell behavior,
    WebView2 rendering, responsive sizing, plugins, dialogs, filesystem access,
    screenshots, or desktop sign-off matter.
  - When the user already has a Tauri process running, do not stop it. Prefer a
    separate WebDriver-controlled session, or attach/reuse existing WebDriver
    state when available.
  - Check at least one constrained/narrow width and one normal desktop width for
    layout-sensitive changes, and include DOM/layout evidence such as no
    horizontal overflow when that was the issue.
- Keep tests deterministic. Prefer temp directories, sample DBs, fakes, and
  local fixtures over network or shared external resources.

## Dev Docs Usage

- `README.md` is the product and development entry point. Update it when setup,
  capabilities, IPC behavior, roadmap, or security expectations change.
- `UI_DESIGN.md` is the active UI/UX baseline. Update it when layout, state model,
  visual conventions, interaction patterns, or validation checklists change.
- If `dev_docs/` is introduced or used, treat it as the project working-doc
  layer:
  - `dev_docs/exec/`: active design docs, execution plans, acceptance notes, and
    current tracking.
  - `dev_docs/archive/`: historical context only; do not treat archive notes as
    the default source of truth.
- Prefer updating an existing primary guide before creating parallel docs.
- Keep docs organized by module/topic, for example `ui`, `ipc`, `backend`,
  `storage`, `desktop`, or `testing`.
- Chinese prose is acceptable and preferred for local design/tracking docs when
  it improves clarity for the maintainers. Keep API names, command names, file
  paths, type names, error codes, log keys, and external protocol terms in
  English when that avoids ambiguity.
- When archiving or moving docs, update nearby indexes and internal links in the
  same change.

## Agenta Usage

- Agenta is a recovery and closeout ledger, not the default implementation task
  tracker.
- When Agenta work is relevant, invoke the repository skill
  `.agents/skills/agenta-workflow/SKILL.md` and follow its referenced workflow.
- Use `.agenta/project.yaml` as the current project/recovery entry source before
  creating or changing projects, versions, tasks, or context manifests.
- Keep detailed implementation plans and task decomposition in current docs
  (`README.md`, `UI_DESIGN.md`, or `dev_docs/exec/` when present). Agenta should
  record lane pointers, reusable findings, validation evidence, risks, and
  closeout state.
- Prefer Agenta MCP tools when available and the user has not requested CLI
  mode.
- Do not create or update Agenta versions/tasks just because a normal code task
  exists. First pass the skill's minimalism gate.
- After any Agenta write, read back the affected project, version, task, note,
  attachment, or context manifest before reporting completion.

## Repo Memory Governance

- Treat `.agents/memory/repo/` as the repository's Git-backed institutional
  memory layer if/when it is present. It is for distilled, durable repo
  knowledge, not scratch notes, chat transcripts, or temporary debugging logs.
- Treat `/memories/repo/` as an external imported cache or explicit sync target
  only. If both locations exist, the repository-tracked `.agents/memory/repo/`
  copy is the source of truth.
- Current code, tests, `README.md`, and `UI_DESIGN.md` are authoritative. If repo
  memory conflicts with live implementation, update or retire the memory rather
  than bending code to stale notes.
- Do not opportunistically rewrite repo memory during ordinary implementation.
  If drift is noticed, mention it briefly and wait for an explicit memory
  maintenance request.
- For explicit repo-memory cleanup, migration, governance, or index work, read
  and follow `.agents/skills/repo-memory-curator/SKILL.md`.
- Keep memory entries schema-aligned with that skill; do not invent ad-hoc
  labels or naming patterns.

## Serena Usage

- If Serena tools are available, call `serena.activate_project` once for this
  project before substantive work unless it is already active.
- Read the Serena Instructions Manual once per context before using Serena for
  project work.
- Prefer Serena symbolic tools for code exploration and edits when they fit the
  task. Use normal shell reads for non-code docs and configs.
- Use Serena memories when they are likely to contain relevant repo conventions,
  command guidance, prior architecture decisions, or task completion rules.
- Keep Serena memories distinct from repo memory: Serena memories are local
  agent working context; `.agents/memory/repo/` is durable repo knowledge.

## SubAgent Usage

- Treat requests to use or parallelize with SubAgents, including vague wording
  like "结合 SubAgent 并行推进", as permission to split execution work when it is
  safe and useful.
- Before delegating substantial work, identify the critical path and independent
  workstreams. Keep urgent, tightly coupled, or high-risk edits local.
- Delegate bounded implementation, exploration, or verification slices with
  clear ownership and non-overlapping write scopes.
- Prefer implementation workers for well-scoped paths/modules and explorer
  workers when the boundary is unclear or discovery is the main value.
- Prompts to SubAgents should state owned paths, expected output, relevant
  assumptions, validation commands, and whether edits are allowed.
- Reuse an existing SubAgent only when the follow-up stays within the same
  bounded context; otherwise spawn a new narrowly scoped one.

## Coding Style

### General

- Commit messages should use Conventional Commits style:
  `feat(scope): concise English summary`.
- Common types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `build`,
  `ci`, `style`, `perf`.
- Use a meaningful lowercase scope when it helps identify the touched area, for
  example `feat(ipc): add scan projection options` or
  `docs(agents): clarify validation rules`.
- Keep commit summaries in English, present tense, and without a trailing
  period.
- Code comments should be in English unless the touched doc already uses Chinese
  prose.
- Prefer small, focused commits and functions.
- Keep user-visible errors friendly and actionable.
- Avoid new dependencies without a clear reason.
- Prefer current code and tests over stale docs. If docs are stale, update the
  docs as part of the change or call out the drift.

### TypeScript / Vue

- Use `<script setup lang="ts">` in Vue SFCs.
- Use Composition API (`ref`, `computed`, `watch`, `watchEffect`) over Options
  API unless an existing file clearly requires otherwise.
- Keep state minimal and derive with `computed` when possible.
- Keep templates declarative; move non-trivial branching or transforms into
  script/composables.
- Use typed props/emits for child components. Prefer props down, events up.
- Split large components into focused components and composables when the file
  owns multiple independent UI sections or side-effect-heavy logic.
- Prefer named exports for utilities and default exports for Vue components.
- Component files use PascalCase; DOM component names in templates use
  kebab-case.
- Imports are managed by Biome. Prefer external packages first, then internal
  modules, then relative files.

### Tauri IPC / Data Handling

- Use `invoke` from `@tauri-apps/api/core` on the frontend.
- Always handle `invoke` errors with `try/catch` and surface actionable UI
  feedback.
- Keep IPC payloads versionable with additive changes where possible.
- Validate inputs on both frontend and backend.
- Avoid sending large blobs via JSON when a structured or streaming format is
  available. Arrow IPC exists for larger scan payloads.
- Cross-boundary type changes should update matching frontend and Rust types in
  the same patch.

### Rust

- Use rustfmt defaults.
- Naming: `snake_case` for functions/modules, `CamelCase` for types,
  `SCREAMING_SNAKE_CASE` for constants.
- Prefer `Result<T, E>` and `?` for error propagation.
- Tauri commands should stay thin and return `Result<T, String>` or a
  serializable error envelope.
- Move reusable backend logic into helpers/services rather than command bodies.
- Log with `log` macros (`info!`, `warn!`, `error!`).
- Treat `panic!` as a bug outside startup/test-only code.

## UI Conventions

- Follow `UI_DESIGN.md` for current layout and visual decisions.
- The main information architecture is a dark global primary nav, a collapsible
  connection sidebar, and a right workspace.
- Use Naive UI for complex controls, Tailwind for layout/spacing/light styling,
  and Lucide for icons.
- Keep UI responsive and semantic. Prefer accessible labels, keyboard focus, and
  clear loading/empty/error states.
- Avoid inline styles unless a dynamic value is clearer than a class.
- Keep repeated UI in small reusable components.
- Do not add decorative animation or large reflow-heavy transitions to data
  tables. Keep motion small and purposeful.

## Security

- Do not hardcode secrets in `src/` or `src-tauri/`.
- Prefer Stronghold or OS-backed secure storage for credentials.
- Store only `secret_ref` and safe metadata in connection profiles.
- Sanitize logs and user-facing errors so credentials, tokens, and local private
  paths are not exposed unnecessarily.

## Housekeeping

- Keep files encoded in UTF-8.
- Keep changes scoped; avoid reformatting unrelated files.
- Update `README.md` when adding setup steps, commands, env vars, capabilities,
  or operational requirements.
- Update `UI_DESIGN.md` when UI architecture, interaction contracts, or validation
  checklists change.
- If new commands, IPC endpoints, or env vars are added, document them in the
  same change.
- If a Tauri/WebDriver debug probe or workflow proves reusable, update
  `.agents/skills/tauri-webdriver/` in the same change: prefer adding
  parameterized script support under `scripts/`, then document the usage in the
  skill or `references/`. Keep helpers generic, stdlib-only where practical,
  and scoped so cleanup never kills unrelated user processes.
