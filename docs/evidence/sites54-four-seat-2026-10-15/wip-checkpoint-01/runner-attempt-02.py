#!/usr/bin/env python3
"""Owned public Site54 four-seat PC game, normal deck UI and gameplay UI only.

No seed selection, midgame setup, direct API moves or hosting owner bypass.
Each policy reads its own seat's hand and the public board. Evidence never
writes hands, credentials, request headers, cookies or browser profiles.
"""
import argparse,asyncio,importlib.util,json,re,time,traceback
from collections import Counter
from pathlib import Path
from playwright.async_api import async_playwright,expect,TimeoutError as BrowserTimeout

ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('abi77_public_frozen',ROOT/'docs/evidence/sites-abi77-2026-10-15/online_pc_abi77.py')
frozen=importlib.util.module_from_spec(spec);spec.loader.exec_module(frozen)
from full_game_persistent import FullGame
from attachment_v026 import AttachmentRun
from common import public_view,write_json

NEW={'JC046','JC060','JC095'}
ENGINE=frozen.ENGINE;POOL=frozen.POOL;CANDIDATE=frozen.CANDIDATE
IDS=['JC046','JC060','JC095','JC084','JC085','JC088','JZ49','XQ37','WM056','JC042','XQ17','JZ31','XQ18','JC029','JZ22','JC069','JC125']
DECK={card_id:2 if card_id=='JC125' else 3 for card_id in IDS}
SACRIFICE='选择一个自己操控的角色牺牲'

class TeamsRun(frozen.PublicGame):
    def __init__(self,p,service,output,resume=False,budget=3000):
        FullGame.__init__(self,p,service,output,'teams',budget)
        self.resume_run=resume;self.refresh_paused_peers=False;self.initial_probe_done=True
        self.online_checks=[];self.room_id=None;self.attempt=1
        self.newcards={key:{'drawnSeats':[],'faceupInstances':[]} for key in sorted(NEW)}
        self.jc046_sequences={};self.jc095_entries=[];self.aura_checks=[];self.aura_seen=set();self.choice_seen=set()
        self.mulligan_probe_done=False;self.privacy_snapshots=[];self.command_observations=[]
        if resume:
            prior=json.loads((self.output/'teams-summary.json').read_text())
            assert prior['mode']=='teams' and not prior['passed']
            self.post_count=prior['uiPostCount'];self.post_results=prior['postResults']
            self.online_checks=prior['checks'];self.room_id=prior['roomId'];self.attempt=prior['attempt']+1
            self.newcards=prior['newCards'];self.jc046_sequences=prior['jc046Sequences'];self.jc095_entries=prior['jc095Entries']
            self.aura_checks=prior['auraChecks'];self.aura_seen={tuple(x['fingerprint']) for x in self.aura_checks}
            self.privacy_snapshots=prior['choicePrivacy'];self.choice_seen={x['choiceId'] for x in self.privacy_snapshots}
            self.mulligan_probe_done=prior['mulliganProbeDone'];self.expiry_races=prior['expiryRaces']
            self.expected_errors=prior['expectedBrowserErrors'];self.coverage.update(prior['coverage'])
            log=self.output/'teams-actions.jsonl'
            if log.exists():self.records=[json.loads(line) for line in log.read_text().splitlines()]
            expired=[r for r in self.records if r['httpStatus']==409 and r.get('error',{}).get('error')=='window_expired' and r['action'].get('kind')=='passResponse']
            for row in expired:
                if not any(e.get('commandId')==row['commandId'] for e in self.expiry_races):
                    self.expiry_races.append(dict(kind='prior-expired-response-rejection',commandId=row['commandId'],seat=row['seat'],beforeVersion=row['before']['version'],afterVersion=row['after']['version'],httpStatus=409,error='window_expired'))
            for e in prior['browserErrors']:
                assert e['kind']=='console' and e['message']=='Failed to load resource: the server responded with a status of 409 (Conflict)' and any(r['seat']==e['seat'] for r in expired), 'Unexpected prior browser error requires defect review'
                self.expected_errors.append(dict(**e,cause='prior-verified-expired-response-409'))
            (self.output/f'attempt-{prior["attempt"]:02d}-summary.json').write_text(json.dumps(prior,ensure_ascii=False,indent=2)+'\n')

    def check(self,name,**data):
        self.online_checks.append(dict(name=name,passed=True,**data))
        write_json(self.output/'pc-checks.json',dict(mode='teams',candidate=CANDIDATE,version=54,
            publicAnonymous=True,viewport=[1366,900],productionUrl=self.base,checks=self.online_checks))

    async def new_seat(self,seat,session=None):
        page=await AttachmentRun.new_seat(self,seat,session)
        await page.set_viewport_size({'width':1366,'height':900});await page.bring_to_front()
        if self.resume_run and self.room_id:
            # Unjoined later seats may still have only their own saved deck.
            if seat==0:await page.wait_for_function('(id)=>window.__cloudView?.roomId===id',arg=self.room_id,timeout=30000)
        if seat==0:
            c=await page.evaluate('window.__cloudCatalog')
            assert c['engineVersion']==ENGINE and c['cardPoolVersion']==POOL and len(c['cards'])==143
            self.catalog={x['id']:x for x in c['cards']}
            health=await page.evaluate('async()=>{const r=await fetch("/api/health");if(!r.ok)throw Error("health "+r.status);return r.json()}')
            assert health['engineVersion']==ENGINE
            manifest=json.loads((ROOT/'docs/evidence/sites-abi77-2026-10-15/client-files-158.json').read_text())['files']
            entries=[x for x in manifest if x['path'].startswith('assets/index-')]
            assert len(entries)==2
            matches=await page.evaluate('''async rows=>{const out=[];for(const row of rows){const r=await fetch('/'+row.path,{cache:'no-cache'});const b=await r.arrayBuffer();const h=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',b))).map(x=>x.toString(16).padStart(2,'0')).join('');out.push({path:row.path,status:r.status,bytes:b.byteLength,sha256:h,passed:r.ok&&b.byteLength===row.bytes&&h===row.sha256})}return out}''',entries)
            assert all(x['passed'] for x in matches)
            write_json(self.output/f'public-identity-attempt-{self.attempt:02d}.json',dict(candidate=CANDIDATE,engine=ENGINE,cardPool=POOL,cards=143,health=health,entryAssets=matches,full158PriorReceipt='docs/evidence/sites-abi77-2026-10-15/online-identity.json',full158RehashedThisRun=False,ownerBypass=False))
            self.check('public-v54-engine-pool-and-entry-assets',engine=ENGINE,cardPool=POOL,matchingEntryFiles=2)
        return page

    async def build_deck(self,seat):
        page=self.pages[seat]
        name=f'四人77新卡验收-{seat+1}'
        await page.get_by_role('button',name='展开组卡工作台',exact=True).click()
        settings=page.get_by_role('button',name='牌组设置与模板',exact=True)
        if await settings.count():await settings.click()
        await page.get_by_role('button',name='新建空白牌组',exact=True).click()
        await page.get_by_role('textbox',name='牌组名称',exact=True).fill(name)
        for card_id,count in DECK.items():
            card=self.catalog[card_id];assert card['supported'] and card['kind']=='character'
            await page.get_by_role('searchbox',name='检索卡牌',exact=True).fill(card_id)
            label=f'{card["name"]}（{card_id}）'
            await page.get_by_role('button',name='添加 '+label,exact=True).click()
            counter=page.get_by_role('spinbutton',name=label+'张数',exact=True)
            await counter.fill(str(count));await expect(counter).to_have_value(str(count))
        save=page.get_by_role('button',name='保存并选择此牌组',exact=True)
        await expect(save).to_be_enabled();await save.click()
        close=page.get_by_role('button',name='返回大厅组卡',exact=True)
        if await close.count() and await close.is_visible():await close.click()
        await expect(page.locator('.hg-entry-form')).to_contain_text(name)
        self.check('legal-50-card-deck-built-and-selected-through-ui',seat=seat,cardCount=sum(DECK.values()),newCardCopies={x:DECK[x] for x in sorted(NEW)})

    async def setup(self):
        host=await self.new_seat(0)
        if not self.room_id:
            await self.build_deck(0)
            await host.get_by_label('你的称呼',exact=True).fill('四人77甲')
            await host.get_by_role('button',name=re.compile('四人协作')).click()
            await host.get_by_role('button',name='创建牌桌 →',exact=True).click()
            await host.wait_for_function('window.__cloudView?.status==="lobby"')
            self.room_id=(await self.view(0))['roomId'];await self.checkpoint('owned-room-created')
        for seat in range(1,4):
            page=await self.new_seat(seat)
            if self.resume_run and any(p['id']==f'p{seat}' for p in (await self.view(0))['players']):
                await page.wait_for_function('(id)=>window.__cloudView?.roomId===id',arg=self.room_id,timeout=30000)
            await asyncio.sleep(.3)
            own=await self.view(seat)
            if not own:
                current=await self.view(0)
                assert current['status']=='lobby' and all(p['id']!=f'p{seat}' for p in current['players']), 'Existing seat must restore its own original browser session'
                await self.build_deck(seat)
                await page.get_by_role('button',name='邀请码加入',exact=True).click()
                await page.get_by_label('你的称呼',exact=True).fill(f'四人77{seat+1}')
                await page.get_by_label('邀请码',exact=True).fill(current['inviteCode'])
                await page.get_by_role('button',name='加入牌桌 →',exact=True).click()
            await page.wait_for_function('(id)=>window.__cloudView?.roomId===id',arg=self.room_id,timeout=30000)
            assert (await self.view(seat))['you']==f'p{seat}'
            await self.checkpoint('owned-seat-restored-or-joined')
        await self.sync(max(v['version'] for v in await self.views()))
        if (await self.view(0))['status']=='lobby':
            for seat in range(4):
                view=await self.view(seat)
                own=next(p for p in view['players'] if p['id']==view['you'])
                if not own['ready']:await self.legal_button(seat,next(a for a in view['legalActions'] if a['kind']=='ready'))
            await self.legal_button(0,next(a for a in (await self.view(0))['legalActions'] if a['kind']=='start'))
        self.check('four-owned-seats-native77',roomId=self.room_id)

    async def sync(self,version):
        await super().sync(version)
        views=await self.views()
        for view in views:
            choice=view.get('pendingChoice')
            if choice:assert choice['playerId']==view['you']

    async def submit(self,seat,click,intended_kind):
        before_records=len(self.records);before_errors=len(self.errors)
        try:return await super().submit(seat,click,intended_kind)
        except AssertionError:
            row=self.records[-1] if len(self.records)>before_records else None
            if not (row and row['httpStatus']==409 and row.get('error',{}).get('error')=='window_expired'
                    and row['action'].get('kind')=='passResponse' and row.get('after')
                    and row['after']['version']>row['before']['version']):raise
            await self.sync(row['after']['version']);current=await self.view(seat)
            window=current.get('responseWindow') or {};member=next((m for m in window.get('members',[]) if m['playerId']==current['you']),{})
            assert window.get('id')!=row['action']['windowId'] or member.get('status')!='undecided' or member.get('deadlineMs',0)<=current['serverNowMs']
            for e in self.errors[before_errors:]:
                if e['seat']==seat and e['kind']=='console' and e['message']=='Failed to load resource: the server responded with a status of 409 (Conflict)':
                    self.errors.remove(e);self.expected_errors.append(dict(**e,cause='verified-expired-response-409'))
            self.expiry_races.append(dict(kind='actual-passResponse-expired-and-view-advanced',commandId=row['commandId'],intendedKind=intended_kind,seat=seat,beforeVersion=row['before']['version'],afterVersion=current['version'],httpStatus=409,error='window_expired'))
            return current

    async def observe(self):
        views=await self.views()
        for seat,v in enumerate(views):
            assert v['roomId']==self.room_id and v['you']==f'p{seat}'
            for c in v['hand']:
                if c.get('cardId') in NEW and seat not in self.newcards[c['cardId']]['drawnSeats']:
                    self.newcards[c['cardId']]['drawnSeats'].append(seat)
            board=[c for r in v['regions'] for c in r['characters']]
            teams={p['id']:p['team'] for p in v['players']}
            for c in board:
                if not c['faceDown'] and c.get('cardId') in NEW and c['instanceId'] not in self.newcards[c['cardId']]['faceupInstances']:
                    self.newcards[c['cardId']]['faceupInstances'].append(c['instanceId'])
                if c['faceDown']:continue
                definition=self.catalog[c['cardId']]
                eligible=definition.get('magicIcon')=='None' and definition['kind']=='character'
                sources=[a for a in board if not a['faceDown'] and a.get('cardId')=='JC060' and teams[a['controller']]==teams[c['controller']]] if eligible else []
                expected=len(sources);actual=sum(d['count'] for d in c.get('currentDomains',[]) if d['magic']=='心灵')
                assert actual==expected, f'JC060 public domain projection differs for {c["instanceId"]}'
                fingerprint=(seat,c['instanceId'],expected,c['controller'])
                if expected and fingerprint not in self.aura_seen:
                    self.aura_seen.add(fingerprint)
                    self.aura_checks.append(dict(fingerprint=list(fingerprint),viewer=seat,version=v['version'],targetId=c['instanceId'],controller=c['controller'],sourceCount=expected,actualMindCount=actual,teammateSource=any(a['controller']!=c['controller'] for a in sources),passed=True))
        primary=views[0]
        for entry in self.jc095_entries:
            if entry.get('resolved'):continue
            r=next((r for r in primary['regions'] if r['index']==entry['region'] and r['cardId']==entry['regionCardId']),None)
            if not r:continue
            if r['influence'][entry['enemyTeam']]==0 and r['influence'][entry['team']]==entry['ownBefore']+entry['enemyBefore']:
                entry.update(resolved=True,after=list(r['influence']),version=primary['version'],nonzeroConversionObserved=entry['enemyBefore']>0)
        return views

    async def privacy(self,seat,choice):
        await self.sync((await self.view(seat))['version']);views=await self.views();own=views[seat]
        assert own.get('pendingChoice',{}).get('id')==choice['id']
        assert all(v['version']==own['version'] for v in views), 'Choice privacy snapshot must use one stable revision'
        for viewer,v in enumerate(views):
            if viewer==seat:assert v['pendingChoice']['playerId']==v['you']
            else:assert v.get('pendingChoice') is None
        if choice['id'] not in self.choice_seen:
            self.choice_seen.add(choice['id'])
            self.privacy_snapshots.append(dict(choiceId=choice['id'],kind=choice['kind'],title=choice['title'],chooserSeat=seat,version=own['version'],viewerPrivateOptionCounts=[len(v['pendingChoice']['options']) if v.get('pendingChoice') else 0 for v in views],fourViewsChecked=True,otherSeatsHaveNoPrivateChoice=True,passed=True))

    async def confirm_draft(self,seat,view,wanted,probe=False):
        page=self.pages[seat];choice=view['pendingChoice'];panel=page.locator('.hg-choice')
        await panel.wait_for(state='visible')
        for option in choice['options']:
            button=page.locator('[data-choice-option='+json.dumps(option['id'])+']')
            selected=await button.get_attribute('aria-pressed')=='true'
            if selected!=(option['id'] in wanted):await button.click()
        if probe and wanted:
            before=self.post_count;option_id=wanted[0]
            await page.reload(wait_until='domcontentloaded')
            await expect(page.locator('[data-choice-option='+json.dumps(option_id)+']')).to_have_attribute('aria-pressed','true')
            restored=await self.view(seat)
            assert restored['roomId']==self.room_id and restored['you']==f'p{seat}' and restored['pendingChoice']['id']==choice['id']
            assert self.post_count==before
            self.check('four-seat-choice-refresh-restores-own-draft',seat=seat,choiceId=choice['id'],kind=choice['kind'],extraPosts=0)
            await self.screenshot(f'draft-restored-{choice["kind"]}-p{seat}',seat)
        prior=self.post_count;version=(await self.view(seat))['version']
        confirm=page.locator('.hg-choice').get_by_role('button',name='确认选择',exact=True)
        body=await self.submit(seat,lambda:confirm.evaluate('(button)=>{button.click();button.click()}') if probe else confirm.click(),'choose')
        if probe:
            await asyncio.sleep(.3)
            assert self.post_count==prior+1 and body['version']==version+1
            actual=self.records[-1]['action'];actual=actual.get('action',actual)
            assert actual['selected']==wanted
            self.check('four-seat-rapid-choice-clicks-one-command',seat=seat,choiceId=choice['id'],kind=choice['kind'],posts=1,versionBefore=version,versionAfter=body['version'])
        self.coverage['choice:'+choice['kind']]+=1

    async def choose(self,seat,view):
        choice=view['pendingChoice'];await self.privacy(seat,choice)
        if choice['kind']=='mulligan':
            kept_body=False;wanted=[]
            for o in choice['options']:
                c=o.get('card',{});key=c.get('cardId')
                if key in NEW:continue
                if not kept_body and c.get('cost',99)<=1:kept_body=True;continue
                wanted.append(o['id'])
            probe=not self.mulligan_probe_done and bool(wanted)
            await self.confirm_draft(seat,view,wanted,probe)
            if probe:self.mulligan_probe_done=True
            return
        if choice['kind']=='target' and choice['title']==SACRIFICE:
            assert choice['min']==choice['max']==1 and not choice['allowDecline']
            assert await self.pages[seat].locator('.hg-choice').get_by_role('button',name='跳过此选择',exact=True).count()==0
            option=min(choice['options'],key=lambda o:(o.get('card',{}).get('cardId') in NEW,self.card_value(o.get('card',{}))))
            frame=next((f['id'] for f in reversed(view['stack']) if f.get('cardId')=='JC046'),None)
            key=frame or getattr(self,'last_046_reveal','unbound')
            assert key!='unbound','Mandatory choice must bind an observed JC046 reveal/frame'
            record=self.jc046_sequences.setdefault(key,dict(frameId=frame,seats=[],choiceIds=[],completedFourSeats=False))
            probe=seat not in record['seats']
            await self.confirm_draft(seat,view,[option['id']],probe)
            record['seats'].append(seat);record['choiceIds'].append(choice['id'])
            record['completedFourSeats']=set(record['seats'])=={0,1,2,3}
            return
        await FullGame.choose(self,seat,view)

    def policy(self,seat,view):
        legal=view.get('legalActions',[])
        if not legal:return None
        hand={c['instanceId']:c for c in view['hand']};board=[c for r in view['regions'] for c in r['characters']]
        own=[c for c in board if c['controller']==view['you']]
        all_have_bodies=all(any(c['controller']==p['id'] for c in board) for p in view['players'] if not p['eliminated'])
        hidden46={c['instanceId'] for c in own if c['faceDown'] and c.get('cardId')=='JC046'}
        covered_four=any(row['completedFourSeats'] for row in self.jc046_sequences.values())
        reveal=next((a for a in legal if a['kind']=='reveal' and a.get('cardId') in hidden46),None)
        if reveal and not covered_four and (all_have_bodies or view['turn']>=7):return reveal
        assets=[a for a in legal if a['kind']=='asset']
        owned_assets=[c for c in view['assets'] if c['controller']==view['you']]
        if assets and len(owned_assets)<8 and len(hand)>1:
            color_counts=Counter(c.get('color') for c in owned_assets);held=Counter(c.get('cardId') for c in hand.values())
            wanted={'黑':3,'红':1,'灰':1,'蓝':2}
            def score(a):
                c=hand[a['cardId']];key=c.get('cardId');d=self.catalog[key]
                bonus=25 if color_counts[d['color']]<wanted.get(d['color'],0) else 0
                if key not in NEW and d['color']=='灰' and held['JC060'] and color_counts['灰']==0:bonus+=35
                if key not in NEW and d['color']=='红' and held['JC046'] and color_counts['红']==0:bonus+=30
                preserve=100 if key in NEW and held[key]==1 else 35 if key in NEW else 0
                return self.card_value(c)-bonus+preserve
            return min(assets,key=score)
        deploy=[a for a in legal if a['kind']=='deploy' and a.get('cardId') in hand]
        preferred=seat if seat<4 else 4
        def placement(a):return (a.get('region')==preferred,-a.get('region',0))
        if len(own)<2:
            bodies=[a for a in deploy if hand[a['cardId']].get('cost',99)<=2 and hand[a['cardId']].get('cardId') not in NEW]
            if bodies:return max(bodies,key=lambda a:(placement(a),-hand[a['cardId']].get('cost',0)))
        conceal=[a for a in legal if a['kind']=='conceal' and hand.get(a.get('cardId'),{}).get('cardId')=='JC046']
        if conceal and not hidden46 and not covered_four:return max(conceal,key=placement)
        adviser=[a for a in deploy if hand[a['cardId']].get('cardId')=='JC060']
        if adviser:return max(adviser,key=placement)
        leader=[a for a in deploy if hand[a['cardId']].get('cardId')=='JC095']
        if leader:
            team=next(p['team'] for p in view['players'] if p['id']==view['you'])
            return max(leader,key=lambda a:(view['regions'][a['region']]['influence'][1-team],placement(a)))
        normal=[a for a in deploy if covered_four or hand[a['cardId']].get('cardId')!='JC046']
        if normal:return max(normal,key=lambda a:(self.card_value(hand[a['cardId']]),placement(a)))
        return next((a for a in legal if a['kind']=='pass'),None)

    async def legal_button(self,seat,action):
        before=await self.view(seat)
        card=next((c for c in before['hand']+[c for r in before['regions'] for c in r['characters']] if c['instanceId']==action.get('cardId')),None)
        await super().legal_button(seat,action)
        if card and card.get('cardId')=='JC046' and action['kind']=='reveal':self.last_046_reveal=self.records[-1]['commandId']
        if card and card.get('cardId')=='JC095' and action['kind'] in ('deploy','reveal'):
            team=next(p['team'] for p in before['players'] if p['id']==before['you']);r=before['regions'][action['region']]
            self.jc095_entries.append(dict(seat=seat,region=r['index'],regionCardId=r['cardId'],team=team,enemyTeam=1-team,ownBefore=r['influence'][team],enemyBefore=r['influence'][1-team],resolved=False))

    async def checkpoint(self,stage,passed=False,failure=None):
        v=await self.view(0) if self.pages else None
        result=dict(passed=passed,completeGame=bool(v and v['status']=='finished'),stage=stage,failure=failure,mode='teams',attempt=self.attempt,
            candidate=CANDIDATE,version=54,providerSource='cec2bd64b847d47ac971a459dc8ef2d61aec9e89',roomId=self.room_id,
            publicAnonymous=True,ownerBypass=False,stateInjection=False,apiMoves=False,viewport=[1366,900],
            uiPostCount=self.post_count,uiPostBudget=self.post_limit,postResults=self.post_results,
            checks=self.online_checks,newCards=self.newcards,jc046Sequences=self.jc046_sequences,jc095Entries=self.jc095_entries,
            auraChecks=self.aura_checks,choicePrivacy=self.privacy_snapshots,mulliganProbeDone=self.mulligan_probe_done,
            coverage=dict(self.coverage),expiryRaces=self.expiry_races,expectedBrowserErrors=self.expected_errors,
            browserErrors=self.errors,transportEvents=self.transport_events,final=public_view(v),deck=DECK,
            actualCoverageOnly=True,notFaceupObserved=[x for x in sorted(NEW) if not self.newcards[x]['faceupInstances']],
            notCovered=[label for label,covered in [
                ('JC046-four-seat-mandatory-sacrifice',any(x['completedFourSeats'] for x in self.jc046_sequences.values())),
                ('JC060-current-domain',bool(self.aura_checks)),
                ('JC060-teammate-aura',any(x['teammateSource'] for x in self.aura_checks)),
                ('JC095-entry-nonzero-conversion',any(x.get('nonzeroConversionObserved') for x in self.jc095_entries))] if not covered])
        write_json(self.output/'teams-summary.json',result)
        return result

    async def play(self):
        result=None
        try:
            await self.setup()
            while self.post_count<self.post_limit:
                views=await self.views();await self.sync(max(v['version'] for v in views));views=await self.observe()
                latest=max(views,key=lambda v:v['version'])
                if latest['status']=='finished':
                    await self.screenshot('complete-four-seat-game')
                    assert not self.errors,'Unexpected browser errors'
                    result=await self.checkpoint('natural-complete',True);break
                chosen=next(((s,v) for s,v in enumerate(views) if v.get('pendingChoice')),None)
                if chosen:await self.choose(*chosen)
                elif latest.get('waitingChoice'):await asyncio.sleep(.1);continue
                elif any(v.get('stack') and v.get('responseWindow') for v in views):
                    acted=False
                    for seat,v in enumerate(views):
                        window=v.get('responseWindow');member=next((m for m in (window or {}).get('members',[]) if m['playerId']==v['you']),None)
                        if member and member['status']=='undecided' and member.get('deadlineMs',0)-time.time()*1000>=1800:
                            button=self.pages[seat].get_by_role('button',name='不连锁，让过',exact=True)
                            if await button.count() and await button.is_enabled():
                                count=self.post_count
                                try:await self.submit(seat,lambda:button.click(timeout=1000),'passResponse')
                                except BrowserTimeout:
                                    if self.post_count!=count:raise
                                    await self.pages[seat].wait_for_function('(id)=>{const v=window.__cloudView;return v?.responseWindow?.id!==id||v?.pendingChoice||v?.waitingChoice||v?.responseWindow?.members.find(m=>m.playerId===v.you)?.status!=="undecided"}',arg=window['id'],timeout=12000)
                                acted=True;break
                    if not acted:await asyncio.sleep(.1);continue
                else:
                    actions=[(s,self.policy(s,v)) for s,v in enumerate(views)]
                    chosen=next(((s,a) for s,a in actions if a and a['kind']!='pass'),None) or next(((s,a) for s,a in actions if a),None)
                    assert chosen,f'No normal UI action at {latest["phase"]}:{latest["step"]}'
                    await self.legal_button(*chosen)
                await self.checkpoint('playing')
                if self.post_count%20==0:
                    v=await self.view(0);print(json.dumps(dict(posts=self.post_count,version=v['version'],turn=v['turn'],score=[p['score'] for p in v['players']],jc046FourSeats=any(x['completedFourSeats'] for x in self.jc046_sequences.values()))),flush=True)
            else:raise AssertionError('Four-seat UI POST budget exhausted')
        except BaseException as e:
            failure=f'{type(e).__name__}: {e}'
            (self.output/f'attempt-{self.attempt:02d}-exception.log').write_text(traceback.format_exc())
            if self.pages:
                try:await self.screenshot('four-seat-failure')
                except Exception:pass
            result=await self.checkpoint('interrupted',False,failure)
            if isinstance(e,asyncio.CancelledError):raise
        finally:
            await asyncio.gather(*(c.close() for c in self.contexts),return_exceptions=True)
        print(json.dumps({k:result.get(k) for k in ['passed','completeGame','roomId','uiPostCount','failure']}),flush=True)
        assert result['passed'],result['failure']

async def main(args):
    class Remote:remote=True;base_url=args.url
    output=Path(args.output).resolve()
    if not args.resume:assert not output.exists(),'Use a new output for a new owned game; resume the existing owned room otherwise'
    async with async_playwright() as p:
        await TeamsRun(p,Remote(),output,args.resume,args.budget).play()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--url',required=True);p.add_argument('--output',required=True);p.add_argument('--resume',action='store_true');p.add_argument('--budget',type=int,default=3000)
    asyncio.run(main(p.parse_args()))
