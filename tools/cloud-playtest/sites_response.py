#!/usr/bin/env python3
"""Remote D1 sacrifice/response UI, using only legal ordinary API preparation.

This is a bounded fixture, not a complete UI game. At the committed mid-response
checkpoint the owning agent republishes the exact same saved Site version, then
resumes via hidden stdin. No production test hooks or storage edits are used.
"""
import argparse
import getpass
import json
import os
import threading
from pathlib import Path
from types import SimpleNamespace
from urllib.parse import urlsplit

from playwright.sync_api import sync_playwright
from common import write_json
from response_v2 import BrowserResponse, Fixture, evidence_view
from sites_protocol import SitesApi
from sites_transport import same_origin_redirect, transport_failure


class PrivateBrowser:
    def __init__(self, browser, base, bypass):
        self.browser, self.origin, self.bypass = browser, urlsplit(base).netloc, bypass
        self.transport_events = []

    def new_context(self, **options):
        context = self.browser.new_context(**options)

        def forward(route):
            request = route.request
            if urlsplit(request.url).netloc != self.origin:
                route.abort()
                return
            try:
                response = route.fetch(headers={**request.headers, 'OAI-Sites-Authorization': 'Bearer ' + self.bypass},
                                       max_redirects=0, max_retries=0, timeout=20000)
                current_url = request.url
                for _ in range(5 if request.method in ('GET', 'HEAD') else 0):
                    destination = same_origin_redirect(response, current_url, self.origin)
                    if destination is None:
                        break
                    response = route.fetch(url=destination, headers={**request.headers, 'OAI-Sites-Authorization': 'Bearer ' + self.bypass},
                                           max_redirects=0, max_retries=0, timeout=20000)
                    current_url = destination
                route.fulfill(response=response)
            except Exception as error:
                # Raw Playwright errors include overridden credential headers.
                self.transport_events.append(transport_failure(error, request.url))
                try:
                    route.fulfill(status=503, content_type='application/json', body='{"error":"qa_transport_unavailable"}')
                except Exception:
                    pass

        context.route('**/*', forward)
        return context


class RemoteResponse(BrowserResponse):
    def wait_for_deployment(self):
        # Keep Playwright's sync event loop pumping while the owning agent uses
        # the native Sites deploy tool. Blocking terminal input on this thread
        # would stall request routes and manufacture browser timeouts.
        result = {}
        ready = threading.Event()
        def receive():
            try:
                result['value'] = json.loads(getpass.getpass('Deployment checkpoint resume JSON (input hidden): '))
            except Exception as error:
                result['error'] = error
            finally:
                ready.set()
        threading.Thread(target=receive, daemon=True).start()
        while not ready.is_set():
            self.pages[0].wait_for_timeout(100)
        if 'error' in result:
            raise result['error']
        return result['value']

    def restart(self, label, command=None):
        before = self.views()
        self.pages[0].reload(wait_until='domcontentloaded')
        self.sync(before[0]['version'])
        assert self.views() == before, 'Refresh changed per-seat response views'
        if command:
            print(json.dumps({'checkpoint': 'republish-exact-saved-private-version', 'label': label,
                              'roomId': self.fixture.room_id, 'version': before[0]['version']}), flush=True)
            result = self.wait_for_deployment()
            assert result.get('status') == 'succeeded' and result.get('version_id') == self.saved_version_id, 'No successful same-version deployment supplied'
            self.pages[0].reload(wait_until='domcontentloaded')
            self.sync(before[0]['version'])
            restored = self.views()
            assert restored == before, 'Same-version deployment changed per-seat response views'
            seat, body, ack = command
            status, replay = self.fixture.api.request('POST', f'/api/rooms/{self.fixture.room_id}/commands', body,
                                                    token=self.fixture.sessions[seat]['token'])
            assert status == 200 and replay == ack
            assert self.fixture.views() == restored
            self.fixture.probes.append({'kind': 'after-republish-original-command-receipt', 'passed': True,
                                        'versionUnchanged': True, 'sameIntent': True})
            self.checks.append({'name': label, 'refreshExactViews': True, 'republishExactViews': True,
                                'deploymentId': result.get('id'), 'savedVersionId': result['version_id'],
                                'independentContexts': len(self.pages), 'originalReceiptIdentical': True})
        else:
            self.checks.append({'name': label, 'refreshExactViews': True, 'independentContexts': len(self.pages)})


def main(args, bypass):
    output = Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    metadata = {'testType': 'actual-private-site-legal-api-fixture-response-ui', 'sourceCommit': args.source_commit,
                'siteUrl': args.base_url, 'mode': args.mode, 'seedOverride': False,
                'intermediateStateInjection': False, 'countsAsCompleteUiGame': False, 'passed': False}
    fixture = run = private = None
    try:
        api = SitesApi(args.base_url, bypass)
        fixture = Fixture(api, output, args.mode, args.max_steps, 'rust-v0.2.1')
        match = fixture.prepare()
        metadata['roomId'] = fixture.room_id
        with sync_playwright() as playwright:
            proxy = os.environ.get('HTTPS_PROXY') or os.environ.get('https_proxy')
            browser = playwright.chromium.launch(executable_path=args.chromium, headless=True,
                                                 proxy={'server': proxy} if proxy else None,
                                                 args=['--no-sandbox', '--disable-dev-shm-usage'])
            try:
                private = PrivateBrowser(browser, args.base_url, bypass)
                run = RemoteResponse(private, fixture, SimpleNamespace(base_url=args.base_url), output)
                run.saved_version_id = args.saved_version_id
                run.open(match[0], match[1]); run.run_chain(*match)
                assert not run.errors, 'Unexpected browser errors'
                metadata.update(passed=True, checks=run.checks, screenshots=run.shots,
                                final=evidence_view(run.views()[0]), independentContexts=len(run.pages))
            finally:
                if run:
                    run.close()
                browser.close()
    except Exception as error:
        metadata['failure'] = f'{type(error).__name__}: {error}'
        raise
    finally:
        if private:
            write_json(output / 'qa-transport-events.json', private.transport_events)
        metadata.update(fixtureAcceptedCommands=len(fixture.records) if fixture else 0,
                        uiAcceptedCommands=len(run.commands) if run else 0,
                        apiProbeRequests=len(fixture.probes) if fixture else 0,
                        browserErrors=run.errors if run else [], checks=run.checks if run else [],
                        screenshots=run.shots if run else [])
        write_json(output / 'response-summary.json', metadata)
        print(json.dumps({key: metadata[key] for key in ('passed', 'fixtureAcceptedCommands', 'uiAcceptedCommands')}, ensure_ascii=False), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--saved-version-id', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--mode', choices=['duel', 'teams'], default='teams')
    parser.add_argument('--max-steps', type=int, default=2200)
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    args = parser.parse_args()
    main(args, getpass.getpass('Private Site QA bearer (input hidden): '))
