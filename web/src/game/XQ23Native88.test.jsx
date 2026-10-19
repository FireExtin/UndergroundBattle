// Actual Native88/WASM views; component checks are not online gameplay.
import {render,screen,fireEvent,within} from '@testing-library/react';
import {beforeEach,afterEach,expect,it,vi} from 'vitest';
import {loadFixture,sha256} from '../../../tools/xq23-oracle.mjs';
import {kernel as abi,moduleBytes,catalog} from '../../../rust-game-wasm/tests/current88-kernel.mjs';
import {Table} from './Table';
import {ReadModal} from './ReadModal';
import {actionForRoom} from './api';
const fixture=loadFixture();expect(sha256(moduleBytes)).toBe(fixture.wasmSha256);expect(catalog).toEqual(fixture.catalog);
beforeEach(()=>sessionStorage.clear());afterEach(()=>sessionStorage.clear());
const rows=fixture.scenarios.flatMap(s=>s.steps.map((row,i)=>({name:s.name,row,views:i?s.steps[i-1].views:s.initial.views})));
const activations=rows.filter(({row})=>!row.expected.errorCode&&row.command.action.action?.kind==='activate'&&row.command.action.action.abilityId==='xq23-return-asset-become-asset');
it.each(activations)('$name selects the actual own asset, submits the Native action and removes the old source after resolution',({row,views})=>{
 const view=views[row.seat],core=row.command.action.action,submit=vi.fn();
 const props={catalog,busy:false,connection:'online',onAction:submit};const m=render(<Table {...props} view={view}/>);
 const legal=view.legalActions.find(a=>a.kind===core.kind&&a.cardId===core.cardId&&a.targetId===core.targetId&&a.abilityId===core.abilityId);expect(legal).toBeDefined();
 fireEvent.click(m.container.querySelector(`[data-card-instance="${core.cardId}"]`));
 fireEvent.click(screen.getByRole('button',{name:legal.label.split(' → ')[0],exact:true}));
 const valid=new Set(view.legalActions.filter(a=>a.cardId===core.cardId&&a.abilityId===core.abilityId).map(a=>a.targetId));
 for(const asset of view.assets){
  const shown=m.container.querySelector(`[data-card-instance="${asset.instanceId}"]`);expect(shown).not.toBeNull();
  expect(shown).toHaveAttribute('data-card-targeted',String(valid.has(asset.instanceId)));expect(asset.cardId).toBeUndefined();
 }
 fireEvent.click(m.container.querySelector(`[data-card-instance="${core.targetId}"]`));
 fireEvent.click(screen.getByRole('button',{name:`确认 · ${legal.label}`,exact:true}));expect(submit).toHaveBeenCalledOnce();
 const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));expect(action).toEqual(row.command.action);
 const result=JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify({...row.command,action}),'0'));expect(result).toEqual(row.expected);
 row.views.forEach((v,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(v));
 m.rerender(<Table {...props} view={row.views[row.seat]} busy/>);expect(submit).toHaveBeenCalledOnce();
});
it('clears a stale asset target after the second real response returns that original UID',()=>{
 const s=fixture.scenarios.find(s=>s.name==='target-return-response'),view=s.initial.views[0],core=s.steps[0].command.action.action,submit=vi.fn();
 const legal=view.legalActions.find(a=>a.cardId===core.cardId&&a.targetId===core.targetId&&a.abilityId===core.abilityId);
 const props={catalog,busy:false,connection:'online',onAction:submit};const m=render(<Table {...props} view={view}/>);
 fireEvent.click(m.container.querySelector(`[data-card-instance="${core.cardId}"]`));fireEvent.click(screen.getByRole('button',{name:legal.label.split(' → ')[0],exact:true}));
 fireEvent.click(m.container.querySelector(`[data-card-instance="${core.targetId}"]`));
 const after=s.steps.find(r=>!r.views[0].assets.some(c=>c.instanceId===core.targetId)).views[0];
 m.rerender(<Table {...props} view={after}/>);expect(screen.queryByRole('button',{name:`确认 · ${legal.label}`,exact:true})).not.toBeInTheDocument();expect(submit).not.toHaveBeenCalled();
});
it.each([0,1,2,3])('reads the converted asset anonymously for seat%s and never offers its original scan',seat=>{
 const s=fixture.scenarios.find(s=>s.name==='actor-0'),view=s.steps.at(-1).views[seat];
 const asset=view.assets.find(c=>c.controller==='p0'&&!s.initial.views[0].assets.some(old=>old.instanceId===c.instanceId));expect(asset).toBeDefined();
 expect(asset).toMatchObject({kind:'asset',name:'资产',exhausted:true});expect(asset.cardId).toBeUndefined();expect(asset.text).toBeUndefined();
 const m=render(<ReadModal card={asset} viewerId={`p${seat}`} onClose={vi.fn()}/>);
 expect(screen.queryByRole('button',{name:'原始牌面'})).not.toBeInTheDocument();m.unmount();
});
it('shows the complete printed XQ23 and its actual original scan before it becomes an asset',()=>{
 const s=fixture.scenarios.find(s=>s.name==='actor-0'),core=s.steps[0].command.action.action;
 const card=s.initial.views[0].regions.flatMap(r=>r.characters).find(c=>c.instanceId===core.cardId),definition=catalog.cards.find(c=>c.id==='XQ23');
 expect(definition).toMatchObject({name:'谨慎的办事员',cost:1,color:'灰',loyalty:['灰色'],magicIcon:'None',subtypes:['人类','文员'],defense:1,unique:false,permanentIcons:{investigation:0,combat:0,influence:1}});
 render(<ReadModal card={card} definition={definition} viewerId='p0' onClose={vi.fn()}/>);const dialog=screen.getByRole('dialog');
 expect(dialog).toHaveTextContent('公开');expect(dialog).toHaveTextContent('撤回');expect(dialog).toHaveTextContent('拥有者手上');expect(dialog).toHaveTextContent('资产区');
 fireEvent.click(within(dialog).getByRole('button',{name:'原始牌面'}));expect(within(dialog).getByRole('img',{name:'谨慎的办事员原始牌面'})).toHaveAttribute('src','/cards/XQ23.jpg');
});
