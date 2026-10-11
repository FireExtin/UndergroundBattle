// Prepared native layouts, rendered React controls and actual WASM transitions.
import {cleanup,fireEvent,render,screen} from '@testing-library/react';
import {afterEach,expect,it,vi} from 'vitest';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import * as wasm from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import native from './threeColorDeathNative63Test.fixture.json';
import {Table} from './Table';
import {CardContent,CardTile} from './CardTile';
import {ReadModal} from './ReadModal';
import {cardScanUrl} from './cardScans';
wasm.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(wasm.catalog()),defs=new Map(catalog.cards.map(c=>[c.id,c]));
afterEach(cleanup);
it.each([
 ['JC045','7b79668cee2cab604341a79f86decb03587046b31f3c5586123bcc95561d3d60'],
 ['JC031','c5d64c9392c3627659b405bc422a49535d35368b5aef5cc50cfc85f2123e6436'],
 ['JC090','7734b37f8e222307c2958461e6d20a66400011707bc77413a4101e096f40ef0c'],
])('%s has the exact pinned original and readable face', (id,hash)=>{
 expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);
 expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(hash);
 const definition=defs.get(id);const card={instanceId:'face-'+id,cardId:id,name:definition.name,kind:definition.kind,owner:'p0',controller:'p0',faceDown:false,exhausted:false};
 render(<ReadModal card={card} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:definition.name+'原始牌面'})).toHaveAttribute('src',`/cards/${id}.jpg`);
});
it('shows five public time markers while exhaustion suppresses participation',()=>{
 const view=JSON.parse(wasm.view(native.priestMarks.state,0));const card=view.regions.flatMap(r=>r.characters).find(c=>c.cardId==='JC045');
 expect(card.timeMarkers).toBe(5);expect(card.icons.influence).toBe(0);
 render(<CardContent card={card} definition={defs.get('JC045')} compact/>);expect(screen.getByText('时间 5')).toBeInTheDocument();
});
it('clears time marker presentation for concealed and asset views',()=>{
 const card=JSON.parse(wasm.view(native.priestMarks.state,0)).regions.flatMap(r=>r.characters).find(c=>c.cardId==='JC045');
 const {rerender}=render(<CardTile card={{...card,faceDown:true}} definition={defs.get('JC045')} viewerId="p1" compact/>);
 expect(screen.queryByText('时间 5')).not.toBeInTheDocument();expect(screen.queryByText('钟摆祭司')).not.toBeInTheDocument();
 rerender(<CardContent card={{...card,kind:'asset'}} definition={defs.get('JC045')} compact/>);expect(screen.queryByText('时间 5')).not.toBeInTheDocument();
});
it('requires exactly one ordinary discard and submits the real private card instance',()=>{
 let state=native.casterDiscard.state;let transition;const view=JSON.parse(wasm.view(state,1));
 const submit=a=>{transition=JSON.parse(wasm.applyRoom(state,1,JSON.stringify({commandId:'ui-death-discard',expectedVersion:view.version,action:{kind:'game',action:a}}),'0'));state=transition.state;};
 const {container}=render(<Table view={view} catalog={catalog} busy={false} onAction={submit}/>);
 const confirm=screen.getByRole('button',{name:'确认选择'});expect(confirm).toBeDisabled();expect(screen.queryByRole('button',{name:'不发动'})).not.toBeInTheDocument();
 fireEvent.click(container.querySelector('[data-choice-option]'));expect(confirm).toBeEnabled();fireEvent.click(confirm);
 expect(transition.outcome).toBe('accepted');expect(JSON.parse(wasm.view(state,1)).hand).toHaveLength(view.hand.length-1);
 expect(JSON.parse(wasm.view(state,1)).pendingChoice).toBeNull();
});
it.each([0,2,3])('ordinary discard choices stay private from seat %s including the teammate',seat=>{
 const {container}=render(<Table view={JSON.parse(wasm.view(native.casterDiscard.state,seat))} catalog={catalog} busy={false} onAction={vi.fn()}/>);
 expect(container.querySelector('[data-choice-option]')).toBeNull();expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();
});
it('optional player declaration declines without consuming the original instance limit',()=>{
 let state=native.casterTrigger.state;const view=JSON.parse(wasm.view(state,0));let result;
 const {container}=render(<Table view={view} catalog={catalog} busy={false} onAction={a=>{result=JSON.parse(wasm.applyRoom(state,0,JSON.stringify({commandId:'ui-death-decline',expectedVersion:view.version,action:{kind:'game',action:a}}),'0'));state=result.state;}}/>);
 expect(container.querySelectorAll('[data-choice-option]')).toHaveLength(4);
 const decline=screen.getByRole('button',{name:/不选|放弃|跳过|不发动/});fireEvent.click(decline);
 expect(result.outcome).toBe('accepted');expect((JSON.parse(state).game.turn_ability_usage??[])).toHaveLength(0);
});
it('random discard has no target-hand picker and consumes only server PRNG at resolution',()=>{
 const view=JSON.parse(wasm.view(native.casterRandomStack.state,1));const {container}=render(<Table view={view} catalog={catalog} busy={false} onAction={vi.fn()}/>);
 expect(container.querySelector('[data-choice-option]')).toBeNull();expect(view.pendingChoice).toBeNull();
});
it('contract declaration and the existing win window restore in all four seats',()=>{
 for(const name of ['contractTrigger','contractWinWindow'])for(let seat=0;seat<4;seat++)expect(JSON.parse(wasm.view(native[name].state,seat))).toEqual(native[name].views[seat]);
 expect(JSON.parse(wasm.view(native.contractWinWindow.state,2)).step).toBe('region:1:win');
});
