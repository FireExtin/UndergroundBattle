import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {cleanup,render,screen} from '@testing-library/react';
import {afterEach,expect,it} from 'vitest';
import {DeckCardFace} from './DeckCardFace';
import {cardScanUrl} from './cardScans';
import {catalog as currentCatalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';

const read=path=>readFileSync(resolve('..',path));
const registry=JSON.parse(read('rust-game/data/cards.json'));
const primary=JSON.parse(read('docs/factions/card-specifications.json'));
const batch=JSON.parse(read('docs/factions/import20-rulings-2026-10-24.json')).batch20;
const scans=JSON.parse(read('web/public/card-scans.json'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const canonical=value=>Array.isArray(value)?value.map(canonical):value&&typeof value==='object'
  ?Object.fromEntries(Object.keys(value).sort().map(key=>[key,canonical(value[key])])):value;
afterEach(cleanup);

it('preserves all241 prior source-index records and all239 prior runtime URLs',()=>{
  const prior=Object.fromEntries(Object.entries(scans).filter(([id])=>!batch.includes(id)));
  expect(Object.keys(prior)).toHaveLength(241);
  expect(sha(JSON.stringify(canonical(prior)))).toBe('8391a760ca9b2d25d1767b53ebb2f092eb3f106f2d326b924a6eaa43e1de79b8');
  const runtime=Object.fromEntries(Object.keys(prior).filter(id=>!['BQ041','JZ38'].includes(id)).map(id=>[id,cardScanUrl(id)]));
  expect(Object.keys(runtime)).toHaveLength(239);
  expect(sha(JSON.stringify(canonical(runtime)))).toBe('b72fae2c401492c362b759fe3fbfdc36197147db936168edd048b90272b64d42');
});

it.each(batch)('%s displays its pinned original pixels through the real collection-card component',id=>{
  const card=registry.cards.find(card=>card.id===id),source=primary.cards[id].sourceVerification.imagePath;
  const expected=card.source.imageSha256;
  expect(sha(read(source))).toBe(expected);
  expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(expected);
  expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source,sha256:expected});
  expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);
  render(<DeckCardFace card={card} large/>);
  expect(screen.getByRole('img',{name:`${card.name}原始牌面`})).toHaveAttribute('src',`/cards/${id}.jpg`);
  expect(screen.queryByText('暂无原始牌面')).not.toBeInTheDocument();
});

it('does not admit any additional card or replace the current kernel while adding artwork',()=>{
  expect(registry.engineVersion).toBe('rust-v0.2.112-shared-keyword-rewards');
  expect(currentCatalog.engineVersion).toBe('rust-v0.2.112-shared-keyword-rewards');
  expect(currentCatalog.cards.map(card=>[card.id,card.supported])).toEqual(registry.cards.map(card=>[card.id,card.supported!==false]));
  expect(registry.cards).toHaveLength(272);
  expect(registry.cards.filter(card=>card.supported!==false)).toHaveLength(233);
  expect(registry.cards.slice(213).filter(card=>card.supported).map(card=>card.id).sort()).toEqual([...batch].sort());
  expect(cardScanUrl('unknown-private-card')).toBeUndefined();
});
