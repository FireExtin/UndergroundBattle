// Compiler-bound Native86 -> actual WASM -> production Table/choice/reading controls.
import {render,screen,fireEvent,within} from '@testing-library/react';
import {beforeEach,afterEach,expect,it,vi} from 'vitest';
import {loadFixture,sha256} from '../../../tools/jc066-oracle.mjs';
import {kernel as abi,moduleBytes,catalog} from '../../../rust-game-wasm/tests/current86-kernel.mjs';
import {Table} from './Table';
import {ReadModal} from './ReadModal';
import {actionForRoom} from './api';

const fixture=loadFixture();
expect(sha256(moduleBytes)).toBe(fixture.wasmSha256);
expect(catalog).toEqual(fixture.catalog);
beforeEach(()=>sessionStorage.clear());
afterEach(()=>sessionStorage.clear());
const choices=fixture.scenarios.flatMap(s=>s.steps.flatMap((row,i)=>{
  const views=i?s.steps[i-1].views:s.initial.views;
  return !row.expected.errorCode&&row.command.action.action?.kind==='choose'
    &&views[row.seat].pendingChoice
    ?[{name:s.name,row,views}]:[];
}));

it.each(choices)('$name preserves the physical optional target, draft and exact accepted room transition',({row,views})=>{
  const view=views[row.seat],selected=row.command.action.action.selected,submit=vi.fn();
  const props={catalog,busy:false,connection:'online',onAction:submit};
  const first=render(<Table {...props} view={view}/>);
  const firstDialog=screen.getByRole('dialog',{name:'待完成的选择'});
  expect(firstDialog.querySelectorAll('[data-choice-option]')).toHaveLength(view.pendingChoice.options.length);
  if(selected.length)fireEvent.click(firstDialog.querySelector(`[data-choice-option="${selected[0]}"]`));
  first.unmount();
  const restored=JSON.parse(abi.view(row.state,row.seat));
  expect(restored).toEqual(view);
  const mounted=render(<Table {...props} view={restored}/>);
  const dialog=screen.getByRole('dialog',{name:'待完成的选择'});
  if(selected.length)expect(dialog.querySelector(`[data-choice-option="${selected[0]}"]`)).toHaveAttribute('aria-pressed','true');
  const confirm=within(dialog).getByRole('button',{name:'确认选择'});
  expect(confirm).toBeEnabled();fireEvent.click(confirm);expect(submit).toHaveBeenCalledOnce();
  const action=JSON.parse(JSON.stringify(actionForRoom(restored,submit.mock.calls[0][0])));
  expect(action).toEqual(row.command.action);
  const result=JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify({...row.command,action}),'0'));
  expect(result).toEqual(row.expected);
  row.views.forEach((v,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(v));
  mounted.rerender(<Table {...props} view={restored} busy/>);
  fireEvent.click(within(dialog).getByRole('button',{name:'正在提交…'}));
  expect(submit).toHaveBeenCalledOnce();
});

it('only the acting seat receives target controls and a foreign hidden target keeps its identity private',()=>{
  const {views,row}=choices.find(s=>s.name==='actor-1');
  const hidden=views[row.seat].pendingChoice.options.find(o=>o.card.kind==='hidden');
  expect(hidden.card.cardId).toBeUndefined();expect(hidden.label).toBe('暗藏者');
  const mounted=render(<Table catalog={catalog} view={views[row.seat]} busy={false} connection="online" onAction={vi.fn()}/>);
  const option=screen.getByRole('dialog',{name:'待完成的选择'}).querySelector(`[data-choice-option="${hidden.id}"]`);
  expect(option).toHaveTextContent('暗藏者');expect(option).not.toHaveTextContent('唐·格鲁尼');mounted.unmount();
  for(let seat=0;seat<4;seat++)if(seat!==row.seat){
    expect(views[seat].pendingChoice).toBeNull();
    const other=render(<Table catalog={catalog} view={views[seat]} busy={false} connection="online" onAction={vi.fn()}/>);
    expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();other.unmount();
  }
});

const sealed=fixture.scenarios.filter(s=>s.steps.at(-1).views[0].sealedCards?.length);
it.each(sealed)('$name publicly reads the actual sealed blank card and its exact live host without an action target',scenario=>{
  const final=scenario.steps.at(-1);
  for(let seat=0;seat<4;seat++){
    const view=JSON.parse(abi.view(final.expected.state,seat));
    expect(view).toEqual(final.views[seat]);
    const sealed=view.sealedCards[0],host=view.regions.flatMap(r=>r.characters).find(c=>c.instanceId===sealed.hostId);
    expect(host.cardId).toBe('JC066');
    const mounted=render(<Table catalog={catalog} view={view} busy={false} connection="online" onAction={vi.fn()}/>);
    fireEvent.click(mounted.container.querySelector('.hg-sealed-cards-menu summary'));
    const reading=screen.getByRole('button',{name:`阅读封印牌${sealed.name}`});
    expect(reading).toHaveAttribute('data-sealed-host',host.instanceId);
    expect(reading).toHaveTextContent('载体：封魔瓶瑟琳娜');
    expect(mounted.container.querySelector(`[data-card-instance="${sealed.instanceId}"]`)).toBeNull();
    fireEvent.click(reading);
    const dialog=screen.getByRole('dialog');
    expect(dialog).toHaveTextContent('此牌已封印，当前视为空白且在场外');
    expect(dialog).not.toHaveTextContent('当前有效');
    expect(dialog.querySelector('[aria-label="印刷图标"]')).toBeNull();
    fireEvent.click(within(dialog).getByRole('button',{name:'原始牌面'}));
    expect(within(dialog).getByRole('img',{name:`${sealed.name}原始牌面`})).toHaveAttribute('src',`/cards/${sealed.cardId}.jpg`);
    mounted.unmount();
  }
});

it.each(['actor-0','actor-2','shared-hand-seal-character','shared-hand-seal-attachment'])('%s displays inherited current icons separately from the complete original print',name=>{
  const scenario=fixture.scenarios.find(s=>s.name===name),view=scenario.steps.at(-1).views[0];
  const card=view.regions.flatMap(r=>r.characters).find(c=>c.cardId==='JC066'),definition=catalog.cards.find(c=>c.id==='JC066');
  expect(definition).toMatchObject({subtitle:'样本B7021',cost:5,color:'灰',loyalty:['灰色','灰色','灰色'],magicIcon:'Mind',unique:true,defense:2,
    permanentIcons:{investigation:0,combat:0,influence:2},temporaryIcons:{investigation:1,combat:0,influence:0}});
  render(<ReadModal card={card} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  const dialog=screen.getByRole('dialog');
  expect(dialog).toHaveTextContent('样本B7021');expect(dialog).toHaveTextContent('被封印在其上的角色牌能力栏内的所有能力图标');
  const current=within(dialog).getByLabelText(`当前有效图标：调查${card.icons.investigation}，战斗${card.icons.combat}，势力${card.icons.influence}`);
  expect(current).toBeInTheDocument();
  expect(within(dialog).getByLabelText('印刷图标：调查0，战斗0，势力2')).toBeInTheDocument();
  fireEvent.click(within(dialog).getByRole('button',{name:'原始牌面'}));
  expect(within(dialog).getByRole('img',{name:'封魔瓶瑟琳娜原始牌面'})).toHaveAttribute('src','/cards/JC066.jpg');
});
