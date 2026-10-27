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
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/core-candidate.json.gz');
afterEach(()=>{cleanup();sessionStorage.clear();});
it.each(['BQ046','BQ055'])('%s preserves original, public scan, index and closed current definition',id=>{
 const spec=JSON.parse(readFileSync(resolve('../docs/factions/card-specifications.json'))).cards[id];
 const scans=JSON.parse(readFileSync(resolve('public/card-scans.json'))),{imagePath,imageSha256}=spec.sourceVerification;
 const hash=p=>createHash('sha256').update(readFileSync(p)).digest('hex');
 expect(hash(resolve('..',imagePath))).toBe(imageSha256);expect(hash(resolve(`public/cards/${id}.jpg`))).toBe(imageSha256);
 expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:imagePath,sha256:imageSha256});expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);
 const sourceCards=JSON.parse(readFileSync(resolve('../rust-game/data/cards.json')));expect(sourceCards.cards.find(c=>c.id===id).subtitle).toBe(spec.fields.subtitle);
 const card=catalog.cards.find(c=>c.id===id);expect(card).toMatchObject({supported:false,name:spec.fields.name,cost:spec.fields.printedCost,permanentIcons:spec.fields.permanentIcons,temporaryIcons:spec.fields.temporaryIcons,defense:spec.fields.defense});
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
