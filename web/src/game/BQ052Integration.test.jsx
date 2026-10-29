import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen,fireEvent,within} from '@testing-library/react';
import {kernel as abi,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {Table} from './Table';
import {CardContent} from './CardTile';
import {actionForRoom} from './api';
import {validateDeckDraft} from './deckLibrary';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/bq052.json.gz');
afterEach(()=>{cleanup();sessionStorage.clear();});
it('keeps book and generated token closed while retaining the real 50+1 input',()=>{
 const scenario=fixture.cases[0],draft=scenario.construction.deck;
 expect(scenario.construction).toMatchObject({ordinaryCards:50,separateCore:1,minimumCommandCards:12,generatedTentacles:3,noLayoutOrResourceMutation:true});
 expect(catalog.cards.find(c=>c.id==='BQ052')).toMatchObject({color:'红',supported:false,cost:2,unique:true});
 expect(catalog.cards.find(c=>c.id==='TK010')).toMatchObject({color:'红',supported:false,subtypes:['突变体','器官']});
 expect(validateDeckDraft(draft,catalog).valid).toBe(false);
 const candidate={...catalog,cards:catalog.cards.map(c=>['BQ046','BQ048','BQ050','BQ063','BQ052'].includes(c.id)?{...c,supported:true}:c),societies:catalog.societies.map(s=>s.id==='MSBQ02'?{...s,supported:true}:s)};
 expect(validateDeckDraft(draft,candidate).valid).toBe(true);
 expect(draft.cards.some(c=>c.cardId==='TK010')).toBe(false);
});
it('submits all three actual sacrifice instances from the book action',()=>{
 const step=fixture.cases[0].steps.find(s=>s.command.action.action?.abilityId==='bq052-sacrifice-three-place-core');expect(step).toBeDefined();
 const view=JSON.parse(abi.view(step.state,step.seat)),wanted=step.command.action.action,submit=vi.fn();
 expect(wanted.costSelected).toHaveLength(3);expect(new Set(wanted.costSelected).size).toBe(3);
 const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
 const legal=view.legalActions.find(a=>a.abilityId===wanted.abilityId&&JSON.stringify(a.costSelected)===JSON.stringify(wanted.costSelected));expect(legal).toBeDefined();
 const card=ui.container.querySelector(`[data-attachment-instance="${wanted.cardId}"]`)||ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`);expect(card).not.toBeNull();fireEvent.click(card);
 const button=Array.from(ui.container.querySelectorAll('[data-action-id]')).find(el=>el.getAttribute('data-action-id')===legal.id);expect(button).toBeDefined();fireEvent.click(button);
 expect(submit).toHaveBeenCalledOnce();expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);
 expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
});
it('selects each real preparation trigger without creating tokens through UI shortcuts',()=>{
 let n=0;
 for(const step of fixture.cases[0].steps){
  if(step.command.action.action?.kind!=='choose')continue;
  const view=JSON.parse(abi.view(step.state,step.seat));
  if(view.pendingChoice?.kind!=='jc035-preparation-order'||!view.pendingChoice.options.some(o=>o.card?.cardId==='BQ052'))continue;
  const submit=vi.fn();render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
  const dialog=screen.getByRole('dialog',{name:'待完成的选择'});
  for(const id of step.command.action.action.selected){fireEvent.click(Array.from(dialog.querySelectorAll('[data-choice-option]')).find(el=>el.getAttribute('data-choice-option')===id));}
  fireEvent.click(within(dialog).getByRole('button',{name:/确认/}));expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);n++;cleanup();
 }
 expect(n).toBe(3);
});

it('shows a generated tentacle as absent printed cost, never a printed zero',()=>{
 const steps=fixture.cases[0].steps;
 const view=steps.map(s=>JSON.parse(abi.view(s.expected.state,0))).find(v=>v.regions.some(r=>r.characters.some(c=>c.cardId==='TK010'&&!c.faceDown)));
 const card=view.regions.flatMap(r=>r.characters).find(c=>c.cardId==='TK010'&&!c.faceDown);
 expect(card.cost).toBeUndefined();expect(card.effectiveCost).toBeUndefined();expect(card.costAppearance).toBe('absent');
 render(<CardContent card={card} definition={catalog.cards.find(c=>c.id==='TK010')}/>);
 expect(screen.getByLabelText('无印刷费用')).toHaveTextContent('无');
 expect(screen.queryByLabelText('费用 0')).toBeNull();
});

it('omits ordinary reveal and rejects a direct command using the real Native negative case',()=>{
 const c=loadCurrentFixture('rust-game-wasm/tests/current113/tentacle.json.gz').cases[0].paidRevealRejection;
 const view=JSON.parse(abi.view(c.state,c.seat));
 expect(view.legalActions.some(a=>a.kind==='reveal'&&a.cardId===c.command.action.action.cardId)).toBe(false);
 expect(JSON.parse(abi.applyRoom(c.state,c.seat,JSON.stringify(c.command),c.serverNowMs))).toEqual(c.expected);
});

it('shows sealed tentacle disappearance instead of the ordinary return-to-hand hint',()=>{
 const scenario=loadCurrentFixture('rust-game-wasm/tests/current113/tentacle.json.gz').cases.find(c=>c.name==='tentacle-prepared-seal-and-host-hide');
 const view=scenario.steps.map(s=>JSON.parse(abi.view(s.expected.state,0))).find(v=>(v.sealedCards??[]).some(s=>s.cardId==='TK010'));
 const card=view.sealedCards.find(s=>s.cardId==='TK010');
 render(<CardContent card={card} definition={catalog.cards.find(c=>c.id==='TK010')}/>);
 expect(screen.getByText(/载体离场或翻暗时触手消失/)).toBeDefined();
 expect(screen.queryByText(/回到拥有者手牌/)).toBeNull();
 expect(screen.queryByLabelText('费用 0')).toBeNull();
});
