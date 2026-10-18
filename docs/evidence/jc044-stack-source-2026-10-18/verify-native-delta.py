from pathlib import Path
import json,hashlib
r=Path(__file__).parent
a=json.loads((r/'original-native85.json').read_bytes());b=json.loads((r/'native-fixed/native85.json').read_bytes())
assert a['catalog']==b['catalog']
assert a['scenarios']==b['scenarios']
old={row['name']:row for row in a['invalid']};new={row['name']:row for row in b['invalid']}
assert len(old)==35 and len(new)==37
for name,row in old.items():assert new[name]==row,name
names=['stack-deploy-source-identity','stack-reveal-source-identity']
for kind,name in zip(['deploy','reveal'],names):
 assert new[name]['state']==(r/('entry-'+kind+'-skip-bad.room')).read_text()
 assert new[name]['gameError']==new[name]['roomError']=='JC044不能挂接其他牌的堆栈程序'
assert set(a['sourceBindings']['files'])==set(b['sourceBindings']['files'])
changed=[k for k,v in a['sourceBindings']['files'].items() if b['sourceBindings']['files'][k]!=v]
assert sorted(changed)==['rust-game/src/jc044.rs','rust-game/src/jc044_tests.rs'],changed
result={'oldSource':'1979d2420bd1644c8d6b04a16ff2caa23608b9c5','catalogExactlyUnchanged':True,'all13NormalScenariosCompleteTransitionsAndViewsExactlyUnchanged':True,'all35PriorMalformedArchivesExactlyUnchanged':True,'newMalformedArchives':names,'newMalformedStatesByteIdenticalToOriginalReviewReproductions':True,'nativeInputCount':len(b['sourceBindings']['files']),'changedCompilerInputs':changed,'nativeRawSha256':hashlib.sha256((r/'native-fixed/native85.json').read_bytes()).hexdigest()}
(r/'native-delta.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
