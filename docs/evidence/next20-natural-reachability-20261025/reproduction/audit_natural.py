from pathlib import Path
import collections, hashlib, json
root=Path(__file__).resolve().parent
source=Path(json.loads((root/'frozen-source.json').read_text())['source'])
raw={c['id']:c for c in json.loads((source/'rust-game/data/cards.json').read_text())['cards']}
cases=[]
for path in sorted((root/'runs/fixed-native-06/native').glob('*.jsonl')):
    result={'file':str(path),'raw_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'assets':[],'milestones':[]}
    previous=None;count=0;commands=set();initial=None;complete=None;moving=None
    with path.open() as stream:
        for line in stream:
            row=json.loads(line)
            if row['kind']=='initial':
                initial=row;catalog={c['id']:c for c in row['catalog']['cards']};result['case']=row['case'];result['seed']=row['seedDecimal'];decks=[]
                for seat,d in enumerate(row['decks']):
                    by_name=collections.Counter();total=0
                    for e in d['cards']:
                        c=catalog[e['cardId']];n=e['count'];total+=n;by_name[c['name']]+=n
                        assert n<=3
                        if raw[e['cardId']].get('unique'):assert n==1
                        assert c['supported'] or e['cardId'] in ['BQ034','CQ25','XG25']
                        assert e['cardId'] not in ['BQ009','BQ033']
                    assert total==50 and max(by_name.values())<=3
                    decks.append({'seat':seat,'cards':total,'max_printed_name_count':max(by_name.values()),'candidate_entries':[e for e in d['cards'] if not catalog[e['cardId']]['supported']]})
                result['decks']=decks;previous=row['state']
            elif row['kind']=='step':
                assert row['state']==previous;assert row['expected']['outcome']=='accepted';assert len(row['views'])==2
                assert row['command']['commandId'] not in commands;commands.add(row['command']['commandId'])
                before=json.loads(row['state']);after=json.loads(row['expected']['state']);g=before['game'];n=after['game'];seat=row['seat'];session=row['command']['action'];action=session.get('action',{})
                assert row['command']['expectedVersion']==before['revision']
                if session['kind']=='game' and action['kind']=='asset':
                    c=next(c for c in g['players'][seat]['hand'] if c['id']==action['cardId'])
                    assert len(n['players'][seat]['assets'])==len(g['players'][seat]['assets'])+1
                    result['assets'].append({'command':row['command']['commandId'],'seat':seat,'definition':c['definition'],'hand_instance':c['id'],'turn':g['turn']})
                if g['players'][0]['society_zone'].get('secrecy',0)==0 and n['players'][0]['society_zone'].get('secrecy',0)>0:
                    assert any(c['definition']=='CQ07' for reg in g['regions'] for c in reg['cards'])
                    result['milestones'].append({'kind':'CQ07_actual_moon_arrival','command':row['command']['commandId'],'before':0,'after':n['players'][0]['society_zone']['secrecy']})
                if not any(c['definition']=='XQ14' for c in g['players'][0]['graveyard']) and any(c['definition']=='XQ14' for c in n['players'][0]['graveyard']):
                    result['milestones'].append({'kind':'real_XQ14_grave_arrival','command':row['command']['commandId']})
                if session['kind']=='game' and action['kind']=='choose' and g.get('pending',{}):
                    p=g['pending']
                    if p['choice']['kind']=='next20-cq25-search':
                        selected=action['selected'][0];where=[zone for zone in ['deck','graveyard'] if any(c['id']==selected and c['definition']=='XG25' for c in g['players'][seat][zone])];assert len(where)==1
                        result['milestones'].append({'kind':'CQ25_nonempty_search','command':row['command']['commandId'],'source_zone':where[0],'actual_instance':selected})
                if session['kind']=='game' and action.get('abilityId')=='next20-xg25-move':
                    sealed=next(s for s in g['sealed_cards'] if s['card']['id']==action['cardId']);assert sealed['card']['owner']==seat==0
                    host=next(c for reg in g['regions'] for c in reg['cards'] if c['id']==sealed['host_id']);assert host['controller']==1
                    target=action['targetId'];old_region=next(i for i,reg in enumerate(g['regions']) if any(c['id']==target for c in reg['cards']))
                    moving={'target':target,'activation_command':row['command']['commandId'],'owner_payer':seat,'host_controller':host['controller'],'from_region':old_region,'usage_after_activation':n['turn_ability_usage']}
                if moving:
                    new_region=next((i for i,reg in enumerate(n['regions']) if any(c['id']==moving['target'] for c in reg['cards'])),None)
                    if new_region is not None and new_region!=moving['from_region']:
                        result['milestones'].append({'kind':'XG25_sealed_owner_action_on_enemy_host',**moving,'resolution_command':row['command']['commandId'],'to_region':new_region});moving=None
                previous=row['expected']['state'];count+=1
            elif row['kind']=='complete':complete=row
    assert initial and complete and complete['commands']==count and complete['finalState']==previous and moving is None
    result.update(commands=count,turn=json.loads(previous)['game']['turn'],details=complete['details'],views=count*2+2)
    result['asset_commands_by_seat']=dict(collections.Counter(x['seat'] for x in result['assets']))
    cases.append(result)
assert len(cases)==3
code=(source/'rust-game/src/next20_natural_room_tests.rs').read_text()
for forbidden in ['make_card(','.assets.push(','.graveyard.push(','society_zone.secrecy=','begin_window(']:assert forbidden not in code,forbidden
out={'source_commit':json.loads((root/'frozen-source.json').read_text())['source_commit'],'publicAdmission':False,'candidateConstructionOnly':['BQ034','CQ25','XG25'],'native_commands':sum(c['commands'] for c in cases),'no_runtime_fixture_injection_in_test_source':True,'cases':cases}
(root/'natural-reachability-audit.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'commands':out['native_commands'],'cases':[{k:c[k] for k in ['case','seed','commands','turn','asset_commands_by_seat','milestones']} for c in cases]},ensure_ascii=False,indent=2))
