#!/usr/bin/env python3
"""Newcomer UI probe using visible DOM only, never API views/legalActions.

This is a short interaction/accessibility/visibility check, not another full game.
It works against a same-origin local or published HTTP service.
"""
import argparse
import asyncio
import json
import re
from pathlib import Path

from playwright.async_api import async_playwright

from common import write_json


class Newcomer:
    def __init__(self, browser, base_url, output, steps):
        self.browser, self.base = browser, base_url.rstrip('/')
        self.output = Path(output); self.output.mkdir(parents=True, exist_ok=True)
        self.max_steps = steps
        self.contexts, self.pages, self.errors, self.actions, self.shots = [], [], [], [], []
        self.developed = set()
        self.hidden_names = {}
        self.hidden_observations = {'owner': set(), 'teammate': set(), 'opponent': set()}
        self.checks = {'fourIndependentContexts': False, 'phone390': False, 'desktop1366': False,
                       'actingAndWaitingVisible': False, 'singleNextStepVisible': False,
                       'choiceReachable': False, 'ownerLabelsVisible': False,
                       'teammateAndOpponentHiddenIdentity': False, 'onePointDamageTapped': False,
                       'readableCardDialog': False}

    async def screenshot(self, name, seat=0):
        file = f'newcomer-{name}-seat{seat + 1}.png'
        await self.pages[seat].screenshot(path=str(self.output / file), full_page=False)
        self.shots.append({'file': file, 'seat': seat, 'privateLocalArtifact': True})

    async def command(self, seat, locator, kind):
        page = self.pages[seat]
        async with page.expect_response(lambda response: response.request.method == 'POST' and '/commands' in response.url, timeout=20000) as pending:
            await locator.click()
        response = await pending.value
        # Deliberately do not read JSON responses or request bodies for this probe.
        self.actions.append({'seat': seat, 'visibleOperation': kind, 'httpStatus': response.status})
        write_json(self.output / 'newcomer-actions.json', self.actions)
        assert response.ok, f'DOM-driven {kind} rejected HTTP {response.status}'
        await asyncio.sleep(.16)

    async def setup(self):
        for seat in range(4):
            mobile = seat % 2 == 1
            context = await self.browser.new_context(viewport={'width': 390 if mobile else 1366, 'height': 844 if mobile else 900},
                                                      is_mobile=mobile, has_touch=mobile, locale='zh-CN')
            page = await context.new_page()
            page.on('pageerror', lambda error, seat=seat: self.errors.append({'seat': seat, 'kind': 'pageerror', 'message': str(error)}))
            page.on('console', lambda message, seat=seat: self.errors.append({'seat': seat, 'kind': 'console', 'message': message.text}) if message.type == 'error' else None)
            self.contexts.append(context); self.pages.append(page)
            await page.goto(self.base, wait_until='domcontentloaded')
            await page.locator('.hg-deck').nth(3).wait_for(state='visible', timeout=30000)
            await page.locator('.hg-deck').nth(seat).click()
            await page.get_by_label('你的称呼').fill('新手' + chr(65 + seat))
            if seat == 0:
                await page.get_by_role('button', name=re.compile('四人协作')).click()
                await page.get_by_role('button', name='创建牌桌 →', exact=True).click()
                await page.get_by_role('heading', name='等待秘社集结').wait_for()
                self.invite = await page.locator('.hg-invite strong').inner_text()
            else:
                await page.get_by_role('button', name='邀请码加入', exact=True).click()
                await page.get_by_label('邀请码', exact=True).fill(self.invite)
                await page.get_by_role('button', name='加入牌桌 →', exact=True).click()
                await page.get_by_role('heading', name='等待秘社集结').wait_for()
        for seat, page in enumerate(self.pages):
            await self.command(seat, page.get_by_role('button', name='准备', exact=True), '准备')
        await self.command(0, self.pages[0].get_by_role('button', name='开始游戏', exact=True), '开始游戏')
        for page in self.pages: await page.locator('.hg-table').wait_for()
        self.checks.update(fourIndependentContexts=True, phone390=True, desktop1366=True)

    async def prompts(self):
        cues = []
        for seat, page in enumerate(self.pages):
            # Only text actually rendered on the page guides the bot's next move.
            status = await page.locator('.hg-phase-line strong').inner_text()
            hints = page.locator('.hg-hand-dock .hg-dock-action > div > strong')
            texts = [await item.inner_text() for item in await hints.all() if await item.is_visible()]
            assert len(texts) == 1, f'Seat {seat + 1} has {len(texts)} visible next-step prompts'
            assert '下一步：' in texts[0] or '等待' in texts[0] or '已结束' in texts[0]
            own_choice = await page.locator('.hg-choice').count() > 0
            assert '你' in status or '等待' in status or (own_choice and '完成' in status), f'Acting/waiting status unclear: {status}'
            box = await hints.first.bounding_box()
            viewport = page.viewport_size
            assert box and viewport and box['y'] >= 0 and box['y'] + box['height'] <= viewport['height'], 'Primary next-step cue is outside the viewport'
            assert await page.locator('.hg-team-score').first.get_by_text('我方秘社', exact=True).is_visible()
            assert not await page.evaluate('document.documentElement.scrollWidth > innerWidth + 1'), 'Horizontal document overflow'
            cues.append({'seat': seat, 'status': status, 'nextStep': texts[0]})
        self.checks.update(actingAndWaitingVisible=True, singleNextStepVisible=True)
        return cues

    async def read_dialog(self, seat, opener):
        await opener.click()
        page = self.pages[seat]
        dialog = page.get_by_role('dialog', name=re.compile('放大阅读'))
        await dialog.wait_for(state='visible')
        box = await dialog.bounding_box()
        assert box and box['width'] <= (390 if seat % 2 else 1366)
        # Record geometry/legibility, not private card prose or identity.
        font = await dialog.locator('.hg-card-text').evaluate('(element) => getComputedStyle(element).fontSize')
        self.checks['readableCardDialog'] = True
        await self.screenshot('readable-card', seat)
        close = dialog.get_by_role('button', name=re.compile('关闭'))
        await close.first.click()
        return font

    async def choice(self, seat):
        page = self.pages[seat]
        panel = page.locator('.hg-choice')
        go_to_choice = page.get_by_role('button', name='前往完成选择 ↑', exact=True)
        if await go_to_choice.count():
            await go_to_choice.click()
            await asyncio.sleep(.25)
        else:
            await panel.scroll_into_view_if_needed()
        self.checks['choiceReachable'] = True
        damage = panel.locator('.hg-damage-meter')
        if await damage.count():
            total = int(await damage.locator('b').nth(0).inner_text())
            rows = panel.locator('.hg-damage-row')
            count = await rows.count()
            guards = []
            for index in range(count):
                # The printed ability can be read through the regular card modal.
                row = rows.nth(index)
                await row.get_by_role('button', name='放大阅读', exact=True).click()
                dialog = page.get_by_role('dialog', name=re.compile('放大阅读'))
                prose = await dialog.locator('.hg-card-text').inner_text()
                if '护卫' in prose: guards.append(index)
                await dialog.get_by_role('button', name=re.compile('关闭')).first.click()
            target_order = guards[:total]
            if len(target_order) < total: target_order += [0] * (total - len(target_order))
            for index in target_order:
                button = rows.nth(index).get_by_role('button', name=re.compile('分配1点伤害$'))
                await button.click()
            assigned = int(await damage.locator('b').nth(1).inner_text())
            assert assigned == total, 'Visible damage meter did not match tap allocations'
            self.checks['onePointDamageTapped'] = True
            if await panel.get_by_role('button', name='放大阅读', exact=True).count():
                await self.read_dialog(seat, panel.get_by_role('button', name='放大阅读', exact=True).first)
            await self.screenshot('actual-damage-choice', seat)
        elif await panel.get_by_role('button', name='保留全部手牌', exact=True).count():
            await self.command(seat, panel.get_by_role('button', name='保留全部手牌', exact=True), '保留全部手牌')
            return
        elif await panel.locator('.hg-order-list').count():
            bottom = panel.get_by_role('button', name='置于底', exact=True)
            if await bottom.count() > 1: await bottom.last.click()
        elif not await panel.get_by_role('button', name='确认选择', exact=True).is_enabled():
            footer = await panel.locator('.hg-choice-footer').inner_text()
            match = re.search(r'请选择\s*(\d+)(?:[–-](\d+))?\s*项', footer)
            minimum = int(match.group(1)) if match else 1
            options = panel.locator('.hg-choice-option')
            for index in range(minimum or min(1, await options.count())):
                option = options.nth(index)
                if await option.get_attribute('aria-pressed') != 'true': await option.click()
        await self.command(seat, panel.get_by_role('button', name='确认选择', exact=True), '确认可见选择')

    async def own_asset_count(self, page):
        strip = page.locator('.hg-player-you')
        item = strip.locator('dl > div').filter(has=page.get_by_text('资产', exact=True))
        return int(await item.locator('dd').inner_text())

    async def act(self, seat):
        page = self.pages[seat]
        hand = page.locator('.hg-hand .hg-card')
        cards = await hand.all()
        # Read visible cost/icons, just as a player scans the hand; no engine view.
        scored = []
        for index, card in enumerate(cards):
            icons = card.locator('.hg-icons').first
            text = await icons.get_attribute('aria-label') if await icons.count() else ''
            combat = re.search(r'战斗(\d+)', text or '')
            cost = card.locator('.hg-cost')
            price = int(await cost.inner_text()) if await cost.count() else 0
            scored.append((int(combat.group(1)) if combat else 0, -price, index))
        asset_count = await self.own_asset_count(page)
        if asset_count < 4 and cards:
            asset_order = sorted(scored)
            for _, _, index in asset_order:
                await cards[index].click()
                button = page.locator('.hg-actions button').filter(has_text='建立资产')
                if await button.count() and await button.first.is_enabled():
                    await self.command(seat, button.first, '建立资产')
                    return True
        # Face-up combat develops before a low-icon concealed piece. Every move
        # is selected from the visible contextual buttons, not a legality array.
        for _, _, index in sorted(scored, reverse=True):
            await cards[index].click()
            deploy = page.locator('.hg-actions button').filter(has_text=re.compile(r'^派遣 '))
            if await deploy.count():
                await self.command(seat, deploy.first, '派遣')
                return True
        if seat not in self.developed:
            for _, _, index in sorted(scored):
                await cards[index].click()
                conceal = page.locator('.hg-actions button').filter(has_text='秘密派遣')
                if await conceal.count():
                    name = await cards[index].locator('.hg-card-name').inner_text()
                    owner = await cards[index].get_attribute('data-card-owner')
                    self.hidden_names[owner] = name  # memory only, never in reports
                    await self.command(seat, conceal.first, '秘密派遣')
                    self.developed.add(seat)
                    return True
        passed = page.get_by_role('button', name='让过', exact=True)
        if await passed.count() and await passed.first.is_enabled():
            await self.command(seat, passed.first, '让过')
            return True
        return False

    async def hidden_check(self):
        if not self.hidden_names: return False
        for seat, page in enumerate(self.pages):
            for owner, private_name in self.hidden_names.items():
                pieces = page.locator('.hg-region-characters .hg-card[data-card-owner=' + json.dumps(owner) + ']')
                hidden = [piece for piece in await pieces.all() if '不受角色伤害' in await piece.inner_text()]
                if not hidden: continue
                for piece in hidden:
                    assert await piece.locator('.hg-card-owner').is_visible()
                    assert ('新手' in await piece.locator('.hg-card-owner').inner_text())
                    instance = await piece.get_attribute('data-card-instance')
                    owner_seat = int(owner[1:])
                    relationship = 'owner' if owner_seat == seat else 'teammate' if owner_seat // 2 == seat // 2 else 'opponent'
                    self.hidden_observations[relationship].add((seat, instance))
                    if owner != f'p{seat}':
                        assert await piece.locator('.hg-card-name').inner_text() == '暗藏者'
                        assert private_name not in await piece.inner_text(), 'Concealed identity visible to another seat'
        self.checks.update(ownerLabelsVisible=bool(self.hidden_observations['owner']),
                           teammateAndOpponentHiddenIdentity=bool(self.hidden_observations['teammate'] and self.hidden_observations['opponent']))
        return self.checks['ownerLabelsVisible'] and self.checks['teammateAndOpponentHiddenIdentity']

    async def run(self):
        try:
            await self.setup()
            for step in range(self.max_steps):
                cues = await self.prompts()
                pending = [seat for seat, page in enumerate(self.pages) if await page.locator('.hg-choice').count()]
                if pending:
                    await self.choice(pending[0])
                else:
                    acted = False
                    for cue in cues:
                        if '轮到你' in cue['status']:
                            acted = await self.act(cue['seat'])
                            if acted: break
                    assert acted, 'Visible prompts did not lead to an actionable control'
                await self.hidden_check()
                if step % 25 == 0:
                    print(json.dumps({'domOnlyStep': step, 'visibleOperations': len(self.actions), 'checks': self.checks}), flush=True)
                if all(self.checks.values()):
                    for seat in range(4):
                        await self.screenshot('perspective', seat)
                    result = {'testType': 'visible-dom-only-newcomer', 'passed': True, 'steps': step + 1,
                              'readsApiViews': False, 'readsLegalActions': False, 'checks': self.checks,
                              'hiddenObservations': {kind: len(items) for kind, items in self.hidden_observations.items()},
                              'browserErrors': self.errors, 'screenshots': self.shots,
                              'acceptedCommands': sum(a['httpStatus'] == 200 for a in self.actions)}
                    assert not self.errors
                    write_json(self.output / 'newcomer-summary.json', result)
                    return
            raise RuntimeError('Did not reach all newcomer interactions within the bounded probe')
        except Exception as error:
            for seat in range(len(self.pages)):
                await self.screenshot('failure', seat)
            write_json(self.output / 'newcomer-summary.json', {'testType': 'visible-dom-only-newcomer', 'passed': False,
                       'checks': self.checks, 'readsApiViews': False, 'readsLegalActions': False,
                       'failure': f'{type(error).__name__}: {error}', 'browserErrors': self.errors, 'screenshots': self.shots})
            raise
        finally:
            for context in self.contexts: await context.close()

    async def mobile_reading(self):
        """Small latest-build reader check; no repeat of a complete game."""
        try:
            await self.setup()
            measurements = []
            for seat in (1, 3):
                page = self.pages[seat]
                await page.locator('.hg-hand .hg-card').first.click()
                opener = page.get_by_role('button', name=re.compile('放大文字与图标'))
                await opener.click()
                dialog = page.get_by_role('dialog', name=re.compile('放大阅读'))
                await dialog.wait_for(state='visible')
                box = await dialog.bounding_box()
                assert box and 0 <= box['x'] and box['x'] + box['width'] <= 390
                font = await dialog.locator('.hg-card-text').evaluate('(element) => getComputedStyle(element).fontSize')
                await page.keyboard.press('Tab')
                assert await page.evaluate('!!document.activeElement.closest("[role=dialog]")'), 'Reader keyboard focus left the modal'
                await self.screenshot('phone-reader', seat)
                await page.keyboard.press('Escape')
                await dialog.wait_for(state='hidden')
                measurements.append({'seat': seat, 'viewportWidth': 390, 'dialogWidth': round(box['width'], 2), 'bodyFont': font,
                                     'tabFocusContained': True, 'escapeClosed': True})
            assert not self.errors
            write_json(self.output / 'newcomer-reader-summary.json', {'testType': 'visible-dom-only-phone-reader', 'passed': True,
                       'readsApiViews': False, 'readsLegalActions': False, 'measurements': measurements,
                       'browserErrors': self.errors, 'screenshots': self.shots})
            print('Phone reader visible DOM checks passed', flush=True)
        except Exception as error:
            for seat in range(len(self.pages)): await self.screenshot('reader-failure', seat)
            write_json(self.output / 'newcomer-reader-summary.json', {'passed': False, 'failure': f'{type(error).__name__}: {error}', 'browserErrors': self.errors})
            raise
        finally:
            for context in self.contexts: await context.close()


async def main(args):
    async with async_playwright() as playwright:
        browser = await playwright.chromium.launch(executable_path=args.chromium, headless=True, args=['--no-sandbox', '--disable-dev-shm-usage'])
        try:
            probe = Newcomer(browser, args.base_url, args.output, args.max_steps)
            if args.reader_only: await probe.mobile_reading()
            else: await probe.run()
        finally:
            await browser.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', default='http://127.0.0.1:8090')
    parser.add_argument('--output', default='/tmp/hegemony-cloud-newcomer')
    parser.add_argument('--max-steps', type=int, default=700)
    parser.add_argument('--reader-only', action='store_true')
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    asyncio.run(main(parser.parse_args()))
