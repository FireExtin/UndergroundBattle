#!/usr/bin/env python3
"""Bounded anonymous browser entry and independent game authentication on Sites.

No ChatGPT login, Site bypass header, seed override or state injection. This is
an entry/privacy/recovery check, not a complete UI game. Game seat credentials
stay in memory and in their own browser's normal application storage.
"""
import argparse
import asyncio
import json
import os
import re
from pathlib import Path
from urllib.parse import urlsplit

from playwright.async_api import async_playwright
from api_acceptance import assert_projection
from browser_playtest import UiRun
from common import write_json
from sites_browser import PrivateContext
from sites_transport import same_origin_redirect, transport_failure


class PublicBrowser:
    def __init__(self, browser, base):
        self.browser, self.origin = browser, urlsplit(base).netloc
        self.transport_events = []
        self.bypass_header_seen = False

    async def new_context(self, **options):
        context = await self.browser.new_context(**options)
        private = PrivateContext(context)
        async def forward(route):
            request = route.request
            if private.offline:
                await route.abort('internetdisconnected')
                return
            if urlsplit(request.url).netloc != self.origin:
                await route.abort()
                return
            assert 'oai-sites-authorization' not in request.headers
            try:
                response = await route.fetch(max_redirects=0, max_retries=0, timeout=20000)
                current_url = request.url
                for _ in range(5 if request.method in ('GET', 'HEAD') else 0):
                    destination = same_origin_redirect(response, current_url, self.origin)
                    if destination is None:
                        break
                    response = await route.fetch(url=destination, max_redirects=0, max_retries=0, timeout=20000)
                    current_url = destination
                await route.fulfill(response=response)
            except Exception as error:
                self.transport_events.append(transport_failure(error, request.url))
                try:
                    await route.fulfill(status=503, content_type='application/json', body='{"error":"qa_transport_unavailable"}')
                except Exception:
                    pass
        await context.route('**/*', forward)
        return private


def instance_ids(value):
    if isinstance(value, list):
        return set().union(*(instance_ids(v) for v in value)) if value else set()
    if isinstance(value, dict):
        return ({value['instanceId']} if 'instanceId' in value else set()).union(
            *(instance_ids(v) for v in value.values()))
    return set()


async def main(args):
    output = Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    evidence = {'testType': 'public-anonymous-browser-entry-privacy-recovery', 'passed': False,
                'siteUrl': args.base_url, 'sourceCommit': args.source_commit,
                'siteBypassUsed': False, 'chatGptLoginUsed': False, 'countsAsCompleteUiGame': False,
                'checks': [], 'apiGameplayCommands': 0}
    run = public = None
    try:
        async with async_playwright() as playwright:
            proxy = os.environ.get('HTTPS_PROXY') or os.environ.get('https_proxy')
            browser = await playwright.chromium.launch(executable_path=args.chromium, headless=True,
                proxy={'server': proxy} if proxy else None, args=['--no-sandbox', '--disable-dev-shm-usage'])
            try:
                public = PublicBrowser(browser, args.base_url)
                run = UiRun(public, args.base_url, output, 70, 'develop', True)
                run.mode = 'teams'
                host = await run.new_seat(0)
                assert not await run.contexts[0].cookies('https://auth.openai.com')
                catalog = await host.evaluate('window.__cloudCatalog')
                run.catalog = {c['id']: c for c in catalog['cards']}
                decks = ['watchers', 'hunters', 'keepers', 'responders']
                await host.get_by_label('你的称呼').fill('匿名云端甲')
                await host.get_by_role('button', name=re.compile('四人协作')).click()
                await host.get_by_role('button', name='创建牌桌 →', exact=True).click()
                await host.wait_for_function('window.__cloudView?.status === "lobby"')
                invitation = (await run.view(0))['inviteCode']
                for seat in range(1, 4):
                    page = await run.new_seat(seat)
                    await page.locator('.hg-deck[data-deck-id=' + json.dumps(decks[seat]) + ']').click()
                    await page.get_by_role('button', name='邀请码加入', exact=True).click()
                    await page.get_by_label('你的称呼').fill('匿名云端' + '乙丙丁'[seat - 1])
                    await page.get_by_label('邀请码', exact=True).fill(invitation)
                    await page.get_by_role('button', name='加入牌桌 →', exact=True).click()
                    await page.wait_for_function('window.__cloudView?.status === "lobby"')
                await run.sync(3)
                room = await run.view(0)
                evidence['roomId'] = room['roomId']
                evidence['checks'].append('four-independent-anonymous-browser-seats-created-and-joined-by-ui')
                for seat in range(4):
                    await run.legal_button(seat, next(a for a in (await run.view(seat))['legalActions'] if a['kind'] == 'ready'))
                await run.legal_button(0, next(a for a in (await run.view(0))['legalActions'] if a['kind'] == 'start'))
                views = await run.views()
                for seat, view in enumerate(views):
                    assert_projection(view)
                    for other, other_view in enumerate(views):
                        if other != seat:
                            assert not {c['instanceId'] for c in other_view['hand']}.intersection(instance_ids(view))
                    assert bool(view.get('pendingChoice')) == (seat == 0)
                evidence['checks'].append('other-hands-and-private-choice-options-absent-from-every-seat-view')
                await run.verify_recovery(0, views[0])
                request = run.contexts[0].request
                endpoint = args.base_url.rstrip('/') + '/api/rooms/' + room['roomId']
                denied_state = await request.get(endpoint + '/state')
                denied_command = await request.post(endpoint + '/commands', data={
                    'commandId': 'anonymous-denied', 'expectedVersion': views[0]['version'], 'action': {'kind': 'pass'}})
                assert denied_state.status == denied_command.status == 401
                foreign = await request.post(args.base_url.rstrip('/') + '/api/rooms', data={
                    'name': '匿名跨房探针', 'mode': 'duel', 'deckId': 'watchers'})
                assert foreign.status == 200
                foreign_session = await foreign.json()
                cross_room = await request.get(endpoint + '/state', headers={'Authorization': 'Bearer ' + foreign_session['token']})
                assert cross_room.status == 401
                listing = await request.get(args.base_url.rstrip('/') + '/api/rooms')
                assert listing.status == 404
                assert await run.views() == views
                evidence['checks'].append('anonymous-state-command-and-cross-room-token-rejected-without-view-change-no-public-room-list')
                for _ in range(70):
                    views = await run.views()
                    if any(c.get('faceDown') for r in views[0]['regions'] for c in r['characters']):
                        break
                    pending = next(((seat, view) for seat, view in enumerate(views) if view.get('pendingChoice')), None)
                    if pending:
                        await run.choose(*pending)
                    else:
                        candidates = [(seat, run.policy(seat, view)) for seat, view in enumerate(views)]
                        chosen = next(((seat, action) for seat, action in candidates if action and action['kind'] != 'pass'), None)
                        chosen = chosen or next(((seat, action) for seat, action in candidates if action), None)
                        assert chosen
                        await run.legal_button(*chosen)
                else:
                    raise AssertionError('No secret deployment reached within bounded legal UI actions')
                for view in await run.views():
                    assert_projection(view)
                await run.pages[0].set_viewport_size({'width': 390, 'height': 844})
                assert not await run.pages[0].evaluate('document.documentElement.scrollWidth > innerWidth + 1')
                await run.screenshot('anonymous-phone-hidden-board')
                evidence['checks'].append('secret-deployment-hidden-from-other-seats-and-phone-board-fits')
                assert not run.errors and not public.transport_events
                evidence.update(passed=True, independentContexts=4, finalVersion=(await run.view(0))['version'])
            finally:
                for context in run.contexts if run else []:
                    await context.close()
                await browser.close()
    except Exception as error:
        # Assertion messages and whitelist network causes are safe; raw request
        # exceptions may contain the application seat header and must be suppressed.
        evidence['failure'] = str(error) if isinstance(error, AssertionError) else type(error).__name__ + ': request error suppressed'
        raise RuntimeError(evidence['failure']) from None
    finally:
        evidence.update(uiAcceptedCommands=len(run.records) if run else 0,
                        browserErrors=run.errors if run else [], recovery=run.recovery if run else [],
                        expectedBrowserErrors=run.expected_errors if run else [],
                        transportEvents=public.transport_events if public else [])
        write_json(output / 'public-summary.json', evidence)
        print(json.dumps(evidence, ensure_ascii=False), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    asyncio.run(main(parser.parse_args()))
