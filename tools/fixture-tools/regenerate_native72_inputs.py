"""Generate fresh Native72 inputs, retaining every prior evidence directory."""
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


def regenerate(source, root, build_and_check):
    validate_empty_output(root)
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
    for card in ['jc061','jc087','jc094','wm056','wm023','xq25','wm038']:
        base=e/card
        tests(card,card,{'GREEN_EVIDENCE_DIR':str(base/'trace'),card.upper()+'_EVIDENCE_DIR':str(base/'native'),card.upper()+'_INVALID_DIR':str(base/'invalid')})
        run(card+'-fixture',['python3','tools/fixture-tools/'+card+'-current.py',str(base),'72'])
    tests('lifecycle-blue','blue_jc032',{'GREEN_EVIDENCE_DIR':str(e/'lifecycle/trace-blue'),'BLUE_FRONTEND_DIR':str(e/'lifecycle/blue')})
    tests('lifecycle-black','black_expansion',{'GREEN_EVIDENCE_DIR':str(e/'lifecycle/trace-black'),'BLACK_EXPANSION_EVIDENCE_DIR':str(e/'lifecycle/black')})
    tests('lifecycle-red','red_time',{'GREEN_EVIDENCE_DIR':str(e/'lifecycle/trace-red'),'RED_TIME_EVIDENCE_DIR':str(e/'lifecycle/red')})
    run('lifecycle-fixture',['python3','tools/fixture-tools/current-four-faction-choices.py',str(e/'lifecycle'),'72','3e2a0cfba34c80b844482ba838776c079acb58c3'])
    tests('death-frontends','death_observer',{'DEATH_OBSERVER_FRONTEND_DIR':str(e/'death')})
    run('current-worker-web-fixtures',['cargo','run','--locked','-p','hegemony-wasm','--example','current_ui_worker_fixtures','--',str(source),str(e/'death'),'72','--web-only'])
    run('seven-card-fixtures',['cargo','run','--locked','-p','hegemony-wasm','--example','seven_card_ui_fixtures','--',str(source/'web/src/game/sevenCardUIV072.fixture.json')])
    tests('win-flow','win_flow',{'WIN_FLOW_EVIDENCE_DIR':str(e/'win')})
    run('fixed-slots-fixture',['python3','tools/fixture-tools/current-win-flow.py',str(e/'win'),'72','--web-only'])
    run('bq030',['cargo','test','--locked','-p','hegemony-server','--test','bq030_attachment_regression','--','--test-threads=1'],{'BQ030_TRACE_DIR':str(e/'bq030/room')})
    run('bq030-fixture',['node','tools/fixture-tools/current-bq030.mjs',str(e/'bq030'),str(source/'rust-game-wasm/pkg/hegemony_wasm.js'),'--web-only'])
    base=root/'evidence'
    tests('jz35','jz35',{'GREEN_EVIDENCE_DIR':str(base/'trace'),'JZ35_EVIDENCE_DIR':str(base/'native'),'JZ35_INVALID_DIR':str(base/'invalid')})
    run('jz35-fixture',['python3','tools/fixture-tools/jz35-current.py',str(base)])
    tests('xq24','xq24_continuous_duel_teams_room_pause_restore_journal_and_receipt_inputs',{'XQ24_TRACE_DIR':str(e/'xq24')})
    rows=[json.loads(p.read_text()) for p in sorted((e/'xq24').glob('*.json'))]
    assert len(rows)==12,len(rows)
    (source/'web/src/game/xq24Native72.fixture.json').write_text(json.dumps({'catalog':rows[0]['catalog'],'scenarios':rows},ensure_ascii=False,separators=(',',':'))+'\n')
    print('native72-input-generation-complete',flush=True)

    if build_and_check:
        run('jz35-parity', ['node', 'rust-game-wasm/tests/jz35_compare.mjs', str(root / 'evidence')])
        for card in ['jc061', 'jc087', 'jc094', 'wm056', 'wm023', 'xq25', 'wm038']:
            run(card + '-parity', ['node', f'rust-game-wasm/tests/{card}_compare.mjs',
                                  str(e / card)])
        run('focused-ui', ['npm', '--prefix', 'web', 'test', '--',
                          'src/game/JZ35Integration.test.jsx', 'src/game/JZ35Face.test.jsx',
                          'src/game/CurrentLifecycleChoices.test.jsx', 'src/game/SevenCardIntegration.test.jsx',
                          '--maxWorkers=1'])
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
        print(f'Native72 generation refused: {error}', file=sys.stderr)
        return 2
    return 0


if __name__ == '__main__':
    sys.exit(main())
