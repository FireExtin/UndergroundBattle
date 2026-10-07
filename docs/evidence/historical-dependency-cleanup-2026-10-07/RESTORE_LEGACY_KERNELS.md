# Restore the removed Git-tracked historical kernels

The exact 40 directories and all 209 source file SHA256 values are in `legacy-deletion-receipt.json`. Source commit: `27b2d573073ff7211facc9ebe8feb2826cecbe40`. Run from the game repository root, adjusting only the receipt location if the reviewed packet has moved:

```sh
python3 - <<'PY'
import json, subprocess
with open('docs/evidence/historical-dependency-cleanup-2026-10-07/legacy-deletion-receipt.json') as f:
    receipt = json.load(f)
subprocess.run(['git', 'restore', '--source=' + receipt['sourceCommit'],
                '--worktree', '--', *receipt['deletedDirectories']], check=True)
PY
```

This restores only the exact 40 removed directories. It does not change HEAD, main or the index. Do not restore the entire `rust-game-wasm` tree: current pkg, comparator and core/WIP changes must be preserved. The compact historical identity/opaque-room/catalog fixtures remain usable without restoring any legacy executable.
