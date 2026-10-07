// Persisted native fixture layouts; these do not claim natural browser play.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import * as prior from '../../../rust-game-wasm/legacy-v0.2.44/hegemony_wasm.js';
import { ReadModal } from './ReadModal';
import { Table } from './Table';
import { DeckLibrary } from './DeckLibraryPanel';
import { createDeckDraft, validateDeckDraft } from './deckLibrary';
import { visibleCard } from './CardTile';

kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
prior.initSync({ module: readFileSync(resolve('../rust-game-wasm/legacy-v0.2.44/hegemony_wasm_bg.wasm')) });
const catalog=JSON.parse(kernel.catalog());
const definitions=new Map(catalog.cards.map(c=>[c.id,c]));
const fixtures=JSON.parse(readFileSync(resolve('src/game/jc089Test.fixture.json'),'utf8')).fixtures;
const view=(kind,seat=0)=>JSON.parse(kernel.view(fixtures.find(f=>f.kind===kind).state,seat));
const host=kind=>view(kind).regions.flatMap(r=>r.characters).find(c=>c.cardId==='LC01');
afterEach(cleanup);

it('admits only JC089 and preserves every previously published card and deck field',()=>{
 const old=JSON.parse(prior.catalog());
 expect(catalog.engineVersion).toBe('rust-v0.2.45-jc089-poison-blood-candidate');
 expect(catalog.cardPoolVersion).toBe('limited-v2.42-jc089-poison-blood-candidate');
 expect(catalog.cards).toHaveLength(103);expect(catalog.cards.filter(c=>c.id!=='JC089')).toEqual(old.cards);
 for(const field of ['world','decks','societies','deckBuildRules'])expect(catalog[field]).toEqual(old[field]);
 expect(definitions.get('JC089')).toMatchObject({name:'毒血诅咒',kind:'attachment',cost:2,loyalty:['黑色'],magicIcon:'Blood',subtypes:['诅咒'],unique:false,deckCopyLimit:3});
 expect(definitions.get('JC089').text).toContain('永久战斗1和临时战斗1');
});

it('restores all four native views through stack, sacrifice response, replacement and cascade',()=>{
 for(const row of fixtures)for(let seat=0;seat<4;seat++)expect(JSON.parse(kernel.view(row.state,seat))).toEqual(row.views[seat]);
 expect(view('glory-sacrifice-response').stack).toHaveLength(2);
 expect(view('glory-response-final').attachments).toHaveLength(0);
 expect(view('glory-response-final').regions[2].influence).toEqual([1,0]);
 expect(view('replacement-region').regions[2].influence).toEqual([0,0]);
 expect(view('jz48-cascade').graveyard.filter(c=>c.cardId==='JZ48')).toHaveLength(2);
 expect(view('glory-score-ten').status).toBe('finished');
});

it('reads current defense and two curse combat bonuses alongside unchanged printed values',()=>{
 const c=host('double-curse');expect(c.defense).toBe(2);expect(c.icons.combat).toBe(4);expect(c.currentCombatGlory).toBe(true);expect(c.currentRenown).toBeUndefined();
 render(<ReadModal card={c} definition={definitions.get('LC01')} viewerId="p0" onClose={vi.fn()}/>);
 expect(screen.getByLabelText('当前有效图标：调查3，战斗4，势力0')).toBeInTheDocument();
 expect(screen.getByLabelText('印刷图标：调查2，战斗0，势力0')).toBeInTheDocument();
 expect(screen.getByText('当前防御 2 · 印刷防御 4')).toBeInTheDocument();
 expect(screen.getByText('威名')).toBeInTheDocument();expect(screen.queryByText('声望')).not.toBeInTheDocument();
});

it('shows permanent combat without initiative and continuous glory on an exhausted host',()=>{
 expect(host('without-initiative').icons.combat).toBe(1);
 const v=view('exhausted-host'),c=host('exhausted-host');expect(c.icons.combat).toBe(0);expect(c.defense).toBe(3);expect(c.currentCombatGlory).toBe(true);
 const m=render(<Table view={v} catalog={catalog} busy={false} onAction={vi.fn()}/>);
 expect(m.container.querySelector(`[data-card-instance="${c.instanceId}"]`)).toHaveAttribute('data-card-exhausted','true');
});

it('keeps curse and stolen host controllers separate and clears glory with the last curse',()=>{
 const v=view('stolen-host'),c=host('stolen-host');expect(c).toMatchObject({owner:'p0',controller:'p2',defense:3,currentCombatGlory:true});
 expect(v.attachments[0]).toMatchObject({owner:'p0',controller:'p0'});
 expect(host('curse-removed')).toMatchObject({defense:4,icons:{combat:0}});expect(host('curse-removed').currentCombatGlory).toBeUndefined();
 expect(view('zero-defense-death').regions.flatMap(r=>r.characters)).toHaveLength(0);
});

it('conceals the granted ability and attributes when an opponent reads a hidden card',()=>{
 const c=view('hidden-host').regions.flatMap(r=>r.characters).find(c=>c.faceDown);
 const e=view('hidden-host',2).regions.flatMap(r=>r.characters).find(x=>x.instanceId===c.instanceId);
 expect(e.cardId).toBeUndefined();expect(e.currentCombatGlory).toBeUndefined();expect(e.defense).toBeUndefined();
 const safe=visibleCard({...c,currentCombatGlory:true,currentRenown:true},'p2');expect(safe.currentCombatGlory).toBeUndefined();expect(safe.currentRenown).toBeUndefined();
 render(<ReadModal card={{...c,currentCombatGlory:true}} definition={definitions.get('LC01')} viewerId="p2" onClose={vi.fn()}/>);
 expect(screen.queryByText('西比尔')).not.toBeInTheDocument();expect(screen.queryByText('威名')).not.toBeInTheDocument();
});

it('offers JC089 through the actual deck library controls',()=>{
 render(<DeckLibrary catalog={catalog}/>);fireEvent.click(screen.getByRole('button',{name:'新建空白牌组'}));
 fireEvent.change(screen.getByRole('searchbox',{name:'检索卡牌'}),{target:{value:'JC089'}});
 expect(screen.getByRole('button',{name:'添加 毒血诅咒（JC089）'})).toBeEnabled();
});

it('validates three curse copies and rejects four without admitting JZ50',()=>{
 const draft={...createDeckDraft(catalog),name:'毒血诅咒有限测试',cards:[{cardId:'JC089',count:3},{cardId:'JC125',count:47}]};
 expect(validateDeckDraft(draft,catalog).valid).toBe(true);
 expect(JSON.parse(kernel.newGameWithDeck('jc089-ui','LOCAL','teams','P0',JSON.stringify(draft),'9')).view.status).toBe('lobby');
 const invalid={...draft,cards:[{cardId:'JC089',count:4},{cardId:'JC125',count:46}]};expect(validateDeckDraft(invalid,catalog).valid).toBe(false);
 expect(()=>kernel.newGameWithDeck('jc089-invalid','LOCAL','teams','P0',JSON.stringify(invalid),'9')).toThrow();
 expect(definitions.has('JZ50')).toBe(false);
});

it('merges a cursed printed glory while preserving separately granted true renown',()=>{
 const v=view('cursed-printed-glory-only'),c=v.regions.flatMap(r=>r.characters).find(c=>c.cardId==='JC018');
 expect(c.currentCombatGlory).toBe(true);expect(c.currentRenown).toBeUndefined();expect(c.defense).toBe(2);expect(c.damage).toBe(1);
 const r=render(<ReadModal card={c} definition={definitions.get('JC018')} viewerId="p0" onClose={vi.fn()}/>);
 expect(screen.getByText('威名')).toBeInTheDocument();expect(screen.queryByText('声望')).not.toBeInTheDocument();r.unmount();
 const both=view('cursed-printed-glory-and-true-renown').regions.flatMap(r=>r.characters).find(c=>c.cardId==='JC018');
 expect(both.currentCombatGlory).toBe(true);expect(both.currentRenown).toBe(true);
 const m=render(<ReadModal card={both} definition={definitions.get('JC018')} viewerId="p0" onClose={vi.fn()}/>);
 expect(screen.getByText('威名')).toBeInTheDocument();expect(screen.getByText('声望')).toBeInTheDocument();m.unmount();
 const removed=view('printed-glory-curse-removed-baseline').regions.flatMap(r=>r.characters).find(c=>c.cardId==='JC018');
 expect(removed.currentCombatGlory).toBeUndefined();expect(removed.currentRenown).toBe(true); // Retained old baseline, not asserted canonically correct.
});
