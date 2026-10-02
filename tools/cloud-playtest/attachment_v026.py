#!/usr/bin/env python3
"""Bounded natural four-seat attachment UI acceptance; no API state setup.

Every seat has its own persistent Chromium profile. The fetch observer reads
only that seat's normal projected responses. Credentials and hands stay local.
All POST requests, including creation/join, share a strict forty-request budget.
"""
import argparse
import asyncio
import json
import os
import re
import time
from pathlib import Path
from urllib.parse import urlsplit

from playwright.async_api import async_playwright

from browser_playtest import OBSERVE_FETCH, UiRun
from common import IsolatedService, public_view, write_json
from sites_transport import same_origin_redirect, transport_failure


class AttachmentRun(UiRun):
    def __init__(self, playwright, service, output):
        super().__init__(None, service.base_url, output, 40)
        self.playwright, self.service = playwright, service
        self.mode = 'teams'
        self.post_count = 0
        self.post_results = []
        self.measurements = []
        self.checks = []
        self.remote = getattr(service, 'remote', False)
        self.transport_events = []

    async def new_seat(self, seat, session=None):
        assert session is None, 'Only normal profile restoration is allowed'
        proxy = os.environ.get('HTTPS_PROXY') or os.environ.get('https_proxy')
        context = await self.playwright.chromium.launch_persistent_context(
            str(self.output / f'profile-p{seat}'), executable_path='/usr/bin/chromium',
            headless=True, viewport={'width': 1180, 'height': 760}, locale='zh-CN',
            **({'proxy': {'server': proxy}} if self.remote and proxy else {}),
            args=['--no-sandbox', '--disable-dev-shm-usage'])
        await context.add_init_script(OBSERVE_FETCH)

        async def budget(route):
            request = route.request
            if request.method == 'POST' and '/api/' in request.url:
                assert self.post_count < 40, 'UI POST budget exhausted'
                self.post_count += 1
            if not self.remote:
                await route.continue_()
                return
            origin = urlsplit(self.base).netloc
            if urlsplit(request.url).netloc != origin:
                await route.abort()
                return
            assert 'oai-sites-authorization' not in request.headers
            try:
                response = await route.fetch(max_redirects=0, max_retries=0, timeout=20000)
                current = request.url
                for _ in range(5 if request.method in ('GET', 'HEAD') else 0):
                    destination = same_origin_redirect(response, current, origin)
                    if destination is None:
                        break
                    response = await route.fetch(url=destination, max_redirects=0, max_retries=0, timeout=20000)
                    current = destination
                await route.fulfill(response=response)
            except Exception as error:
                self.transport_events.append(transport_failure(error, request.url))
                await route.fulfill(status=503, content_type='application/json', body='{"error":"qa_transport_unavailable"}')

        await context.route('**/*' if self.remote else '**/api/**', budget)
        page = context.pages[0] if context.pages else await context.new_page()
        page.on('pageerror', lambda error: self.errors.append({'seat': seat, 'kind': 'pageerror', 'message': str(error)}))
        page.on('console', lambda msg: self.errors.append({'seat': seat, 'kind': 'console', 'message': msg.text}) if msg.type == 'error' else None)
        page.on('response', lambda r: self.post_results.append({'seat': seat, 'status': r.status, 'endpoint': r.url.split('/api/', 1)[-1]}) if r.request.method == 'POST' else None)
        self.contexts.append(context)
        self.pages.append(page)
        await page.goto(self.base, wait_until='domcontentloaded')
        await page.wait_for_function('window.__cloudCatalog && window.__cloudCatalog.decks.length >= 4', timeout=30000)
        return page

    async def confirm_mulligan(self, seat, view):
        choice = view['pendingChoice']
        page = self.pages[seat]
        panel = page.locator('.hg-choice')
        await panel.wait_for(state='visible')
        # Keep equipment and one cheap human in this player's own hand; replace
        # other dealt cards through the actual one-time mulligan UI.
        kept_host = False
        for index, option in enumerate(choice['options']):
            card = option.get('card', {})
            if card.get('cardId') == 'BQ022':
                continue
            if not kept_host and card.get('cardId') == 'JC125':
                kept_host = True
                continue
            await panel.locator('.hg-choice-option').nth(index).click()
        footer = panel.locator('.hg-choice-footer')
        box = await footer.bounding_box()
        assert box and box['y'] >= 0 and box['y'] + box['height'] <= 761, 'Mulligan confirmation left the desktop viewport'
        self.measurements.append({'seat': seat, 'mulliganFooterVisible': True, 'viewport': [1180, 760]})
        await self.submit(seat, panel.get_by_role('button', name='确认选择', exact=True).click, 'choose')

    async def wait_stack(self):
        # Let the real server expire undecided five-second windows. No fake
        # timers, hand inspection, scripted pass API, or automatic client click.
        if not any(c['name'] == 'real-five-second-response-window' for c in self.checks):
            for seat in range(4):
                v = await self.view(seat)
                window = v.get('responseWindow')
                if window and any(m.get('deadlineMs', 0) - v['serverNowMs'] <= 5000 and m.get('deadlineMs', 0) > v['serverNowMs'] for m in window['members']):
                    assert await self.pages[seat].get_by_role('timer', name='连锁决定倒计时').count() or not any(m['playerId'] == v['you'] and m['status'] == 'undecided' for m in window['members'])
                    self.checks.append({'name': 'real-five-second-response-window', 'passed': True})
                    break
        await self.pages[0].wait_for_function('window.__cloudView && !window.__cloudView.stack.length', timeout=18000)
        await self.sync((await self.view(0))['version'])

    async def inspect_attachment(self):
        v = await self.view(0)
        attachment = v['attachments'][0]
        host_id = attachment['hostId']
        for seat in range(4):
            own = await self.view(seat)
            assert own['attachments'] == v['attachments']
            host = next(c for r in own['regions'] for c in r['characters'] if c['instanceId'] == host_id)
            printed = self.catalog[host['cardId']]['permanentIcons']['combat']
            assert host['icons']['combat'] == printed + 1
            page = self.pages[seat]
            tile = page.locator('[data-card-instance=' + json.dumps(host_id) + ']')
            await tile.click()
            badge = tile.locator('.hg-card-attachment-count')
            assert await badge.count() == 1
            link = page.locator('[data-attachment-instance=' + json.dumps(attachment['instanceId']) + ']')
            await link.click()
            dialog = page.get_by_role('dialog', name='放大阅读合金指虎', exact=True)
            await dialog.wait_for(state='visible')
            assert '拥有' in await dialog.inner_text()
            await dialog.get_by_role('button', name='原始牌面', exact=True).click()
            scan = dialog.get_by_role('img', name='合金指虎原始牌面', exact=True)
            await scan.wait_for(state='visible')
            await page.wait_for_function('(id) => { const image=document.querySelector(`[data-reading-card="${id}"] img`); return image && image.complete && image.naturalWidth > 0; }', arg=attachment['instanceId'])
            await page.screenshot(path=str(self.output / f'teams-attachment-reader-p{seat}.png'), full_page=False)
            await dialog.get_by_role('button', name='关闭放大阅读').click()
            await page.get_by_role('button', name='关闭卡牌详情').click()
        self.checks.append({'name': 'four-seats-host-current-combat-and-owner-reading', 'passed': True})
        self.checks.append({'name': 'four-seats-original-attachment-scan-loads', 'passed': True})

    async def run(self, resume=False):
        started = time.monotonic()
        try:
            if resume:
                previous = json.loads((self.output / 'attachment-ui-summary.json').read_text())
                self.post_count = previous['uiPostCount']
                self.post_results = previous['postResults']
                self.checks = previous['checks']
                self.measurements = previous['measurements']
                for seat in range(4):
                    await self.new_seat(seat)
                await self.pages[0].wait_for_function('window.__cloudView')
                await self.sync((await self.view(0))['version'])
                assert (await self.view(0))['roomId'] == previous['roomId']
                c = await self.pages[0].evaluate('window.__cloudCatalog')
                self.catalog = {card['id']: card for card in c['cards']}
                await self.inspect_attachment()
                result = await self.finish_recovery(started)
                return result
            host = await self.new_seat(0)
            c = await host.evaluate('window.__cloudCatalog')
            self.catalog = {card['id']: card for card in c['cards']}
            await host.locator('.hg-deck[data-deck-id="hunters"]').click()
            await host.get_by_label('你的称呼').fill('附属验收甲')
            await host.get_by_role('button', name=re.compile('四人协作')).click()
            await host.get_by_role('button', name='创建牌桌 →', exact=True).click()
            await host.wait_for_function('window.__cloudView && window.__cloudView.status === "lobby"')
            invitation = (await self.view(0))['inviteCode']
            for seat in range(1, 4):
                page = await self.new_seat(seat)
                await page.locator('.hg-deck[data-deck-id="hunters"]').click()
                await page.get_by_role('button', name='邀请码加入', exact=True).click()
                await page.get_by_label('你的称呼').fill(f'附属验收{seat + 1}')
                await page.get_by_label('邀请码', exact=True).fill(invitation)
                await page.get_by_role('button', name='加入牌桌 →', exact=True).click()
                await page.wait_for_function('window.__cloudView && window.__cloudView.status === "lobby"')
            await self.sync((await self.view(3))['version'])
            for seat in range(4):
                v = await self.view(seat)
                await self.legal_button(seat, next(a for a in v['legalActions'] if a['kind'] == 'ready'))
            await self.legal_button(0, next(a for a in (await self.view(0))['legalActions'] if a['kind'] == 'start'))
            while (await self.view(0)).get('waitingChoice') or (await self.view(0)).get('pendingChoice'):
                for seat in range(4):
                    v = await self.view(seat)
                    if v.get('pendingChoice'):
                        assert v['pendingChoice']['kind'] == 'mulligan'
                        await self.confirm_mulligan(seat, v)
                        break
            assert self.post_count == 13, f'Unexpected setup POST count {self.post_count}'
            for seat in range(4):
                page = self.pages[seat]
                sizes = await page.locator('.hg-hand .hg-card').evaluate_all('(es) => es.map(e=>e.getBoundingClientRect().width)')
                assert sizes and min(sizes) >= 127
                assert not await page.evaluate('document.documentElement.scrollWidth > innerWidth + 1')
                assert await page.locator('.hg-region').count() == 5
                self.measurements.append({'seat': seat, 'minimumHandWidth': round(min(sizes), 2), 'fiveRegions': True, 'horizontalOverflow': False})
            await self.screenshot('natural-setup-1180x760')
            while self.post_count < 36:
                latest = await self.view(0)
                if latest.get('stack'):
                    await self.wait_stack()
                    continue
                if latest.get('attachments'):
                    await self.inspect_attachment()
                    break
                candidate = None
                # Each decision below uses only the acting seat's own projection.
                for seat in range(4):
                    v = await self.view(seat)
                    hand = {c['instanceId']: c for c in v['hand']}
                    actions = v['legalActions']
                    equip = next((a for a in actions if a['kind'] == 'play' and hand.get(a.get('cardId'), {}).get('cardId') == 'BQ022'), None)
                    if equip:
                        candidate = (0, seat, equip)
                        break
                    need_fee = any(c.get('cardId') == 'BQ022' for c in v['hand']) and not any(c['controller'] == v['you'] and not c['exhausted'] for c in v['assets'])
                    asset = next((a for a in actions if a['kind'] == 'asset' and hand.get(a.get('cardId'), {}).get('cardId') != 'BQ022'), None)
                    cheap_host = next((a for a in actions if a['kind'] == 'deploy' and hand.get(a.get('cardId'), {}).get('cardId') == 'JC125'), None)
                    move = asset if need_fee and asset else cheap_host or next((a for a in actions if a['kind'] == 'pass'), None)
                    if move:
                        score = 1 if move is asset else 2 if move is cheap_host else 3
                        if candidate is None or score < candidate[0]:
                            candidate = (score, seat, move)
                assert candidate, 'No natural UI action available'
                _, seat, action = candidate
                await self.legal_button(seat, action)
                print(json.dumps({'postCount': self.post_count, 'seat': seat, 'action': action['kind'], 'version': (await self.view(seat))['version']}), flush=True)
            assert (await self.view(0)).get('attachments'), 'Natural draw did not reach an attachment within the POST budget'
            result = await self.finish_recovery(started)
        except Exception as error:
            if self.pages:
                await self.screenshot('failure')
            result = {'passed': False, 'failure': f'{type(error).__name__}: {error}'}
        finally:
            final = await self.view(0) if self.pages else None
            result.update(testType='natural-four-seat-persistent-browser-ui', stateInjection=False,
                          apiMoves=False, uiPostCount=self.post_count, uiPostBudget=40,
                          deployedSite=self.remote, siteBypassUsed=False, transportEvents=self.transport_events,
                          postResults=self.post_results, checks=self.checks, measurements=self.measurements,
                          browserErrors=self.errors, final=public_view(final),
                          roomId=final.get('roomId') if final else None,
                          durationSeconds=round(time.monotonic() - started, 2))
            write_json(self.output / 'attachment-ui-summary.json', result)
            for context in self.contexts:
                await context.close()
        print(json.dumps(result, ensure_ascii=False), flush=True)
        assert result['passed'], result.get('failure')

    async def finish_recovery(self, started):
        version = (await self.view(0))['version']
        if not self.remote:
            self.service.restart()
        for page in self.pages:
            await page.reload(wait_until='domcontentloaded')
        await self.sync(version)
        self.checks.append({'name': 'four-own-seats-and-attachment-restored-after-page-reload' if self.remote else 'native-service-restart-restores-four-own-seats-and-attachment', 'passed': True})
        for context in self.contexts:
            await context.close()
        self.contexts, self.pages = [], []
        for seat in range(4):
            await self.new_seat(seat)
        await self.sync(version)
        assert (await self.view(0))['attachments']
        self.checks.append({'name': 'four-persistent-browser-profile-reopens', 'passed': True})
        assert not self.errors, f'Browser errors: {self.errors}'
        assert all(r['status'] == 200 for r in self.post_results)
        return {'passed': True}


async def main(args):
    class Remote:
        remote = True
        base_url = args.public_url
        def start(self): pass
        def stop(self): pass
    assert args.public_url or args.binary, 'Provide a native binary or the selected public Site URL'
    service = Remote() if args.public_url else IsolatedService(args.binary, args.cwd, args.output, 8106, args.static_dir)
    service.start()
    try:
        async with async_playwright() as playwright:
            await AttachmentRun(playwright, service, Path(args.output).resolve()).run(args.resume)
    finally:
        service.stop()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary')
    parser.add_argument('--public-url', help='Selected existing public Site; anonymous UI only, no bypass token')
    parser.add_argument('--cwd', default='.')
    parser.add_argument('--static-dir', default='web/dist')
    parser.add_argument('--output', required=True)
    parser.add_argument('--resume', action='store_true', help='Read-only inspection/recovery of this runner\'s existing four profiles')
    asyncio.run(main(parser.parse_args()))
