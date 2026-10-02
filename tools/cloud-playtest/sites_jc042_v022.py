#!/usr/bin/env python3
"""Bounded new-kernel JC042 UI check on a normally dealt owned Site fixture."""
import argparse
import json
import os
from pathlib import Path
from types import SimpleNamespace
from urllib.parse import urlsplit

from playwright.sync_api import sync_playwright
from response_v2 import Fixture, BrowserResponse, character_cards, evidence_view
from common import write_json
from sites_transport import same_origin_redirect, transport_failure


class PublicApi:
    def __init__(self, request, base): self.client, self.base = request, base.rstrip('/')
    def request(self, method, path, body=None, token=None):
        response = self.client.fetch(self.base + path, method=method,
            headers={'Content-Type': 'application/json', **({'Authorization': 'Bearer ' + token} if token else {})},
            data=body, max_redirects=0, timeout=20000)
        return response.status, response.json() if response.status != 204 else None
    def state(self, session):
        status, view = self.request('GET', '/api/rooms/' + session['roomId'] + '/state', token=session['token'])
        assert status == 200
        return view


class PublicBrowser:
    def __init__(self, browser, base):
        self.browser, self.origin, self.transport_events = browser, urlsplit(base).netloc, []
    def new_context(self, **options):
        context = self.browser.new_context(**options)
        def forward(route):
            request = route.request
            if urlsplit(request.url).netloc != self.origin:
                route.abort(); return
            assert 'oai-sites-authorization' not in request.headers
            try:
                response = route.fetch(max_redirects=0, max_retries=0, timeout=20000)
                current = request.url
                for _ in range(5 if request.method in ('GET', 'HEAD') else 0):
                    destination = same_origin_redirect(response, current, self.origin)
                    if not destination: break
                    response = route.fetch(url=destination, max_redirects=0, max_retries=0, timeout=20000)
                    current = destination
                route.fulfill(response=response)
            except Exception as error:
                self.transport_events.append(transport_failure(error, request.url))
                try: route.fulfill(status=503, content_type='application/json', body='{"error":"qa_transport_unavailable"}')
                except Exception: pass
        context.route('**/*', forward)
        return context


def main(args):
    output = Path(args.output); output.mkdir(parents=True, exist_ok=True)
    result = {'testType': 'new-v022-jc042-real-browser-ui-with-legal-api-preparation', 'passed': False,
              'siteBypassUsed': False, 'seedOverride': False, 'intermediateStateInjection': False,
              'countsAsCompleteGame': False, 'engineVersion': 'rust-v0.2.2'}
    fixture = run = public = None
    with sync_playwright() as playwright:
        request = playwright.request.new_context(proxy={'server': os.environ['HTTPS_PROXY']})
        browser = None
        try:
            fixture = Fixture(PublicApi(request, args.base_url), output, 'duel', args.max_steps, 'rust-v0.2.2')
            result['roomId'] = fixture.room_id
            attacker, defender, murder, target = fixture.prepare()
            browser = playwright.chromium.launch(executable_path=args.chromium, headless=True,
                proxy={'server': os.environ['HTTPS_PROXY']}, args=['--no-sandbox', '--disable-dev-shm-usage'])
            public = PublicBrowser(browser, args.base_url)
            run = BrowserResponse(public, fixture, SimpleNamespace(base_url=args.base_url), output)
            run.open(attacker, defender)
            assert all(v['versions']['engine'] == 'rust-v0.2.2' for v in run.views())
            run.action(attacker, murder)
            original = run.views()[0]['stack'][0]
            assert original['cardId'] == 'JC091'
            while run.views()[0]['priorityTeam'] == run.views()[attacker]['players'][attacker]['team']:
                view = run.views()[attacker]
                run.action(attacker, next(a for a in view['legalActions'] if a['kind'] == 'pass'))
            fast = next(a for a in run.views()[defender]['legalActions']
                if a['kind'] == 'activate' and a.get('cardId') == target and a.get('abilityId') == 'reduce-next')
            command, receipt = run.action(defender, fast)
            views = run.views()
            assert all(len(v['stack']) == 1 and v['stack'][0]['cardId'] == 'JC091' for v in views)
            assert all(c['instanceId'] != target for c in character_cards(views[0]))
            assert any(c.get('cardId') == 'JC042' and c['owner'] == 'p' + str(defender) for c in views[0]['graveyard'])
            assert views[0]['stack'][0]['targetSummaries'][0]['status'] == 'missing'
            run.screenshot('immediate-reduction-single-original-stack', defender)
            for page in run.pages: page.reload(wait_until='domcontentloaded')
            run.sync(receipt['version']); assert run.views() == views
            status, replay = fixture.api.request('POST', '/api/rooms/' + fixture.room_id + '/commands', command, fixture.sessions[defender]['token'])
            assert status == 200 and replay == receipt and fixture.views() == views
            run.all_pass_once(1, 'single-pass-round-resolves-original-murder')
            final = run.views()[attacker]
            assert not final['stack']
            assert sum(c['controller'] == final['you'] and not c['exhausted'] for c in final['assets']) == 0
            assert any(c.get('cardId') == 'JC091' and c['owner'] == final['you'] for c in final['graveyard'])
            result.update(immediateReductionHasNoResponseObject=True, refreshExact=True, originalReceiptExact=True,
                noDuplicateSacrifice=True, murderTargetMissing=True, murderCostsNotRefunded=True,
                uiCommands=len(run.commands), final=evidence_view(final), checks=run.checks)
            assert not run.errors and not public.transport_events
            result['passed'] = True
        except Exception as error:
            result['failureType'] = type(error).__name__
        finally:
            if fixture: result['legalApiPreparationCommands'] = len(fixture.records)
            if run: result['browserErrors'] = run.errors
            if public: write_json(output / 'qa-transport-events.json', public.transport_events)
            if browser: browser.close()
            request.dispose()
            write_json(output / 'summary.json', result)
    print(json.dumps({k: v for k, v in result.items() if k not in ('final', 'checks')}, ensure_ascii=False), flush=True)
    return result['passed']


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--max-steps', type=int, default=650)
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    raise SystemExit(0 if main(parser.parse_args()) else 1)
