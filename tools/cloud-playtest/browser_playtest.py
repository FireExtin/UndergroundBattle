#!/usr/bin/env python3
"""Real Chromium UI play: distinct browser seats, contextual actions, no API moves."""
from __future__ import annotations

import argparse
import asyncio
import collections
import json
import re
import time
from pathlib import Path

from playwright.async_api import async_playwright

from common import public_view, wait_health, write_json
from api_acceptance import assert_projection
from initial_policy import choose_initial


# Observe accepted fetch responses, including each seat's authenticated SSE stream.
# This adds no game commands and never changes the application's response stream.
OBSERVE_FETCH = r"""(() => {
  const original = window.fetch.bind(window);
  window.__cloudView = null;
  window.__cloudCatalog = null;
  window.__cloudAccepted = [];
  function accept(value, source) {
    if (value && value.decks && value.cards) window.__cloudCatalog = value;
    const view = value && (value.view || (value.roomId && value.players ? value : null));
    if (view && (!window.__cloudView || view.version >= window.__cloudView.version)) {
      window.__cloudView = view;
      window.__cloudAccepted.push({version:view.version, source});
      if (window.__cloudAccepted.length > 50) window.__cloudAccepted.shift();
    }
  }
  window.fetch = async (...args) => {
    const response = await original(...args);
    const path = String(args[0] instanceof Request ? args[0].url : args[0]);
    if (path.includes('/api/')) {
      const clone = response.clone();
      if (path.includes('/events') && response.ok) {
        (async () => {
          const reader = clone.body.getReader(); const decoder = new TextDecoder(); let buffer = '';
          try { while (true) {
            const chunk = await reader.read(); if (chunk.done) break;
            buffer += decoder.decode(chunk.value, {stream:true}).replace(/\r\n/g,'\n');
            let end; while ((end=buffer.indexOf('\n\n')) >= 0) {
              const event=buffer.slice(0,end); buffer=buffer.slice(end+2);
              const data=event.split('\n').filter(line=>line.startsWith('data:')).map(line=>line.slice(5).trimStart()).join('\n');
              if (data) accept(JSON.parse(data), 'sse');
            }
          }} catch (_) {} finally {reader.releaseLock();}
        })();
      } else clone.json().then(value=>accept(value, response.ok ? 'http' : 'rejected')).catch(()=>{});
    }
    return response;
  };
})();"""


class UiRun:
    def __init__(self, browser, base_url, output, max_steps, policy='develop', response_deck=False):
        self.browser, self.base, self.output = browser, base_url.rstrip('/'), Path(output)
        self.output.mkdir(parents=True, exist_ok=True)
        self.max_steps = max_steps
        self.policy_name = policy
        self.response_deck = response_deck
        self.contexts, self.pages, self.errors = [], [], []
        self.expected_errors = []
        self.offline_probe_active = False
        self.records, self.coverage = [], collections.Counter()
        self.screenshots, self.recovery = [], []
        self.seen_choices = set()
        self.asset_turns = set()
        self.catalog = {}
        self.mode = ''

    async def new_seat(self, seat, session=None):
        context = await self.browser.new_context(viewport={"width": 1512, "height": 1040}, locale='zh-CN')
        if session is not None:
            await context.add_init_script('if(!localStorage.getItem("hegemony.session.v1"))localStorage.setItem("hegemony.session.v1",' + json.dumps(json.dumps(session)) + ');')
        await context.add_init_script(OBSERVE_FETCH)
        page = await context.new_page()
        page.on('pageerror', lambda error: self.errors.append({"seat": seat, "kind": "pageerror", "message": str(error)}))
        def console(message):
            if message.type != 'error': return
            entry = {"seat": seat, "kind": "console", "message": message.text}
            if self.offline_probe_active and 'net::ERR_INTERNET_DISCONNECTED' in message.text:
                self.expected_errors.append({**entry, "cause": "deliberate-offline-recovery-probe"})
            else:
                self.errors.append(entry)
        page.on('console', console)
        self.contexts.append(context)
        self.pages.append(page)
        await page.goto(self.base, wait_until='domcontentloaded')
        await page.wait_for_function('window.__cloudCatalog && window.__cloudCatalog.decks.length >= 4', timeout=30000)
        return page

    async def view(self, seat):
        return await self.pages[seat].evaluate('window.__cloudView')

    async def views(self):
        return await asyncio.gather(*(self.view(seat) for seat in range(len(self.pages))))

    async def sync(self, version):
        # Each independent client must receive the latest version via its own SSE.
        await asyncio.gather(*(page.wait_for_function('(v) => window.__cloudView && window.__cloudView.version >= v', arg=version, timeout=25000) for page in self.pages))
        # Let React commit the accepted view before contextual buttons are inspected.
        await asyncio.sleep(.045)
        for seat, view in enumerate(await self.views()):
            assert view['you'] == f'p{seat}'
            assert_projection(view)

    async def screenshot(self, name, seat=0):
        filename = f'{self.mode}-{name}.png'
        await self.pages[seat].screenshot(path=str(self.output / filename), full_page=True)
        self.screenshots.append({"file": filename, "seat": seat, "privateLocalArtifact": True})

    async def submit(self, seat, click, intended_kind):
        page = self.pages[seat]
        before = await self.view(seat)
        async with page.expect_response(lambda response: response.request.method == 'POST' and '/commands' in response.url, timeout=20000) as pending:
            await click()
        response = await pending.value
        body = await response.json()
        submitted = json.loads(response.request.post_data or '{}')
        actual = submitted.get('action', {})
        # Actual request fields have opaque instance IDs, no hand definitions or credentials.
        record = {"seat": seat, "kind": intended_kind, "httpStatus": response.status,
                  "before": public_view(before), "action": actual,
                  "commandId": submitted.get("commandId"), "expectedVersion": submitted.get("expectedVersion"),
                  "after": public_view(body if response.ok else body.get('view'))}
        if not response.ok:
            record['error'] = {key: body.get(key) for key in ('error', 'message')}
        self.records.append(record)
        with (self.output / f'{self.mode}-actions.jsonl').open('a') as stream:
            stream.write(json.dumps(record, ensure_ascii=False) + '\n')
        assert response.ok, f'{self.mode} seat {seat} {intended_kind} rejected HTTP {response.status}: {body.get("error")} {body.get("message")}'
        self.coverage[intended_kind] += 1
        await self.sync(body['version'])
        return body

    async def legal_button(self, seat, action):
        page = self.pages[seat]
        identity = action.get('cardId') or action.get('targetId')
        if identity:
            card = page.locator('[data-card-instance=' + json.dumps(identity) + ']')
            if await card.count() and not await card.first.is_visible():
                details = card.first.locator('xpath=ancestor::details[1]')
                if await details.count() and await details.get_attribute('open') is None:
                    await details.locator('summary').click()
            assert await card.count(), f'Contextual card {identity} absent from UI'
            await card.first.click()
        elif action.get('region') is not None:
            await page.locator('.hg-region-center').nth(action['region']).click()
        button = page.locator('[data-action-id=' + json.dumps(action['id']) + ']')
        if not await button.count():
            button = page.get_by_role('button', name=action['label'], exact=True)
        await button.first.wait_for(state='visible')
        await self.submit(seat, button.first.click, action['kind'])

    async def choose(self, seat, view):
        choice = view['pendingChoice']
        page = self.pages[seat]
        panel = page.locator('.hg-choice')
        await panel.wait_for(state='visible')
        self.coverage['choice:' + choice['kind']] += 1
        if choice['kind'] not in self.seen_choices:
            await self.screenshot('private-choice-' + choice['kind'], seat)
            self.seen_choices.add(choice['kind'])
        options = choice['options']
        kind = choice['kind']
        if kind in ('investigation', 'order'):
            # Exercise both ordering and top/bottom controls when there is a choice.
            if len(options) > 1:
                await panel.get_by_role('button', name='置于底', exact=True).last.click()
                self.coverage['ordering:bottom'] += 1
            if len(options) > 2:
                await panel.locator('.hg-order-list').first.locator('li').nth(1).get_by_role('button', name=re.compile('上移$')).click()
                self.coverage['ordering:reorder'] += 1
        elif kind == 'damage':
            amount = choice.get('amount', 0)
            allocation = [0] * len(options)
            # Printed guard cards each require their mandatory first damage point.
            guards = [i for i, option in enumerate(options) if option.get('card', {}).get('cardId') in ('LC21', 'LC22')]
            for i in guards:
                if not amount: break
                allocation[i] = 1; amount -= 1
            if amount and options:
                target = min(range(len(options)), key=lambda i: (options[i].get('card', {}).get('defense', 999), i))
                allocation[target] += amount
            for i, value in enumerate(allocation):
                if value:
                    await panel.get_by_role('spinbutton').nth(i).fill(str(value))
        else:
            selected = []
            if kind == 'mulligan':
                # Replace an expensive card to use the actual one-time mulligan.
                selected = sorted(range(len(options)), key=lambda i: options[i].get('card', {}).get('cost', 0), reverse=True)[:1]
            elif kind == 'region_return':
                # Newer UI defaults to all selected; reconcile rather than toggle
                # away that valid default. Then enact a custom bottom order.
                for index in range(len(options)):
                    button = panel.locator('.hg-choice-option').nth(index)
                    if await button.get_attribute('aria-pressed') != 'true':
                        await button.click()
                for index in range(len(options) - 1, 0, -1):
                    await panel.locator('.hg-return-order > div').nth(index).get_by_role('button', name=re.compile('上移$')).click()
                self.coverage['ordering:region-return'] += 1
            elif kind == 'discard':
                count = choice.get('min', 0) or min(1, choice.get('max', 0))
                selected = sorted(range(len(options)), key=lambda i: options[i].get('card', {}).get('cost', 0), reverse=True)[:count]
            elif kind == 'trigger' and any(o['id'] == 'discard' for o in options):
                selected = [next(i for i, o in enumerate(options) if o['id'] == 'discard')]
            elif options:
                if kind == 'search':
                    selected = [max(range(len(options)), key=lambda i: self.card_value(options[i].get('card', {})))]
                elif kind == 'target' and any(o['id'].startswith('p') for o in options):
                    you_team = next(p['team'] for p in view['players'] if p['id'] == view['you'])
                    selected = [next((i for i, o in enumerate(options) if any(p['id'] == o['id'] and p['team'] != you_team for p in view['players'])), 0)]
                elif kind == 'target' and any(o.get('card') for o in options):
                    you_team = next(p['team'] for p in view['players'] if p['id'] == view['you'])
                    enemy = [i for i, o in enumerate(options) if any(p['id'] == o.get('card', {}).get('controller') and p['team'] != you_team for p in view['players'])]
                    selected = [enemy[0] if enemy else 0]
                else:
                    selected = [0]
            for index in selected:
                await panel.locator('.hg-choice-option').nth(index).click()
        await self.submit(seat, panel.get_by_role('button', name='确认选择', exact=True).click, 'choose')

    def card_value(self, card):
        definition = self.catalog.get(card.get('cardId'), {})
        icons = definition.get('permanentIcons') or definition.get('icons', {}).get('permanent') or card.get('icons', {})
        temporary = definition.get('temporaryIcons') or definition.get('icons', {}).get('temporary', {})
        return (icons.get('influence', 0) * 5 + icons.get('investigation', 0) * 2 + icons.get('combat', 0) * 2
                + temporary.get('influence', 0) * 2 + (2 if definition.get('kind', card.get('kind')) == 'character' else -6)
                - card.get('cost', definition.get('cost', 0)) * .7)

    def policy(self, seat, view):
        if self.policy_name == 'initial': return choose_initial(self, seat, view)
        legal = view.get('legalActions', [])
        if not legal: return None
        own = next(p for p in view['players'] if p['id'] == view['you'])
        lookup = {c['instanceId']: c for c in view['hand'] + view['assets'] + view['graveyard'] + [c for r in view['regions'] for c in r['characters']]}
        assets = [a for a in legal if a['kind'] == 'asset']
        asset_count = sum(c['controller'] == view['you'] for c in view['assets'])
        if assets and asset_count < 4 and (len(view['hand']) > 1 or asset_count < 3 or not any(self.card_value(c) > -3 for c in view['hand'])):
            # Save efficient characters, build an asset with a weaker card each round.
            available = collections.Counter()
            for card in view['assets']:
                if card['controller'] == view['you']:
                    available[card.get('color', '') + '色'] += 1
                    if card.get('magic'): available[card['magic']] += 1
            needed = collections.Counter()
            for card in view['hand']:
                requirements = collections.Counter(self.catalog.get(card.get('cardId'), {}).get('loyalty', []))
                for symbol, count in requirements.items(): needed[symbol] = max(needed[symbol], count)
            def asset_score(action):
                card = lookup.get(action.get('cardId'), {})
                definition = self.catalog.get(card.get('cardId'), {})
                symbols = [card.get('color', definition.get('color', '')) + '色', card.get('magic', definition.get('magic', ''))]
                bonus = sum(20 for symbol in symbols if symbol and available[symbol] < needed[symbol])
                return self.card_value(card) - bonus
            return min(assets, key=asset_score)
        deploy = [a for a in legal if a['kind'] == 'deploy' and self.card_value(lookup.get(a.get('cardId'), {})) > 2]
        conceal = [a for a in legal if a['kind'] == 'conceal']
        # First-round contested deployment creates actual combat decisions; later
        # teams also claim other locations so the play reaches a points victory.
        if self.mode == 'duel':
            preferred = 0 if own['team'] == 0 or view['turn'] == 1 else 1
        else:
            preferred = 2 if view['turn'] <= 2 else (0 if seat % 2 == 0 else 4)
        def deploy_score(action):
            region = view['regions'][action['region']]
            enemy_count = sum(next(p['team'] for p in view['players'] if p['id'] == c['controller']) != own['team'] for c in region['characters'])
            focus = 12 if action['region'] == preferred else 0
            return self.card_value(lookup.get(action.get('cardId'), {})) + focus - enemy_count * .5 + region['influence'][own['team']] * .1
        # Conceal once early in each game, then develop face-up icons.
        weak_conceal = [a for a in conceal if self.card_value(lookup.get(a.get('cardId'), {})) <= 2]
        if weak_conceal:
            return max(weak_conceal, key=deploy_score)
        if deploy:
            return max(deploy, key=deploy_score)
        reveals = [a for a in legal if a['kind'] == 'reveal' and self.card_value(lookup.get(a.get('cardId'), {})) > 2]
        if reveals:
            return max(reveals, key=lambda a: self.card_value(lookup.get(a.get('cardId'), {})))
        if conceal:
            return max(conceal, key=deploy_score)
        privileges = [a for a in legal if a['kind'] == 'privilege']
        if privileges: return privileges[0]
        # Play interaction only against opponents; avoid a looping heal activation.
        enemy_ids = {p['id'] for p in view['players'] if p['team'] != own['team']}
        control = [a for a in legal if a['kind'] == 'activate' and lookup.get(a.get('cardId'), {}).get('cardId') == 'JC003'
                   and lookup.get(a.get('targetId'), {}).get('controller') in enemy_ids and not lookup.get(a.get('targetId'), {}).get('exhausted')]
        if control: return control[0]
        plays = [a for a in legal if a['kind'] == 'play' and a.get('targetId') and lookup.get(a['targetId'], {}).get('controller') in enemy_ids]
        if plays and self.coverage['play'] < len(self.pages) * 2: return plays[0]
        return next((a for a in legal if a['kind'] == 'pass'), None)

    async def verify_recovery(self, seat, view):
        choice = view['pendingChoice']
        page = self.pages[seat]
        saved = {'version': view['version'], 'choiceId': choice['id'], 'handIds': [c['instanceId'] for c in view['hand']]}
        await page.reload(wait_until='domcontentloaded')
        await page.wait_for_function('(id) => window.__cloudView && window.__cloudView.pendingChoice && window.__cloudView.pendingChoice.id === id', arg=choice['id'], timeout=20000)
        refreshed = await self.view(seat)
        assert refreshed['version'] == saved['version']
        assert [c['instanceId'] for c in refreshed['hand']] == saved['handIds']
        self.recovery.append({'kind': 'refresh-pending-choice', 'seat': seat, 'version': saved['version'], 'passed': True})
        self.offline_probe_active = True
        await self.contexts[seat].set_offline(True)
        failed = await page.evaluate("async () => {try {await fetch('/api/health');return false} catch (_) {return true}}")
        assert failed, 'Chromium offline mode did not block the service read'
        await self.screenshot('offline-pending-choice', seat)
        await self.contexts[seat].set_offline(False)
        await page.reload(wait_until='domcontentloaded')
        await page.wait_for_function('(id) => window.__cloudView && window.__cloudView.pendingChoice && window.__cloudView.pendingChoice.id === id', arg=choice['id'], timeout=25000)
        restored = await self.view(seat)
        assert restored['version'] == saved['version']
        assert [c['instanceId'] for c in restored['hand']] == saved['handIds']
        self.recovery.append({'kind': 'offline-reconnect-pending-choice', 'seat': seat, 'version': saved['version'], 'passed': True})
        self.offline_probe_active = False

    async def play(self, mode):
        self.mode = mode
        started = time.time()
        capacity = 2 if mode == 'duel' else 4
        try:
            host = await self.new_seat(0)
            catalog = await host.evaluate('window.__cloudCatalog')
            self.catalog = {card['id']: card for card in catalog['cards']}
            decks = catalog['decks']
            chosen_decks = [decks[seat]['id'] for seat in range(capacity)]
            if self.response_deck:
                assert any(deck['id'] == 'responders' for deck in decks), 'Response deck unavailable in this binary'
                chosen_decks[-1] = 'responders'
            await self.screenshot('initial-lobby')
            await host.locator('.hg-deck[data-deck-id=' + json.dumps(chosen_decks[0]) + ']').click()
            await host.get_by_label('你的称呼').fill(f'Cloud {mode} A')
            if mode == 'teams':
                await host.get_by_role('button', name=re.compile('四人协作')).click()
            await host.get_by_role('button', name='创建牌桌 →', exact=True).click()
            await host.wait_for_function('window.__cloudView && window.__cloudView.status === "lobby"')
            invitation = (await self.view(0))['inviteCode']
            for seat in range(1, capacity):
                page = await self.new_seat(seat)
                await page.locator('.hg-deck[data-deck-id=' + json.dumps(chosen_decks[seat]) + ']').click()
                await page.get_by_role('button', name='邀请码加入', exact=True).click()
                await page.get_by_label('你的称呼').fill(f'Cloud {mode} {chr(65 + seat)}')
                await page.get_by_label('邀请码', exact=True).fill(invitation)
                await page.get_by_role('button', name='加入牌桌 →', exact=True).click()
                await page.wait_for_function('window.__cloudView && window.__cloudView.status === "lobby"')
            await self.sync((await self.view(capacity - 1))['version'])
            room = await self.view(0)
            assert len(set(p['deckId'] for p in room['players'])) == capacity
            await self.screenshot('full-room-lobby')
            for seat in range(capacity):
                view = await self.view(seat)
                await self.legal_button(seat, next(a for a in view['legalActions'] if a['kind'] == 'ready'))
            await self.legal_button(0, next(a for a in (await self.view(0))['legalActions'] if a['kind'] == 'start'))
            return await self.play_loop(started, capacity)
        except Exception as error:
            view = await self.view(0) if self.pages else None
            if self.pages:
                await self.screenshot('failure')
            result = self.summary(view, started, len(self.records))
            result.update(passed=False, failure=f'{type(error).__name__}: {error}')
            write_json(self.output / f'{mode}-summary.json', result)
            raise
        finally:
            for context in self.contexts:
                await context.close()

    async def resume(self, sessions):
        """Restore this runner's existing seats; never create, join, start or seed a game."""
        self.mode = 'teams'
        started = time.time()
        assert len(sessions) == 4 and [s['seat'] for s in sessions] == list(range(4))
        assert len({s['roomId'] for s in sessions}) == 1
        try:
            for seat, session in enumerate(sessions):
                await self.new_seat(seat, session)
            await self.pages[0].wait_for_function('window.__cloudView')
            await self.sync((await self.view(0))['version'])
            views = await self.views()
            assert all(v['roomId'] == sessions[0]['roomId'] and v['versions']['engine'] == 'rust-v0.2.1' for v in views)
            catalog = await self.pages[0].evaluate('window.__cloudCatalog')
            self.catalog = {card['id']: card for card in catalog['cards']}
            self.recovery.append({'kind': 'restore-four-original-seats', 'version': views[0]['version'], 'passed': True})
            await self.screenshot('resumed-original-room')
            return await self.play_loop(started, 4)
        except Exception as error:
            view = await self.view(0) if self.pages else None
            if self.pages:
                await self.screenshot('continuation-failure')
            result = self.summary(view, started, len(self.records))
            result.update(passed=False, failureType=type(error).__name__)
            write_json(self.output / 'teams-summary.json', result)
            raise
        finally:
            for context in self.contexts:
                await context.close()

    async def play_loop(self, started, capacity):
        mode = self.mode
        spatial = False
        for step in range(self.max_steps):
            views = await self.views()
            current = max(views, key=lambda v: v['version'])
            if current['status'] == 'finished':
                await self.screenshot('finished', 0)
                # Mobile reuses one authenticated browser seat, preserving its view.
                await self.pages[0].set_viewport_size({'width': 390, 'height': 844})
                await self.screenshot('mobile-finished', 0)
                overflow = await self.pages[0].evaluate('document.documentElement.scrollWidth > innerWidth + 1')
                assert not overflow, 'Mobile viewport has horizontal document overflow'
                return self.summary(current, started, step)
            chosen = next(((seat, view) for seat, view in enumerate(views) if view.get('pendingChoice')), None)
            if chosen:
                seat, view = chosen
                if not self.recovery:
                    await self.verify_recovery(seat, view)
                    view = await self.view(seat)
                await self.choose(seat, view)
            else:
                candidates = [(seat, self.policy(seat, view)) for seat, view in enumerate(views)]
                # In team action steps, every eligible teammate develops before passing.
                chosen = next(((seat, action) for seat, action in candidates if action and action['kind'] != 'pass'), None)
                if chosen is None:
                    chosen = next(((seat, action) for seat, action in candidates if action), None)
                assert chosen, f'No legal UI action at version {current["version"]}, {current["phase"]} {current["step"]}'
                await self.legal_button(*chosen)
            latest = await self.view(0)
            if not spatial and sum(len(r['characters']) for r in latest['regions']) >= capacity:
                await self.screenshot('spatial-board')
                await self.pages[0].set_viewport_size({'width': 390, 'height': 844})
                await self.screenshot('mobile-board')
                assert not await self.pages[0].evaluate('document.documentElement.scrollWidth > innerWidth + 1'), 'Mobile board has horizontal document overflow'
                await self.pages[0].set_viewport_size({'width': 1512, 'height': 1040})
                spatial = True
            if step % 50 == 0:
                print(json.dumps({'mode': mode, 'step': step, 'version': latest['version'], 'turn': latest['turn'], 'scores': [p['score'] for p in latest['players']]}, ensure_ascii=False), flush=True)
        raise RuntimeError(f'{mode} exceeded {self.max_steps} legal UI actions')

    def summary(self, view, started, steps):
        write_json(self.output / f'{self.mode}-actions.json', self.records)
        result = {'scenario': self.mode, 'testType': 'real-browser-ui', 'passed': bool(view and view['status'] == 'finished'),
                  'durationSeconds': round(time.time() - started, 2), 'steps': steps,
                  'acceptedCommands': sum(record['httpStatus'] == 200 for record in self.records),
                  'rejectedCommands': sum(record['httpStatus'] != 200 for record in self.records),
                  'independentContexts': len(self.contexts), 'coverage': dict(self.coverage),
                  'policy': self.policy_name,
                  'responseDeckRequested': self.response_deck,
                  'recovery': self.recovery, 'screenshots': self.screenshots,
                  'browserErrors': self.errors, 'expectedBrowserErrors': self.expected_errors, 'final': public_view(view)}
        write_json(self.output / f'{self.mode}-summary.json', result)
        return result


async def main(args):
    wait_health(args.base_url, args.ready_timeout)
    async with async_playwright() as playwright:
        browser = await playwright.chromium.launch(executable_path=args.chromium, headless=True, args=['--no-sandbox', '--disable-dev-shm-usage'])
        results = []
        try:
            for mode in args.modes.split(','):
                run = UiRun(browser, args.base_url, args.output, args.max_steps, args.policy, args.response_deck)
                results.append(await run.play(mode))
            write_json(Path(args.output) / 'browser-summary.json', results)
            print(json.dumps([{'scenario': r['scenario'], 'passed': r['passed'], 'steps': r['steps'], 'winnerTeam': r['final'].get('winnerTeam'), 'turn': r['final']['turn']} for r in results]), flush=True)
        finally:
            await browser.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', default='http://127.0.0.1:8090')
    parser.add_argument('--output', default='/tmp/hegemony-cloud-playtest')
    parser.add_argument('--modes', default='duel,teams')
    parser.add_argument('--max-steps', type=int, default=4500)
    parser.add_argument('--policy', choices=['develop', 'initial'], default='develop')
    parser.add_argument('--response-deck', action='store_true', help='Use responders for the final seat, keeping all seat decks distinct')
    parser.add_argument('--ready-timeout', type=int, default=45)
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    asyncio.run(main(parser.parse_args()))
