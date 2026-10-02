"""Local cloud acceptance helpers. Never persist credentials or private card lists."""
from __future__ import annotations

import json
import os
import signal
import subprocess
import time
import urllib.error
import urllib.request
from pathlib import Path


def write_json(path: Path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + f'.{os.getpid()}.tmp')
    temporary.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")
    temporary.replace(path)


def public_view(view):
    """Keep acceptance evidence useful without writing anyone's private hand."""
    if not view:
        return None
    result = {key: view.get(key) for key in (
        "version", "mode", "status", "you", "players", "firstTeam", "activeTeam",
        "priorityTeam", "turn", "phase", "step", "winScore", "winnerTeam", "versions",
    ) if key in view}
    result["handCount"] = len(view.get("hand", []))
    result["assetCounts"] = {
        player["id"]: sum(c.get("controller") == player["id"] for c in view.get("assets", []))
        for player in view.get("players", [])
    }
    result["regions"] = []
    for region in view.get("regions", []):
        clean = {k: region.get(k) for k in ("index", "cardId", "name", "threshold", "points", "influence")}
        clean["characters"] = [{
            k: c.get(k) for k in (
                "instanceId", "owner", "controller", "kind", "exhausted", "faceDown",
                *(() if c.get("faceDown") else ("cardId", "name", "icons", "defense", "damage", "wounds", "shield")),
            ) if k in c
        } for c in region.get("characters", [])]
        result["regions"].append(clean)
    choice = view.get("pendingChoice")
    result["pendingChoice"] = None if not choice else {
        k: choice.get(k) for k in ("id", "kind", "playerId", "min", "max", "amount", "allowDecline") if k in choice
    } | {"optionCount": len(choice.get("options", []))}
    result["waitingChoice"] = {k: view["waitingChoice"].get(k) for k in ("playerId", "kind")} if view.get("waitingChoice") else None
    result["legalActionKinds"] = sorted(set(a["kind"] for a in view.get("legalActions", [])))
    result["stackCount"] = len(view.get("stack", []))
    result["graveyardCount"] = len(view.get("graveyard", []))
    result["scoreCards"] = [{k: c.get(k) for k in ("instanceId", "cardId", "owner", "name")} for c in view.get("scoreCards", [])]
    return result


def action_payload(action):
    return {key: action[key] for key in (
        "kind", "cardId", "targetId", "region", "option", "choiceId", "selected", "top", "bottom", "allocations",
    ) if key in action}


class Api:
    def __init__(self, base_url):
        self.base = base_url.rstrip("/")

    def request(self, method, path, body=None, token=None):
        headers = {"Content-Type": "application/json"}
        if token:
            headers["Authorization"] = "Bearer " + token
        request = urllib.request.Request(self.base + path, data=None if body is None else json.dumps(body).encode(), method=method, headers=headers)
        try:
            response = urllib.request.urlopen(request, timeout=12)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            raw = response.read()
            return response.status, json.loads(raw) if raw else {}

    def state(self, session):
        status, view = self.request("GET", f'/api/rooms/{session["roomId"]}/state', token=session["token"])
        assert status == 200, f"state returned {status}"
        return view


def wait_health(base_url, timeout=45):
    deadline = time.monotonic() + timeout
    last = "not attempted"
    api = Api(base_url)
    while time.monotonic() < deadline:
        try:
            status, _ = api.request("GET", "/api/health")
            if status == 200:
                return
            last = f"HTTP {status}"
        except (OSError, ValueError) as error:
            last = type(error).__name__
        time.sleep(.2)
    raise RuntimeError(f"Service did not become healthy within {timeout}s ({last})")


class IsolatedService:
    """Own one child PID, database and log. Never touch an existing cloud service."""
    def __init__(self, binary, cwd, output, port=8091, static_dir=None):
        self.binary = str(Path(binary).resolve())
        self.cwd = str(Path(cwd).resolve())
        self.output = Path(output).resolve()
        self.output.mkdir(parents=True, exist_ok=True)
        self.port = port
        self.static_dir = static_dir
        self.process = None
        self.log = None

    @property
    def base_url(self):
        return f"http://127.0.0.1:{self.port}"

    def start(self):
        assert self.process is None, "service already owned"
        # Refuse to reuse another person's listening port.
        import socket
        with socket.socket() as sock:
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            sock.bind(("127.0.0.1", self.port))
        environment = os.environ.copy()
        environment.update(PORT=str(self.port), HEGEMONY_DB=str(self.output / "acceptance.sqlite3"))
        if self.static_dir:
            environment["WEB_DIST"] = str(Path(self.static_dir).resolve())
        self.log = (self.output / "service.log").open("ab")
        self.process = subprocess.Popen([self.binary], cwd=self.cwd, env=environment, stdout=self.log, stderr=subprocess.STDOUT)
        try:
            wait_health(self.base_url)
        except Exception:
            self.stop()
            raise

    def stop(self):
        if self.process:
            self.process.send_signal(signal.SIGINT)
            try:
                self.process.wait(timeout=8)
            except subprocess.TimeoutExpired:
                self.process.terminate()
                self.process.wait(timeout=5)
            self.process = None
        if self.log:
            self.log.close()
            self.log = None

    def restart(self):
        self.stop()
        self.start()
