from pathlib import Path
import json,subprocess,sys
r=Path(__file__).resolve().parent
p=subprocess.run(json.loads((r/'native-probe-compile-args.json').read_text()))
raise SystemExit(p.returncode)
