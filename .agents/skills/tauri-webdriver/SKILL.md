---
name: tauri-webdriver
description: Inspect and automate real Tauri desktop WebView shells through tauri-driver and native WebDriver. Use when Codex needs to validate, screenshot, click, type into, resize, or smoke-test a Tauri UI instead of a browser-only Vite/Playwright preview, especially for WebView2 behavior, desktop chrome, tray/window flows, managed backend lifecycle, rendered model/stage layout, or desktop sign-off evidence.
---

# Tauri WebDriver

## Overview

Use this skill to inspect and operate a real Tauri desktop shell through `tauri-driver` plus the native platform WebDriver. Prefer this path when browser-only inspection would miss Tauri host behavior, WebView2 rendering, managed backend lifecycle, native window chrome, tray flows, or desktop sign-off evidence.

Browser Playwright remains useful for fast Vite diagnostics. Tauri WebDriver is the authoritative path for real desktop UI evidence.

## Terms

- **Tauri WebDriver**: The automation path exposed by `tauri-driver`.
- **Native driver**: The platform driver used underneath `tauri-driver`; on Windows WebView2 use `msedgedriver`.
- **Session**: One WebDriver-controlled Tauri app instance.
- **Application path**: The executable passed to `tauri:options.application`, for example `apps/desktop-tauri/src-tauri/target/debug/yachiyo-live.exe`.
- **Dev URL dependency**: Debug Tauri executables may still load the Vite `devUrl`; keep the dev server/backend running when the app was built for dev mode.

## Resource Index

Load only the file needed for the current task.

- `references/windows-tauri.md`: Windows/WebView2 setup, common launch commands, and troubleshooting.
- `references/webdriver-patterns.md`: Selector strategy, click/type/screenshot patterns, session lifecycle, and failure modes.
- `scripts/tauri_webdriver.py`: Python stdlib CLI for composable WebDriver operations.
- `scripts/tauri-webdriver.ps1`: Windows wrapper around the Python CLI.

## Workflow

1. Confirm `tauri-driver` and the native driver are on `PATH`.
2. Ensure the target Tauri executable exists and its runtime dependencies are available. For this repo's dev executable, keep `bun run --cwd apps/desktop-tauri tauri:dev` or equivalent Vite/backend services running.
3. Use `scripts/tauri_webdriver.py smoke ...` for one-shot title/DOM/screenshot evidence.
4. Use `open`, `wait`, `click`, `type`, `screenshot`, `status`, and `close` when interactive, multi-step inspection is needed.
5. Prefer stable English `aria-label`, `data-testid`, or CSS selectors for native WebDriver element lookup. For localized or dynamic text, use the script's JS-backed `--aria` or `--text` options.
6. Close sessions and stop drivers when finished.

## Minimal Commands

From the skill directory:

```powershell
python .\scripts\tauri_webdriver.py check-tools
python .\scripts\tauri_webdriver.py smoke --application E:\path\to\app.exe --wait-css "[aria-label='Agent homepage shell']" --screenshot smoke.png
```

For a composable session:

```powershell
python .\scripts\tauri_webdriver.py start-driver
python .\scripts\tauri_webdriver.py open --application E:\path\to\app.exe --wait-css "[aria-label='Agent homepage shell']"
python .\scripts\tauri_webdriver.py click --aria "Open agent thread list"
python .\scripts\tauri_webdriver.py screenshot --output thread-drawer.png
python .\scripts\tauri_webdriver.py close --stop-driver
```

Use the PowerShell wrapper when a Windows shell entrypoint is more convenient:

```powershell
.\scripts\tauri-webdriver.ps1 smoke --application E:\path\to\app.exe --wait-css "[aria-label='Agent homepage shell']"
```
