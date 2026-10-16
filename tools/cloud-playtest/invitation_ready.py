"""Isolated real Worker/D1 invitation and two-player readiness acceptance."""
import argparse
import asyncio
import datetime
import json
import re
from pathlib import Path
from urllib.parse import urlsplit
from playwright.async_api import async_playwright, expect

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--url',default='http://127.0.0.1:8113')
parser.add_argument('--source',required=True)
parser.add_argument('--mode',choices=['baseline','fixed'],required=True)
parser.add_argument('--output',required=True)
args=parser.parse_args()
assert urlsplit(args.url).hostname in {'localhost','127.0.0.1'}, 'Only isolated localhost tables are allowed'
OUT=Path(args.output);OUT.mkdir(exist_ok=False)


async def main():
    result={'source':args.source,'mode':args.mode,'kind':'isolated-production-worker-d1-browser',
            'atUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'checks':[],
            'posts':[],'pageErrors':[],'realUserTableAccessed':False,'remoteDeployment':False}
    views={'host':[],'friend':[]}
    def check(name,value):
        result['checks'].append({'name':name,'passed':bool(value)});assert value,name
    async with async_playwright() as p:
        browser=await p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage'])
        host_context=await browser.new_context(viewport={'width':1366,'height':900},locale='zh-CN')
        friend_context=await browser.new_context(viewport={'width':390,'height':844},locale='zh-CN')
        host=await host_context.new_page();friend=await friend_context.new_page()
        for label,page in [('host',host),('friend',friend)]:
            page.on('pageerror',lambda e:result['pageErrors'].append(str(e)))
            async def response(r,label=label):
                if '/api/rooms' not in urlsplit(r.url).path:return
                try:
                    data=await r.json();view=data.get('view',data)
                    if isinstance(view,dict) and 'players' in view and 'inviteCode' in view:views[label].append(view)
                except Exception:pass
            def request(r,label=label):
                if r.method!='POST':return
                parsed=urlsplit(r.url);assert parsed.hostname in {'localhost','127.0.0.1'}
                body=r.post_data_json or {};action=body.get('action',{})
                result['posts'].append({'player':label,'operation':'command' if parsed.path.endswith('/commands') else 'join' if parsed.path.endswith('/join') else 'create','action':action.get('kind'),'nestedAction':action.get('action',{}).get('kind')})
            page.on('response',response);page.on('request',request)
        async def create(page,name):
            await page.goto(args.url,wait_until='networkidle')
            await page.get_by_role('navigation',name='大厅导航').get_by_role('button',name='入席',exact=True).click()
            await page.get_by_role('textbox',name='你的称呼',exact=True).fill(name)
            await page.get_by_role('button',name='创建牌桌 →',exact=True).click()
            await page.locator('.hg-invite strong').wait_for()
            invite=await page.locator('.hg-invite strong').inner_text()
            await page.get_by_role('button',name='准备',exact=True).click()
            await expect(page.locator('.hg-room-ready > span')).to_have_text(re.compile(r'1\s*/\s*2'))
            return invite
        try:
            target=await create(host,'隔离房主')
            old=await create(friend,'隔离朋友旧桌')
            check('two-isolated-tables-have-different-invitations',target!=old)
            await friend.goto(args.url+'/?invite='+target+('#home' if args.mode=='fixed' else ''),wait_until='networkidle')
            if args.mode=='baseline':
                await friend.locator('.hg-invite strong').wait_for()
                check('v56-restores-old-table-despite-different-invitation',await friend.locator('.hg-invite strong').inner_text()==old)
                check('address-still-contains-target-invitation',urlsplit(friend.url).query=='invite='+target)
                check('no-join-request-was-made',not any(x['operation']=='join' for x in result['posts']))
                await expect(host.locator('.hg-room-ready > span')).to_have_text(re.compile(r'1\s*/\s*2'))
                await expect(friend.locator('.hg-room-ready > span')).to_have_text(re.compile(r'1\s*/\s*2'))
                check('both-own-tables-show-cancel-ready',await host.get_by_role('button',name='取消准备',exact=True).count()==1 and await friend.get_by_role('button',name='取消准备',exact=True).count()==1)
                result['reproducedReportedSymptom']=True
            else:
                await expect(friend.get_by_role('textbox',name='邀请码',exact=True)).to_have_value(target)
                check('different-invitation-opens-join-screen',await friend.get_by_text('等待秘社集结',exact=True).count()==0)
                check('invitation-has-no-create-or-mode-controls',await friend.get_by_role('button',name=re.compile('创建牌桌')).count()==0 and await friend.get_by_role('group',name='对战模式').count()==0)
                check('invitation-target-is-readonly',await friend.get_by_role('textbox',name='邀请码',exact=True).get_attribute('readonly') is not None)
                old_saved=await friend.evaluate('JSON.parse(localStorage.getItem("hegemony.seats.v1")||"[]").map(s=>({inviteCode:s.inviteCode,seat:s.seat}))')
                check('old-seat-is-preserved-before-join',any(s['inviteCode']==old and s['seat']==0 for s in old_saved))
                await friend.get_by_role('textbox',name='你的称呼',exact=True).fill('   ')
                await friend.get_by_role('button',name='加入牌桌 →',exact=True).click()
                await expect(friend.locator('.hg-invite strong')).to_have_text(target)
                generated_name=await friend.evaluate('localStorage.getItem("hegemony.playerName.v1")')
                check('blank-player-name-is-readable-and-persisted',bool(re.fullmatch(r'旅人·[A-F0-9]{6}',generated_name or '')))
                await expect(host.locator('.hg-seat').filter(has_text=generated_name)).to_have_count(1)
                check('host-and-friend-share-target-invitation',await host.locator('.hg-invite strong').inner_text()==target)
                await friend.get_by_role('button',name='准备',exact=True).click()
                await expect(host.locator('.hg-room-ready > span')).to_have_text(re.compile(r'2\s*/\s*2'))
                await expect(friend.locator('.hg-room-ready > span')).to_have_text(re.compile(r'2\s*/\s*2'))
                check('both-players-synchronize-to-two-ready',True)
                # Keep the original joined deck A, then select custom B through the real editor.
                actual=friend.get_by_label('实际上桌牌组',exact=True)
                original_deck=await actual.locator('h2').inner_text()
                await friend.get_by_role('button',name='返回大厅 / 新建牌桌',exact=True).click()
                await friend.get_by_role('navigation',name='大厅导航').get_by_role('button',name='入席',exact=True).click()
                check('browser-remembers-generated-player-name',await friend.get_by_role('textbox',name='你的称呼',exact=True).input_value()==generated_name)
                await friend.get_by_role('navigation',name='大厅导航').get_by_role('button',name='卡牌收藏',exact=True).click()
                check('deck-name-is-visible-before-settings',await friend.get_by_role('textbox',name='牌组名称',exact=True).is_visible())
                await friend.get_by_role('button',name=re.compile('牌组设置与模板')).click()
                await friend.get_by_role('button',name='复制为草稿',exact=True).click()
                await friend.get_by_role('textbox',name='牌组名称',exact=True).fill('隔离已选牌组B')
                copies=friend.get_by_role('spinbutton',name='无知路人（JC125）张数',exact=True)
                old_copies=int(await copies.input_value())
                await copies.fill(str(old_copies+1))
                await friend.get_by_role('button',name='保存并选择此牌组',exact=True).click()
                await expect(friend.get_by_label('所选牌组',exact=True)).to_contain_text('隔离已选牌组B · 51 张')
                await expect(friend.get_by_role('combobox',name='已保存牌组',exact=True)).to_contain_text('隔离已选牌组B')
                await friend.get_by_role('navigation',name='大厅导航').get_by_role('button',name='卡牌收藏',exact=True).click()
                await friend.get_by_role('button',name='收起牌组设置',exact=True).click()
                check('saved-deck-list-and-name-remain-visible-with-settings-closed',await friend.get_by_role('list',name='已保存牌组',exact=True).is_visible() and await friend.get_by_role('textbox',name='牌组名称',exact=True).is_visible())
                # Saving later edits does not silently replace the chosen 51-card snapshot.
                await friend.get_by_role('spinbutton',name='无知路人（JC125）张数',exact=True).fill(str(old_copies+2))
                await friend.get_by_role('button',name='保存牌组',exact=True).click()
                await expect(friend.get_by_role('list',name='已保存牌组',exact=True)).to_contain_text('52 张')
                await friend.goto(args.url+'/?invite='+target+'#home',wait_until='networkidle')
                await expect(friend.get_by_label('所选牌组',exact=True)).to_contain_text('隔离已选牌组B · 51 张')
                check('same-invitation-requires-explicit-join',await friend.get_by_text('等待秘社集结',exact=True).count()==0)
                await friend.get_by_role('button',name='加入牌桌 →',exact=True).click()
                await expect(friend.get_by_label('实际上桌牌组',exact=True).locator('h2')).to_have_text(original_deck)
                await expect(friend.get_by_role('region',name='待应用的牌组',exact=True)).to_contain_text('隔离已选牌组B')
                check('restored-seat-keeps-deck-A-until-explicit-apply',await friend.get_by_role('button',name='取消准备',exact=True).count()==1)
                await friend.screenshot(path=str(OUT/'phone-restored-before-apply.png'),full_page=True)
                await friend.get_by_role('button',name='应用所选牌组到此席位',exact=True).click()
                await expect(friend.get_by_label('实际上桌牌组',exact=True)).to_contain_text('隔离已选牌组B')
                await expect(friend.get_by_label('实际上桌牌组',exact=True)).to_contain_text('原始组成 51 张')
                await expect(friend.locator('.hg-room-ready > span')).to_have_text(re.compile(r'1\s*/\s*2'))
                await expect(host.locator('.hg-room-ready > span')).to_have_text(re.compile(r'1\s*/\s*2'))
                check('explicit-deck-B-apply-uses-selected-snapshot-and-cancels-ready',True)
                # A later explicit choice inside this room supersedes the old snapshot.
                await friend.locator('.hg-room-prepare > summary').click()
                await friend.get_by_role('button',name='保存并选择此牌组',exact=True).click()
                await expect(friend.get_by_label('实际上桌牌组',exact=True)).to_contain_text('原始组成 52 张')
                await expect(friend.get_by_role('region',name='待应用的牌组',exact=True)).to_have_count(0)
                current_selection=await friend.evaluate('JSON.parse(sessionStorage.getItem("hegemony.lobbyDeckSelection.v1"))')
                check('later-explicit-room-selection-replaces-the-stale-lobby-snapshot',current_selection['kind']=='draft' and sum(card['count'] for card in current_selection['draft']['cards'])==52)
                result['deckProgression']={'restoredOriginalCount':50,'chosenSnapshotCount':51,'savedLaterCount':52,'firstExplicitApplicationCount':51,'laterExplicitRoomApplicationCount':52}
                await friend.get_by_role('button',name='准备',exact=True).click()
                await expect(host.locator('.hg-room-ready > span')).to_have_text(re.compile(r'2\s*/\s*2'))
                await friend.reload(wait_until='networkidle')
                await expect(friend.locator('.hg-room-ready > span')).to_have_text(re.compile(r'2\s*/\s*2'))
                check('refresh-keeps-second-seat-and-readiness',any(v['inviteCode']==target and v['you']=='p1' for v in views['friend']))
                check('only-one-target-join-request',sum(x['operation']=='join' for x in result['posts'])==1)
                saved=await friend.evaluate('JSON.parse(localStorage.getItem("hegemony.seats.v1")||"[]").map(s=>({inviteCode:s.inviteCode,seat:s.seat}))')
                check('both-original-and-joined-seat-remain-saved',any(s['inviteCode']==old and s['seat']==0 for s in saved) and any(s['inviteCode']==target and s['seat']==1 for s in saved))
                await host.locator('.hg-room-ready').get_by_role('button',name=re.compile('开始')).click()
                await expect(host.get_by_text('等待秘社集结',exact=True)).to_have_count(0)
                await expect(friend.get_by_text('等待秘社集结',exact=True)).to_have_count(0)
                check('host-start-reaches-playing-for-both-seats',all(any(v['inviteCode']==target and v['status']=='playing' for v in views[label]) for label in ['host','friend']))
                # Complete the two initial mulligans with ordinary UI controls.
                for page in [host,friend]:
                    await page.get_by_role('dialog',name='待完成的选择',exact=True).get_by_role('button',name='保留全部手牌',exact=True).click()
                await expect(host.get_by_role('dialog',name='待完成的选择',exact=True)).to_have_count(0)
                await expect(friend.get_by_role('dialog',name='待完成的选择',exact=True)).to_have_count(0)
                await expect(host.locator('.hg-phase-line')).not_to_contain_text('再调度')
                await expect(friend.locator('.hg-phase-line')).not_to_contain_text('再调度')
                check('both-seats-complete-initial-choices-through-real-ui',True)
                await expect(friend.locator('.hg-player-you .hg-player-strip > small')).to_contain_text('隔离已选牌组B · 原始 52 张')
                check('started-table-keeps-applied-deck-B-and-no-deck-change-control',await friend.get_by_role('button',name='应用所选牌组到此席位',exact=True).count()==0)
                await friend.locator('.hg-hand .hg-archive-art img').first.wait_for(state='visible')
                await friend.wait_for_function("""() => [...document.querySelectorAll('.hg-hand .hg-archive-art img')].some(img => img.complete && img.naturalWidth > 0 && img.getBoundingClientRect().height > 30 && img.parentElement.getBoundingClientRect().height >= 40)""")
                result['mobileArtwork']=await friend.locator('.hg-hand .hg-archive-art img').evaluate_all("""images => images.map(img => ({loaded:img.complete&&img.naturalWidth>0,artHeight:img.parentElement.getBoundingClientRect().height,visible:getComputedStyle(img.parentElement).display!=='none'}))""")
                check('phone-hand-artwork-is-loaded-and-visible',any(item['loaded'] and item['visible'] and item['artHeight']>=40 for item in result['mobileArtwork']))
                check('concealed-cards-and-assets-have-no-print-image',await friend.locator('.hg-card[data-card-face-down=true] .hg-archive-art img, .hg-archive-asset-art img, .hg-other-hand img').count()==0)
                await friend.screenshot(path=str(OUT/'phone-playing-artwork.png'),full_page=True)
                await host.screenshot(path=str(OUT/'desktop-playing.png'),full_page=True)
                result['targetReadySummary']={label:[{'you':v['you'],'version':v['version'],'status':v['status'],'ready':sum(bool(p['ready']) for p in v['players']),'players':len(v['players'])} for v in history if v['inviteCode']==target] for label,history in views.items()}
            check('no-page-errors',not result['pageErrors']);result['passed']=True
        except Exception as e:
            result.update(passed=False,failure=type(e).__name__+': '+str(e))
        finally:
            await host_context.close();await friend_context.close();await browser.close()
    (OUT/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(result,ensure_ascii=False),flush=True)
    return 0 if result['passed'] else 1


raise SystemExit(asyncio.run(main()))
