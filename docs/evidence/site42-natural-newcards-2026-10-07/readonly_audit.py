"""Only read local D1. Never export seat tokens or write state to the game."""
import hashlib,json,sqlite3,sys
from pathlib import Path
ROOT=Path('/workspace/game-publication-evidence/site42-natural-newcards')
ROOM='d61b44bbe38eada719bcc4d8'
p=next(Path('/tmp/site42-natural-newcards-local-d1/v3/d1/miniflare-D1DatabaseObject').glob('*.sqlite'))
db=sqlite3.connect('file:'+str(p)+'?mode=ro',uri=True)
def digest(value):return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()
def snapshot():
    raw,version=db.execute('SELECT state,version FROM rooms WHERE id=?',(ROOM,)).fetchone();s=json.loads(raw);g=s['game']
    rows=[]
    for player in g['players']:
        deck=player['deck'];grave=player['graveyard'];hand=player['hand']
        rows.append({'seat':player['seat'],'deckCount':len(deck),'deckOrderSha256':digest([c['id'] for c in deck]),'deckIdentitySetSha256':digest(sorted(c['id'] for c in deck)),'deckIds':[c['id'] for c in deck],
          'graveCount':len(grave),'grave':[[c['id'],c['definition'],c['owner'],c['controller']] for c in grave],'handCount':len(hand),'hand':[[c['id'],c['definition']] for c in hand]})
    return {'roomId':ROOM,'roomVersion':version,'turn':g['turn'],'gameVersion':g['version'],'random':g['random'],'privateStateSha256':digest(s),'players':rows,'dbMode':'read-only','boardInjection':False}
if __name__=='__main__':
    name=sys.argv[1];s=snapshot();(ROOT/name).write_text(json.dumps(s,ensure_ascii=False,indent=2)+'\n');print({k:s[k] for k in ['roomVersion','turn','random','dbMode']})
