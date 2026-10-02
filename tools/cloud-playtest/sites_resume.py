#!/usr/bin/env python3
"""Continue an owned stopped Site game using its four original browser seats.

Credentials arrive on hidden stdin and remain in memory/application storage.
No new room, join, start, seed override, intermediate-state injection or Site
bypass credential is used. Earlier failures remain in their original evidence.
"""
import argparse
import asyncio
import getpass
import json
import os
from pathlib import Path

from playwright.async_api import async_playwright
from browser_playtest import UiRun
from sites_public import PublicBrowser
from common import write_json


async def main(args, sessions):
    output = Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    metadata = {'testType': 'public-site-original-four-seat-continuation', 'passed': False,
                'roomId': sessions[0]['roomId'], 'newRoomsCreated': 0, 'joinCommands': 0,
                'apiGameplayCommands': 0, 'siteBypassUsed': False, 'originalStrictFailurePreserved': True,
                'expectedEngine': 'rust-v0.2.1', 'sourceCommit': args.source_commit}
    public = None
    try:
        async with async_playwright() as playwright:
            proxy = os.environ.get('HTTPS_PROXY') or os.environ.get('https_proxy')
            browser = await playwright.chromium.launch(executable_path=args.chromium, headless=True,
                proxy={'server': proxy} if proxy else None, args=['--no-sandbox', '--disable-dev-shm-usage'])
            try:
                public = PublicBrowser(browser, args.base_url)
                run = UiRun(public, args.base_url, output, args.max_steps, 'develop', True)
                result = await run.resume(sessions)
                metadata['passed'] = result['passed'] and not result['browserErrors'] and not public.transport_events
                metadata['finished'] = result['final']['status'] == 'finished'
                metadata['acceptedCommands'] = result['acceptedCommands']
                metadata['finalVersion'] = result['final']['version']
            finally:
                await browser.close()
    except Exception as error:
        metadata['failureType'] = type(error).__name__
    finally:
        if public:
            write_json(output / 'qa-transport-events.json', public.transport_events)
        write_json(output / 'continuation-evidence.json', metadata)
        print(json.dumps(metadata, ensure_ascii=False), flush=True)
    return metadata['passed']


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--max-steps', type=int, default=4999)
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    args = parser.parse_args()
    sessions = json.loads(getpass.getpass('Own four original seat sessions (input hidden): '))
    raise SystemExit(0 if asyncio.run(main(args, sessions)) else 1)
