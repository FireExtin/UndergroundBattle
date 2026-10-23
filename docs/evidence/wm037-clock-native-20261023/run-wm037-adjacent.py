from pathlib import Path
import json,subprocess,sys
r=Path(__file__).resolve().parent
s=json.loads((r/"wm037-clock-20261023/normal-adjacent-selection.json").read_text())["selection"]
sys.exit(subprocess.call([sys.executable,str(r/"run-wm037-clock.py"),"native-normal-adjacent-08","native","--lib","--",*s,"--nocapture"]))
