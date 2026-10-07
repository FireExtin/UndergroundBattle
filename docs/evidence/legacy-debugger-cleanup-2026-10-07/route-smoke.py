"""Exercise the actual built client with an intentionally absent local backend.

This checks routing/chunk loading and the debugger's designed fallback. It does
not exercise live Go, Rust gameplay, or the public deployment.
"""
import argparse
import functools
import json
import threading
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit

from playwright.sync_api import sync_playwright


class ClientHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        route = urlsplit(self.path).path
        if route.startswith("/api/"):
            self.send_error(501, "No backend in this client route smoke check")
            return
        if route == "/" or route.startswith("/legacy-debugger"):
            self.path = "/index.html"
        super().do_GET()

    def log_message(self, *_args):
        pass


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--dist", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    handler = functools.partial(ClientHandler, directory=str(args.dist.resolve()))
    server = ThreadingHTTPServer(("127.0.0.1", 0), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    checks = []
    errors = []
    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(
                executable_path="/usr/bin/chromium",
                headless=True,
                args=["--no-sandbox", "--disable-dev-shm-usage", "--disable-gpu",
                      "--single-process", "--no-zygote"],
            )
            context = browser.new_context(viewport={"width": 1280, "height": 900})
            page = context.new_page()
            page.on("pageerror", lambda error: errors.append(str(error)))
            requests = []
            page.on("request", lambda request: requests.append(urlsplit(request.url).path))

            page.goto(base + "/", wait_until="networkidle")
            page.get_by_role("button", name="上手指南 ?", exact=True).wait_for()
            assert page.locator(".hg-app").count() == 1
            assert page.get_by_role("heading", name="最小对局骨架").count() == 0
            assert not any("LegacyDebuggerEntry-" in route for route in requests)
            assert "/api/catalog" in requests
            page.screenshot(path=str(args.output / "default-game-shell.png"), full_page=True)
            checks.append({"route": "/", "result": "pass", "game_shell": True,
                           "debugger_chunk_requests": [], "requests": requests.copy()})

            requests.clear()
            page.goto(base + "/legacy-debugger", wait_until="networkidle")
            page.get_by_role("heading", name="最小对局骨架", exact=True).wait_for()
            page.get_by_text("Source: Mock Fallback", exact=True).wait_for()
            assert page.locator(".hg-app").count() == 0
            chunks = [route for route in requests if "LegacyDebuggerEntry-" in route]
            assert len(chunks) == 1, chunks
            assert "/api/debugger/messages" in requests
            assert page.get_by_role("button", name="Pass Priority", exact=True).is_disabled()
            page.screenshot(path=str(args.output / "legacy-debugger-fallback.png"), full_page=True)
            checks.append({"route": "/legacy-debugger", "result": "pass",
                           "designed_mock_fallback": True, "debugger_chunk_requests": chunks,
                           "requests": requests.copy(), "submit_disabled_without_live_server": True})
            assert errors == [], errors
            browser.close()
        report = {"scope": "Actual built client route/chunk checks, local backend absent; no fixtures or HTTP response interception",
                  "public_deployment_checked": False, "live_backend_checked": False,
                  "checks": checks, "uncaught_javascript_errors": errors}
        (args.output / "route-smoke.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
        print(json.dumps({"checks_passed": len(checks), "uncaught_javascript_errors": errors}))
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)


if __name__ == "__main__":
    main()
