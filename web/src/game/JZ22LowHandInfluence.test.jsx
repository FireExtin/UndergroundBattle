// Fresh engine57 Native inputs; React controls plus actual WASM, not natural UI play.
import {cleanup,fireEvent,render,screen} from '@testing-library/react';
import {afterEach,expect,it,vi} from 'vitest';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import * as wasm from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import native from './jz22Native60Test.fixture.json';
import prior53 from './threeColorDeathNative53Test.fixture.json';
import {Table} from './Table';
import {ReadModal} from './ReadModal';
import {cardScanUrl} from './cardScans';
wasm.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(wasm.catalog()),definition=catalog.cards.find(c=>c.id==='JZ22');
afterEach(cleanup);
it('pins the whole original JZ22 face, blue loyalty, Blood and both combat icons',()=>{
 expect(cardScanUrl('JZ22')).toBe('/cards/JZ22.jpg');
 expect(createHash('sha256').update(readFileSync(resolve('public/cards/JZ22.jpg'))).digest('hex')).toBe('b368bc3d75306dd0112cff180c8b10501c191e30b0c8d06d78e901ea20cd6160');
 expect(definition.cost).toBe(2);expect(definition.loyalty).toEqual(['蓝色']);expect(definition.magic).toBe('鲜血');expect(definition.permanentIcons.combat).toBe(1);expect(definition.temporaryIcons.combat).toBe(1);
 render(<ReadModal card={{instanceId:'face-jz22',cardId:'JZ22',name:definition.name,kind:'character',owner:'p0',controller:'p0',faceDown:false,exhausted:false}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:definition.name+'原始牌面'})).toHaveAttribute('src','/cards/JZ22.jpg');
});
it('submits the optional entry through real React controls and matches Native exactly',()=>{
 const d=native.accept,view=JSON.parse(wasm.view(d.state,d.seat));let result;
 const {container}=render(<Table view={view} catalog={catalog} busy={false} onAction={action=>{result=JSON.parse(wasm.applyRoom(d.state,d.seat,JSON.stringify({...d.command,action:{kind:'game',action}}),'0'));}}/>);
 expect(container.querySelectorAll('[data-choice-option]')).toHaveLength(1);
 fireEvent.click(container.querySelector('[data-choice-option]'));fireEvent.click(screen.getByRole('button',{name:'确认选择'}));expect(result).toEqual(d.expected);
});
it('skips the optional entry through real controls without adding influence',()=>{
 const d=native.decline,view=JSON.parse(wasm.view(d.state,d.seat));let result;
 render(<Table view={view} catalog={catalog} busy={false} onAction={action=>{result=JSON.parse(wasm.applyRoom(d.state,d.seat,JSON.stringify({...d.command,action:{kind:'game',action}}),'0'));}}/>);
 fireEvent.click(screen.getByRole('button',{name:'跳过此选择'}));expect(result).toEqual(d.expected);expect(result.view.pendingChoice).toBeNull();
});
it.each([1,2,3])('keeps the optional entry controls private from seat %s',seat=>{
 const {container}=render(<Table view={JSON.parse(wasm.view(native.accept.state,seat))} catalog={catalog} busy={false} onAction={vi.fn()}/>);
 expect(container.querySelector('[data-choice-option]')).toBeNull();expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();
});
it('places exactly one after the response passes and restores every seat projection',()=>{
 const d=native.placement,result=JSON.parse(wasm.applyRoom(d.state,d.seat,JSON.stringify(d.command),'0'));expect(result).toEqual(d.expected);
 for(let seat=0;seat<4;seat++)expect(JSON.parse(wasm.view(result.state,seat))).toEqual(d.views[seat]);
 const before=JSON.parse(wasm.view(d.state,0));expect(result.view.regions[2].influence.reduce((a,b)=>a+b,0)-before.regions[2].influence.reduce((a,b)=>a+b,0)).toBe(1);
});

it('accepts existing granted combat glory on JZ22 without blocking the response stack',()=>{
 const d=native.glory,view=JSON.parse(wasm.view(d.state,d.seat));let result;
 const {container}=render(<Table view={view} catalog={catalog} busy={false} onAction={action=>{result=JSON.parse(wasm.applyRoom(d.state,d.seat,JSON.stringify({...d.command,action:{kind:'game',action}}),'0'));}}/>);
 fireEvent.click(container.querySelector('[data-choice-option]'));fireEvent.click(screen.getByRole('button',{name:'确认选择'}));expect(result).toEqual(d.expected);
 for(let seat=0;seat<4;seat++)expect(JSON.parse(wasm.view(result.state,seat))).toEqual(d.views[seat]);
});

it('rejects the preserved previous engine53 room without silent migration',()=>{expect(()=>wasm.view(prior53.contractTrigger.state,0)).toThrow();});
