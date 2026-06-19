#!/usr/bin/env python3
"""Small WebDriver CLI for Tauri desktop UI inspection.

The script intentionally uses only Python stdlib so it can run in constrained
developer environments without installing Selenium.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import shutil
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any

DEFAULT_DRIVER_URL = "http://127.0.0.1:4444"
DEFAULT_DRIVER_PORT = 4444
DEFAULT_NATIVE_PORT = 9515
SESSION_KEY = "element-6066-11e4-a52e-4f735466cecf"


class WebDriverError(RuntimeError):
    pass


def log(message: str) -> None:
    print(message, file=sys.stderr)


def print_json(payload: Any) -> None:
    print(json.dumps(payload, indent=2, ensure_ascii=True))


def http_json(
    method: str,
    url: str,
    payload: dict[str, Any] | None = None,
    timeout: float = 30.0,
) -> dict[str, Any]:
    data = None
    headers = {"Accept": "application/json"}
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"
    request = urllib.request.Request(url, data=data, method=method, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            raw = response.read()
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        raise WebDriverError(f"{method} {url} failed: HTTP {error.code}: {body}") from error
    except urllib.error.URLError as error:
        raise WebDriverError(f"{method} {url} failed: {error.reason}") from error
    if not raw:
        return {}
    return json.loads(raw.decode("utf-8"))


def port_is_open(host: str, port: int, timeout: float = 0.25) -> bool:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.settimeout(timeout)
        return sock.connect_ex((host, port)) == 0


def wait_for_port(host: str, port: int, timeout: float) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if port_is_open(host, port):
            return True
        time.sleep(0.1)
    return False


def read_json_file(path: Path) -> dict[str, Any]:
    if not path.exists():
        raise WebDriverError(f"Missing state file: {path}")
    return json.loads(path.read_text(encoding="utf-8"))


def write_json_file(path: Path, payload: dict[str, Any]) -> None:
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def base_url(driver_url: str) -> str:
    return driver_url.rstrip("/")


def session_url(driver_url: str, session_id: str, suffix: str = "") -> str:
    return f"{base_url(driver_url)}/session/{session_id}{suffix}"


def load_session_id(args: argparse.Namespace) -> str:
    if args.session_id:
        return args.session_id
    state = read_json_file(Path(args.session_file))
    session_id = state.get("session_id")
    if not isinstance(session_id, str) or not session_id:
        raise WebDriverError(f"No session_id in {args.session_file}")
    return session_id


def load_driver_pid(path: str) -> int | None:
    state_path = Path(path)
    if not state_path.exists():
        return None
    try:
        value = read_json_file(state_path).get("pid")
    except (OSError, json.JSONDecodeError):
        return None
    return value if isinstance(value, int) else None


def create_session(args: argparse.Namespace) -> str:
    application = Path(args.application).resolve()
    if not application.exists():
        raise WebDriverError(f"Application does not exist: {application}")
    payload = {
        "capabilities": {
            "alwaysMatch": {
                "browserName": "wry",
                "tauri:options": {
                    "application": str(application),
                },
            },
        },
    }
    response = http_json("POST", f"{base_url(args.driver_url)}/session", payload)
    value = response.get("value", response)
    session_id = value.get("sessionId")
    if not isinstance(session_id, str) or not session_id:
        raise WebDriverError(f"Could not create session: {response}")
    write_json_file(
        Path(args.session_file),
        {
            "session_id": session_id,
            "driver_url": args.driver_url,
            "application": str(application),
        },
    )
    return session_id


def execute_js(
    driver_url: str,
    session_id: str,
    script: str,
    args: list[Any] | None = None,
) -> Any:
    payload = {"script": script, "args": args or []}
    response = http_json("POST", session_url(driver_url, session_id, "/execute/sync"), payload)
    return response.get("value")


def find_element(driver_url: str, session_id: str, css: str) -> str:
    payload = {"using": "css selector", "value": css}
    response = http_json("POST", session_url(driver_url, session_id, "/element"), payload)
    value = response.get("value") or {}
    element_id = value.get(SESSION_KEY)
    if not isinstance(element_id, str) or not element_id:
        raise WebDriverError(f"Element id missing for selector {css!r}: {response}")
    return element_id


def wait_for_condition(args: argparse.Namespace, session_id: str) -> dict[str, Any]:
    deadline = time.time() + args.timeout
    last: Any = None
    while time.time() < deadline:
        last = inspect_state(args.driver_url, session_id)
        if args.wait_css:
            try:
                find_element(args.driver_url, session_id, args.wait_css)
                return {"matched": "css", "value": args.wait_css, "state": last}
            except WebDriverError:
                pass
        if args.wait_aria and js_query_exists(args.driver_url, session_id, "aria", args.wait_aria):
            return {"matched": "aria", "value": args.wait_aria, "state": last}
        if args.wait_text and js_query_exists(args.driver_url, session_id, "text", args.wait_text):
            return {"matched": "text", "value": args.wait_text, "state": last}
        if not args.wait_css and not args.wait_aria and not args.wait_text:
            return {"matched": "none", "state": last}
        time.sleep(args.interval)
    raise WebDriverError(
        "Timed out waiting for "
        f"css={args.wait_css!r} aria={args.wait_aria!r} text={args.wait_text!r}; "
        f"last_state={json.dumps(last, ensure_ascii=False)[:1000]}"
    )


def js_query_exists(driver_url: str, session_id: str, mode: str, value: str) -> bool:
    script = JS_FIND_ELEMENT + "\nreturn Boolean(findCandidate(arguments[0], arguments[1]));"
    return bool(execute_js(driver_url, session_id, script, [mode, value]))


def inspect_state(driver_url: str, session_id: str) -> dict[str, Any]:
    script = """
return {
  title: document.title,
  url: location.href,
  readyState: document.readyState,
  bodyText: document.body ? document.body.innerText.slice(0, 1000) : "",
  buttons: Array.from(document.querySelectorAll("button")).map((button, index) => ({
    index,
    text: button.innerText,
    aria: button.getAttribute("aria-label"),
    title: button.getAttribute("title")
  })).slice(0, 80),
  inputs: Array.from(document.querySelectorAll("input, textarea")).map((input, index) => ({
    index,
    tag: input.tagName,
    value: input.value,
    placeholder: input.getAttribute("placeholder"),
    aria: input.getAttribute("aria-label")
  })).slice(0, 40)
};
"""
    return execute_js(driver_url, session_id, script) or {}


JS_FIND_ELEMENT = r"""
function visibleTextOf(element) {
  return (element.innerText || element.textContent || "").trim();
}
function findCandidate(mode, expected) {
  const elements = Array.from(document.querySelectorAll("button, input, textarea, [role], a, [aria-label]"));
  if (mode === "aria") {
    return elements.find((element) => element.getAttribute("aria-label") === expected) || null;
  }
  if (mode === "text") {
    return elements.find((element) => visibleTextOf(element).includes(expected)) || null;
  }
  if (mode === "button-text") {
    return Array.from(document.querySelectorAll("button")).find((element) => visibleTextOf(element).includes(expected)) || null;
  }
  throw new Error(`Unsupported lookup mode: ${mode}`);
}
"""


def click_by_js(driver_url: str, session_id: str, mode: str, value: str) -> Any:
    script = JS_FIND_ELEMENT + """
const element = findCandidate(arguments[0], arguments[1]);
if (!element) {
  return { clicked: false, reason: "not-found" };
}
element.scrollIntoView({ block: "center", inline: "center" });
element.click();
return { clicked: true, tag: element.tagName, text: visibleTextOf(element), aria: element.getAttribute("aria-label") };
"""
    result = execute_js(driver_url, session_id, script, [mode, value])
    if not result or not result.get("clicked"):
        raise WebDriverError(f"Could not click {mode}={value!r}: {result}")
    return result


def type_by_js(driver_url: str, session_id: str, mode: str, lookup: str, value: str, clear: bool) -> Any:
    script = JS_FIND_ELEMENT + """
const element = findCandidate(arguments[0], arguments[1]);
if (!element) {
  return { typed: false, reason: "not-found" };
}
element.scrollIntoView({ block: "center", inline: "center" });
element.focus();
if (arguments[3]) {
  element.value = "";
}
element.value = `${element.value || ""}${arguments[2]}`;
element.dispatchEvent(new Event("input", { bubbles: true }));
element.dispatchEvent(new Event("change", { bubbles: true }));
return { typed: true, tag: element.tagName, aria: element.getAttribute("aria-label"), value: element.value };
"""
    result = execute_js(driver_url, session_id, script, [mode, lookup, value, clear])
    if not result or not result.get("typed"):
        raise WebDriverError(f"Could not type into {mode}={lookup!r}: {result}")
    return result


def command_check_tools(_args: argparse.Namespace) -> int:
    tools = {
        "tauri-driver": shutil.which("tauri-driver"),
        "msedgedriver": shutil.which("msedgedriver"),
    }
    print_json(tools)
    return 0 if all(tools.values()) else 1


def command_start_driver(args: argparse.Namespace) -> int:
    if port_is_open(args.host, args.driver_port):
        print_json({"status": "already-running", "port": args.driver_port})
        return 0
    command = [
        "tauri-driver",
        "--port",
        str(args.driver_port),
        "--native-port",
        str(args.native_port),
    ]
    if args.native_driver:
        command.extend(["--native-driver", args.native_driver])
    creationflags = 0
    if os.name == "nt":
        creationflags = subprocess.CREATE_NEW_PROCESS_GROUP | subprocess.DETACHED_PROCESS
    process = subprocess.Popen(
        command,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        creationflags=creationflags,
    )
    if not wait_for_port(args.host, args.driver_port, args.driver_timeout):
        raise WebDriverError(f"tauri-driver did not listen on {args.host}:{args.driver_port}")
    write_json_file(
        Path(args.driver_state_file),
        {
            "pid": process.pid,
            "driver_port": args.driver_port,
            "native_port": args.native_port,
        },
    )
    print_json({"status": "started", "pid": process.pid, "port": args.driver_port})
    return 0


def command_stop_driver(args: argparse.Namespace) -> int:
    pid = getattr(args, "pid", None) or load_driver_pid(args.driver_state_file)
    if not pid:
        print_json({"status": "no-pid"})
        return 0
    try:
        if os.name == "nt":
            subprocess.run(["taskkill", "/PID", str(pid), "/F"], check=False, stdout=subprocess.DEVNULL)
        else:
            os.kill(pid, 15)
    finally:
        Path(args.driver_state_file).unlink(missing_ok=True)
    print_json({"status": "stopped", "pid": pid})
    return 0


def command_open(args: argparse.Namespace) -> int:
    session_id = create_session(args)
    if args.wait_css or args.wait_aria or args.wait_text:
        wait_for_condition(args, session_id)
    print_json({"session_id": session_id})
    return 0


def command_close(args: argparse.Namespace) -> int:
    session_id = load_session_id(args)
    http_json("DELETE", session_url(args.driver_url, session_id))
    Path(args.session_file).unlink(missing_ok=True)
    if args.stop_driver:
        command_stop_driver(args)
    print_json({"status": "closed", "session_id": session_id})
    return 0


def command_status(args: argparse.Namespace) -> int:
    session_id = load_session_id(args)
    rect = http_json("GET", session_url(args.driver_url, session_id, "/window/rect")).get("value")
    state = inspect_state(args.driver_url, session_id)
    print_json({"rect": rect, "state": state})
    return 0


def command_resize(args: argparse.Namespace) -> int:
    session_id = load_session_id(args)
    payload = {"width": args.width, "height": args.height}
    if args.x is not None:
        payload["x"] = args.x
    if args.y is not None:
        payload["y"] = args.y
    rect = http_json("POST", session_url(args.driver_url, session_id, "/window/rect"), payload).get("value")
    print_json({"rect": rect})
    return 0


def command_wait(args: argparse.Namespace) -> int:
    session_id = load_session_id(args)
    result = wait_for_condition(args, session_id)
    print_json(result)
    return 0


def command_screenshot(args: argparse.Namespace) -> int:
    session_id = load_session_id(args)
    response = http_json("GET", session_url(args.driver_url, session_id, "/screenshot"))
    encoded = response.get("value")
    if not isinstance(encoded, str):
        raise WebDriverError(f"Screenshot payload missing: {response}")
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(base64.b64decode(encoded))
    print_json({"output": str(output.resolve())})
    return 0


def command_click(args: argparse.Namespace) -> int:
    session_id = load_session_id(args)
    if args.css:
        element_id = find_element(args.driver_url, session_id, args.css)
        http_json("POST", session_url(args.driver_url, session_id, f"/element/{element_id}/click"), {})
        result = {"clicked": True, "mode": "css", "value": args.css}
    elif args.aria:
        result = click_by_js(args.driver_url, session_id, "aria", args.aria)
    elif args.text:
        result = click_by_js(args.driver_url, session_id, "text", args.text)
    elif args.button_text:
        result = click_by_js(args.driver_url, session_id, "button-text", args.button_text)
    else:
        raise WebDriverError("click requires --css, --aria, --text, or --button-text")
    print_json(result)
    return 0


def command_type(args: argparse.Namespace) -> int:
    session_id = load_session_id(args)
    if args.css:
        element_id = find_element(args.driver_url, session_id, args.css)
        if args.clear:
            http_json("POST", session_url(args.driver_url, session_id, f"/element/{element_id}/clear"), {})
        http_json(
            "POST",
            session_url(args.driver_url, session_id, f"/element/{element_id}/value"),
            {"text": args.value, "value": list(args.value)},
        )
        result = {"typed": True, "mode": "css", "value": args.css}
    elif args.aria:
        result = type_by_js(args.driver_url, session_id, "aria", args.aria, args.value, args.clear)
    elif args.text:
        result = type_by_js(args.driver_url, session_id, "text", args.text, args.value, args.clear)
    else:
        raise WebDriverError("type requires --css, --aria, or --text")
    print_json(result)
    return 0


def command_execute(args: argparse.Namespace) -> int:
    session_id = load_session_id(args)
    if args.script_file:
        script = Path(args.script_file).read_text(encoding="utf-8")
    else:
        script = args.script
    result = execute_js(args.driver_url, session_id, script, [])
    print_json(result)
    return 0


def command_layout_check(args: argparse.Namespace) -> int:
    session_id = load_session_id(args)
    script = r"""
const selector = arguments[0];
const parentSelector = arguments[1];
const absentSelector = arguments[2];
const element = document.querySelector(selector);
const parent = parentSelector ? document.querySelector(parentSelector) : null;
const targetRect = element ? element.getBoundingClientRect() : null;
const parentRect = parent ? parent.getBoundingClientRect() : null;
const viewport = {
  width: window.innerWidth,
  height: window.innerHeight,
  scrollWidth: document.documentElement.scrollWidth,
  clientWidth: document.documentElement.clientWidth,
  scrollHeight: document.documentElement.scrollHeight,
  clientHeight: document.documentElement.clientHeight,
};
const hasHorizontalOverflow = viewport.scrollWidth > viewport.clientWidth;
const hasVerticalOverflow = viewport.scrollHeight > viewport.clientHeight;
const overflowsViewport = targetRect
  ? targetRect.right > window.innerWidth + 1 || targetRect.left < -1
  : null;
const overflowsParent = targetRect && parentRect
  ? targetRect.right > parentRect.right + 1 || targetRect.left < parentRect.left - 1
  : null;
return {
  selector,
  exists: Boolean(element),
  text: element ? (element.innerText || element.textContent || "").replace(/\s+/g, " ").trim() : null,
  rect: targetRect ? targetRect.toJSON() : null,
  parentSelector,
  parentRect: parentRect ? parentRect.toJSON() : null,
  absentSelector,
  absentSelectorExists: absentSelector ? Boolean(element && element.querySelector(absentSelector)) : null,
  viewport,
  hasHorizontalOverflow,
  hasVerticalOverflow,
  overflowsViewport,
  overflowsParent,
};
"""
    result = execute_js(args.driver_url, session_id, script, [args.selector, args.parent, args.absent])
    print_json(result)
    if args.fail_on_overflow and result and (
        result.get("hasHorizontalOverflow")
        or result.get("overflowsViewport")
        or result.get("overflowsParent")
    ):
        return 2
    if args.fail_on_absent and result and result.get("absentSelectorExists"):
        return 3
    return 0


def command_smoke(args: argparse.Namespace) -> int:
    started_driver = False
    if not port_is_open(args.host, args.driver_port):
        command_start_driver(args)
        started_driver = True
    session_id = create_session(args)
    try:
        wait_result = wait_for_condition(args, session_id)
        state = inspect_state(args.driver_url, session_id)
        screenshot_path = None
        if args.screenshot:
            screenshot_args = argparse.Namespace(**vars(args))
            screenshot_args.session_id = session_id
            screenshot_args.output = args.screenshot
            command_screenshot(screenshot_args)
            screenshot_path = str(Path(args.screenshot).resolve())
        print(
            json.dumps(
                {
                    "session_id": session_id,
                    "wait": wait_result.get("matched"),
                    "state": state,
                    "screenshot": screenshot_path,
                },
                indent=2,
                ensure_ascii=True,
            )
        )
    finally:
        try:
            http_json("DELETE", session_url(args.driver_url, session_id))
        finally:
            Path(args.session_file).unlink(missing_ok=True)
            if args.stop_driver or started_driver:
                command_stop_driver(args)
    return 0


def add_common(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--driver-url", default=DEFAULT_DRIVER_URL)
    parser.add_argument("--session-id")
    parser.add_argument("--session-file", default=".tauri-webdriver-session.json")
    parser.add_argument("--driver-state-file", default=".tauri-webdriver-driver.json")


def add_driver_options(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--driver-port", default=DEFAULT_DRIVER_PORT, type=int)
    parser.add_argument("--native-port", default=DEFAULT_NATIVE_PORT, type=int)
    parser.add_argument("--native-driver")
    parser.add_argument("--driver-timeout", default=15.0, type=float)


def add_wait_options(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--wait-css")
    parser.add_argument("--wait-aria")
    parser.add_argument("--wait-text")
    parser.add_argument("--timeout", default=20.0, type=float)
    parser.add_argument("--interval", default=0.5, type=float)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="tauri_webdriver.py")
    subparsers = parser.add_subparsers(dest="command", required=True)

    check = subparsers.add_parser("check-tools")
    check.set_defaults(func=command_check_tools)

    start = subparsers.add_parser("start-driver")
    add_driver_options(start)
    add_common(start)
    start.set_defaults(func=command_start_driver)

    stop = subparsers.add_parser("stop-driver")
    add_driver_options(stop)
    add_common(stop)
    stop.add_argument("--pid", type=int)
    stop.set_defaults(func=command_stop_driver)

    open_cmd = subparsers.add_parser("open")
    add_common(open_cmd)
    add_wait_options(open_cmd)
    open_cmd.add_argument("--application", required=True)
    open_cmd.set_defaults(func=command_open)

    close = subparsers.add_parser("close")
    add_common(close)
    add_driver_options(close)
    close.add_argument("--stop-driver", action="store_true")
    close.set_defaults(func=command_close)

    status = subparsers.add_parser("status")
    add_common(status)
    status.set_defaults(func=command_status)

    resize = subparsers.add_parser("resize")
    add_common(resize)
    resize.add_argument("--width", required=True, type=int)
    resize.add_argument("--height", required=True, type=int)
    resize.add_argument("--x", type=int)
    resize.add_argument("--y", type=int)
    resize.set_defaults(func=command_resize)

    wait = subparsers.add_parser("wait")
    add_common(wait)
    add_wait_options(wait)
    wait.set_defaults(func=command_wait)

    screenshot = subparsers.add_parser("screenshot")
    add_common(screenshot)
    screenshot.add_argument("--output", required=True)
    screenshot.set_defaults(func=command_screenshot)

    click = subparsers.add_parser("click")
    add_common(click)
    click.add_argument("--css")
    click.add_argument("--aria")
    click.add_argument("--text")
    click.add_argument("--button-text")
    click.set_defaults(func=command_click)

    type_cmd = subparsers.add_parser("type")
    add_common(type_cmd)
    type_cmd.add_argument("--css")
    type_cmd.add_argument("--aria")
    type_cmd.add_argument("--text")
    type_cmd.add_argument("--value", required=True)
    type_cmd.add_argument("--clear", action="store_true")
    type_cmd.set_defaults(func=command_type)

    execute = subparsers.add_parser("execute")
    add_common(execute)
    group = execute.add_mutually_exclusive_group(required=True)
    group.add_argument("--script")
    group.add_argument("--script-file")
    execute.set_defaults(func=command_execute)

    layout = subparsers.add_parser("layout-check")
    add_common(layout)
    layout.add_argument("--selector", required=True)
    layout.add_argument("--parent")
    layout.add_argument("--absent")
    layout.add_argument("--fail-on-overflow", action="store_true")
    layout.add_argument("--fail-on-absent", action="store_true")
    layout.set_defaults(func=command_layout_check)

    smoke = subparsers.add_parser("smoke")
    add_common(smoke)
    add_driver_options(smoke)
    add_wait_options(smoke)
    smoke.add_argument("--application", required=True)
    smoke.add_argument("--screenshot")
    smoke.add_argument("--stop-driver", action="store_true")
    smoke.set_defaults(func=command_smoke)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return int(args.func(args))
    except WebDriverError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
