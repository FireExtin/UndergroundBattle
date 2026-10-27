import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen,fireEvent,within} from '@testing-library/react';
import {kernel as abi,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {Table} from './Table';
import {DeckCardFace} from './DeckCardFace';
import {cardScanUrl} from './cardScans';
import {actionForRoom} from './api';
import {groupObjectActions} from './objectActions';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/core-candidate.json.gz');
afterEach(()=>{cleanup();sessionStorage.clear();});
it.each(['BQ046','BQ055','BQ048','BQ056'])('%s preserves original, public scan, index and closed current definition',id=>{
 const spec=JSON.parse(readFileSync(resolve('../docs/factions/card-specifications.json'))).cards[id];
 const scans=JSON.parse(readFileSync(resolve('public/card-scans.json'))),{imagePath,imageSha256}=spec.sourceVerification;
 const hash=p=>createHash('sha256').update(readFileSync(p)).digest('hex');
 expect(hash(resolve('..',imagePath))).toBe(imageSha256);expect(hash(resolve(`public/cards/${id}.jpg`))).toBe(imageSha256);
 expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:imagePath,sha256:imageSha256});expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);
 const sourceCards=JSON.parse(readFileSync(resolve('../rust-game/data/cards.json')));expect((sourceCards.cards.find(c=>c.id===id).subtitle??null)).toBe(spec.fields.subtitle??null);
 const card=catalog.cards.find(c=>c.id===id);expect(card).toMatchObject({supported:false,name:spec.fields.name,cost:spec.fields.printedCost,permanentIcons:spec.fields.permanentIcons,temporaryIcons:spec.fields.temporaryIcons,...(spec.fields.defense===null?{}:{defense:spec.fields.defense})});
 render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:`${card.name}原始牌面`})).toHaveAttribute('src',`/cards/${id}.jpg`);
});
it('shows the actual parked core outside all regions without inventing a core activation button',()=>{
 const row=fixture.cases.find(c=>c.name==='bq046-prepared-core-true'),view=JSON.parse(abi.view(row.steps.at(-1).expected.state,0));
 const submit=vi.fn();render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
 const zone=screen.getByRole('region',{name:'P0的核心区'});expect(within(zone).getByText('核心区')).toBeInTheDocument();
 expect(view.coreZones[0].card.cardId).toBe('BQ046');expect(view.regions.flatMap(r=>r.characters).some(c=>c.instanceId===view.coreZones[0].card.instanceId)).toBe(false);
 expect(view.legalActions.some(a=>a.abilityId==='bq046-core-forecast')).toBe(false);expect(submit).not.toHaveBeenCalled();
});
const choices=fixture.cases.flatMap(row=>row.steps.filter(s=>s.command.action.action?.kind==='choose').map(step=>({name:row.name+' '+JSON.parse(abi.view(step.state,step.seat)).pendingChoice.kind,step})));
it.each(choices)('$name sends the exact Native choice including a nontrivial forecast partition',({step})=>{
 const view=JSON.parse(abi.view(step.state,step.seat)),wanted=step.command.action.action,submit=vi.fn();render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
 const dialog=screen.getByRole('dialog',{name:'待完成的选择'});
 for(const id of wanted.selected||[]){const option=Array.from(dialog.querySelectorAll('[data-choice-option]')).find(el=>el.getAttribute('data-choice-option')===id);expect(option).toBeDefined();fireEvent.click(option);}
 const orderRow=id=>Array.from(dialog.querySelectorAll('[data-choice-order-option]')).find(el=>el.getAttribute('data-choice-order-option')===id);
 if(wanted.top){for(const id of wanted.bottom){fireEvent.click(within(orderRow(id)).getByRole('button',{name:'置于底'}));}
  // Move the desired first item before the other top item in these three-card fixtures.
  const first=orderRow(wanted.top[0]);const up=within(first).getByRole('button',{name:/上移$/});if(!up.disabled)fireEvent.click(up);
 }
 fireEvent.click(within(dialog).getByRole('button',{name:'确认选择'}));expect(submit).toHaveBeenCalled();expect(actionForRoom(view,submit.mock.calls.at(-1)[0])).toEqual(step.command.action);
 expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
});
const deployments=fixture.cases.flatMap(row=>row.steps.filter(s=>s.command.action.action?.kind==='deploy').map((step,index)=>({name:row.name+' '+index,step})));
it.each(deployments)('$name selects the real core card and submits its exact paid deployment',({step})=>{
 const view=JSON.parse(abi.view(step.state,step.seat)),wanted=step.command.action.action,submit=vi.fn();
 const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
 const legal=view.legalActions.find(a=>a.kind==='deploy'&&a.cardId===wanted.cardId&&a.region===wanted.region);expect(legal).toBeDefined();
 const source=ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`);expect(source).toBeInTheDocument();fireEvent.click(source);
 const group=groupObjectActions(view.legalActions.filter(a=>a.cardId===wanted.cardId)).find(g=>g.actions.some(a=>a.id===legal.id));fireEvent.click(screen.getByRole('button',{name:group.label}));
 const region=view.regions.find(r=>r.index===wanted.region);fireEvent.click(screen.getByRole('button',{name:`查看地区${wanted.region+1} ${region.name}`}));
 const confirm=ui.container.querySelector(`[data-action-kind="deploy"][data-action-region="${wanted.region}"]`);expect(confirm).toHaveAttribute('data-action-id',legal.id);fireEvent.click(confirm);
 expect(submit).toHaveBeenCalledOnce();expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);
 expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
 if(view.coreZones[0].rest===1)expect(screen.getByText('休整 1 · 再派遣额外费用 2')).toBeInTheDocument();
});
const commands=fixture.cases.filter(c=>c.name.startsWith('bq048-command-core-')).map(row=>({name:row.name,step:row.steps[0]}));
it.each(commands)('$name uses the printed paid command mode through the actual card action button',({step})=>{
 const view=JSON.parse(abi.view(step.state,step.seat)),wanted=step.command.action.action,submit=vi.fn();const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
 const legal=view.legalActions.find(a=>a.kind==='play'&&a.cardId===wanted.cardId&&a.option==='core');expect(legal).toBeDefined();
 fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`));const button=Array.from(ui.container.querySelectorAll('[data-action-id]')).find(el=>el.getAttribute('data-action-id')===legal.id);expect(button).toBeEnabled();fireEvent.click(button);
 expect(submit).toHaveBeenCalledOnce();expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
});
it('BQ048 damage chooses a printed mode and real character target through the card UI',()=>{
 const step=fixture.cases.find(c=>c.name==='bq048-command-damage').steps[0];
 const view=JSON.parse(abi.view(step.state,step.seat)),wanted=step.command.action.action,submit=vi.fn();const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
 const legal=view.legalActions.find(a=>a.kind==='play'&&a.cardId===wanted.cardId&&a.option==='damage'&&a.targetId===wanted.targetId);expect(legal).toBeDefined();
 fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`));
 const group=groupObjectActions(view.legalActions.filter(a=>a.cardId===wanted.cardId)).find(g=>g.actions.some(a=>a.id===legal.id));fireEvent.click(screen.getByRole('button',{name:group.label}));
 fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.targetId}"]`));fireEvent.click(screen.getByRole('button',{name:`确认 · ${legal.label}`}));
 expect(submit).toHaveBeenCalledOnce();expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
});
