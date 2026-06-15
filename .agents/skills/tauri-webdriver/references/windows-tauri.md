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
  --application .\apps\desktop-tauri\src-tauri\target\debug\yachiyo-live.exe `
  --wait-css "[aria-label='Agent homepage shell']" `
  --screenshot tauri-smoke.png
```

## Dev Executable Notes

Debug Tauri executables usually load the configured `devUrl` from `tauri.conf.json`. If a session opens but the page is blank or partially loaded:

- Start the Vite dev server expected by `devUrl`.
- Start the backend service expected by the UI.
- Check whether an existing Tauri dev run already owns the correct dev server/backend.

For `yachiyo-live`, the real Tauri dev flow can launch a managed backend on a non-default port. Browser-only Vite runs may use `.env` defaults instead, so do not assume browser success and Tauri success are equivalent.

## Cleanup

- Close WebDriver sessions with the script's `close` command.
- Stop a driver started by the script with `close --stop-driver` or `stop-driver`.
- If a port remains occupied, identify the owner with:

```powershell
Get-NetTCPConnection -LocalPort 4444,9515 -State Listen -ErrorAction SilentlyContinue
```
