"""Compare reviewer-exported states with exact Python integers, not JS numbers."""
import json,sys
from pathlib import Path
p=Path(sys.argv[1]);data=json.loads(p.read_text())
for row in data['reviewStates']:
    assert json.loads(row['actual']) == json.loads(row['expected'])
print(json.dumps({'reviewNativeSemanticStatesEqual':len(data['reviewStates']),
                  'note':'Object export loses key order; no byte-equality claim for review states'}))
