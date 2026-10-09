// Prepared Native room input through real React response controls and current WASM.
import {cleanup,fireEvent,render,screen} from '@testing-library/react';
import {afterEach,expect,it,vi} from 'vitest';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import * as wasm from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import native from './xq27Native56Test.fixture.json';
import previous from './jc050Native55Test.fixture.json';
import {ResponseWindow} from './ResponseWindow';
import {ReadModal} from './ReadModal';
import {cardScanUrl} from './cardScans';
import {createDeckDraft,validateDeckDraft} from './deckLibrary';
wasm.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(wasm.catalog()),definition=catalog.cards.find(c=>c.id==='XQ27');
afterEach(cleanup);
it('pins complete gray original and opens the actual scan',()=>{
 expect(cardScanUrl('XQ27')).toBe('/cards/XQ27.jpg');
 expect(createHash('sha256').update(readFileSync(resolve('public/cards/XQ27.jpg'))).digest('hex')).toBe('78f2d5e2422b611bbf159a33083d70c75bf2ac83103325550430a394e8a803b5');
 expect(definition).toMatchObject({name:'定点清除行动',kind:'spell',cost:4,loyalty:['灰色','灰色'],magic:'',subtypes:['阴谋'],unique:false});
 expect(definition.abilities).toMatchObject([{key:'hidden-sweep',timing:'standard',triggered:false}]);
 render(<ReadModal card={{instanceId:'face-xq27',cardId:'XQ27',name:definition.name,kind:'spell',owner:'p0',controller:'p0',faceDown:false,exhausted:false}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:definition.name+'原始牌面'})).toHaveAttribute('src','/cards/XQ27.jpg');
});
it('passes the final response using real controls and matches the exact whole Native transition',()=>{
 const d=native,view=JSON.parse(wasm.view(d.state,d.seat));let result;
 render(<ResponseWindow view={view} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} onAction={action=>{expect(action).toEqual(d.command.action);result=JSON.parse(wasm.applyRoom(d.state,d.seat,JSON.stringify({...d.command,action}),'0'));}}/>);
 expect(screen.getByText('定点清除行动')).toBeInTheDocument();
 expect(screen.queryByLabelText('公开目标')).not.toBeInTheDocument();
 fireEvent.click(screen.getByRole('button',{name:'不连锁，让过'}));expect(result).toEqual(d.expected);
 expect(result.view.regions.flatMap(r=>r.characters).filter(c=>c.faceDown)).toHaveLength(0);
 for(let seat=0;seat<4;seat++)expect(JSON.parse(wasm.view(result.state,seat))).toEqual(d.views[seat]);
});
it('does not expose hidden victim definitions to other seats before resolution',()=>{
 const input=JSON.parse(native.state).game;
 const hidden=input.regions.flatMap(r=>r.cards).filter(c=>c.face_down);
 for(let seat=0;seat<4;seat++){
  const view=JSON.parse(wasm.view(native.state,seat));
  for(const c of hidden.filter(c=>c.controller!==seat)){
   const projected=view.regions.flatMap(r=>r.characters).find(p=>p.instanceId===c.id);
   expect(projected).not.toHaveProperty('cardId');expect(projected.name).toBe('暗藏者');
  }
 }
});
it('admits three copies and enforces the existing fifty-card deck limit',()=>{
 const draft={...createDeckDraft(catalog),cards:[{cardId:'XQ27',count:3},{cardId:'JC125',count:47}]};expect(validateDeckDraft(draft,catalog).valid).toBe(true);
 expect(()=>wasm.newGameWithDeck('xq27-ui','LOCAL','teams','P0',JSON.stringify(draft),'9')).not.toThrow();
 const bad={...draft,cards:[{cardId:'XQ27',count:4},{cardId:'JC125',count:46}]};expect(validateDeckDraft(bad,catalog).valid).toBe(false);expect(()=>wasm.newGameWithDeck('xq27-bad','LOCAL','teams','P0',JSON.stringify(bad),'9')).toThrow();
});
it('preserves engine55 identity without silently migrating its rooms',()=>{expect(()=>wasm.view(previous.state,0)).toThrow();});
