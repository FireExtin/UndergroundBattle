from pathlib import Path
import json,hashlib,shutil
root=Path(__file__).resolve().parent
env=json.loads((root/'native-env.json').read_text());source=Path(env['GREEN_EVIDENCE_DIR'])
outputs={k:root/'selected-traces-v2'/k for k in ['xq37','jz22','death']}
for p in outputs.values():p.mkdir(parents=True,exist_ok=False)
manifest=[]
for p in sorted(source.glob('step-*.json')):
 raw=p.read_bytes();row=json.loads(raw);game=json.loads(row['state'])['game']
 pending=game.get('pending') or {};resolution=pending.get('resolution') or {}
 if not isinstance(resolution,dict):resolution={}
 declarations=[resolution.get('Declare',{}).get('declaration',{})]
 for effect in game.get('effects',[]):
  if isinstance(effect,dict) and 'Declare' in effect:declarations.append(effect['Declare'].get('declaration',{}))
 definitions={d.get('source',{}).get('card',{}).get('definition') for d in declarations}
 definitions.update(item.get('frame',{}).get('source',{}).get('card',{}).get('definition') for item in game.get('stack',[]) if item.get('frame'))
 definitions.update(c.get('definition') for region in game['regions'] for c in region['cards'])
 action=(row.get('command',{}).get('action',{}).get('action') or {})
 definitions.update(c['definition'] for player in game['players'] for c in player['hand'] if c['id']==action.get('cardId'))
 groups=[]
 if 'XQ37' in definitions:groups.append('xq37')
 if 'JZ22' in definitions:groups.append('jz22')
 if definitions.intersection({'JC031','JC045','JC090'}):groups.append('death')
 for group in groups:
  dest=outputs[group]/p.name;dest.write_bytes(raw);manifest.append({'source':str(p),'selected':str(dest),'sha256':hashlib.sha256(raw).hexdigest()})
(root/'selected-traces-v2/manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'selected':{k:len(list(p.glob('*.json'))) for k,p in outputs.items()},'sourceCommands':len(list(source.glob('step-*.json')))}))
