---
name: tauri-webdriver
description: Inspect and automate real Tauri desktop WebView shells through tauri-driver and native WebDriver. Use when Codex needs to validate, screenshot, click, type into, resize, or smoke-test a Tauri UI instead of a browser-only Vite/Playwright preview, especially for WebView2 behavior, desktop chrome, tray/window flows, managed backend lifecycle, rendered model/stage layout, or desktop sign-off evidence.
---

# Tauri WebDriver

## Overview

Use this skill to inspect and operate a real Tauri desktop shell through `tauri-driver` plus the native platform WebDriver. Prefer this path for LanceDB Viewer UI work after code changes, because browser-only inspection can miss WebView2 rendering, desktop shell behavior, native sizing, dialogs, filesystem access, and layout regressions.

Browser Playwright remains useful for fast Vite diagnostics. Tauri WebDriver is the authoritative path for real desktop UI evidence.

## Terms

- **Tauri WebDriver**: The automation path exposed by `tauri-driver`.
- **Native driver**: The platform driver used underneath `tauri-driver`; on Windows WebView2 use `msedgedriver`.
- **Session**: One WebDriver-controlled Tauri app instance.
- **Application path**: The executable passed to `tauri:options.application`, for example `apps/desktop-tauri/src-tauri/target/debug/yachiyo-live.exe`.
- **Dev URL dependency**: Debug Tauri executables may still load the Vite `devUrl`; keep the dev server/backend running when the app was built for dev mode.

## Resource Index

Load only the file needed for the current task.

- `references/windows-tauri.md`: Windows/WebView2 setup, LanceDB Viewer debug executable, common launch commands, and troubleshooting.
- `references/webdriver-patterns.md`: Selector strategy, click/type/screenshot/resize/layout-check patterns, session lifecycle, and failure modes.
- `scripts/tauri_webdriver.py`: Python stdlib CLI for composable WebDriver operations.
- `scripts/tauri-webdriver.ps1`: Windows wrapper around the Python CLI.

## Workflow

1. Confirm `tauri-driver` and the native driver are on `PATH`.
2. Ensure the target Tauri executable exists and its runtime dependencies are available. For this repo, the normal debug executable is `src-tauri/target/debug/lancedb-viewer.exe` and it loads `http://localhost:1420` from `src-tauri/tauri.conf.json`.
3. Use `scripts/tauri_webdriver.py smoke ...` for one-shot title/DOM/screenshot evidence.
4. Use `open`, `wait`, `click`, `type`, `execute`, `resize`, `layout-check`, `screenshot`, `status`, and `close` when interactive, multi-step inspection is needed.
5. Prefer stable English `aria-label`, `data-testid`, or CSS selectors for native WebDriver element lookup. For localized or dynamic text, use the script's JS-backed `--aria` or `--text` options.
6. Check at least one normal desktop width and one constrained/narrow width when the task touches layout, panels, tables, dialogs, or responsive behavior.
7. Close sessions and stop only the driver/session you started. Do not kill unrelated `bun`, `node`, `tauri`, or WebView processes owned by the user.

## Skill Maintenance

When a reusable WebDriver probe gets hand-written more than once, update `scripts/tauri_webdriver.py` instead of leaving the logic in chat. Keep additions generic, parameterized, stdlib-only, and documented in `references/webdriver-patterns.md`.

## Minimal Commands

From the skill directory:

```powershell
python .\scripts\tauri_webdriver.py check-tools
python .\scripts\tauri_webdriver.py smoke --application ..\..\..\src-tauri\target\debug\lancedb-viewer.exe --wait-css "#app" --screenshot smoke.png
```

For a composable session:

```powershell
python .\scripts\tauri_webdriver.py start-driver
python .\scripts\tauri_webdriver.py open --application ..\..\..\src-tauri\target\debug\lancedb-viewer.exe --wait-css "#app"
python .\scripts\tauri_webdriver.py resize --width 1495 --height 995
python .\scripts\tauri_webdriver.py layout-check --selector ".search-breadcrumb-shell" --parent "main" --absent ".n-select" --fail-on-overflow --fail-on-absent
python .\scripts\tauri_webdriver.py screenshot --output evidence.png
python .\scripts\tauri_webdriver.py close --stop-driver
```

Use the PowerShell wrapper when a Windows shell entrypoint is more convenient:

```powershell
.\scripts\tauri-webdriver.ps1 smoke --application ..\..\..\src-tauri\target\debug\lancedb-viewer.exe --wait-css "#app"
```
