# Windows Tauri WebDriver Reference

## Setup

- `tauri-driver` must be on `PATH`.
- `msedgedriver` must be on `PATH` for Windows WebView2 automation.
- `msedgedriver` must match the installed WebView2 runtime version.
- If only `msedgedriver-tool` is available, run it once to download the matching driver, then place the resulting `msedgedriver.exe` on `PATH`.

## Common Commands

Start the intermediary driver:

```powershell
tauri-driver --port 4444 --native-port 9515
```

Create and operate a session through the bundled script:

```powershell
python .\.agents\skills\tauri-webdriver\scripts\tauri_webdriver.py smoke `
  --application .\src-tauri\target\debug\lancedb-viewer.exe `
  --wait-css "#app" `
  --screenshot tauri-smoke.png
```

For a stateful LanceDB Viewer debug pass:

```powershell
python .\.agents\skills\tauri-webdriver\scripts\tauri_webdriver.py start-driver
python .\.agents\skills\tauri-webdriver\scripts\tauri_webdriver.py open `
  --application .\src-tauri\target\debug\lancedb-viewer.exe `
  --wait-css "#app"
python .\.agents\skills\tauri-webdriver\scripts\tauri_webdriver.py status
python .\.agents\skills\tauri-webdriver\scripts\tauri_webdriver.py resize --width 1495 --height 995
python .\.agents\skills\tauri-webdriver\scripts\tauri_webdriver.py layout-check `
  --selector ".workspace-selector-or-panel" `
  --parent "main" `
  --fail-on-overflow
python .\.agents\skills\tauri-webdriver\scripts\tauri_webdriver.py close --stop-driver
```

## Dev Executable Notes

Debug Tauri executables usually load the configured `devUrl` from `tauri.conf.json`. In this repo, `src-tauri/tauri.conf.json` points at `http://localhost:1420`, and the product title is `LanceDB Studio`. If a session opens but the page is blank or partially loaded:

- Start the Vite dev server expected by `devUrl`.
- Check whether an existing `bun tauri dev` run already owns the correct dev server/backend.
- If the user already has a Tauri process running, do not stop it. Open a separate WebDriver-controlled session only when needed for reproducible inspection.

## Cleanup

- Close WebDriver sessions with the script's `close` command.
- Stop a driver started by the script with `close --stop-driver` or `stop-driver`.
- Remove only the session/driver files created by this script if cleanup is needed: `.tauri-webdriver-session.json` and `.tauri-webdriver-driver.json`.
- Do not kill broad process groups such as every `node`, `bun`, `tauri`, or `msedgewebview2` process. They may belong to the user's active dev run.
- If a port remains occupied, identify the owner with:

```powershell
Get-NetTCPConnection -LocalPort 4444,9515 -State Listen -ErrorAction SilentlyContinue
```
