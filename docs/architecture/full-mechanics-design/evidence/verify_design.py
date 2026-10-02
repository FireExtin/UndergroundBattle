#!/usr/bin/env python3
"""Check documentation coverage/provenance; this does not validate game rules."""
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
DESIGN = HERE.parent
ROOT = HERE.parents[3]


def read(path):
    return json.loads(path.read_text())


def main():
    manifest = read(HERE / "manifest.json")
    matrix = read(DESIGN / "mechanism-matrix.json")
    tests = read(DESIGN / "test-matrix.json")
    coverage = read(ROOT / "rust-game/data/faction-coverage.json")
    errors = []

    def check(condition, message):
        if not condition:
            errors.append(message)

    baseline_ids = [m["id"] for m in coverage["mechanisms"]]
    documented_ids = [m["id"] for m in matrix["mechanisms"]]
    check(documented_ids == baseline_ids, "Baseline mechanism IDs/order differ")
    check(len(set(documented_ids)) == len(documented_ids), "Duplicate mechanisms")
    check(matrix["reviewedCode"] == manifest["codeCommit"], "Conflicting code SHA")
    check(matrix["baselineDirectory"] == manifest["directoryCommit"], "Conflicting directory SHA")
    for item in manifest["originals"] + manifest["codeSources"] + manifest["metadata"]:
        path = ROOT / item["path"]
        check(path.is_file(), f"Missing evidence: {path}")
        if path.is_file():
            check(hashlib.sha256(path.read_bytes()).hexdigest() == item["sha256"],
                  f"Evidence changed from reviewed snapshot: {item['path']}")
    case_ids = {x["id"] for x in tests["cases"]}
    for item in matrix["mechanisms"] + matrix["addenda"]:
        check(bool(item["acceptance_cases"]), f"No test link: {item['id']}")
        check(set(item["acceptance_cases"]) <= case_ids, f"Unknown test ID: {item['id']}")
        check(set(item["abstractions"]) <= {f"K{i}" for i in range(1, 8)},
              f"Unknown abstraction: {item['id']}")
    source = "\n".join((ROOT / p).read_text() for p in (
        "rust-game/src/engine.rs", "rust-game/src/rules.rs", "rust-game/tests/service.rs"))
    for case in tests["cases"]:
        for name in case["existingRustTests"]:
            check(re.search(r"fn\s+" + re.escape(name) + r"\s*\(", source),
                  f"Missing claimed existing test: {name}")
        check(case["fullyPassed"] is False, f"Future case falsely marked passed: {case['id']}")
    for path in DESIGN.rglob("*.md"):
        for raw in re.findall(r"\]\(([^)]+)\)", path.read_text()):
            target = raw.strip("<>")
            if "://" in target or target.startswith("#"):
                continue
            check((path.parent / target.split("#", 1)[0]).exists(),
                  f"Broken documentation link: {path.name} -> {target}")
    print(json.dumps({"ok": not errors, "baselineMechanisms": len(documented_ids),
                      "laterAdditions": len(matrix["addenda"]), "acceptanceCases": len(case_ids),
                      "errors": errors, "scope": "documentation/provenance only"},
                     ensure_ascii=False, indent=2))
    return bool(errors)


if __name__ == "__main__":
    raise SystemExit(main())
