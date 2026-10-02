#!/usr/bin/env python3
"""Complete desktop games through persistent seats and normal UI only.

No seed/state API setup, private-state edits, foreign sessions or mobile checks.
The action policy reads only its acting seat's hand and public board. Responses
use the real intent buttons; full-game budgets are distinct from slice probes.
"""
import argparse, asyncio, json, re, time
from pathlib import Path
from playwright.async_api import async_playwright, TimeoutError as BrowserTimeout
from attachment_v026 import AttachmentRun
from common import IsolatedService, public_view, write_json

class FullGame(AttachmentRun):
    def __init__(self,p,service,output,mode,budget,grave_probe=False,resume=False):
        super().__init__(p,service,output)
        self.mode=mode; self.post_limit=budget
        self.grave_probe=grave_probe; self.grave_checks=[]; self.grave_entries=[]
        self.condition_checks=[]; self.condition_seen=set()
        self.expiry_races=[]
        self.resume=resume; self.previous=None
        if resume:
            self.previous=json.loads((self.output/'full-game-summary.json').read_text())
            assert self.previous['mode']==mode and not self.previous['passed']
            self.post_count=self.previous['uiPostCount']; self.coverage.update(self.previous['coverage'])
            self.grave_checks=self.previous.get('graveChecks',[])
            self.condition_checks=self.previous.get('conditionChecks',[])
            self.expiry_races=self.previous.get('expiryRaces',[])
    def observe_conditions(self,views):
        for v in views:
            teams={p['id']:p['team'] for p in v['players']}
            for region in v['regions']:
                for c in region['characters']:
                    if c.get('cardId') not in ('JC084','JC085') or c['faceDown']:continue
                    team=teams[c['controller']];first=team==v['firstTeam'] and not c['exhausted']
                    own_death=any(a['controller']==c['controller'] and a.get('magic')=='死亡' for a in v['assets'])
                    team_death=any(teams[a['controller']]==team and a.get('magic')=='死亡' for a in v['assets'])
                    if c['cardId']=='JC085': expected=(int(first and own_death),0)
                    else:expected=(int(first and region['influence'][team]>0),int(first and region['influence'][1-team]>0))
                    actual=(c['icons']['influence'],c['icons']['investigation'])
                    assert actual==expected,f'Conditional icons differ for {c["cardId"]}: {actual} vs {expected}'
                    key=(c['cardId'],first,c['exhausted'],own_death,team_death,tuple(region['influence']),actual)
                    if key not in self.condition_seen:
                        self.condition_seen.add(key)
                        self.condition_checks.append({'cardId':c['cardId'],'viewer':v['you'],'controller':c['controller'],'version':v['version'],'firstTeamEligible':first,'exhausted':c['exhausted'],'ownDeathAsset':own_death,'teamDeathAsset':team_death,'regionInfluence':region['influence'],'effectiveInfluence':actual[0],'effectiveInvestigation':actual[1],'passed':True})
    async def legal_button(self, seat, action):
        before=await self.view(seat)
        grave=next((c for c in before.get('graveyard',[]) if c['instanceId']==action.get('cardId') and c['owner']==before['you']),None)
        await super().legal_button(seat,action)
        if grave and action['kind']=='deploy':
            after=await self.view(seat)
            assert grave['cardId']=='JC085'
            assert all(c['instanceId']!=grave['instanceId'] for c in after['hand']+after['graveyard'])
            assert after['stack'] and after['stack'][-1]['cardId']=='JC085'
            exhausted=lambda v: sum(c['controller']==v['you'] and c['exhausted'] for c in v['assets'])
            assert exhausted(after)-exhausted(before)==2
            self.grave_entries.append({'seat':seat,'oldIds':{c['instanceId'] for r in before['regions'] for c in r['characters']},'region':action['region']})
            self.grave_checks.append({'seat':seat,'kind':'natural-grave-face-up-frame','version':after['version'],'passed':True})
            await self.screenshot('natural-grave-paid-frame',seat)
    def policy(self,seat,view):
        if self.grave_probe:
            own_grave={c['instanceId'] for c in view.get('graveyard',[]) if c['owner']==view['you']}
            grave=next((a for a in view.get('legalActions',[]) if a['kind']=='deploy' and a.get('cardId') in own_grave),None)
            if grave: return grave
            own_recur={c['instanceId'] for c in view['hand'] if c.get('cardId')=='JC085'}
            contested=next((a for a in view.get('legalActions',[]) if a['kind']=='deploy' and a.get('cardId') in own_recur and a.get('region')==2),None)
            if contested:return contested
        return super().policy(seat,view)
    async def play_full(self):
        start=time.monotonic(); capacity=2 if self.mode=='duel' else 4
        try:
            page=await self.new_seat(0)
            catalog=await page.evaluate('window.__cloudCatalog')
            self.catalog={c['id']:c for c in catalog['cards']}
            decks=['reclaimers','keepers','hunters','watchers'] if self.grave_probe and self.mode=='teams' else ['reclaimers','hunters','keepers','watchers'] if self.grave_probe else ['watchers','hunters','keepers','reclaimers']
            if self.resume:
                await page.wait_for_function('window.__cloudView?.status === "playing"')
                for seat in range(1,capacity):await self.new_seat(seat)
                for seat,v in enumerate(await self.views()):
                    assert v['roomId']==self.previous['roomId'] and v['you']==f'p{seat}'
                    assert v['versions']['engine']==self.previous['final']['versions']['engine']
                await self.sync(max(v['version'] for v in await self.views()))
            else:
                await page.locator(f'.hg-deck[data-deck-id="{decks[0]}"]').click()
                await page.get_by_label('你的称呼').fill(f'完整局{self.mode}甲')
                if self.mode=='teams': await page.get_by_role('button',name=re.compile('四人协作')).click()
                await page.get_by_role('button',name='创建牌桌 →',exact=True).click()
                await page.wait_for_function('window.__cloudView?.status === "lobby"')
                invite=(await self.view(0))['inviteCode']
                for seat in range(1,capacity):
                    page=await self.new_seat(seat)
                    await page.locator(f'.hg-deck[data-deck-id="{decks[seat]}"]').click()
                    await page.get_by_role('button',name='邀请码加入',exact=True).click()
                    await page.get_by_label('你的称呼').fill(f'完整局{self.mode}{seat+1}')
                    await page.get_by_label('邀请码',exact=True).fill(invite)
                    await page.get_by_role('button',name='加入牌桌 →',exact=True).click()
                    await page.wait_for_function('window.__cloudView?.status === "lobby"')
                await self.sync((await self.view(capacity-1))['version'])
                for seat in range(capacity):
                    await self.legal_button(seat,next(a for a in (await self.view(seat))['legalActions'] if a['kind']=='ready'))
                await self.legal_button(0,next(a for a in (await self.view(0))['legalActions'] if a['kind']=='start'))
            while self.post_count < self.post_limit:
                views=await self.views(); latest=max(views,key=lambda v:v['version'])
                await self.sync(latest['version'])
                views=await self.views(); latest=max(views,key=lambda v:v['version'])
                if self.grave_probe:self.observe_conditions(views)
                if self.grave_entries and not latest['stack']:
                    for entry in self.grave_entries:
                        v=views[entry['seat']]
                        entered=next(c for c in v['regions'][entry['region']]['characters'] if c['instanceId'] not in entry['oldIds'] and c.get('cardId')=='JC085' and c['controller']==v['you'])
                        assert not entered['faceDown']
                        self.grave_checks.append({'seat':entry['seat'],'kind':'natural-grave-fresh-face-up-entry','version':v['version'],'passed':True})
                    self.grave_entries.clear()
                if latest['status']=='finished':
                    await self.screenshot('complete-game')
                    result={'passed':True}; break
                chosen=next(((s,v) for s,v in enumerate(views) if v.get('pendingChoice')),None)
                if chosen:
                    seat,v=chosen; await self.choose(seat,v)
                elif latest.get('waitingChoice'):
                    # A timed transition can create a private chooser while its
                    # own client is still receiving that same public revision.
                    await asyncio.sleep(.1); continue
                elif latest.get('stack') and latest.get('responseWindow'):
                    acted=False
                    for seat,v in enumerate(views):
                        w=v.get('responseWindow'); member=next((m for m in (w or {}).get('members',[]) if m['playerId']==v['you']),None)
                        if member and member['status']=='undecided':
                            if member.get('deadlineMs',0)-time.time()*1000<1800:continue
                            button=self.pages[seat].get_by_role('button',name='不连锁，让过',exact=True)
                            if await button.count() and await button.is_enabled():
                                before_posts=self.post_count
                                try:await self.submit(seat,lambda:button.click(timeout=1000),'passResponse')
                                except BrowserTimeout as error:
                                    if self.post_count!=before_posts:raise
                                    with (self.output/'response-click-diagnostics.jsonl').open('a') as stream:
                                        stream.write(json.dumps({'seat':seat,'windowId':w['id'],'browserCallLog':str(error)})+'\n')
                                    if 'intercepts pointer events' in str(error):raise
                                    # Five seconds plus the ordinary polling/render interval,
                                    # rather than a timeout shorter than the response itself.
                                    await self.pages[seat].wait_for_function('(id)=>{const v=window.__cloudView;return v?.responseWindow?.id!==id || v?.pendingChoice || v?.waitingChoice || v?.responseWindow?.members.find(m=>m.playerId===v.you)?.status!=="undecided"}',arg=w['id'],timeout=8000)
                                    self.expiry_races.append({'seat':seat,'windowId':w['id'],'kind':'normal-response-ui-changed-before-post','gamePosts':0})
                                acted=True; break
                    if not acted:
                        await asyncio.sleep(.1)
                        continue
                else:
                    candidates=[(s,self.policy(s,v)) for s,v in enumerate(views)]
                    chosen=next(((s,a) for s,a in candidates if a and a['kind']!='pass'),None) or next(((s,a) for s,a in candidates if a),None)
                    assert chosen, f'No normal UI action at {latest["phase"]}:{latest["step"]}'
                    await self.legal_button(*chosen)
                if self.post_count % 50 == 0:
                    v=await self.view(0)
                    print(json.dumps({'mode':self.mode,'posts':self.post_count,'version':v['version'],'turn':v['turn'],'score':[p['score'] for p in v['players']]}),flush=True)
            else: raise AssertionError('Full-game UI budget exhausted')
            assert not self.errors, 'Browser errors during normal play'
            if self.grave_probe: assert any(c['kind']=='natural-grave-fresh-face-up-entry' for c in self.grave_checks), 'Complete game finished without naturally reaching grave play; mechanism UI remains untested'
        except Exception as error:
            result={'passed':False,'failure':f'{type(error).__name__}: {error}'}
            if self.pages: await self.screenshot('full-game-failure')
        finally:
            v=await self.view(0) if self.pages else None
            result.update(testType='complete-natural-desktop-ui-game',mode=self.mode,stateInjection=False,apiMoves=False,uiPostCount=self.post_count,uiPostBudget=self.post_limit,browserErrors=self.errors,coverage=dict(self.coverage),graveChecks=self.grave_checks,conditionChecks=self.condition_checks,expiryRaces=self.expiry_races,final=public_view(v),roomId=v['roomId'] if v else None,durationSeconds=round(time.monotonic()-start,2),resumedExistingRoom=self.resume,priorUiPostCount=self.previous['uiPostCount'] if self.previous else 0)
            write_json(self.output/'full-game-summary.json',result)
            for c in self.contexts: await c.close()
        print(json.dumps({'mode':self.mode,'passed':result['passed'],'posts':self.post_count,'failure':result.get('failure'),'roomId':result['roomId']}),flush=True)
        assert result['passed'], result.get('failure')
async def main(args):
    service=IsolatedService(args.binary,args.cwd,args.output,args.port,args.static_dir)
    service.start()
    try:
        async with async_playwright() as p: await FullGame(p,service,Path(args.output).resolve(),args.mode,args.budget,args.grave_probe,args.resume).play_full()
    finally: service.stop()
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary',required=True);p.add_argument('--cwd',default='.');p.add_argument('--static-dir',default='web/dist');p.add_argument('--output',required=True)
    p.add_argument('--mode',choices=['duel','teams'],required=True);p.add_argument('--budget',type=int,default=2000);p.add_argument('--port',type=int,default=8107)
    p.add_argument('--grave-probe',action='store_true')
    p.add_argument('--resume',action='store_true')
    asyncio.run(main(p.parse_args()))
