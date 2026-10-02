#!/usr/bin/env python3
"""Small real UI check for the optional auto-pass preference and its pause gates."""
import argparse
import asyncio
import re
from pathlib import Path

from playwright.async_api import async_playwright

from browser_playtest import UiRun
from common import public_view, wait_health, write_json


async def main(args):
    wait_health(args.base_url)
    output = Path(args.output)
    async with async_playwright() as playwright:
        browser = await playwright.chromium.launch(executable_path=args.chromium, headless=True, args=['--no-sandbox', '--disable-dev-shm-usage'])
        run = UiRun(browser, args.base_url, output, 200)
        run.mode = 'autopass'
        try:
            host = await run.new_seat(0)
            await host.get_by_label('你的称呼').fill('Auto A')
            await host.get_by_role('button', name='创建牌桌 →', exact=True).click()
            await host.wait_for_function('window.__cloudView && window.__cloudView.status === "lobby"')
            invitation = (await run.view(0))['inviteCode']
            guest = await run.new_seat(1)
            await guest.locator('.hg-deck').nth(1).click()
            await guest.get_by_role('button', name='邀请码加入', exact=True).click()
            await guest.get_by_label('你的称呼').fill('Auto B')
            await guest.get_by_label('邀请码', exact=True).fill(invitation)
            await guest.get_by_role('button', name='加入牌桌 →', exact=True).click()
            await guest.wait_for_function('window.__cloudView && window.__cloudView.status === "lobby"')
            await run.sync((await run.view(1))['version'])
            for seat in (0, 1):
                await run.legal_button(seat, next(a for a in (await run.view(seat))['legalActions'] if a['kind'] == 'ready'))
            await run.legal_button(0, next(a for a in (await run.view(0))['legalActions'] if a['kind'] == 'start'))
            switches = [page.get_by_role('switch', name='无可用行动时自动让过') for page in run.pages]
            assert all([not await switch.is_checked() for switch in switches])
            version = (await run.view(0))['version']
            for switch in switches: await switch.check()
            await asyncio.sleep(1.25)
            assert all([(await run.view(seat))['version'] == version for seat in (0, 1)]), 'Auto-pass crossed a pending-choice boundary'
            await run.screenshot('enabled-private-choice')
            # All required decisions still come through the ordinary choice form.
            while True:
                choices = [(seat, view) for seat, view in enumerate(await run.views()) if view.get('pendingChoice')]
                if not choices: break
                await run.choose(*choices[0])
            version_after_choices = (await run.view(0))['version']
            await host.wait_for_function('window.__cloudView && window.__cloudView.phase === "action"', timeout=25000)
            views = await run.views()
            current = max(views, key=lambda v: v['version'])
            assert current['version'] > version_after_choices, 'No sole-pass window advanced after opting in'
            assert any(len(v['legalActions']) > 1 for v in views), 'Did not reach a real decision window'
            await run.sync(current['version'])
            await asyncio.sleep(1.5)
            assert all([(await run.view(seat))['version'] == current['version'] for seat in (0, 1)]), 'Auto-pass consumed a window with other legal actions'
            await run.screenshot('paused-for-actions')
            for switch in switches: await switch.uncheck()
            assert all([not await switch.is_checked() for switch in switches])
            assert not run.errors
            write_json(output / 'autopass-summary.json', {'testType': 'real-browser-ui-autopass', 'passed': True,
                       'defaultOff': True, 'pendingChoicePaused': True, 'solePassAdvanced': True,
                       'otherActionsPaused': True, 'toggleOff': True, 'browserErrors': run.errors,
                       'final': public_view(current), 'screenshots': run.screenshots})
            print('Auto-pass real UI gates passed', flush=True)
        except Exception as error:
            write_json(output / 'autopass-summary.json', {'passed': False, 'failure': f'{type(error).__name__}: {error}'})
            raise
        finally:
            for context in run.contexts: await context.close()
            await browser.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', default='http://127.0.0.1:8091')
    parser.add_argument('--output', default='/tmp/hegemony-cloud-autopass')
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    asyncio.run(main(parser.parse_args()))
