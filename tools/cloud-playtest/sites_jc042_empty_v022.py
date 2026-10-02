#!/usr/bin/env python3
"""UI-only JC042 check continuing the same normally dealt bounded fixture."""
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
    output = Path(args.output); output.mkdir(parents=True, exist_ok=True)
    result = {'passed': False, 'testType': 'new-v022-jc042-empty-stack-same-fixture-ui-continuation',
        'roomId': sessions[0]['roomId'], 'newRoomsCreated': 0, 'apiGameplayCommands': 0,
        'siteBypassUsed': False, 'seedOverride': False, 'intermediateStateInjection': False,
        'originalMurderFixtureBoundFailurePreserved': True, 'countsAsCompleteGame': False}
    public = run = None
    async with async_playwright() as playwright:
        browser = await playwright.chromium.launch(executable_path=args.chromium, headless=True,
            proxy={'server': os.environ['HTTPS_PROXY']}, args=['--no-sandbox', '--disable-dev-shm-usage'])
        try:
            assert len(sessions) == 2 and [s['seat'] for s in sessions] == [0, 1]
            public = PublicBrowser(browser, args.base_url)
            run = UiRun(public, args.base_url, output, 120, 'develop', True); run.mode = 'duel'
            for seat, session in enumerate(sessions): await run.new_seat(seat, session)
            await run.pages[0].wait_for_function('window.__cloudView')
            await run.sync((await run.view(0))['version'])
            if args.confirm_original:
                result['verificationStage'] = 'original-record'
                records = json.loads(Path(args.confirm_original).read_text())
                record = records[-1]; seat = record['seat']; action = record['action']
                assert record['httpStatus'] == 200 and action['kind'] == 'activate' and action['abilityId'] == 'reduce-next'
                snapshots = await run.views(); after = snapshots[seat]
                result['verificationStage'] = 'restored-original-version'
                assert after['version'] == record['after']['version'] and after['versions']['engine'] == 'rust-v0.2.2'
                assert record['before']['stackCount'] == record['after']['stackCount'] == 0
                assert not after['stack'] and not after.get('pendingChoice')
                result['verificationStage'] = 'sacrifice-zone-transition'
                assert any(c['cardId'] == 'JC042' and c['owner'] == after['you'] for c in after['graveyard'])
                assert all(c['instanceId'] != action['cardId'] for r in after['regions'] for c in r['characters'])
                for page in run.pages: await page.reload(wait_until='domcontentloaded')
                result['verificationStage'] = 'refresh'
                await run.sync(after['version']); assert await run.views() == snapshots
                command = {k: record[k] for k in ('commandId', 'expectedVersion', 'action')}
                response = await run.contexts[seat].request.post(args.base_url + '/api/rooms/' + sessions[seat]['roomId'] + '/commands',
                    headers={'Authorization': 'Bearer ' + sessions[seat]['token']}, data=command, max_redirects=0)
                result['verificationStage'] = 'original-receipt'
                assert response.status == 200 and await response.json() == after and await run.views() == snapshots
                await run.screenshot('original-immediate-action-exact-refresh', seat)
                assert not run.errors and not public.transport_events
                result.update(passed=True, verificationStage='complete', noResponseObject=True, sourceSacrificedOnce=True, refreshExact=True,
                    originalReceiptExact=True, version=after['version'], engineVersion=after['versions']['engine'],
                    originalUiCommands=len(records), newUiGameplayCommands=0, originalFailuresUnmodified=True,
                    handCostObservation='No eligible remaining hand card in the original UI run; not verified by this supplement')
                return result
            for step in range(120):
                views = await run.views()
                assert all(v['roomId'] == sessions[0]['roomId'] and v['versions']['engine'] == 'rust-v0.2.2' for v in views)
                if any(v.get('pendingChoice') for v in views):
                    seat, view = next((i, v) for i, v in enumerate(views) if v.get('pendingChoice'))
                    await run.choose(seat, view); continue
                source = next(((seat, action) for seat, view in enumerate(views) for action in view['legalActions']
                    if action['kind'] == 'activate' and action.get('abilityId') == 'reduce-next' and not view['stack']), None)
                if source:
                    seat, action = source; before = views[seat]
                    await run.legal_button(seat, action)
                    after = await run.view(seat)
                    assert after['stack'] == [] and not after.get('pendingChoice')
                    assert any(c['cardId'] == 'JC042' and c['owner'] == after['you'] for c in after['graveyard'])
                    assert all(c['instanceId'] != action['cardId'] for r in after['regions'] for c in r['characters'])
                    hand_before = {c['instanceId']: c for c in before['hand']}
                    reductions = [{'cardId': c['cardId'], 'before': hand_before[c['instanceId']].get('effectiveCost'),
                        'after': c.get('effectiveCost')} for c in after['hand']
                        if c['instanceId'] in hand_before and isinstance(c.get('effectiveCost'), int)
                        and c['effectiveCost'] < hand_before[c['instanceId']].get('effectiveCost', 0)]
                    assert reductions, 'No qualifying normally dealt hand card demonstrates the immediate reduction'
                    snapshots = await run.views()
                    for page in run.pages: await page.reload(wait_until='domcontentloaded')
                    await run.sync(after['version']); assert await run.views() == snapshots
                    record = run.records[-1]
                    command = {k: record[k] for k in ('commandId', 'expectedVersion', 'action')}
                    response = await run.contexts[seat].request.post(args.base_url + '/api/rooms/' + sessions[seat]['roomId'] + '/commands',
                        headers={'Authorization': 'Bearer ' + sessions[seat]['token']}, data=command, max_redirects=0)
                    assert response.status == 200 and await response.json() == after
                    assert await run.views() == snapshots
                    result.update(immediateReduction=True, reductionEvidence=reductions, noResponseObject=True,
                        refreshExact=True, originalReceiptExact=True, noRepeatedSacrifice=True, version=after['version'],
                        engineVersion=after['versions']['engine'], uiCommands=len(run.records))
                    await run.screenshot('immediate-reduction-and-exact-refresh', seat)
                    assert not run.errors and not public.transport_events
                    result['passed'] = True; break
                deploy = next(((seat, action) for seat, view in enumerate(views) for action in view['legalActions']
                    if action['kind'] == 'deploy' and any(c['instanceId'] == action.get('cardId') and c['cardId'] == 'JC042' for c in view['hand'])), None)
                if deploy and not views[0]['stack']: await run.legal_button(*deploy); continue
                passing = next(((seat, action) for seat, view in enumerate(views) for action in view['legalActions'] if action['kind'] == 'pass'), None)
                assert passing, 'No legal continuation action'
                await run.legal_button(*passing)
            assert result['passed']
        except Exception as error:
            result['failureType'] = type(error).__name__
        finally:
            if run:
                result['browserErrors'] = run.errors; write_json(output / 'ui-actions.json', run.records)
            if public: write_json(output / 'qa-transport-events.json', public.transport_events)
            await browser.close()
    write_json(output / 'summary.json', result); print(json.dumps(result, ensure_ascii=False), flush=True)
    return result['passed']


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    parser.add_argument('--confirm-original', help='Verify an already accepted action from this owned fixture, without new gameplay')
    args = parser.parse_args()
    sessions = json.loads(getpass.getpass('Same two owned fixture seats (input hidden): '))
    result = asyncio.run(main(args, sessions))
    if isinstance(result, dict):
        write_json(Path(args.output) / 'summary.json', result); print(json.dumps(result, ensure_ascii=False), flush=True)
        result = result['passed']
    raise SystemExit(0 if result else 1)
