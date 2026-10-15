#!/usr/bin/env python3
"""Bounded local PC recovery checks with real UI commands and an owned SQLite service.

This does not play a complete match, publish a site, or use production credentials.
Only two pause acknowledgements are replaced with explicit synthetic HTTP 503s.
The service still receives their original command identity and persists normally.
"""
from __future__ import annotations

import argparse
import asyncio
import json
from pathlib import Path

from playwright.async_api import async_playwright, expect

from browser_playtest import OBSERVE_FETCH
from common import IsolatedService, write_json


async def main(args):
    output = Path(args.output).resolve()
    if output.exists():
        raise ValueError('Choose a new output directory; prior evidence is never overwritten')
    service = IsolatedService(args.binary, args.cwd, output, args.port, args.static_dir)
    result = {'passed': False, 'scope': 'local Chromium, current native server, bounded two-seat UI recovery',
              'fullMatch': False, 'remoteDeployment': False, 'sourceCommit': args.source_commit,
              'checks': [], 'synthetic503s': 0, 'pageErrors': []}
    service.start()
    try:
        async with async_playwright() as playwright:
            browser = await playwright.chromium.launch(executable_path=args.chromium, headless=True,
                                                       args=['--no-sandbox', '--disable-dev-shm-usage'])
            commands = []

            async def new_context():
                context = await browser.new_context(viewport={'width': 1366, 'height': 900})
                await context.add_init_script(OBSERVE_FETCH)
                context.on('page', lambda page: page.on('pageerror', lambda error: result['pageErrors'].append(str(error))))
                return context

            async def view(page):
                return await page.evaluate('window.__cloudView')

            async def submit(page, name):
                async with page.expect_response(lambda response: response.request.method == 'POST' and '/commands' in response.url) as received:
                    await page.get_by_role('button', name=name, exact=True).click()
                response = await received.value
                assert response.ok, f'UI command returned {response.status}'
                current = await response.json()
                await page.wait_for_function('(version) => window.__cloudView?.version >= version', arg=current['version'])
                return current

            host_context = await new_context()
            guest_context = await new_context()
            host = await host_context.new_page()
            guest = await guest_context.new_page()
            await host.goto(service.base_url)
            await host.get_by_label('你的称呼', exact=True).fill('Recovery host')
            await host.get_by_role('button', name='创建牌桌 →', exact=True).click()
            await host.wait_for_function('window.__cloudView?.status === "lobby"')
            invitation = (await view(host))['inviteCode']
            await guest.goto(service.base_url)
            await guest.get_by_role('button', name='开始新的独立玩家会话', exact=True).click()
            await guest.get_by_role('button', name='邀请码加入', exact=True).click()
            await guest.get_by_label('你的称呼', exact=True).fill('Recovery guest')
            await guest.get_by_label('邀请码', exact=True).fill(invitation)
            await guest.get_by_role('button', name='加入牌桌 →', exact=True).click()
            await guest.wait_for_function('window.__cloudView?.status === "lobby"')

            # window.open copies this independent page's sessionStorage, as browser duplicate-tab does.
            async with guest.expect_popup() as popup:
                await guest.evaluate('window.open(location.href, "_blank")')
            copied_independent = await popup.value
            await expect(copied_independent.get_by_role('status', name='同席标签提示')).to_be_visible()
            await expect(guest.get_by_role('status', name='同席标签提示')).to_be_visible()
            await expect(host.get_by_role('status', name='同席标签提示')).to_have_count(0)
            await copied_independent.close()
            await expect(guest.get_by_role('status', name='同席标签提示')).to_have_count(0)
            result['checks'].append('copied independent same seat detected; other seat isolated; close clears warning')

            joined = await view(guest)
            await host.wait_for_function('(version) => window.__cloudView?.version >= version', arg=joined['version'])
            ready = await submit(host, '准备')
            await guest.wait_for_function('(version) => window.__cloudView?.version >= version', arg=ready['version'])
            await submit(guest, '准备')
            await host.wait_for_function('window.__cloudView?.legalActions.some(action => action.kind === "start")')
            start = next(action for action in (await view(host))['legalActions'] if action['kind'] == 'start')
            await submit(host, start['label'])
            await host.get_by_role('dialog', name='待完成的选择').wait_for()
            current = await view(host)
            choice = current['pendingChoice']
            assert choice and choice['kind'] == 'mulligan' and choice['options'], 'Expected the ordinary initial private choice'
            option_id = choice['options'][0]['id']
            option = host.locator('[data-choice-option=' + json.dumps(option_id) + ']')
            await option.click()
            await expect(option).to_have_attribute('aria-pressed', 'true')
            await host.reload()
            await expect(host.locator('[data-choice-option=' + json.dumps(option_id) + ']')).to_have_attribute('aria-pressed', 'true')
            result['checks'].append('private choice and selected draft survive a real browser reload')
            await host.screenshot(path=str(output / 'choice-restored.png'), full_page=True)

            peer = await host_context.new_page()  # No opener: independent tab-local draft storage.
            await peer.goto(service.base_url)
            await expect(peer.get_by_role('status', name='同席标签提示')).to_be_visible()
            await expect(host.get_by_role('status', name='同席标签提示')).to_be_visible()
            await expect(peer.locator('[data-choice-option=' + json.dumps(option_id) + ']')).to_have_attribute('aria-pressed', 'false')
            await expect(host.locator('[data-choice-option=' + json.dumps(option_id) + ']')).to_have_attribute('aria-pressed', 'true')
            await host.screenshot(path=str(output / 'ordinary-seat-conflict.png'), full_page=True)
            await peer.close()
            await expect(host.get_by_role('status', name='同席标签提示')).to_have_count(0)
            result['checks'].append('ordinary same-seat conflict is symmetric; drafts remain tab-local; close clears warning')

            # Exercise the real storage-event fallback in separate browser documents.
            await host_context.add_init_script('window.BroadcastChannel = undefined')
            await host.reload()
            peer = await host_context.new_page()
            await peer.goto(service.base_url)
            await expect(peer.get_by_role('status', name='同席标签提示')).to_be_visible()
            await expect(host.get_by_role('status', name='同席标签提示')).to_be_visible()
            await peer.close()
            await expect(host.get_by_role('status', name='同席标签提示')).to_have_count(0)
            assert await host.evaluate('localStorage.getItem("hegemony.seatPresence.v1")') is None
            result['checks'].append('storage-event fallback detects separate tabs without leaving a presence record')

            async def lose_pause_ack(route):
                command = route.request.post_data_json
                commands.append(command)
                response = await route.fetch(max_retries=0)
                assert response.ok, f'Original local command returned {response.status}'
                if command['action']['kind'] == 'pauseRoom' and result['synthetic503s'] < 2:
                    result['synthetic503s'] += 1
                    await route.fulfill(status=503, content_type='application/json', body='{"error":"synthetic_lost_pause_ack"}')
                else:
                    await route.fulfill(response=response)

            await host.route('**/commands', lose_pause_ack)
            await host.get_by_role('button', name='暂停并保存此桌', exact=True).click()
            await host.wait_for_function('window.__cloudView?.pause')
            await expect(host.get_by_text('上一行动等待确认；牌桌将在恢复连接后继续。', exact=True)).to_be_visible()
            await expect(host.get_by_text('上一行动等待确认；牌桌将在恢复连接后继续。', exact=True)).to_have_count(0, timeout=20000)
            pause_commands = [command for command in commands if command['action']['kind'] == 'pauseRoom']
            assert len(pause_commands) == 3 and all(command == pause_commands[0] for command in pause_commands)
            paused = await view(host)
            assert paused['version'] == pause_commands[0]['expectedVersion'] + 1
            result['checks'].append('lost pause ACK automatically recovers the identical tuple; server version advances once')
            await host.screenshot(path=str(output / 'paused-confirmed.png'), full_page=True)

            service.restart()
            await host.reload()
            await host.wait_for_function('window.__cloudView?.pause')
            restored = await view(host)
            assert restored['version'] == paused['version'] and restored['pendingChoice'] == paused['pendingChoice']
            assert restored['hand'] == paused['hand']
            await submit(host, '恢复对局')
            await expect(host.locator('[data-choice-option=' + json.dumps(option_id) + ']')).to_have_attribute('aria-pressed', 'true')
            result['checks'].append('SQLite service reopen preserves paused private choice; resume restores local selection')

            # Two immediate DOM clicks must still pass through the same synchronous command lock.
            await host.get_by_role('button', name='确认选择', exact=True).evaluate('(button) => { button.click(); button.click(); }')
            await host.wait_for_function('(id) => window.__cloudView && window.__cloudView.pendingChoice?.id !== id', arg=choice['id'])
            choices = [command for command in commands if command['action'].get('action', command['action']).get('kind') == 'choose']
            assert len(choices) == 1 and choices[0]['action'].get('action', choices[0]['action'])['selected'] == [option_id]
            assert await host.evaluate('Object.keys(sessionStorage).filter(key => key.startsWith("hegemony.choiceDraft.v1.")).length') == 0
            result['checks'].append('rapid duplicate choice clicks emit one command; authoritative completion clears draft')
            result['versions'] = restored['versions']
            result['commandCounts'] = {'pausePosts': len(pause_commands), 'choicePosts': len(choices)}
            assert not result['pageErrors'], 'Unexpected browser page errors'
            result['passed'] = True
            await browser.close()
    except Exception as error:
        result['failure'] = f'{type(error).__name__}: {error}'
        raise
    finally:
        service.stop()
        write_json(output / 'summary.json', result)
        print(json.dumps({key: result.get(key) for key in ('passed', 'checks', 'synthetic503s', 'failure')}, ensure_ascii=False))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', default='target/debug/hegemony-server')
    parser.add_argument('--cwd', default='.')
    parser.add_argument('--static-dir', default='web/dist')
    parser.add_argument('--output', required=True)
    parser.add_argument('--port', type=int, default=8194)
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    parser.add_argument('--source-commit', required=True, help='Exact base commit plus a dirty/WIP label when source is uncommitted')
    asyncio.run(main(parser.parse_args()))
