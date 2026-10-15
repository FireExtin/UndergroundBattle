#!/usr/bin/env python3
"""Bounded JC095 nonzero conversion on one new owned v54 table, real UI only.

Both seats use normal 50-card decks and random draws. Policies read only their
own hand and the public table. No fixture state, seed, API moves or owner bypass.
The previous four-player terminal table is never opened or changed.
"""
import argparse,asyncio,json,re,time,traceback
from pathlib import Path
from playwright.async_api import async_playwright,expect,TimeoutError as BrowserTimeout
from public_abi77_teams import TeamsRun,frozen,CANDIDATE,ENGINE,POOL
from full_game_persistent import FullGame
from common import public_view,write_json

BLACK_IDS=['JC086','JC084','JC085','JC088','BQ083','JC096','XQ38','XQ36','JZ49',
           'JZ48','JZ50','XQ37','WM059','BQ078','JC087','WM056','JC095']
DECK={key:2 if key=='JZ50' else 3 for key in BLACK_IDS}
OLD_ROOM='1609f4e9dabfb4d970646b44'
TRIGGER='唐·格鲁尼：进场触发：移除原地区所有敌方势力并放置等量本方势力'

class TargetRun(TeamsRun):
    def __init__(self,p,service,output,resume=False,budget=900):
        super().__init__(p,service,output,False,budget)
        self.mode='duel';self.resume_run=resume;self.entry=None;self.goal=None
        self.marker_regions=[];self.auto_started=False;self.auto_enabled=False
        self.target_privacy=[];self.drawn_seats=[];self.progress_seen=None
        if resume:
            prior=json.loads((self.output/'target-summary.json').read_text())
            assert not prior['passed'] and prior['roomId']!=OLD_ROOM
            assert not prior['browserErrors'],'Review unexpected browser errors before continuing'
            self.attempt=prior['attempt']+1;self.room_id=prior['roomId']
            self.post_count=prior['uiPostCount'];self.post_results=prior['postResults']
            self.online_checks=prior['checks'];self.target_privacy=prior['choicePrivacy']
            self.marker_regions=prior['markerRegions'];self.entry=prior['entry'];self.goal=prior['goal']
            self.drawn_seats=prior['jc095DrawnSeats'];self.expiry_races=prior['expiryRaces']
            self.expected_errors=prior['expectedBrowserErrors']
            self.coverage.update(prior['coverage'])
            self.records=[json.loads(l) for l in (self.output/'duel-actions.jsonl').read_text().splitlines()]
            write_json(self.output/f'attempt-{prior["attempt"]:02d}-summary.json',prior)

    def check(self,name,**data):
        self.online_checks.append(dict(name=name,passed=True,**data))
        write_json(self.output/'target-pc-checks.json',dict(mode='duel',scope='JC095 nonzero only',checks=self.online_checks))

    async def build_deck(self,seat):
        page=self.pages[seat];name=f'JC095非零专项-{seat+1}'
        await page.get_by_role('button',name='展开组卡工作台',exact=True).click()
        settings=page.get_by_role('button',name='牌组设置与模板',exact=True)
        if await settings.count():await settings.click()
        await page.get_by_role('button',name='新建空白牌组',exact=True).click()
        await page.get_by_role('textbox',name='牌组名称',exact=True).fill(name)
        for card_id,count in DECK.items():
            card=self.catalog[card_id]
            assert card['supported'] and card['kind']=='character' and card['color']=='黑'
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
        self.check('legal-black-50-card-deck-through-ui',seat=seat,count=sum(DECK.values()),jc095Copies=3)

    async def setup(self):
        host=await self.new_seat(0)
        if not self.room_id:
            await self.build_deck(0)
            await host.get_by_label('你的称呼',exact=True).fill('非零验收甲')
            await host.get_by_role('button',name=re.compile('两人对决')).click()
            await host.get_by_role('button',name='创建牌桌 →',exact=True).click()
            await host.wait_for_function('window.__cloudView?.status==="lobby"')
            self.room_id=(await self.view(0))['roomId']
            assert self.room_id!=OLD_ROOM
            await self.checkpoint('new-owned-target-room-created')
        peer=await self.new_seat(1)
        joined=any(p['id']=='p1' for p in (await self.view(0))['players'])
        if joined:
            await peer.wait_for_function('(id)=>window.__cloudView?.roomId===id',arg=self.room_id,timeout=30000)
        else:
            assert (await self.view(0))['status']=='lobby'
            await self.build_deck(1)
            await peer.get_by_role('button',name='邀请码加入',exact=True).click()
            await peer.get_by_label('你的称呼',exact=True).fill('非零验收乙')
            await peer.get_by_label('邀请码',exact=True).fill((await self.view(0))['inviteCode'])
            await peer.get_by_role('button',name='加入牌桌 →',exact=True).click()
            await peer.wait_for_function('(id)=>window.__cloudView?.roomId===id',arg=self.room_id)
        await self.sync(max(v['version'] for v in await self.views()))
        if self.resume_run:await self.resume_preflight()
        if (await self.view(0))['status']=='lobby':
            for seat in range(2):
                view=await self.view(seat)
                if not next(p['ready'] for p in view['players'] if p['id']==view['you']):
                    await self.legal_button(seat,next(a for a in view['legalActions'] if a['kind']=='ready'))
            await self.legal_button(0,next(a for a in (await self.view(0))['legalActions'] if a['kind']=='start'))
        view=await self.view(0)
        assert view['mode']=='duel' and len(view['players'])==2
        if not self.marker_regions:
            self.marker_regions=[r['index'] for r in sorted(view['regions'],key=lambda r:(-r['threshold'],r['index']))[:2]]
        self.check('two-owned-native77-seats-same-new-table',roomId=self.room_id)

    async def set_auto(self,enabled):
        for page in self.pages:
            switch=page.get_by_role('switch',name='无可用行动时自动让过',exact=True)
            await switch.set_checked(enabled)
        self.auto_enabled=enabled;self.auto_started=True
        self.check('normal-ui-auto-pass-preference',enabled=enabled)

    async def choose(self,seat,view):
        choice=view['pendingChoice'];await self.sync(view['version']);views=await self.views()
        assert all(v['roomId']==self.room_id for v in views)
        assert all(i==seat or v.get('pendingChoice') is None for i,v in enumerate(views))
        if not any(x['choiceId']==choice['id'] for x in self.target_privacy):
            self.target_privacy.append(dict(choiceId=choice['id'],kind=choice['kind'],title=choice['title'],
                seat=seat,version=view['version'],ownOptionCount=len(choice['options']),otherSeatHasNoPrivateOptions=True))
        if choice['kind']=='mulligan':
            # Normal once-only mulligan; each seat preserves its own JC095 and
            # one cheap permanent-influence body, replacing other cards normally.
            body_kept=False;wanted=[]
            for option in choice['options']:
                card=option.get('card',{});definition=self.catalog.get(card.get('cardId'),{})
                if card.get('cardId')=='JC095':continue
                if not body_kept and definition.get('cost',99)<=2 and definition.get('permanentIcons',{}).get('influence',0)>0:
                    body_kept=True;continue
                wanted.append(option['id'])
            return await self.confirm_draft(seat,view,wanted)
        if choice['title']==TRIGGER:
            assert self.entry and self.entry['actorSeat']==seat and self.entry['enemyBefore']>0
            region=view['regions'][self.entry['region']]
            assert region['cardId']==self.entry['regionCardId']
            assert region['influence']==self.entry['beforeInfluence']
            self.entry['triggerChoiceId']=choice['id'];self.entry['beforeTriggerView']=public_view(view)
            await self.screenshot('jc095-nonzero-before-trigger',seat)
            await self.confirm_draft(seat,view,['accept'])
            record=self.records[-1];actual=record['action'].get('action',record['action'])
            assert record['httpStatus']==200 and actual['selected']==['accept']
            self.entry['triggerCommandId']=record['commandId'];self.entry['triggerAccepted']=True
            return
        if choice['kind']=='trigger' and choice.get('allowDecline'):
            skip=self.pages[seat].locator('.hg-choice').get_by_role('button',name='跳过此选择',exact=True)
            if await skip.count():return await self.submit(seat,skip.click,'choose')
            return await self.confirm_draft(seat,view,[])
        if choice['kind']=='discard':
            wanted=sorted(choice['options'],key=lambda o:(o.get('card',{}).get('cardId')=='JC095',self.card_value(o.get('card',{}))))[:choice['min']]
            return await self.confirm_draft(seat,view,[o['id'] for o in wanted])
        return await FullGame.choose(self,seat,view)

    def policy(self,seat,view):
        legal=view.get('legalActions',[]);hand={c['instanceId']:c for c in view['hand']}
        team=next(p['team'] for p in view['players'] if p['id']==view['you'])
        leaders=[a for a in legal if a['kind']=='deploy' and hand.get(a.get('cardId'),{}).get('cardId')=='JC095'
            and view['regions'][a['region']]['influence'][1-team]>0
            and sum(view['regions'][a['region']]['influence'])<view['regions'][a['region']]['threshold']]
        if leaders and not self.entry:return max(leaders,key=lambda a:view['regions'][a['region']]['influence'][1-team])
        assets=[a for a in legal if a['kind']=='asset' and hand.get(a['cardId'],{}).get('cardId')!='JC095']
        owned_assets=sum(c['controller']==view['you'] for c in view['assets'])
        if assets and owned_assets<8:return min(assets,key=lambda a:self.card_value(hand[a['cardId']]))
        counts={p['id']:sum(c['controller']==p['id'] for c in view['assets']) for p in view['players']}
        own_board=[c for r in view['regions'] for c in r['characters'] if c['controller']==view['you']]
        if min(counts.values())>=7 and not own_board:
            target=self.marker_regions[seat]
            bodies=[a for a in legal if a['kind']=='deploy' and a.get('cardId') in hand and a['region']==target
                and hand[a['cardId']]['cardId']!='JC095'
                and self.catalog[hand[a['cardId']]['cardId']]['permanentIcons']['influence']>0]
            if bodies:return min(bodies,key=lambda a:(hand[a['cardId']].get('cost',99),-self.catalog[hand[a['cardId']]['cardId']]['permanentIcons']['influence']))
        if len(legal)==1 and legal[0]['kind']=='pass' and self.auto_enabled:return None
        return next((a for a in legal if a['kind']=='pass'),None)

    async def legal_button(self,seat,action):
        before=await self.view(seat)
        card=next((c for c in before['hand'] if c['instanceId']==action.get('cardId')),None)
        if action['kind']=='deploy' and card and card['cardId']=='JC095':
            await self.set_auto(False)
            before=await self.view(seat);team=next(p['team'] for p in before['players'] if p['id']==before['you']);region=before['regions'][action['region']]
            assert region['influence'][1-team]>0
            self.entry=dict(actorSeat=seat,actorTeam=team,region=region['index'],regionCardId=region['cardId'],
                beforeInfluence=list(region['influence']),ownBefore=region['influence'][team],enemyBefore=region['influence'][1-team],
                beforeEntryVersion=before['version'],beforeEntryView=public_view(before),triggerAccepted=False)
        await frozen.PublicGame.legal_button(self,seat,action)
        if self.entry and action['kind']=='deploy' and card and card['cardId']=='JC095':
            self.entry['deployCommandId']=self.records[-1]['commandId']

    async def observe_goal(self,views):
        for seat,view in enumerate(views):
            assert view['roomId']==self.room_id and self.room_id!=OLD_ROOM and view['you']==f'p{seat}'
            if any(c['cardId']=='JC095' for c in view['hand']) and seat not in self.drawn_seats:self.drawn_seats.append(seat)
        if not self.entry or not self.entry.get('triggerAccepted'):return False
        if any(v.get('stack') or v.get('pendingChoice') or v.get('waitingChoice') for v in views):return False
        entry=self.entry;expected=list(entry['beforeInfluence']);expected[entry['actorTeam']]+=entry['enemyBefore'];expected[1-entry['actorTeam']]=0
        proof=[]
        for seat,view in enumerate(views):
            region=view['regions'][entry['region']]
            assert region['cardId']==entry['regionCardId'],'Original region changed before this bounded observation'
            assert region['influence']==expected,'Nonzero conversion did not produce exact expected counters'
            sources=[c['instanceId'] for c in region['characters'] if c.get('cardId')=='JC095' and c['controller']==f'p{entry["actorSeat"]}' and not c['faceDown']]
            assert len(sources)==1
            proof.append(dict(seat=seat,version=view['version'],influence=list(region['influence']),sourceInstanceId=sources[0],publicView=public_view(view)))
        self.goal=dict(nonzeroEnemyRemoved=entry['enemyBefore'],equalFriendlyMarkersAdded=entry['enemyBefore'],expectedInfluence=expected,views=proof)
        self.check('actual-JC095-nonzero-original-region-conversion',enemyBefore=entry['enemyBefore'],ownBefore=entry['ownBefore'],after=expected,viewsChecked=2)
        return True

    async def checkpoint(self,stage,passed=False,failure=None):
        views=await self.views() if self.pages else [];view=views[0] if views else None
        result=dict(passed=passed,stage=stage,failure=failure,attempt=self.attempt,mode='duel',boundedMechanismProbe=True,
            completeGame=bool(view and view['status']=='finished'),roomId=self.room_id,oldFourSeatRoomId=OLD_ROOM,
            originalFourSeatRoomUntouched=True,siteVersion=54,candidate=CANDIDATE,providerSource='cec2bd64b847d47ac971a459dc8ef2d61aec9e89',
            publicAnonymous=True,ownerBypass=False,stateInjection=False,apiMoves=False,newDeployment=False,viewport=[1366,900],
            uiPostCount=self.post_count,postResults=self.post_results,unansweredUiPostAttempts=self.post_count-len(self.post_results),
            uiPostBudget=self.post_limit,checks=self.online_checks,choicePrivacy=self.target_privacy,entry=self.entry,goal=self.goal,
            markerRegions=self.marker_regions,jc095DrawnSeats=self.drawn_seats,deck=DECK,coverage=dict(self.coverage),
            browserErrors=self.errors,expectedBrowserErrors=self.expected_errors,expiryRaces=self.expiry_races,transportEvents=self.transport_events,
            final=public_view(view),views=[public_view(v) for v in views],pausedViews=[bool(v.get('pause')) for v in views])
        write_json(self.output/'target-summary.json',result);return result

    async def play(self):
        result=None
        try:
            await self.setup()
            while self.post_count<self.post_limit:
                views=await self.views();await self.sync(max(v['version'] for v in views));views=await self.views()
                if await self.observe_goal(views):
                    await self.screenshot('jc095-nonzero-converted',self.entry['actorSeat'])
                    pause=self.pages[0].get_by_role('button',name='暂停并保存',exact=True)
                    await self.submit(0,pause.click,'pauseRoom')
                    assert all(v.get('pause') for v in await self.views())
                    assert not self.errors
                    result=await self.checkpoint('nonzero-observed-and-owned-table-paused',True);break
                latest=max(views,key=lambda v:v['version'])
                assert latest['status']!='finished','Natural game ended before the bounded nonzero observation'
                assert latest['turn']<=16,'No suitable natural draw/opportunity within the bounded 16 turns'
                pending=next(((s,v) for s,v in enumerate(views) if v.get('pendingChoice')),None)
                if pending:await self.choose(*pending)
                elif latest.get('waitingChoice'):await asyncio.sleep(.1);continue
                elif any(v.get('stack') and v.get('responseWindow') for v in views):
                    acted=False
                    for seat,view in enumerate(views):
                        window=view.get('responseWindow') or {};member=next((m for m in window.get('members',[]) if m['playerId']==view['you']),{})
                        if member.get('status')=='undecided' and member.get('deadlineMs',0)-time.time()*1000>=1800:
                            button=self.pages[seat].get_by_role('button',name='不连锁，让过',exact=True)
                            if await button.count() and await button.is_enabled():
                                count=self.post_count
                                try:await self.submit(seat,lambda:button.click(timeout=1000),'passResponse')
                                except BrowserTimeout:
                                    if self.post_count!=count:raise
                                acted=True;break
                    if not acted:await asyncio.sleep(.1);continue
                else:
                    if not self.auto_started:await self.set_auto(True)
                    actions=[(s,self.policy(s,v)) for s,v in enumerate(views)]
                    chosen=next(((s,a) for s,a in actions if a and a['kind']!='pass'),None) or next(((s,a) for s,a in actions if a),None)
                    if chosen:await self.legal_button(*chosen)
                    else:await asyncio.sleep(.1)
                current=await self.view(0);progress=(current['version'],self.post_count)
                if progress!=self.progress_seen:
                    self.progress_seen=progress;await self.checkpoint('building-natural-nonzero-opportunity')
                    if self.post_count%20==0:print(json.dumps(dict(posts=self.post_count,version=current['version'],turn=current['turn'],assets=[sum(c['controller']==p['id'] for c in current['assets']) for p in current['players']],entryStarted=bool(self.entry))),flush=True)
            else:raise AssertionError('Bounded JC095 UI POST budget exhausted')
        except BaseException as error:
            frames=[dict(file=Path(x.filename).name,line=x.lineno,function=x.name) for x in traceback.extract_tb(error.__traceback__)]
            message=str(error).split('\n')[0][:350]
            message=re.sub(r'Bearer\s+\S+','Bearer [redacted]',message)
            write_json(self.output/f'attempt-{self.attempt:02d}-failure.json',dict(type=type(error).__name__,message=message,frames=frames))
            result=await self.checkpoint('interrupted',False,f'{type(error).__name__}: {message}')
            if isinstance(error,asyncio.CancelledError):raise
        finally:
            await asyncio.gather(*(context.close() for context in self.contexts),return_exceptions=True)
            if self.network_body_tasks:
                _,pending=await asyncio.wait(self.network_body_tasks,timeout=3)
                for task in pending:task.cancel()
                await asyncio.gather(*pending,return_exceptions=True)
            if result:
                result.update(postResults=self.post_results,browserErrors=self.errors,transportEvents=self.transport_events,
                    unansweredUiPostAttempts=self.post_count-len(self.post_results),shutdownTelemetryCollected=True)
                write_json(self.output/'target-summary.json',result)
        print(json.dumps({k:result.get(k) for k in ['passed','stage','roomId','uiPostCount','failure']}),flush=True)
        assert result['passed'],result['failure']

async def main(args):
    class Remote:remote=True;base_url=args.url
    output=Path(args.output).resolve()
    if not args.resume:assert not output.exists(),'One new owned table only; resume its original profiles after interruption'
    async with async_playwright() as p:await TargetRun(p,Remote(),output,args.resume,args.budget).play()

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--url',required=True);parser.add_argument('--output',required=True)
    parser.add_argument('--resume',action='store_true');parser.add_argument('--budget',type=int,default=900)
    asyncio.run(main(parser.parse_args()))
