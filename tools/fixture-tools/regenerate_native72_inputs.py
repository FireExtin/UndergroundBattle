"""Generate current Native72/73/74/75/76 inputs, retaining all recorded inputs and evidence."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys


def validated_paths(source_value, output_value):
    if not source_value or not source_value.strip() or not output_value or not output_value.strip():
        raise ValueError('Source and output paths must be nonempty.')
    source = Path(source_value).resolve(strict=True)
    required = ['Cargo.toml', 'rust-game/Cargo.toml', 'rust-game-wasm/Cargo.toml',
                'rust-game-wasm/build.sh', 'tools/build-storage.mjs']
    if not source.is_dir() or any(not (source / name).is_file() for name in required):
        raise ValueError('Source must be a project checkout with its existing build-storage guard.')
    requested = Path(output_value)
    if requested.is_symlink():
        raise ValueError('Output must be an empty directory, not a symlink.')
    root = requested.resolve()
    if root == source or root in source.parents or source in root.parents:
        raise ValueError('Source and output directories must be separate and non-overlapping.')
    validate_empty_output(root)
    ancestor = root
    while not ancestor.exists():
        ancestor = ancestor.parent
    if not ancestor.is_dir():
        raise ValueError('Output parent must be a directory.')
    return source, root


def validate_empty_output(root):
    if root.is_symlink() or (root.exists() and (not root.is_dir() or any(root.iterdir()))):
        raise ValueError('Output must be new or empty; existing input and logs are preserved.')


def validate_new_input_targets(source, version):
    web = [f'{card}Native{version}.fixture.json'
           for card in ['jc061', 'jc087', 'jc094', 'wm056', 'wm023', 'xq25',
                        'wm038', 'jz35', 'xq24', 'bq030', 'fixedSlots']]
    web += [f'currentLifecycle{version}.fixture.json', f'sevenCardUIV0{version}.fixture.json']
    web += [f'{name}Native{version}Test.fixture.json'
            for name in ['threeColorDeath', 'xq37', 'jz22', 'deckSeal', 'xq27',
                         'boundedEntry', 'jc050', 'xq18']]
    web.append(f'jc050Native{version}QuoteTest.fixture.json')
    if version in {'73', '74', '75', '76'}:
        web.append(f'jc052Native{version}Current.fixture.json')
    sites = []
    if version in {'74', '75', '76'}:
        web.append(f'jc037Native{version}.fixture.json')
        sites = [f'{name}-v0{version}.json' for name in [
            'xq37-native', 'gray-lock-native', 'jc050-native', 'xq27-invalid-rootface',
            'deck-seal-native', 'three-color-death-native', 'xq37-invalid-rootface',
            'jz50-room-native', 'bounded-entry-native', 'xq27-native', 'pause-resume-native',
            'jz22-native', 'sealing-room-native', 'sealed-restart-native', 'win-flow-native',
            'bq030-native', 'xq24-native', 'xq24-container-invalid', 'jc052-native', 'jc037-native']]
    if version == '76':
        web.append(f'jc060Native{version}.fixture.json')
        sites.append(f'jc060-native-v0{version}.json')
    paths = [source / 'web/src/game' / name for name in web]
    paths += [source / 'sites/test/fixtures' / name for name in sites]
    for path in paths:
        if path.exists() or path.is_symlink():
            raise ValueError(f'Native input already exists; preserve it and use fresh destinations: {path}')


def regenerate(source, root, build_and_check):
    validate_empty_output(root)
    pool = json.loads((source / 'rust-game/data/cards.json').read_text())
    versions = {
        'rust-v0.2.72-jz35-host-return-candidate': ('72', 'limited-v2.65-jz35-host-return-candidate'),
        'rust-v0.2.73-jc052-original-region-candidate': ('73', 'limited-v2.66-jc052-original-region-candidate'),
        'rust-v0.2.74-jc037-hidden-investigation-candidate': ('74', 'limited-v2.67-jc037-hidden-investigation-candidate'),
        'rust-v0.2.75-jc060-occult-adviser-candidate': ('75', 'limited-v2.68-jc060-occult-adviser-candidate'),
        'rust-v0.2.76-jc095-influence-conversion-candidate': ('76', 'limited-v2.69-jc095-influence-conversion-candidate'),
    }
    if pool.get('engineVersion') not in versions:
        raise ValueError('Only explicitly supported ABI72/73/74/75/76 candidate generation is supported.')
    version, expected_pool = versions[pool['engineVersion']]
    web_only = [] if version in {'74', '75', '76'} else ['--web-only']
    if pool.get('cardPoolVersion') != expected_pool:
        raise ValueError('Source engine/card-pool versions do not match.')
    validate_new_input_targets(source, version)
    # Reuse the project guard before creating evidence or touching any source.
    # Do not trust an inherited BUILD_STORAGE_READY flag at either entrypoint.
    guard = ['node', str(source / 'tools/build-storage.mjs'),
             '--output-dir', str(root), '--output-dir', str(source / 'rust-game-wasm/pkg'),
             '--output-dir', str(source / 'web/dist')]
    preflight = subprocess.run(guard, cwd=source, text=True, capture_output=True)
    if preflight.returncode:
        sys.stderr.write(preflight.stderr)
        raise SystemExit(preflight.returncode)
    validate_empty_output(root)
    validate_new_input_targets(source, version)
    root.mkdir(parents=True, exist_ok=True)
    e = root / 'evidence/ui-refresh'
    e.mkdir(parents=True, exist_ok=False)
    with (e / 'preflight.json').open('x') as log:
        log.write(preflight.stdout)
    records = []

    def run(name, args, extra=None):
        env = os.environ.copy()
        env.update(extra or {})
        # Every Cargo launch rechecks space through the same existing guard.
        with (e / (name + '.log')).open('x') as log:
            result = subprocess.run([*guard, '--', *args], cwd=source, env=env,
                                    stdout=log, stderr=subprocess.STDOUT)
        records.append({'name': name, 'exit_code': result.returncode,
                        'log': str(e / (name + '.log'))})
        with (e / f'progress-{len(records):03}.json').open('x') as progress:
            progress.write(json.dumps(records, indent=2) + '\n')
        print(name, result.returncode, flush=True)
        if result.returncode:
            raise SystemExit(result.returncode)

    def tests(name, pattern, extra):
        run(name, ['cargo', 'test', '--locked', '-p', 'hegemony-server', '--lib',
                   pattern, '--', '--test-threads=1'], extra)

    if build_and_check:
        # Archive mtimes can otherwise select a previous source snapshot's cache.
        for directory in ['rust-game/src', 'rust-game-wasm/src']:
            for path in (source / directory).rglob('*'):
                if path.is_file():
                    path.touch()
        run('wasm-build', ['bash', 'rust-game-wasm/build.sh'])
    # Check real bindings before any generator can overwrite a prior ABI input.
    run('abi-version-check', ['node', '--input-type=module', '-e',
        "import assert from 'node:assert/strict';"
        "import {catalog} from './rust-game-wasm/tests/current-kernel.mjs';"
        f"assert.equal(catalog.engineVersion,{json.dumps(pool['engineVersion'])});"
        f"assert.equal(catalog.cardPoolVersion,{json.dumps(expected_pool)});"])
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=source, text=True).strip()
    for card in ['jc061','jc087','jc094','wm056','wm023','xq25','wm038']:
        base=e/card
        tests(card,card,{'GREEN_EVIDENCE_DIR':str(base/'trace'),card.upper()+'_EVIDENCE_DIR':str(base/'native'),card.upper()+'_INVALID_DIR':str(base/'invalid')})
        run(card+'-fixture',['python3','tools/fixture-tools/'+card+'-current.py',str(base),version])
    tests('lifecycle-blue','blue_jc032',{'GREEN_EVIDENCE_DIR':str(e/'lifecycle/trace-blue'),'BLUE_FRONTEND_DIR':str(e/'lifecycle/blue')})
    tests('lifecycle-black','black_expansion',{'GREEN_EVIDENCE_DIR':str(e/'lifecycle/trace-black'),'BLACK_EXPANSION_EVIDENCE_DIR':str(e/'lifecycle/black')})
    tests('lifecycle-red','red_time',{'GREEN_EVIDENCE_DIR':str(e/'lifecycle/trace-red'),'RED_TIME_EVIDENCE_DIR':str(e/'lifecycle/red')})
    run('lifecycle-fixture',['python3','tools/fixture-tools/current-four-faction-choices.py',str(e/'lifecycle'),version,revision])
    tests('death-frontends','death_observer',{'DEATH_OBSERVER_FRONTEND_DIR':str(e/'death')})
    run('current-worker-web-fixtures',['cargo','run','--locked','-p','hegemony-wasm','--example','current_ui_worker_fixtures','--',str(source),str(e/'death'),version,*web_only])
    run('seven-card-fixtures',['cargo','run','--locked','-p','hegemony-wasm','--example','seven_card_ui_fixtures','--',str(source/f'web/src/game/sevenCardUIV0{version}.fixture.json')])
    tests('win-flow','win_flow',{'WIN_FLOW_EVIDENCE_DIR':str(e/'win')})
    run('fixed-slots-fixture',['python3','tools/fixture-tools/current-win-flow.py',str(e/'win'),version,*web_only])
    run('bq030',['cargo','test','--locked','-p','hegemony-server','--test','bq030_attachment_regression','--','--test-threads=1'],{'BQ030_TRACE_DIR':str(e/'bq030/room')})
    run('bq030-fixture',['node','tools/fixture-tools/current-bq030.mjs',str(e/'bq030'),str(source/'rust-game-wasm/pkg/hegemony_wasm.js'),*web_only])
    base=root/'evidence'
    tests('jz35','jz35',{'GREEN_EVIDENCE_DIR':str(base/'trace'),'JZ35_EVIDENCE_DIR':str(base/'native'),'JZ35_INVALID_DIR':str(base/'invalid')})
    run('jz35-fixture',['python3','tools/fixture-tools/jz35-current.py',str(base),version])
    tests('xq24','xq24_continuous_duel_teams_room_pause_restore_journal_and_receipt_inputs',{'XQ24_TRACE_DIR':str(e/'xq24')})
    rows=[json.loads(p.read_text()) for p in sorted((e/'xq24').glob('*.json'))]
    assert len(rows)==12,len(rows)
    assert all(row['catalog']['engineVersion'] == pool['engineVersion']
               and row['catalog']['cardPoolVersion'] == expected_pool for row in rows)
    xq24_payload = json.dumps({'catalog':rows[0]['catalog'],'scenarios':rows},ensure_ascii=False,separators=(',',':'))+'\n'
    with (source/f'web/src/game/xq24Native{version}.fixture.json').open('x') as output:
        output.write(xq24_payload)
    if version in {'73', '74', '75', '76'}:
        jc052 = e / 'jc052'
        tests('jc052', 'jc052', {'GREEN_EVIDENCE_DIR':str(jc052/'trace'),
              'JC052_EVIDENCE_DIR':str(jc052/'native'), 'JC052_INVALID_DIR':str(jc052/'invalid')})
        # Keep the author-produced ABI-hashed evidence immutable as well.
        run('jc052-fixture', ['python3','tools/fixture-tools/jc052-current.py',str(jc052),
                             str(source/f'web/src/game/jc052Native{version}Current.fixture.json')])
    if version in {'74', '75', '76'}:
        (source/'sites/test/fixtures').mkdir(parents=True,exist_ok=True)
        with (source/f'sites/test/fixtures/xq24-native-v0{version}.json').open('x') as output:
            output.write(xq24_payload)
        tests('xq24-container', 'xq24_entry_frame_container_restore_rejects_response_bypass',
              {'XQ24_CONTAINER_EVIDENCE_FILE':str(e/'xq24-container-invalid.json')})
        with (source/f'sites/test/fixtures/xq24-container-invalid-v0{version}.json').open('xb') as output:
            output.write((e/'xq24-container-invalid.json').read_bytes())
        run('jc052-worker-fixtures',['cargo','run','--locked','-p','hegemony-wasm','--example','current_jc052_worker_fixtures','--',str(source/f'web/src/game/jc052Native{version}Current.fixture.json'),str(source/f'sites/test/fixtures/jc052-native-v0{version}.json')])
        tests('jc037', 'jc037', {'JC037_NATIVE_DIR':str(e/'jc037/native')})
        run('jc037-fixture', ['python3','tools/fixture-tools/jc037-current.py',str(e/'jc037/native')])
    if version == '76':
        tests('jc060', 'jc060', {'JC060_NATIVE_DIR':str(e/'jc060/native')})
        run('jc060-fixture', ['python3','tools/fixture-tools/jc060-current.py',str(e/'jc060/native')])
    print(f'native{version}-input-generation-complete',flush=True)

    if build_and_check:
        run('jz35-parity', ['node', 'rust-game-wasm/tests/jz35_compare.mjs', str(root / 'evidence')])
        for card in ['jc061', 'jc087', 'jc094', 'wm056', 'wm023', 'xq25', 'wm038']:
            run(card + '-parity', ['node', f'rust-game-wasm/tests/{card}_compare.mjs',
                                  str(e / card)])
        if version in {'73', '74', '75', '76'}:
            run('jc052-parity', ['node','rust-game-wasm/tests/jc052_compare.mjs',str(e/'jc052')])
        if version in {'74', '75', '76'}:
            run('jc037-parity', ['node','rust-game-wasm/tests/jc037_compare.mjs',str(source/f'web/src/game/jc037Native{version}.fixture.json')])
        if version == '76':
            run('jc060-parity', ['node','rust-game-wasm/tests/jc060_compare.mjs',str(source/f'web/src/game/jc060Native{version}.fixture.json')])
        focused = ['src/game/JZ35Integration.test.jsx', 'src/game/JZ35Face.test.jsx',
                   'src/game/CurrentLifecycleChoices.test.jsx', 'src/game/SevenCardIntegration.test.jsx']
        if version in {'73', '74', '75', '76'}:
            focused.extend(['src/game/JC052Integration.test.jsx', 'src/game/JC052Face.test.jsx'])
        run('focused-ui', ['npm', '--prefix', 'web', 'test', '--', *focused, '--maxWorkers=1'])
        run('full-ui', ['npm', '--prefix', 'web', 'test', '--', '--maxWorkers=1'])
        run('web-build', ['npm', '--prefix', 'web', 'run', 'build'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build-and-check', action='store_true')
    parser.add_argument('source', nargs='?')
    parser.add_argument('output', nargs='?')
    args = parser.parse_args()
    if args.source is None and args.output is None:
        source_value = os.environ.get('JZ35_SOURCE_ROOT')
        output_value = os.environ.get('JZ35_REVIEW_ROOT')
    elif args.source is not None and args.output is not None:
        source_value, output_value = args.source, args.output
    else:
        parser.error('Provide both source and output paths.')
    try:
        source, root = validated_paths(source_value, output_value)
        regenerate(source, root, args.build_and_check)
    except (OSError, ValueError) as error:
        print(f'Native input generation refused: {error}', file=sys.stderr)
        return 2
    return 0


if __name__ == '__main__':
    sys.exit(main())
