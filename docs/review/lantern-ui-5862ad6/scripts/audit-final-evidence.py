import copy,hashlib,json,pathlib,sqlite3,subprocess,sys
run=sys.argv[1] if len(sys.argv)>1 else 'four-seat-run-7'
p=pathlib.Path('/workspace/lantern-ui-e2e-evidence')/run
progress=json.loads((p/'progress.json').read_text())
assert progress['checks']['success'],progress['checks'].get('error')
snapshot=json.loads((p/'final-snapshot.json').read_text())
requests=json.loads((p/'requests.json').read_text())
room=progress['roomId']
assert snapshot['room']['id']==room
journal=snapshot['journal'];assert [r['version'] for r in journal]==list(range(1,snapshot['room']['version']+1))
receipts={r['command_id']:r for r in snapshot['commands']}
assert len(receipts)==len(snapshot['commands'])
accepted=[];rejected=[]
for request in requests:
 if not request['url'].endswith('/commands'):continue
 cid=request['body']['commandId']
 if request['status']==200:
  assert cid in receipts
  assert json.loads(receipts[cid]['response'])==request['response']
  accepted.append(request)
 else:
  assert cid not in receipts
  rejected.append(request)
assert len({r['body']['commandId'] for r in accepted})==len(receipts)
assert len(snapshot['seats'])==4 and len(snapshot['entries'])==4
paths=list((p/'sqlite/d1/miniflare-D1DatabaseObject').glob('*.sqlite'))
db=next(f for f in paths if f.name!='metadata.sqlite')
def hashes():
 return {f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in [db,pathlib.Path(str(db)+'-wal')] if f.exists()}
before=hashes()
audit=subprocess.run(['/tmp/lantern-ui-native-target/debug/hegemony-audit',str(db),room],capture_output=True,text=True)
(p/'native-replay-audit.stderr').write_text(audit.stderr)
assert audit.returncode==0,audit.stderr
native=json.loads(audit.stdout);assert native['matches']
after=hashes();assert before==after
(p/'native-replay-audit.json').write_text(json.dumps(native,indent=2))
proof={'auditMode':'Native Store::open_read_only, consistent SELECT read transaction; no migrations or room writes','persistentDatabaseFilesUnchanged':True,'beforeSha256':before,'afterSha256':after,'sharedMemoryLockBytesExcluded':True,'audit':native}
(p/'native-replay-read-only-proof.json').write_text(json.dumps(proof,indent=2))
effect=next(s for s in progress['steps'] if s['command']['action']['kind']=='submitResponse' and s['command']['action'].get('action',{}).get('abilityId')=='funeral')
pre=progress['steps'][progress['steps'].index(effect)-1]['view']
ready=lambda view:sum(c['controller']=='p2' and not c['exhausted'] for c in view['assets'])
assert ready(pre)-ready(effect['view'])==1
final_view=json.loads(snapshot['commands'][-1]['response'])
same=[c for region in final_view['regions'] if region['index']==0 for c in region['characters'] if c.get('cardId')=='JC125']
assert len({c['instanceId'] for c in same})==3 and {c['controller'] for c in same}=={'p0','p2'}
summary={'run':run,'roomId':room,'fixedSeed':9,'finalTurn':progress['checks']['finalTurn'],'version':snapshot['room']['version'],'journalEntries':len(journal),'acceptedUniqueCommands':len(receipts),'acceptedCommandHttpResponses':len(accepted),'duplicateCommandHttpResponses':len(accepted)-len(receipts),'rejectedCommands':[{'id':r['body']['commandId'],'status':r['status']} for r in rejected],'allSuccessfulPostReceiptsMatchStoredResponses':True,'journalVersionsContiguous':True,'fourSeats':True,'fourEntryReceipts':True,'nativeReplayMatches':True,'persistedDigest':native['persistedDigest'],'stateStringSha256':hashlib.sha256(snapshot['room']['state'].encode()).hexdigest(),'initialStateStringSha256':hashlib.sha256(snapshot['room']['initial_state'].encode()).hexdigest(),'databaseRelativePath':str(db.relative_to(p))}
summary['funeralAssetsPaidExactlyOne']=True
summary['sameNameRegion0IndependentInstances']=[{k:c[k] for k in ['instanceId','owner','controller','cardId']} for c in same]
(p/'evidence-integrity.json').write_text(json.dumps(summary,indent=2))
# Shared review copies omit disposable local seat bearer tokens. The actual
# D1 database, browser storage, and original receipts stay intact in the cloud.
def redact(value):
 if isinstance(value,dict):return {k:('[redacted local seat token]' if k=='token' else redact(v)) for k,v in value.items()}
 if isinstance(value,list):return [redact(v) for v in value]
 return value
public_snapshot=copy.deepcopy(snapshot)
for entry in public_snapshot['entries']:
 entry['originalResponseSha256']=hashlib.sha256(entry['response'].encode()).hexdigest()
 entry['response']=json.dumps(redact(json.loads(entry['response'])),ensure_ascii=False,separators=(',',':'))
(p/'final-snapshot.redacted.json').write_text(json.dumps(public_snapshot,ensure_ascii=False,indent=2))
(p/'requests.redacted.json').write_text(json.dumps(redact(requests),ensure_ascii=False,indent=2))
print(json.dumps(summary,ensure_ascii=False))
