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
const newlyAdmitted=['BQ034','CQ25','XG25'];
const closedCandidates=["BQ045", "BQ052", "XQ35", "BQ084", "CQ10", "CQ09", "BQ086", "BQ046", "BQ055", "BQ048", "BQ056", "BQ050", "BQ051", "TEST034", "BQ063"];
const scans=JSON.parse(read('web/public/card-scans.json'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const canonical=value=>Array.isArray(value)?value.map(canonical):value&&typeof value==='object'
  ?Object.fromEntries(Object.keys(value).sort().map(key=>[key,canonical(value[key])])):value;
afterEach(cleanup);

it('preserves all241 prior source-index records and all239 prior runtime URLs',()=>{
  const prior=Object.fromEntries(Object.entries(scans).filter(([id])=>!batch.includes(id)&&!newlyAdmitted.includes(id)&&!closedCandidates.includes(id)&&id!=="TK010"&&!["MSBQ02","MSJC12","MSWM02","MSCQ02","MSBQ03"].includes(id)));
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

it.each(newlyAdmitted)('%s delivers the tracked original scan through both indexes and the card component',id=>{
  const card=registry.cards.find(card=>card.id===id);
  const {imagePath:source,imageSha256:expected}=primary.cards[id].sourceVerification;
  expect(typeof expected).toBe('string');
  expect(sha(read(source))).toBe(expected);
  expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(expected);
  expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source,sha256:expected});
  expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);
  expect(card.supported).toBe(true);
  render(<DeckCardFace card={card} large/>);
  expect(screen.getByRole('img',{name:`${card.name}原始牌面`})).toHaveAttribute('src',`/cards/${id}.jpg`);
  expect(screen.queryByText('暂无原始牌面')).not.toBeInTheDocument();
});

it('keeps the current kernel and reviewed admission set while delivering artwork',()=>{
  expect(registry.engineVersion).toBe('rust-v0.2.113-bq001-anchor-source-wip');
  expect(currentCatalog.engineVersion).toBe('rust-v0.2.113-bq001-anchor-source-wip');
  expect(currentCatalog.cards.map(card=>[card.id,card.supported])).toEqual(registry.cards.map(card=>[card.id,card.supported!==false]));
  expect(registry.cards).toHaveLength(294);
  for(const id of ['BQ033','BQ009','BQ001',...closedCandidates])expect(registry.cards.find(card=>card.id===id).supported,id).toBe(false);
  for(const id of ['BQ034','CQ25','XG25'])expect(registry.cards.find(card=>card.id===id).supported,id).toBe(true);
  expect(registry.cards.filter(card=>card.supported!==false)).toHaveLength(236);
  expect(registry.cards.slice(213).filter(card=>card.supported).map(card=>card.id).sort()).toEqual([...batch,'BQ034','CQ25','XG25'].sort());
  expect(cardScanUrl('unknown-private-card')).toBeUndefined();
});

it.each(["MSJC12","MSWM02","MSCQ02","MSBQ03"])("%s preserves pinned original and remains a closed society",id=>{const source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);const card=currentCatalog.societies.find(c=>c.id===id);expect(card.supported).toBe(false);render(<DeckCardFace card={card} large/>);expect(screen.getByRole("img",{name:`${card.name}原始牌面`})).toHaveAttribute("src",`/cards/${id}.jpg`);});

it('TK010 preserves its original PNG and stays outside admission',()=>{const id='TK010',card=registry.cards.find(c=>c.id===id),source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read('web/public/cards/TK010.png'))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:'/cards/TK010.png',source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/TK010.png');expect(card.supported).toBe(false);expect(card.subtypes).toEqual(['突变体','器官']);render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:'触手原始牌面'})).toHaveAttribute('src','/cards/TK010.png');});

it('BQ052 original book artwork, loyalty and closed status stay aligned',()=>{const id='BQ052',card=registry.cards.find(c=>c.id===id),source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read('web/public/cards/BQ052.jpg'))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:'/cards/BQ052.jpg',source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/BQ052.jpg');expect(card.supported).toBe(false);expect(card.loyalty).toEqual(['红色']);expect(card.unique).toBe(true);render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src','/cards/BQ052.jpg');});

it('BQ045 keeps the original core cube separate from its single gray loyalty',()=>{const id='BQ045',card=registry.cards.find(c=>c.id===id),source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read('web/public/cards/BQ045.jpg'))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:'/cards/BQ045.jpg',source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/BQ045.jpg');expect(card.supported).toBe(false);expect(card.loyalty).toEqual(['灰色']);expect(card.subtitle).toBe('修格斯');expect(card.cost).toBe(3);render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src','/cards/BQ045.jpg');});
