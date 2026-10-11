"""Select unchanged current Native exports for UI and real D1 regressions.

Usage: python3 tools/fixture-tools/current-win-flow.py <WIN_FLOW_EVIDENCE_DIR>
Run win_flow_tests with that directory first. No opaque state is edited here.
"""
import hashlib
import json
import pathlib
import re
import sys

root = pathlib.Path(__file__).resolve().parents[2]
source = pathlib.Path(sys.argv[1])
names = ["entry-XQ37-0-first-0", "nested-reveal-Before(1, 0)",
         "privilege-renown-true", "privilege-already-wins", "world-empty-nested-2-1",
         "world-empty-jc050", "world-empty-conservation", "world-empty-duel"]


def read(path):
    raw = path.read_bytes()
    value = json.loads(raw)
    value["inputName"] = path.name
    value["inputSha256"] = hashlib.sha256(raw).hexdigest()
    return value


def steps(name):
    paths = [p for p in source.glob("*-step.json")
             if re.fullmatch(re.escape(name) + r"-\d+-step\.json", p.name)]
    paths.sort(key=lambda p: int(p.name.removeprefix(name + "-").split("-")[0]))
    values = [read(p) for p in paths]
    assert values, name
    for before, after in zip(values, values[1:]):
        assert before["expected"]["state"] == after["state"], name
    return values


def before_views(step):
    prefix = step["inputName"].removesuffix("-step.json")
    checkpoint = json.loads((source / (prefix + "-checkpoint.json")).read_bytes())
    assert checkpoint["state"] == step["state"]
    return checkpoint["views"]


def write(relative, value):
    (root / relative).write_text(json.dumps(value, ensure_ascii=False, separators=(",", ":")) + "\n")


write("sites/test/fixtures/win-flow-native-v063.json", {
    "scenarios": [{"name": name, "steps": steps(name)} for name in names]})
captures = []
for name, empty_index in [("world-empty-duel", 1), ("world-empty-slot-2", 2)]:
    candidates = []
    for step in steps(name):
        before = before_views(step)
        if before[0]["regions"][empty_index]["cardId"] and not step["views"][0]["regions"][empty_index]["cardId"]:
            candidates.append({"name": name, "emptyIndex": empty_index,
                               "beforeState": step["state"], "beforeViews": before, "step": step})
    assert len(candidates) == 1, name
    captures.extend(candidates)
sparse = [s for s in steps("world-empty-jc050")
          if before_views(s)[s["seat"]].get("pendingChoice")
          and [o["id"] for o in before_views(s)[s["seat"]]["pendingChoice"]["options"]]
          == ["region:0", "region:2", "region:3", "region:4"]
          and not s["expected"].get("errorCode")]
assert len(sparse) == 1
write("web/src/game/fixedSlotsNative63.fixture.json", {
    "captures": captures, "sparseChoice": {"beforeViews": before_views(sparse[0]), "step": sparse[0]}})
print(json.dumps({"continuousD1Scenarios": len(names), "captures": len(captures), "sparseChoices": 1}))
