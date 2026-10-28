import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen,fireEvent} from '@testing-library/react';
import {kernel as abi,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {Table} from './Table';
import {actionForRoom} from './api';
import {groupObjectActions} from './objectActions';
import {validateDeckDraft} from './deckLibrary';
import {cardScanUrl} from './cardScans';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/test034.json.gz');
afterEach(()=>{cleanup();sessionStorage.clear();});
it('pins the blue TEST image without granting formal admission',()=>{
 const definition=catalog.cards.find(c=>c.id==='TEST034');
 expect(definition).toMatchObject({name:'兽性直觉',kind:'attachment',color:'蓝',cost:1,loyalty:['蓝色'],magic:'心灵',unique:false,supported:false,subtypes:['状态','超能力'],keywords:['非资产']});
 const source='resource/ymsj-fun.github.io/cards/TEST034 兽性直觉.jpg',original=readFileSync(resolve('..',source));
 const sha256='51b211a96598671a2fc92dd3192dac1fb146a1bdfdde2e219febe8e7754b0ee5';
 expect(createHash('sha256').update(original).digest('hex')).toBe(sha256);
 expect(readFileSync(resolve('public/cards/TEST034.jpg')).equals(original)).toBe(true);
 expect(JSON.parse(readFileSync(resolve('public/card-scans.json'))).TEST034).toEqual({url:'/cards/TEST034.jpg',source,sha256});
 expect(cardScanUrl('TEST034')).toBe('/cards/TEST034.jpg');
 for(const row of fixture.cases){expect(validateDeckDraft(row.construction.deck,catalog).valid).toBe(false);expect(validateDeckDraft(row.construction.deck,{...catalog,cards:catalog.cards.map(c=>c.id==='TEST034'?{...c,supported:true}:c)}).valid).toBe(true);}
});
it.each(fixture.cases)('$name submits the actual natural Native action after restored view',row=>{
 const step=row.steps.find(s=>{const a=s.command.action.action;if(!a||!['play','activate'].includes(a.kind))return false;const v=JSON.parse(abi.view(s.state,s.seat));return v.hand.some(c=>c.instanceId===a.cardId&&c.cardId==='TEST034');});
 expect(step).toBeDefined();const view=JSON.parse(abi.view(step.state,step.seat)),wanted=step.command.action.action,submit=vi.fn();
 const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
 const legal=view.legalActions.find(a=>a.kind===wanted.kind&&a.cardId===wanted.cardId&&a.abilityId===wanted.abilityId&&a.targetId===wanted.targetId);
 expect(legal).toBeDefined();fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`));
 const group=groupObjectActions(view.legalActions.filter(a=>a.cardId===wanted.cardId)).find(g=>g.actions.some(a=>a.id===legal.id));
 fireEvent.click(screen.getByRole('button',{name:group.label}));
 if(wanted.targetId){fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.targetId}"]`));fireEvent.click(screen.getByRole('button',{name:`确认 · ${legal.label}`}));}
 expect(submit).toHaveBeenCalledOnce();expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);
 expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
});
