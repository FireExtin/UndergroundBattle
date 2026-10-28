import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen,fireEvent,within} from '@testing-library/react';
import {kernel as abi,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {Table} from './Table';
import {actionForRoom} from './api';
import {validateDeckDraft} from './deckLibrary';
import {cardScanUrl} from './cardScans';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/bq063.json.gz');
afterEach(()=>{cleanup();sessionStorage.clear();});
it('keeps the formal gray command closed and delivers its exact original image',()=>{
 const definition=catalog.cards.find(c=>c.id==='BQ063');expect(definition).toMatchObject({name:'神话威严',kind:'spell',color:'灰',cost:3,loyalty:['灰色'],subtypes:['命令','精神'],supported:false});
 const source='resource/ymsj-fun.github.io/cards/BQ063 神话威严.jpg',original=readFileSync(resolve('..',source)),sha256='1d9702f3d600ce4c7c1f50776a3a008c15b7b5d34706be8bd8ac2bef42246c98';
 expect(createHash('sha256').update(original).digest('hex')).toBe(sha256);expect(readFileSync(resolve('public/cards/BQ063.jpg')).equals(original)).toBe(true);expect(JSON.parse(readFileSync(resolve('public/card-scans.json'))).BQ063).toEqual({url:'/cards/BQ063.jpg',source,sha256});expect(cardScanUrl('BQ063')).toBe('/cards/BQ063.jpg');
 const draft=fixture.cases[0].construction.deck;expect(validateDeckDraft(draft,catalog).valid).toBe(false);const candidate={...catalog,cards:catalog.cards.map(c=>['BQ046','BQ048','BQ050','BQ063'].includes(c.id)?{...c,supported:true}:c),societies:catalog.societies.map(s=>s.id==='MSBQ02'?{...s,supported:true}:s)};expect(validateDeckDraft(draft,candidate).valid).toBe(true);expect(draft.cards.reduce((n,e)=>n+e.count,0)).toBe(50);expect(draft.coreCardId).toBe('BQ046');
});
it.each(fixture.cases)('$name selects the exact actual command mode from the card',row=>{
 const step=row.steps.find(s=>s.command.action.action?.abilityId==='bq063-command');expect(step).toBeDefined();const view=JSON.parse(abi.view(step.state,step.seat)),wanted=step.command.action.action,submit=vi.fn();const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`));const legal=view.legalActions.find(a=>a.cardId===wanted.cardId&&a.abilityId===wanted.abilityId&&a.option===wanted.option);expect(legal).toBeDefined();const button=Array.from(ui.container.querySelectorAll('[data-action-id]')).find(el=>el.getAttribute('data-action-id')===legal.id);expect(button).toBeEnabled();fireEvent.click(button);expect(submit).toHaveBeenCalledOnce();expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
});
it('keeps the actual two forecast choices ordered before repression',()=>{
 const choices=fixture.cases[0].steps.filter(s=>s.command.action.action?.kind==='choose');const forecasts=choices.filter(s=>s.command.action.action.top!==undefined);expect(forecasts).toHaveLength(2);
 for(const step of forecasts){const view=JSON.parse(abi.view(step.state,step.seat)),submit=vi.fn();render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);const dialog=screen.getByRole('dialog',{name:'待完成的选择'});expect(view.pendingChoice.kind).toBe('investigation');expect(dialog).not.toHaveTextContent('选择遏制');fireEvent.click(within(dialog).getByRole('button',{name:'确认选择'}));expect(submit).toHaveBeenCalledOnce();expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);cleanup();}
 const last=choices.at(-1);expect(JSON.parse(abi.view(last.state,last.seat)).pendingChoice.kind).toBe('bq063-opponent');
});
it('submits both stages of repression using the actual actor and enemy options',()=>{
 const steps=fixture.cases[1].steps.filter(s=>s.command.action.action?.kind==='choose');expect(steps).toHaveLength(2);
 for(const step of steps){const view=JSON.parse(abi.view(step.state,step.seat)),submit=vi.fn();render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);const dialog=screen.getByRole('dialog',{name:'待完成的选择'});for(const id of step.command.action.action.selected){fireEvent.click(dialog.querySelector(`[data-choice-option="${id}"]`));}fireEvent.click(within(dialog).getByRole('button',{name:/确认/}));expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);cleanup();}
});
