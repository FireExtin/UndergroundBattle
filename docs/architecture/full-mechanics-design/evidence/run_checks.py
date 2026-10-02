#!/usr/bin/env python3
"""Run reviewed-code checks in a temporary archive; never mutate runtime sources."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
MANIFEST = json.loads((HERE / "manifest.json").read_text())
RECIPES = {
    "native": [["cargo", "test", "--locked", "-p", "hegemony-server"]],
    "storage": [["node", "--test", "sites/test/storage-retry.test.mjs"]],
    "wasm": [["bash", "rust-game-wasm/verify.sh"]],
    "receipt-probe": [["cargo", "run", "--locked", "-p", "hegemony-server",
                       "--example", "design_receipt_probe"]],
    "coverage": [["python3", "tools/factions/validate_coverage.py", "--allow-code-drift"],
                 ["python3", "-m", "unittest", "discover", "-s", "tools/factions",
                  "-p", "test_coverage.py"]],
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--list", action="store_true")
    group.add_argument("--run", choices=RECIPES)
    args = parser.parse_args()
    if args.list:
        print(json.dumps({"commit": MANIFEST["codeCommit"], "checks": RECIPES,
                          "known_failure": "coverage",
                          "observes_bug_not_correctness": "receipt-probe"}, indent=2))
        return 0
    work = Path(tempfile.mkdtemp(prefix="ub-design-check-"))
    archive = work / "source.tar"
    with archive.open("wb") as out:
        subprocess.run(["git", "archive", MANIFEST["codeCommit"]], cwd=ROOT,
                       stdout=out, check=True)
    subprocess.run(["tar", "-xf", str(archive), "-C", str(work)], check=True)
    archive.unlink()
    if args.run == "receipt-probe":
        example = work / "rust-game/examples/design_receipt_probe.rs"
        example.parent.mkdir(exist_ok=True)
        shutil.copyfile(HERE / "native_receipt_probe.rs", example)
    env = os.environ.copy()
    env.setdefault("CARGO_TARGET_DIR", str(work / "target"))
    results = []
    for i, command in enumerate(RECIPES[args.run]):
        log = work / f"{args.run}-{i}.log"
        try:
            with log.open("w") as out:
                result = subprocess.run(command, cwd=work, env=env, stdout=out,
                                        stderr=subprocess.STDOUT)
            code = result.returncode
        except OSError as error:
            log.write_text(str(error) + "\n")
            code = 127
        results.append({"command": command, "exitCode": code, "log": str(log)})
    print(json.dumps({"commit": MANIFEST["codeCommit"], "temporaryCopy": str(work),
                      "results": results}, ensure_ascii=False, indent=2))
    return 0 if all(r["exitCode"] == 0 for r in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
