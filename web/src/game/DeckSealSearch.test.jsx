import {cleanup,fireEvent,render,screen} from '@testing-library/react';
import {afterEach,expect,it,vi} from 'vitest';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import * as wasm from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import native from './deckSealNative59Test.fixture.json';
import {Table} from './Table';
import {ChoicePanel} from './ChoicePanel';
import {ReadModal} from './ReadModal';
import {cardScanUrl} from './cardScans';
wasm.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(wasm.catalog()),defs=new Map(catalog.cards.map(c=>[c.id,c]));
afterEach(cleanup);
it.each(['xq44Choice','jz02Choice'])('%s requires one and submits the selected actual instance into candidate WASM',name=>{
 let state=native.states[name].state;const view=JSON.parse(wasm.view(state,0));let transition;
 const submit=a=>{transition=JSON.parse(wasm.applyRoom(state,0,JSON.stringify({commandId:'ui-'+name,expectedVersion:view.version,action:{kind:'game',action:a}}),'0'));state=transition.state;};
 const {container,rerender}=render(<Table view={view} catalog={catalog} busy={false} onAction={submit}/>);
 expect(screen.getByRole('button',{name:'确认选择'})).toBeDisabled();expect(screen.queryByRole('button',{name:'跳过此选择'})).not.toBeInTheDocument();
 const [first,second]=view.pendingChoice.options;fireEvent.click(container.querySelector(`[data-choice-option="${first.id}"]`));
 if(second)expect(container.querySelector(`[data-choice-option="${second.id}"]`)).toBeDisabled();
 fireEvent.click(screen.getByRole('button',{name:'确认选择'}));expect(transition.outcome).toBe('accepted');
 const next=JSON.parse(wasm.view(state,0));expect(next.sealedCards).toHaveLength(1);expect(next.sealedCards[0].cardId).toBe(first.card.cardId);
 expect(JSON.parse(state).game.players[0].deck).toHaveLength(3);
 rerender(<Table view={next} catalog={catalog} busy={false} onAction={submit}/>);expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();
});
it.each(['xq44Empty','jz02Empty'])('%s explicitly completes zero and runs the actual shuffle',name=>{
 let state=native.states[name].state;const view=JSON.parse(wasm.view(state,0));let transition;
 const submit=a=>{transition=JSON.parse(wasm.applyRoom(state,0,JSON.stringify({commandId:'ui-'+name,expectedVersion:view.version,action:{kind:'game',action:a}}),'0'));state=transition.state;};
 render(<Table view={view} catalog={catalog} busy={false} onAction={submit}/>);
 const confirm=screen.getByRole('button',{name:'不取牌并洗牌'});expect(confirm).toBeEnabled();fireEvent.click(confirm);
 expect(transition.outcome).toBe('accepted');expect(JSON.parse(wasm.view(state,0)).pendingChoice).toBeNull();expect(JSON.parse(wasm.view(state,0)).sealedCards??[]).toHaveLength(0);
 expect(JSON.parse(state).game.random).not.toEqual(JSON.parse(native.states[name].state).game.random);
});
it.each(['xq44Choice','jz02Choice'])('%s remains private from its teammate and both opponents and restores unchanged',name=>{
 const fixture=native.states[name],v=JSON.parse(wasm.view(fixture.state,0));
 const {container,rerender}=render(<Table view={v} catalog={catalog} busy={false} onAction={vi.fn()}/>);
 expect(container.querySelector('[data-choice-option]')).not.toBeNull();
 for(const seat of [1,2,3]){rerender(<Table view={JSON.parse(wasm.view(fixture.state,seat))} catalog={catalog} busy={false} onAction={vi.fn()}/>);
  expect(container.querySelector('[data-choice-option]')).toBeNull();expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();}
 rerender(<Table view={JSON.parse(wasm.view(fixture.state,0))} catalog={catalog} busy={false} onAction={vi.fn()}/>);expect(container.querySelector('[data-choice-option]')).not.toBeNull();
});
it('does not enable zero for unrelated or malformed mandatory choices',()=>{
 const view=native.states.xq44Empty.views[0];render(<ChoicePanel choice={{...view.pendingChoice,kind:'unrelated'}} action={view.legalActions.find(a=>a.kind==='choose')} definitions={defs} busy={false} onSubmit={vi.fn()}/>);
 expect(screen.getByRole('button',{name:'确认选择'})).toBeDisabled();
});
it.each([['XQ44','d4f41bf42958d15358bc7fcb1119967ab94bd187c758273952fd6b70475de047'],['JZ02','59546782075c26039bdc7d4da34a621bb86de1d8d34c25710a0d59d6761a19e8']])('%s original scan and printed reader match the checked source', (id,sha)=>{
 expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(sha);expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);
 const d=defs.get(id),card={...d,instanceId:'reader-'+id,cardId:id,owner:'p0',controller:'p0',faceDown:false,exhausted:false};
 render(<ReadModal card={card} definition={d} viewerId="p0" onClose={vi.fn()}/>);fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:d.name+'原始牌面'})).toHaveAttribute('src',`/cards/${id}.jpg`);
});
