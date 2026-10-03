#!/usr/bin/env python3
"""Read-only source, frozen-kernel, preset and build-byte audit for the local v028 candidate."""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CARD_BASE = '9413068a944529032a686b39b77dc88890c657a1'
UI_BASE = '13123619e315f986d1d9934046ee76338e5327aa'
SITE_BASE = '074def9eb5bdf49fe6a5df5ef18ebd70bbce5453'
ADDED = ['BQ083', 'JC001', 'JC006', 'JC007', 'JC047', 'JC075', 'JC088', 'JC104', 'XQ16']

def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)

def digest(data):
    return hashlib.sha256(data).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('published_checkout', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    site = args.published_checkout.resolve()
    # Read the fixed published baseline even after this checkout advances.
    subprocess.run(['git', 'cat-file', '-e', SITE_BASE + '^{commit}'], cwd=site, check=True)
    def published_file(name):
        return subprocess.check_output(['git', 'show', SITE_BASE + ':' + name], cwd=site)
    for name in ('hegemony_wasm.js', 'hegemony_wasm_bg.wasm'):
        assert (ROOT / 'rust-game-wasm/legacy-v0.2.7' / name).read_bytes() == published_file('rust-game-wasm/pkg/' + name)
    for base in (CARD_BASE, UI_BASE):
        subprocess.run(['git', 'merge-base', '--is-ancestor', base, 'HEAD'], cwd=ROOT, check=True)

    # Inspect real WASM catalogs, never manually reconstructed supported flags.
    js = r'''
import {readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
const root=process.cwd(), published=process.argv[2];
const load=async dir=>{const k=await import(pathToFileURL(resolve(dir,'hegemony_wasm.js')));k.initSync({module:readFileSync(resolve(dir,'hegemony_wasm_bg.wasm'))});return JSON.parse(k.catalog());};
const current=await load(resolve(root,'rust-game-wasm/pkg'));
// Its bytes were first proved equal to the fixed Site15 Git objects in Python.
const old=await load(resolve(root,'rust-game-wasm/legacy-v0.2.7'));
const frozen=[];
for(let i=1;i<=7;i++) frozen.push(await load(resolve(root,`rust-game-wasm/legacy-v0.2.${i}`)));
console.log(JSON.stringify({current,published:old,frozen}));
'''
    catalogs = json.loads(subprocess.check_output(['node', '--input-type=module', '-', str(site)], input=js.encode(), cwd=ROOT))
    current, published = catalogs['current'], catalogs['published']
    assert (current['rulesVersion'], current['cardPoolVersion'], current['engineVersion']) == ('hegemony-pdf-v1', 'limited-v2.6', 'rust-v0.2.8')
    assert len(current['cards']) == 48 and len(published['cards']) == 39
    assert sorted(set(c['id'] for c in current['cards']) - set(c['id'] for c in published['cards'])) == ADDED
    assert catalogs['frozen'][-1] == published
    assert [c['engineVersion'] for c in catalogs['frozen']] == [f'rust-v0.2.{i}' for i in range(1, 8)]

    frozen_files = []
    for path in sorted((ROOT / 'rust-game-wasm').glob('legacy-v0.2.*/*')):
        if not path.is_file():
            continue
        relative = path.relative_to(ROOT).as_posix()
        data = path.read_bytes()
        assert data == git('show', f'{CARD_BASE}:{relative}'), relative
        deployed = site / 'rust-game-wasm' / path.relative_to(ROOT / 'rust-game-wasm')
        if path.parent.name == 'legacy-v0.2.7' and path.name in {'hegemony_wasm.js', 'hegemony_wasm_bg.wasm'}:
            deployed = site / 'rust-game-wasm/pkg' / path.name
        compared = subprocess.run(['git', 'cat-file', '-e', SITE_BASE + ':' + deployed.relative_to(site).as_posix()], cwd=site, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0
        if compared:
            assert data == published_file(deployed.relative_to(site).as_posix()), relative
        frozen_files.append({'file': relative, 'bytes': len(data), 'sha256': digest(data), 'unchangedFromCardChain': True, 'comparedToPublishedSite15': compared})
    assert len([r for r in frozen_files if r['file'].endswith('.wasm')]) == 7

    unchanged_card_files = []
    for name in git('ls-tree', '-r', '--name-only', CARD_BASE, '--', 'rust-game/src', 'rust-game/data/cards.json', 'rust-game-wasm/src').decode().splitlines():
        data = (ROOT / name).read_bytes()
        assert data == git('show', f'{CARD_BASE}:{name}'), name
        unchanged_card_files.append({'file': name, 'sha256': digest(data)})
    ui_files = []
    for name in git('diff', '--name-only', '37a09a2', UI_BASE, '--', 'web/src').decode().splitlines():
        if '.test.' in name:
            continue
        data = (ROOT / name).read_bytes()
        assert data == git('show', f'{UI_BASE}:{name}'), name
        assert data == published_file(name), name
        ui_files.append({'file': name, 'sha256': digest(data)})
    persistent_files = []
    for name in git('ls-tree', '-r', '--name-only', UI_BASE, '--', 'sites/src', 'sites/db', 'sites/drizzle', 'sites/.openai/hosting.json', 'sites/wrangler.jsonc').decode().splitlines():
        if name == 'sites/src/kernel.mjs':
            continue
        data = (ROOT / name).read_bytes()
        assert data == git('show', f'{UI_BASE}:{name}'), name
        deployed = site / name.removeprefix('sites/')
        assert data == published_file(deployed.relative_to(site).as_posix()), name
        persistent_files.append({'file': name, 'sha256': digest(data)})

    old_decks = {d['id']: d for d in published['decks']}
    changed_decks = [d['id'] for d in current['decks'] if d != old_decks[d['id']]]
    assert changed_decks == ['reclaimers']
    reclaimers = next(d for d in current['decks'] if d['id'] == 'reclaimers')
    old_reclaimers = old_decks['reclaimers']
    assert {k: v for k, v in reclaimers.items() if k != 'cards'} == {k: v for k, v in old_reclaimers.items() if k != 'cards'}
    new_cards = {e['cardId']: e['count'] for e in reclaimers['cards']}
    old_cards = {e['cardId']: e['count'] for e in old_reclaimers['cards']}
    assert new_cards['JC125'] == 8 and old_cards['JC125'] == 11 and new_cards['JC088'] == 3 and 'JC088' not in old_cards
    assert {k: v for k, v in new_cards.items() if k not in {'JC125', 'JC088'}} == {k: v for k, v in old_cards.items() if k != 'JC125'}

    raw = json.loads((ROOT / 'rust-game/data/cards.json').read_text())['cards']
    images = []
    for cid in ADDED:
        definition = next(c for c in raw if c['id'] == cid)
        original = ROOT / (definition['source'].get('image') or definition['source']['file'])
        web = ROOT / f'web/public/cards/{cid}.jpg'
        built = ROOT / f'sites/dist/client/cards/{cid}.jpg'
        assert original.read_bytes() == web.read_bytes() == built.read_bytes(), cid
        images.append({'id': cid, 'original': original.relative_to(ROOT).as_posix(), 'sha256': digest(original.read_bytes()), 'webAndBuiltBytesEqualOriginal': True})
    expected_wasm = {digest((ROOT / 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes())}
    expected_wasm.update(row['sha256'] for row in frozen_files if row['file'].endswith('.wasm'))
    built_wasm = [{'file': p.relative_to(ROOT).as_posix(), 'bytes': p.stat().st_size, 'sha256': digest(p.read_bytes())} for p in sorted((ROOT / 'sites/dist/server').glob('*.wasm'))]
    assert len(built_wasm) == 8 and {r['sha256'] for r in built_wasm} == expected_wasm

    result = {'sourceCommitAtAudit': git('rev-parse', 'HEAD').decode().strip(), 'cardParent': CARD_BASE, 'publishedUiParent': UI_BASE, 'publishedSiteSource': SITE_BASE,
        'scope': 'Local read-only audit; no external Site/room/database access, push, save-version or deployment.',
        'versions': {k: current[k] for k in ('rulesVersion', 'cardPoolVersion', 'engineVersion')},
        'catalogCounts': {'candidate': 48, 'published': 39, 'added': 9, 'candidatePlayerCards': sum(c['kind'] != 'region' for c in current['cards'])},
        'addedIds': ADDED, 'frozenKernelCount': 7, 'totalKernelCount': 8,
        'frozenCatalogs': [{'rules': c['rulesVersion'], 'pool': c['cardPoolVersion'], 'engine': c['engineVersion'], 'cards': len(c['cards']), 'decks': c['decks']} for c in catalogs['frozen']],
        'frozenFiles': frozen_files, 'cardChainFilesUnchanged': unchanged_card_files, 'publishedUiFilesUnchanged': ui_files, 'persistentServiceFilesUnchanged': persistent_files,
        'presetDifferenceFromPublishedV027': {'unchanged': [d['id'] for d in current['decks'] if d['id'] != 'reclaimers'], 'changedOnly': changed_decks, 'reclaimersBefore': old_reclaimers, 'reclaimersAfter': reclaimers},
        'rawImageEntryChecks': images, 'currentWasm': {'bytes': (ROOT / 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm').stat().st_size, 'sha256': digest((ROOT / 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes())},
        'builtWasmModules': built_wasm, 'rulingBoundary': 'JC016+BQ022 remains individually admitted and jointly legal/reachable; Win retreat/recycling priority pending. No ban or priority change.',
        'restorationBoundary': 'Default real Worker/D1 local regression restores v021-v027 catalogs, opaque states and receipts after workerd reopen; no public historical room acceptance claimed.'}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'catalogCounts': result['catalogCounts'], 'totalKernels': 8, 'unchangedUiFiles': len(ui_files), 'unchangedPersistenceFiles': len(persistent_files), 'wasm': result['currentWasm']}, ensure_ascii=False))

if __name__ == '__main__':
    main()
