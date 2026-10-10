// Exact Native inputs through existing React entry controls and the current WASM.
import {cleanup,fireEvent,render,screen} from '@testing-library/react';
import {afterEach,expect,it,vi} from 'vitest';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import * as wasm from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import native from './xq37Native59Test.fixture.json';
import prior56 from './xq27Native56Test.fixture.json';
import {Table} from './Table';
import {ResponseWindow} from './ResponseWindow';
import {ReadModal} from './ReadModal';
import {cardScanUrl} from './cardScans';
import {createDeckDraft,validateDeckDraft} from './deckLibrary';
wasm.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(wasm.catalog()),definition=catalog.cards.find(c=>c.id==='XQ37');
afterEach(cleanup);
it('pins complete black original and opens its actual face',()=>{
 expect(createHash('sha256').update(readFileSync(resolve('public/cards/XQ37.jpg'))).digest('hex')).toBe('93778bc4ffa69e5b79ad08738e3f31a78a669a049c42a43e68d04bd6398856b9');
 expect(cardScanUrl('XQ37')).toBe('/cards/XQ37.jpg');expect(definition).toMatchObject({name:'夜总会看门人',cost:2,color:'黑',loyalty:['黑色'],magic:'',subtypes:['人类','罪犯'],defense:1,permanentIcons:{combat:1},temporaryIcons:{combat:0}});
 render(<ReadModal card={{instanceId:'face-xq37',cardId:'XQ37',name:definition.name,kind:'character',owner:'p0',controller:'p0',faceDown:false,exhausted:false}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:definition.name+'原始牌面'})).toHaveAttribute('src','/cards/XQ37.jpg');
});
it('accepts the optional untargeted entry through real controls and matches Native exactly',()=>{
 const d=native.accept,view=JSON.parse(wasm.view(d.state,d.seat));let result;
 const {container}=render(<Table view={view} catalog={catalog} busy={false} onAction={action=>{expect(action).toMatchObject(d.command.action.action);result=JSON.parse(wasm.applyRoom(d.state,d.seat,JSON.stringify({...d.command,action:{kind:'game',action}}),'0'));}}/>);
 expect(container.querySelectorAll('[data-choice-option]')).toHaveLength(1);fireEvent.click(container.querySelector('[data-choice-option]'));fireEvent.click(screen.getByRole('button',{name:'确认选择'}));expect(result).toEqual(d.expected);
});
it('declines through real controls without placing influence',()=>{
 const d=native.decline,view=JSON.parse(wasm.view(d.state,d.seat));let result;
 render(<Table view={view} catalog={catalog} busy={false} onAction={action=>{result=JSON.parse(wasm.applyRoom(d.state,d.seat,JSON.stringify({...d.command,action:{kind:'game',action}}),'0'));}}/>);
 fireEvent.click(screen.getByRole('button',{name:'跳过此选择'}));expect(result).toEqual(d.expected);expect(result.view.pendingChoice).toBeNull();
});
it.each([1,2,3])('keeps optional entry controls private from seat %s',seat=>{
 const {container}=render(<Table view={JSON.parse(wasm.view(native.accept.state,seat))} catalog={catalog} busy={false} onAction={vi.fn()}/>);
 expect(container.querySelector('[data-choice-option]')).toBeNull();expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();
});
it('passes the response with real controls and places exactly one',()=>{
 const d=native.placement,view=JSON.parse(wasm.view(d.state,d.seat));let result;
 render(<ResponseWindow view={view} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} onAction={action=>{expect(action).toEqual(d.command.action);result=JSON.parse(wasm.applyRoom(d.state,d.seat,JSON.stringify({...d.command,action}),'0'));}}/>);
 fireEvent.click(screen.getByRole('button',{name:'不连锁，让过'}));expect(result).toEqual(d.expected);
 const before=JSON.parse(wasm.view(d.state,0));expect(result.view.regions[2].influence.reduce((a,b)=>a+b,0)-before.regions[2].influence.reduce((a,b)=>a+b,0)).toBe(1);
 for(let seat=0;seat<4;seat++)expect(JSON.parse(wasm.view(result.state,seat))).toEqual(d.views[seat]);
});
it('admits three copies in an existing fifty-card deck and rejects four',()=>{
 const draft={...createDeckDraft(catalog),cards:[{cardId:'XQ37',count:3},{cardId:'JC125',count:47}]};expect(validateDeckDraft(draft,catalog).valid).toBe(true);expect(()=>wasm.newGameWithDeck('xq37-ui','LOCAL','teams','P0',JSON.stringify(draft),'9')).not.toThrow();
 const bad={...draft,cards:[{cardId:'XQ37',count:4},{cardId:'JC125',count:46}]};expect(validateDeckDraft(bad,catalog).valid).toBe(false);expect(()=>wasm.newGameWithDeck('xq37-bad','LOCAL','teams','P0',JSON.stringify(bad),'9')).toThrow();
});
it('rejects the exact preceding engine56 Room without silent migration',()=>{expect(()=>wasm.view(prior56.state,0)).toThrow();});
