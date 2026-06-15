# WebDriver Patterns

## Selector Strategy

Prefer selectors in this order:

1. `data-testid` or another stable test attribute.
2. Stable English `aria-label` values.
3. CSS selectors tied to semantic structure.
4. JS-backed text or localized aria matching.

Native WebDriver element lookup can be awkward with localized strings or heavy escaping. Use the bundled script's `--aria` and `--text` options when a direct CSS selector is brittle.

## Session Lifecycle

Use one-shot `smoke` when the goal is only evidence capture.

Use composable commands when stateful interaction matters:

```powershell
python .\scripts\tauri_webdriver.py start-driver
python .\scripts\tauri_webdriver.py open --application E:\path\app.exe --wait-css "[aria-label='Agent homepage shell']"
python .\scripts\tauri_webdriver.py click --aria "Open agent thread list"
python .\scripts\tauri_webdriver.py type --aria "Agent thread prompt" --value "Inspect the workspace"
python .\scripts\tauri_webdriver.py screenshot --output evidence.png
python .\scripts\tauri_webdriver.py close --stop-driver
```

## Useful Operations

- `status`: title, URL, window rect, and short body text.
- `wait`: wait for CSS selector, aria label, or visible text.
- `click`: click by CSS selector, aria label, visible text, or button text.
- `type`: type into an input or textarea by CSS selector, aria label, or visible text.
- `screenshot`: save PNG evidence.
- `execute`: run a short JavaScript snippet or script file.

## Failure Modes

- **Session opens but selectors never appear**: the dev URL or backend is not running, or the app is still loading.
- **Native lookup fails on localized text**: use JS-backed `--aria` or `--text`.
- **Click reports intercepted or stale element**: wait again, then retry with JS-backed click.
- **Driver port already occupied**: reuse the existing driver or stop the stale process.
- **Screenshots show browser-like fallback state**: verify the application path is a Tauri executable, not a Vite browser URL.
