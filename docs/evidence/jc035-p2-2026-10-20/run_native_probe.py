import datetime
import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys

root = pathlib.Path(__file__).parent
mode = sys.argv[1]
assert mode in {"before", "after"}
destination = root / mode
destination.mkdir(exist_ok=True)
source = root / "native_restore_probe.rs"
target = pathlib.Path(os.environ["CARGO_TARGET_DIR"]) / "release"
library = target / "libhegemony_server.rlib"
assert library.is_file()
binary = destination / "native-restore-probe"
assert not binary.exists(), "Preserve the previous probe; do not overwrite"
command = ["rustc", "--edition=2021", "--crate-name", "jc035_p2_native_probe", str(source),
           "--extern", "hegemony_server=" + str(library), "-L", "dependency=" + str(target / "deps"),
           "-o", str(binary)]
subprocess.run(command, check=True)
cases = [
    ("original", "original-game.json", "original-room.json", True),
    ("missing-pending", "missing-pending-game.json", "missing-pending-room.json", mode == "before"),
    ("accepted-original", "accepted-original-game.json", "accepted-original-room.json", True),
    ("accepted-missing-frame", "accepted-missing-frame-game.json", "accepted-missing-frame-room.json", mode == "before"),
]
args = [str(binary)]
for label, game, room, expected in cases:
    args.extend([label, str(root / "before" / game), str(root / "before" / room)])
result = subprocess.run(args, check=True, capture_output=True, text=True)
print(result.stdout, end="", flush=True)
observed = {label: (game == "true", room == "true") for label, game, room in
            re.findall(r"^CASE (\S+) GAME_ACCEPTED=(true|false) ROOM_ACCEPTED=(true|false)$", result.stdout, re.M)}
assert len(observed) == len(cases), observed
for label, game, room, expected in cases:
    assert observed[label] == (expected, expected), (label, observed[label], expected)
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
proof = {
    "phase": mode,
    "capturedUtc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "sourceProbeSha256": sha(source),
    "compiledProbeSha256": sha(binary),
    "linkedGameLibrarySha256": sha(library),
    "compiler": subprocess.check_output(["rustc", "--version", "--verbose"], text=True),
    "compileArgv": command,
    "runArgv": args,
    "stdout": result.stdout,
    "observed": {label: {"GameAccepted": values[0], "RoomAccepted": values[1]} for label, values in observed.items()},
    "beforePhaseConfirmsVulnerabilityRatherThanClaimingFunctionalPass": mode == "before",
    "sameExactStateFilesInBothPhases": True,
}
output = destination / "native-two-loaders.json"
with output.open("x") as file:
    json.dump(proof, file, ensure_ascii=False, indent=2)
    file.write("\n")
print(json.dumps({"phase": mode, "observed": proof["observed"], "proof": str(output)}, ensure_ascii=False), flush=True)
