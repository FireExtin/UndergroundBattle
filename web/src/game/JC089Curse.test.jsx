// Persisted native fixture layouts; these do not claim natural browser play.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { kernel } from './testKernel';
import { ReadModal } from './ReadModal';
import { Table } from './Table';
import { DeckLibrary } from './DeckLibraryPanel';
import { createDeckDraft, validateDeckDraft } from './deckLibrary';
import { visibleCard } from './CardTile';

const catalog=JSON.parse(kernel.catalog());
const definitions=new Map(catalog.cards.map(c=>[c.id,c]));
const fixtures=JSON.parse(readFileSync(resolve('src/game/jc089Test.fixture.json'),'utf8')).fixtures;
const view=(kind,seat=0)=>fixtures.find(f=>f.kind===kind).views[seat];
const host=kind=>view(kind).regions.flatMap(r=>r.characters).find(c=>c.cardId==='LC01');
afterEach(cleanup);

it('checks the complete JC089 printed definition',()=>{
 expect(definitions.get('JC089')).toMatchObject({name:'毒血诅咒',kind:'attachment',cost:2,loyalty:['黑色'],magicIcon:'Blood',subtypes:['诅咒'],unique:false,deckCopyLimit:3});
 expect(definitions.get('JC089').text).toContain('永久战斗1和临时战斗1');
});

it('checks recorded stack, sacrifice, replacement and cascade results',()=>{
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

it('validates three curse copies and rejects four while unreviewed JZ51 stays unavailable',()=>{
 const draft={...createDeckDraft(catalog),name:'毒血诅咒有限测试',cards:[{cardId:'JC089',count:3},{cardId:'JC125',count:47}]};
 expect(validateDeckDraft(draft,catalog).valid).toBe(true);
 expect(JSON.parse(kernel.newGameWithDeck('jc089-ui','LOCAL','teams','P0',JSON.stringify(draft),'9')).view.status).toBe('lobby');
 const invalid={...draft,cards:[{cardId:'JC089',count:4},{cardId:'JC125',count:46}]};expect(validateDeckDraft(invalid,catalog).valid).toBe(false);
 expect(()=>kernel.newGameWithDeck('jc089-invalid','LOCAL','teams','P0',JSON.stringify(invalid),'9')).toThrow();
 expect(definitions.has('JZ50')).toBe(true);
 expect(definitions.has('JZ51')).toBe(false);
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
 const removed=view('printed-glory-curse-removed').regions.flatMap(r=>r.characters).find(c=>c.cardId==='JC018');
 expect(removed.currentCombatGlory).toBe(true);expect(removed.currentRenown).toBe(true); // Last loop has independently paid JC074.
});

it('keeps printed glory after paid last-curse removal without a false renown reward',()=>{
 for(const kind of ['last-curse-removed-during-glory','last-curse-removed-after-glory']){
  const v=view(kind),c=v.regions[2].characters.find(c=>c.cardId==='JC018');
  expect(v.attachments).toHaveLength(0);expect(v.regions[2].influence).toEqual([1,0]);
  expect(c.currentCombatGlory).toBe(true);expect(c.currentRenown).toBeUndefined();
 }
 const c=view('uncursed-printed-glory').regions[2].characters.find(c=>c.cardId==='JC018');
 const m=render(<ReadModal card={c} definition={definitions.get('JC018')} viewerId="p0" onClose={vi.fn()}/>);
 expect(screen.getByText('威名')).toBeInTheDocument();expect(screen.queryByText('声望')).not.toBeInTheDocument();m.unmount();
 expect(definitions.get('JC018').ruleTraits.renown).toBeUndefined();
});
