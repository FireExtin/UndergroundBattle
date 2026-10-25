// New narrow reachability checks, not a rerun of the existing660 UI checks.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {cleanup,fireEvent,render,screen,within} from '@testing-library/react';
import {afterEach,beforeEach,expect,it} from 'vitest';
import {kernel,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {cardScanUrl} from './cardScans';
import {deckCardAdmission,validateDeckDraft} from './deckLibrary';
import {DeckLibrary} from './DeckLibraryPanel';

const gaps=['BQ041','JZ38','JZ40','JZ36','XG16','XG18','XQ21','BQ027','BQ031','BQ032','BQ036','BQ038','WM021','WM022','WM032','WM061','BQ082','XG21','JZ56','XQ20','LC16','WM037','WM044','WM043'];
const next20Wip=['BQ033','BQ034','CQ25','XG25','BQ009'];
const unavailableFaces=[...gaps,...next20Wip];
const scans=JSON.parse(readFileSync(resolve('../web/public/card-scans.json')));
const pool=JSON.parse(readFileSync(resolve('../rust-game/data/cards.json')));
const batch=JSON.parse(readFileSync(resolve('../docs/factions/import20-rulings-2026-10-24.json'))).batch20;
const definition=id=>catalog.cards.find(card=>card.id===id);
const base=()=>({id:'scan-gap-control',name:'索引准入边界对照',description:'synthetic current-catalog test',societyId:null,
  cards:structuredClone(catalog.decks[0].cards),rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,
  engineVersion:catalog.engineVersion,updatedAt:'2026-10-24T00:00:00Z'});
beforeEach(()=>{localStorage.clear();sessionStorage.clear();});
afterEach(()=>{cleanup();localStorage.clear();sessionStorage.clear();});

it('keeps the historical24 and five WIP index gaps outside every supported definition and public preset',()=>{
  expect(catalog.engineVersion).toBe('rust-v0.2.112-shared-keyword-rewards');
  const actual=catalog.cards.filter(card=>!scans[card.id]||cardScanUrl(card.id)!==`/cards/${card.id}.jpg`).map(card=>card.id);
  expect(actual).toEqual(unavailableFaces);
  expect(batch).toHaveLength(20);
  expect(batch.filter(id=>gaps.includes(id))).toEqual([]);
  expect(catalog.cards.filter(card=>card.supported)).toHaveLength(233);
  expect(unavailableFaces.every(id=>definition(id).supported===false)).toBe(true);
  expect(catalog.cards.filter(card=>card.supported).every(card=>scans[card.id]&&cardScanUrl(card.id)===scans[card.id].url)).toBe(true);
  for(const deck of catalog.decks)expect(deck.cards.filter(entry=>unavailableFaces.includes(entry.cardId))).toEqual([]);
  expect(pool.world).toHaveLength(10); //The server deliberately omits world from the public catalog.
  expect(pool.world.filter(entry=>gaps.includes(entry.cardId))).toEqual([]);
});

it('rejects all29 unavailable faces through client validation and actual WASM create/join',()=>{
  const control=base();expect(validateDeckDraft(control,catalog).valid).toBe(true);
  const created=JSON.parse(kernel.newGameWithDeck('scan-gap-control','ABCDEF123456','duel','owner',JSON.stringify(control),'9007199254740993'));
  expect(typeof created.state).toBe('string');
  const joined=JSON.parse(kernel.joinGameWithDeck(created.state,'guest',JSON.stringify(control)));
  expect(typeof joined.state).toBe('string');
  const original=created.state;
  for(const id of unavailableFaces){
    const card=definition(id),draft=base();draft.cards.push({cardId:id,count:1});
    expect(deckCardAdmission(card).status,id).toBe('unavailable');
    expect(validateDeckDraft(draft,catalog).issues,id).toContainEqual(expect.objectContaining({code:'unsupported',cardId:id}));
    const reject=error=>String(error).includes(`${card.name}不能加入当前玩家卡组`);
    assert.throws(()=>kernel.newGameWithDeck('scan-gap-reject','ABCDEF123456','duel','owner',JSON.stringify(draft),'9007199254740993'),reject,id);
    assert.throws(()=>kernel.joinGameWithDeck(original,'guest',JSON.stringify(draft)),reject,id);
    expect(created.state).toBe(original);
  }
});

it('shows no gap in the default buildable directory and explicit unavailable placeholders under all',()=>{
  const ui=render(<DeckLibrary catalog={catalog} storage={null} pageMode/>);
  const ids=()=>[...ui.container.querySelectorAll('[data-catalog-card]')].map(row=>row.dataset.catalogCard);
  expect(ids()).toHaveLength(223); //233 supported minus the10 world regions.
  expect(ids().filter(id=>unavailableFaces.includes(id))).toEqual([]);
  expect(batch.every(id=>ids().includes(id))).toBe(true);
  fireEvent.change(screen.getByLabelText('组牌准入'),{target:{value:'all'}});
  expect(ids()).toHaveLength(277);
  for(const id of unavailableFaces){
    const card=definition(id),row=ui.container.querySelector(`[data-catalog-card="${id}"]`);
    expect(row).toHaveAttribute('data-card-admission','unavailable');
    expect(row.querySelector('[data-deck-face]')).toHaveAttribute('data-face-state','missing');
    expect(within(row).getByRole('button',{name:`添加 ${card.name}（${id}）`})).toBeDisabled();
  }
  const card=definition(gaps[0]);fireEvent.click(screen.getByRole('button',{name:`预览 ${card.name}（${card.id}）`}));
  const dialog=screen.getByRole('dialog',{name:`卡面预览 ${card.name}（${card.id}）`});
  expect(within(dialog).getByText('暂无原始牌面')).toBeInTheDocument();
  expect(within(dialog).getByText(/卡牌准入：暂不可用/)).toBeInTheDocument();
});
