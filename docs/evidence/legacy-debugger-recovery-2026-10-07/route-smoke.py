"""Exercise the actual built client with an intentionally absent local backend.

The local server returns503 for the first debugger chunk, then serves the exact
build. The real reload link must cause a new document and a successful import.
This does not exercise live Go, Rust gameplay, or the public deployment.
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
    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def do_GET(self):
        route = urlsplit(self.path).path
        if route.startswith("/api/"):
            self.send_error(501, "No backend in this client route smoke check")
            return
        if "LegacyDebuggerEntry-" in route and self.server.debugger_failures_remaining:
            self.server.debugger_failures_remaining -= 1
            self.send_error(503, "One local chunk download failure for recovery testing")
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
    server.debugger_failures_remaining = 0
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
            responses = []
            page.on("request", lambda request: requests.append(urlsplit(request.url).path))
            page.on("response", lambda response: responses.append({"path": urlsplit(response.url).path, "status": response.status}))

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
            responses.clear()
            server.debugger_failures_remaining = 1
            page.goto(base + "/legacy-debugger", wait_until="networkidle")
            page.get_by_role("heading", name="调试器加载失败", exact=True).wait_for()
            assert page.get_by_role("alert").is_visible()
            assert any("LegacyDebuggerEntry-" in row["path"] and row["status"] == 503 for row in responses)
            assert page.locator(".hg-app").count() == 0
            page.screenshot(path=str(args.output / "debugger-import-failure.png"), full_page=True)
            checks.append({"route": "/legacy-debugger", "result": "pass",
                           "chunk503_contained_by_boundary": True, "requests": requests.copy(),
                           "responses": responses.copy()})

            requests.clear()
            responses.clear()
            # Clicking the actual plain link navigates the whole document: no
            # rerender/reset of a rejected cached React.lazy component.
            previous_document_time_origin = page.evaluate("performance.timeOrigin")
            with page.expect_navigation(wait_until="domcontentloaded"):
                page.get_by_role("link", name="重新加载调试器", exact=True).click()
            page.get_by_role("heading", name="最小对局骨架", exact=True).wait_for()
            page.get_by_text("Source: Mock Fallback", exact=True).wait_for()
            assert page.locator(".hg-app").count() == 0
            chunks = [route for route in requests if "LegacyDebuggerEntry-" in route]
            assert len(chunks) == 1, chunks
            assert "/legacy-debugger" in requests
            assert page.evaluate("performance.timeOrigin") != previous_document_time_origin
            assert any("LegacyDebuggerEntry-" in row["path"] and row["status"] == 200 for row in responses)
            assert "/api/debugger/messages" in requests
            assert page.get_by_role("button", name="Pass Priority", exact=True).is_disabled()
            page.screenshot(path=str(args.output / "legacy-debugger-fallback.png"), full_page=True)
            checks.append({"route": "/legacy-debugger", "result": "pass",
                           "full_document_reload_recovers_failed_import": True,
                           "document_time_origin_changed": True,
                           "designed_mock_fallback": True, "debugger_chunk_requests": chunks,
                           "requests": requests.copy(), "responses": responses.copy(),
                           "submit_disabled_without_live_server": True})
            assert errors == [], errors
            browser.close()
        report = {"scope": "Actual built client; local static server fails first debugger chunk with503 then serves build. Backend absent; no browser response interception or gameplay fixtures",
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
