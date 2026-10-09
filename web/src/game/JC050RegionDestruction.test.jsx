// Prepared Native room input through React controls and the real current WASM.
import {cleanup,fireEvent,render,screen} from '@testing-library/react';
import {afterEach,expect,it,vi} from 'vitest';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import * as wasm from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import native from './jc050Native55Test.fixture.json';
import quote from './jc050Native55QuoteTest.fixture.json';
import previous from './jz22Native54Test.fixture.json';
import {Table} from './Table';
import {ReadModal} from './ReadModal';
import {cardScanUrl} from './cardScans';
import {createDeckDraft,validateDeckDraft} from './deckLibrary';
wasm.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(wasm.catalog()),definition=catalog.cards.find(c=>c.id==='JC050');
afterEach(cleanup);
it('pins the complete original red disaster and opens its real scan',()=>{
 expect(cardScanUrl('JC050')).toBe('/cards/JC050.jpg');
 expect(createHash('sha256').update(readFileSync(resolve('public/cards/JC050.jpg'))).digest('hex')).toBe('2dbbe996c5fec273a1340ca4acadc504689b1af6151b56fe5ae0c217d2df159e');
 expect(definition).toMatchObject({name:'丧钟回响',kind:'spell',cost:5,loyalty:['红色','红色'],magic:'鲜血',subtypes:['灾难'],unique:false});
 expect(definition.abilities).toMatchObject([{key:'region-destruction',timing:'standard',triggered:false}]);
 render(<ReadModal card={{instanceId:'face-jc050',cardId:'JC050',name:definition.name,kind:'spell',owner:'p0',controller:'p0',faceDown:false,exhausted:false}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:definition.name+'原始牌面'})).toHaveAttribute('src','/cards/JC050.jpg');
});
it('chooses a region through real React controls and matches the complete original Native transition',()=>{
 const d=native,view=JSON.parse(wasm.view(d.state,d.seat));let result;
 const {container}=render(<Table view={view} catalog={catalog} busy={false} onAction={action=>{result=JSON.parse(wasm.applyRoom(d.state,d.seat,JSON.stringify({...d.command,action:{kind:'game',action}}),'0'));}}/>);
 expect(screen.getByText('选择要消灭其中角色与暗藏者的地区')).toBeInTheDocument();
 expect(container.querySelectorAll('[data-choice-option]')).toHaveLength(view.regions.length);
 fireEvent.click(container.querySelectorAll('[data-choice-option]')[2]);fireEvent.click(screen.getByRole('button',{name:'确认选择'}));expect(result).toEqual(d.expected);
 expect(result.view.regions[2].characters).toHaveLength(0);
 for(let seat=0;seat<4;seat++)expect(JSON.parse(wasm.view(result.state,seat))).toEqual(d.views[seat]);
});
it.each([1,2,3])('keeps the region choice private from seat %s',seat=>{
 const {container}=render(<Table view={JSON.parse(wasm.view(native.state,seat))} catalog={catalog} busy={false} onAction={vi.fn()}/>);
 expect(container.querySelector('[data-choice-option]')).toBeNull();
});
it('keeps deck copy limits and admits the new card in a fifty-card deck',()=>{
 const draft={...createDeckDraft(catalog),cards:[{cardId:'JC050',count:3},{cardId:'JC125',count:47}]};
 expect(validateDeckDraft(draft,catalog).valid).toBe(true);
 expect(()=>wasm.newGameWithDeck('jc050-ui','LOCAL','teams','P0',JSON.stringify(draft),'9')).not.toThrow();
 const bad={...draft,cards:[{cardId:'JC050',count:4},{cardId:'JC125',count:46}]};expect(validateDeckDraft(bad,catalog).valid).toBe(false);
 expect(()=>wasm.newGameWithDeck('jc050-ui-bad','LOCAL','teams','P0',JSON.stringify(bad),'9')).toThrow();
});
it('rejects preserved engine54 rooms without silently migrating them',()=>{expect(()=>wasm.view(previous.accept.state,0)).toThrow();});

it('rejects a predeclared region in the actual quote ABI without changing state',()=>{
 const d=quote;expect(JSON.parse(wasm.quoteRoom(d.state,d.seat,JSON.stringify(d.request)))).toEqual(d.expected);
 expect(d.expected.ready).toBe(false);expect(d.expected.error).toContain('JC050');
});
