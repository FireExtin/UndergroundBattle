#!/usr/bin/env python3
"""Full UI acceptance on the private deployed Site; no API gameplay moves.

The owner bypass is read from hidden stdin and attached only to the exact Site
origin. The route forwards browser requests once, without following redirects.
No bearer tokens or full hidden server state are written to evidence.
"""
import argparse
import asyncio
import getpass
import json
import os
import threading
from pathlib import Path
from urllib.parse import urlsplit

from playwright.async_api import async_playwright
from browser_playtest import UiRun
from common import write_json
from sites_transport import same_origin_redirect, transport_failure


class PrivateContext:
    def __init__(self, context):
        self.context, self.offline = context, False

    def __getattr__(self, name):
        return getattr(self.context, name)

    async def set_offline(self, offline):
        self.offline = offline
        await self.context.set_offline(offline)


class PrivateBrowser:
    def __init__(self, browser, base, bypass):
        self.browser, self.origin, self.bypass = browser, urlsplit(base).netloc, bypass
        self.transport_events = []

    def accept_credential_updates(self):
        # A native token rotation can be handled without saving a credential or
        # abandoning the current game. This input is only for the owning agent.
        def receive():
            while True:
                try:
                    replacement = getpass.getpass('Private QA credential updates (input hidden): ')
                    if replacement and all(c.isalnum() or c in '-_' for c in replacement):
                        self.bypass = replacement
                        print('\nPrivate QA credential updated in memory', flush=True)
                except (EOFError, KeyboardInterrupt):
                    return
        threading.Thread(target=receive, daemon=True).start()

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
            original_credential = self.bypass
            headers = {**request.headers, 'OAI-Sites-Authorization': 'Bearer ' + original_credential}
            try:
                response = await route.fetch(headers=headers, max_redirects=0, max_retries=0, timeout=20000)
                if response.status in (401, 403) or (
                    response.status in (302, 303, 307, 308) and same_origin_redirect(response, request.url, self.origin) is None
                ):
                    for _ in range(5):
                        if self.bypass != original_credential:
                            headers = {**request.headers, 'OAI-Sites-Authorization': 'Bearer ' + self.bypass}
                            response = await route.fetch(headers=headers,
                                                         max_redirects=0, max_retries=0, timeout=20000)
                            break
                        await asyncio.sleep(1)
                current_url = request.url
                for _ in range(5 if request.method in ('GET', 'HEAD') else 0):
                    destination = same_origin_redirect(response, current_url, self.origin)
                    if destination is None:
                        break
                    response = await route.fetch(url=destination, headers=headers, max_redirects=0, max_retries=0, timeout=20000)
                    current_url = destination
                await route.fulfill(response=response)
            except Exception as error:
                # Playwright exceptions can include every overridden request header.
                # Never print them or let them escape to an event-listener traceback.
                self.transport_events.append(transport_failure(error, request.url))
                try:
                    await route.fulfill(status=503, content_type='application/json',
                                        body='{"error":"qa_transport_unavailable"}')
                except Exception:
                    pass  # A closing/offline browser can cancel its route.

        await context.route('**/*', forward)
        return private


async def main(args, bypass):
    output = Path(args.output).resolve()
    metadata = {'testType': 'deployed-private-site-complete-ui', 'siteUrl': args.base_url,
                'apiGameplayCommands': 0, 'ownerBypassForQa': True,
                'independentHumanAccountAccessProven': False, 'modes': args.modes,
                'githubSourceCommit': args.source_commit, 'siteSourceCommit': args.site_commit}
    results = []
    private = None
    try:
        async with async_playwright() as playwright:
            proxy = os.environ.get('HTTPS_PROXY') or os.environ.get('https_proxy')
            browser = await playwright.chromium.launch(executable_path=args.chromium, headless=True,
                                                       proxy={'server': proxy} if proxy else None,
                                                       args=['--no-sandbox', '--disable-dev-shm-usage'])
            try:
                private = PrivateBrowser(browser, args.base_url, bypass)
                private.accept_credential_updates()
                for mode in args.modes.split(','):
                    run = UiRun(private, args.base_url, output, args.max_steps, 'develop', True)
                    results.append(await run.play(mode))
            finally:
                await browser.close()
        metadata['passed'] = all(r['passed'] and not r['browserErrors'] for r in results)
        write_json(output / 'browser-summary.json', results)
        assert metadata['passed'], 'Browser acceptance did not pass'
    finally:
        if private:
            write_json(output / 'qa-transport-events.json', private.transport_events)
        write_json(output / 'deployed-site-evidence.json', metadata)
        print(json.dumps({'passed': metadata.get('passed', False), 'completedModes': [r['scenario'] for r in results]}), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--site-commit', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--modes', default='duel,teams')
    parser.add_argument('--max-steps', type=int, default=7000)
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    arguments = parser.parse_args()
    secret = getpass.getpass('Private Site QA bearer (input hidden): ')
    asyncio.run(main(arguments, secret))
