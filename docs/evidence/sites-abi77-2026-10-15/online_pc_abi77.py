#!/usr/bin/env python3
"""One owned public Sites PC game. Normal UI only; no hosting owner bypass."""
import argparse, asyncio, json, sys
from pathlib import Path
from playwright.async_api import async_playwright, expect

sys.path.insert(0, '/workspace/UndergroundBattle/tools/cloud-playtest')
from full_game_persistent import FullGame
from common import write_json

ENGINE = 'rust-v0.2.77-jc046-mandatory-sacrifice-candidate'
POOL = 'limited-v2.70-jc046-mandatory-sacrifice-candidate'
CANDIDATE = '4918cfb6372fb9ec900f037e8ed518385d6183e6'

class PublicGame(FullGame):
    def __init__(self, playwright, service, output, resume=False):
        super().__init__(playwright, service, output, 'duel', 2000, resume=resume)
        self.online_checks = []
        if resume and (self.output/'online-pc-checks.json').exists():
            self.online_checks = json.loads((self.output/'online-pc-checks.json').read_text())['checks']
        self.initial_probe_done = any(c['name']=='rapid-choice-clicks-one-command' for c in self.online_checks)
        self.refresh_paused_peers = False

    def check(self, name, **data):
        self.online_checks.append(dict(name=name, passed=True, **data))
        write_json(self.output/'online-pc-checks.json', dict(candidate=CANDIDATE, publicAnonymous=True,
            viewport=[1180,760], productionUrl=self.base, checks=self.online_checks))

    async def new_seat(self, seat, session=None):
        page = await super().new_seat(seat, session)
        if self.resume:
            await page.wait_for_function('(id)=>window.__cloudView?.roomId===id',arg=self.previous['roomId'],timeout=30000)
        if seat == 0:
            catalog = await page.evaluate('window.__cloudCatalog')
            assert catalog['engineVersion']==ENGINE and catalog['cardPoolVersion']==POOL
            assert len(catalog['cards'])==143
            health = await page.evaluate('async()=>{const r=await fetch("/api/health");if(!r.ok)throw Error("health "+r.status);return r.json()}')
            assert health['engineVersion']==ENGINE
            rows = json.loads(Path('/workspace/abi77-publication-20261015/client-files-158.json').read_text())['files']
            hashes = await page.evaluate('''async rows=>{
                const result=[];let index=0;
                async function worker(){while(index<rows.length){const row=rows[index++];
                    const r=await fetch('/'+row.path,{cache:'no-cache'});
                    const bytes=await r.arrayBuffer();
                    const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))).map(x=>x.toString(16).padStart(2,'0')).join('');
                    result.push({path:row.path,status:r.status,bytes:bytes.byteLength,sha256:hash,
                        passed:r.ok&&bytes.byteLength===row.bytes&&hash===row.sha256});
                }}
                await Promise.all([worker(),worker(),worker(),worker()]);
                return result.sort((a,b)=>a.path.localeCompare(b.path));
            }''', rows)
            identity=dict(candidate=CANDIDATE,publicAnonymous=True,health=health,
                catalog=dict(engine=catalog['engineVersion'],cardPool=catalog['cardPoolVersion'],cards=len(catalog['cards'])),
                frontendFiles=hashes,all158Match=len(hashes)==158 and all(row['passed'] for row in hashes))
            write_json(self.output/'online-identity.json',identity)
            assert identity['all158Match'], 'Public frontend identity differs from the reviewed 158-file build'
            self.check('public-engine-pool-and-all158-frontend-hashes',engine=ENGINE,cardPool=POOL,cards=143,matchingFiles=158)
        return page

    async def sync(self, version):
        if self.refresh_paused_peers:
            for seat,v in enumerate(await self.views()):
                if v and v.get('pause') and v['version']<version:
                    await self.pages[seat].reload(wait_until='domcontentloaded')
                    self.check('paused-peer-page-refreshed-after-other-seat-resume',seat=seat,pausedVersion=v['version'],targetVersion=version)
        await super().sync(version)
        for v in await self.views():
            assert v['versions']['engine']==ENGINE and v['versions']['cardPool']==POOL

    async def submit(self, seat, click, intended_kind):
        self.refresh_paused_peers = intended_kind=='resumeRoom'
        try:
            return await super().submit(seat,click,intended_kind)
        finally:
            self.refresh_paused_peers = False

    async def legal_button(self, seat, action):
        # The current UI groups destination variants: source -> action group ->
        # highlighted target/region -> confirmation. Never send an API move.
        page=self.pages[seat]
        identity=action.get('cardId') or action.get('targetId')
        if identity:
            card=page.locator('[data-card-instance='+json.dumps(identity)+']').first
            if not await card.is_visible():
                details=card.locator('xpath=ancestor::details[1]')
                if await details.count() and await details.get_attribute('open') is None:
                    await details.locator('summary').click()
            await card.click()
        elif action.get('region') is not None:
            await page.locator(f'#hg-region-{action["region"]} .hg-region-center').click()
        button=page.locator('[data-action-id='+json.dumps(action['id'])+']')
        if not await button.count() and action.get('cardId') and (action.get('targetId') or action.get('region') is not None):
            payload={key:action[key] for key in sorted(action) if key not in ('id','label','description','sourceZoneId','targetId','region')}
            label=action['label'];arrow=label.find('→');cost=label.find('（费用：')
            if arrow>=0:label=label[:arrow].strip()+(label[cost:] if cost>arrow else '')
            key=json.dumps([payload,'targetId' in action,'region' in action,label,action.get('description')],ensure_ascii=False,separators=(',',':'))
            group=page.locator('[data-object-action='+json.dumps(key,ensure_ascii=False)+']')
            assert await group.count()==1, f'Current UI action group absent for {action["kind"]}'
            await group.click()
            if action.get('targetId'):
                await page.locator('[data-card-instance='+json.dumps(action['targetId'])+']').first.click()
            if action.get('region') is not None:
                await page.locator(f'#hg-region-{action["region"]} .hg-region-center').click()
            await expect(button).to_be_visible()
            self.coverage['ui:grouped-destination-confirmation']+=1
        if not await button.count():button=page.get_by_role('button',name=action['label'],exact=True)
        await self.submit(seat,button.first.click,action['kind'])

    async def choose(self, seat, view):
        choice=view['pendingChoice']
        if seat!=0 or choice['kind']!='mulligan' or self.initial_probe_done:
            return await super().choose(seat,view)
        page=self.pages[seat]; option_id=choice['options'][0]['id']
        option=lambda:page.locator('[data-choice-option='+json.dumps(option_id)+']')
        if await option().get_attribute('aria-pressed')=='true': await option().click()
        await option().click(); await expect(option()).to_have_attribute('aria-pressed','true')
        await option().click(); await expect(option()).to_have_attribute('aria-pressed','false')
        await option().click(); await expect(option()).to_have_attribute('aria-pressed','true')
        self.check('real-choice-select-unselect-reselect',seat=seat,choiceId=choice['id'])
        prior_posts=self.post_count
        await page.reload(wait_until='domcontentloaded')
        await expect(option()).to_have_attribute('aria-pressed','true')
        restored=await self.view(seat)
        assert restored['roomId']==view['roomId'] and restored['you']==view['you']
        assert restored['version']==view['version'] and self.post_count==prior_posts
        self.check('reload-retains-own-seat-choice-and-selected-draft',seat=seat,roomId=restored['roomId'],version=restored['version'],extraGamePosts=0)
        await self.screenshot('online-choice-draft-restored',seat)
        await self.submit(seat,page.get_by_role('button',name='暂停并保存此桌',exact=True).click,'pauseRoom')
        paused=await self.view(seat)
        assert paused.get('pause') and paused['pendingChoice']['id']==choice['id']
        await page.reload(wait_until='domcontentloaded')
        await page.wait_for_function('window.__cloudView?.pause')
        assert (await self.view(seat))['roomId']==view['roomId']
        await self.submit(seat,page.get_by_role('button',name='恢复对局',exact=True).click,'resumeRoom')
        await expect(option()).to_have_attribute('aria-pressed','true')
        resumed=await self.view(seat)
        assert not resumed.get('pause') and resumed['pendingChoice']['id']==choice['id']
        self.check('online-pause-reload-resume-keeps-choice-and-draft',pausedVersion=paused['version'],resumedVersion=resumed['version'])
        before_posts=self.post_count;before_records=len(self.records)
        body=await self.submit(seat,lambda:page.get_by_role('button',name='确认选择',exact=True).evaluate('(button)=>{button.click();button.click()}'),'choose')
        await asyncio.sleep(.7)
        assert self.post_count==before_posts+1 and len(self.records)==before_records+1
        action=self.records[-1]['action'];action=action.get('action',action)
        assert action['kind']=='choose' and action['selected']==[option_id]
        assert body['version']==resumed['version']+1
        assert await page.evaluate('Object.keys(sessionStorage).filter(k=>k.startsWith("hegemony.choiceDraft.v1.")).length')==0
        self.coverage['choice:mulligan']+=1
        self.initial_probe_done=True
        self.check('rapid-choice-clicks-one-command',gamePosts=1,versionBefore=resumed['version'],versionAfter=body['version'],completedDraftCleared=True)

async def main(args):
    class Remote:
        remote=True
        base_url=args.url
    output=Path(args.output).resolve()
    if not args.resume: assert not output.exists(), 'Preserve prior browser evidence; use --resume for this owned room'
    async with async_playwright() as p:
        run=PublicGame(p,Remote(),output,args.resume)
        try:
            await run.play_full()
        finally:
            write_json(output/'online-transport.json',dict(publicAnonymous=True,hostingOwnerBypass=False,
                transportEvents=run.transport_events,infrastructurePosts=run.infrastructure_posts,postResults=run.post_results))
    summary=json.loads((output/'full-game-summary.json').read_text())
    assert summary['completeGame'] and summary['passed']
    print(json.dumps(dict(passed=True,completeOnlineGame=True,roomId=summary['roomId'],uiPostCount=summary['uiPostCount'],checks=len(run.online_checks))),flush=True)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--url',required=True);p.add_argument('--output',required=True);p.add_argument('--resume',action='store_true')
    asyncio.run(main(p.parse_args()))
