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
python .\scripts\tauri_webdriver.py open --application ..\..\..\src-tauri\target\debug\lancedb-viewer.exe --wait-css "#app"
python .\scripts\tauri_webdriver.py click --aria "检索"
python .\scripts\tauri_webdriver.py screenshot --output evidence.png
python .\scripts\tauri_webdriver.py close --stop-driver
```

Use `resize` and `layout-check` for layout bugs:

```powershell
python .\scripts\tauri_webdriver.py resize --width 800 --height 600
python .\scripts\tauri_webdriver.py layout-check --selector ".target-panel" --parent "main" --fail-on-overflow
python .\scripts\tauri_webdriver.py resize --width 1495 --height 995
python .\scripts\tauri_webdriver.py layout-check --selector ".target-panel" --parent "main" --absent ".unexpected-child" --fail-on-overflow --fail-on-absent
```

The `layout-check` output reports selector text, target rect, optional parent rect, viewport scroll size, horizontal overflow, viewport overflow, parent overflow, and whether the optional absent child exists.

## Useful Operations

- `status`: title, URL, window rect, and short body text.
- `resize`: set the native window width/height through WebDriver `/window/rect`.
- `wait`: wait for CSS selector, aria label, or visible text.
- `click`: click by CSS selector, aria label, visible text, or button text.
- `type`: type into an input or textarea by CSS selector, aria label, or visible text.
- `screenshot`: save PNG evidence.
- `execute`: run a short JavaScript snippet or script file.
- `layout-check`: inspect element bounds and fail on overflow or unexpected descendants.

## Debug Trace Pattern

1. Run `status` first and confirm the route, active text, buttons, and inputs match the UI state you intend to inspect.
2. Navigate with `click`, `wait`, and small `execute` snippets only after the state is confirmed.
3. For layout fixes, check both a constrained window and a larger desktop window. Use `layout-check --fail-on-overflow` so regressions become non-zero exits.
4. Prefer ASCII JavaScript snippets for `execute`. If a localized label is difficult to pass through the shell, use `--aria`, `--text`, `--button-text`, or generate non-ASCII strings inside JavaScript.
5. Keep screenshots as temporary evidence unless the task asks to attach or preserve them.

## Failure Modes

- **Session opens but selectors never appear**: the dev URL or backend is not running, or the app is still loading.
- **Native lookup fails on localized text**: use JS-backed `--aria` or `--text`.
- **Click reports intercepted or stale element**: wait again, then retry with JS-backed click.
- **Driver port already occupied**: reuse the existing driver or stop the stale process.
- **Screenshots show browser-like fallback state**: verify the application path is a Tauri executable, not a Vite browser URL.
- **Probe reports the wrong UI**: verify the current route and selected table/connection in `status` before evaluating a component selector.
- **Cleanup disrupts the user**: close only the WebDriver session and driver process recorded in the skill's state files.
