"""Join freshly exported Native snapshots and original accepted Room commands.

States remain opaque strings. This never adapts an old save or touches live data.
Usage: python3 tools/fixture-tools/current-four-faction-choices.py <native-dir>
"""
from pathlib import Path
import hashlib
import json
import sys

root = Path(sys.argv[1]).resolve()
suffix = sys.argv[2] if len(sys.argv) > 2 else "64"
assert suffix.isdigit()
rows = []
for color in ["blue", "black", "red"]:
    steps = [(f, json.loads(f.read_text())) for f in sorted((root / ("trace-" + color)).glob("step-*.json"))]
    for f in sorted((root / color).glob("*.json")):
        if f.name not in ["jc032-positive.json", "jc032-zero-hit.json"] and "corpse-choice-" not in f.name and "private-forecast" not in f.name:
            continue
        snap = json.loads(f.read_text())
        matches = [(path, row) for path, row in steps if row["state"] == snap["state"]
                   and row["command"]["action"].get("kind") == "game"
                   and row["command"]["action"].get("action", {}).get("kind") == "choose"
                   and row["expected"]["outcome"] == "accepted"]
        assert matches, f
        path, step = matches[0]
        assert all(row["command"]["action"] == step["command"]["action"] for _, row in matches), f
        seat = step["seat"]
        assert snap["views"][seat]["pendingChoice"]
        assert step["expected"]["view"] == step["views"][seat]
        rows.append({"name": f"{color}:{f.stem}", "color": color, "seat": seat,
                     "before": snap["state"], "beforeViews": snap["views"],
                     "command": step["command"], "expected": step["expected"], "afterViews": step["views"],
                     "provenance": {"snapshot": f.relative_to(root).as_posix(),
                                    "snapshotSha256": hashlib.sha256(f.read_bytes()).hexdigest(),
                                    "nativeStep": path.relative_to(root).as_posix(),
                                    "nativeStepSha256": hashlib.sha256(path.read_bytes()).hexdigest()}})
assert len(rows) == 10, len(rows)
result = {"scope": "Explicit offline Native layouts; real accepted Room commands and four-seat views. Not natural browser play or migration.",
          "sourceBaseline": sys.argv[3] if len(sys.argv) > 3 else "8dd1ebfa3700ca1793b12a2c8fc6c679c752e053",
          "wasmSha256": hashlib.sha256(Path("rust-game-wasm/pkg/hegemony_wasm_bg.wasm").read_bytes()).hexdigest(),
          "cases": rows}
out = Path(f"web/src/game/currentLifecycle{suffix}.fixture.json")
out.write_text(json.dumps(result, ensure_ascii=False, separators=(",", ":")) + "\n")
print(json.dumps({"path": str(out), "cases": len(rows), "bytes": out.stat().st_size,
                  "sha256": hashlib.sha256(out.read_bytes()).hexdigest()}))
