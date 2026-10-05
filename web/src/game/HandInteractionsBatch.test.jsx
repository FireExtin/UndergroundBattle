import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen, fireEvent, within } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { ReadModal } from './ReadModal';
import { Table } from './Table';
import { ChoicePanel } from './ChoicePanel';
import { groupObjectActions } from './objectActions';
import { testView, testCard, testCatalog } from './testFixtures';
kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
for (const [id, sha, fields] of [
  ['JC114', '2cd5cc66ceb3c9a8f9d07d341558489adb170dcb680331c7620e5f1162be12c7', { name: '好奇的黑客', subtitle: '无知而无畏', cost: 2, loyalty: [], color: '中立', kind: 'character', magic: '', unique: false, subtypes: ['人类', '工程师'], defense: 1, temporaryIcons: { investigation: 1, combat: 0, influence: 0 } }],
  ['XQ34', '2ca1f5a4b25d308f8af13a7a80018010deaf93f9abe2275bef1fa7656be13e04', { name: '灵感', cost: 1, loyalty: ['白色'], color: '白', kind: 'spell', magic: '', subtypes: ['事务', '研究'] }],
  ['XQ38', '26c8311674325491157577a3d295c148ce1c6f27ded59c4d27979faf58e9204e', { name: '金特·易卜拉欣', subtitle: '死灵学专家', cost: 3, loyalty: ['黑色', '黑色'], magic: '', kind: 'character', unique: true, defense: 2, permanentIcons: { investigation: 0, combat: 0, influence: 1 }, temporaryIcons: { investigation: 2, combat: 0, influence: 0 } }],
  ['JZ67', 'a6554b81e081c7292a32f2bf13ce269f276580f9a852e1df232c60c2c36ff477', { name: '失忆', cost: 1, loyalty: ['紫色'], color: '紫', magic: '心灵', kind: 'spell', subtypes: ['事务', '突发状况'] }],
]) it(`reads the unchanged original ${id} and printed fields`, () => {
  const d = catalog.cards.find(c => c.id === id); expect(d).toMatchObject(fields);
  expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(sha);
  const mounted = render(<ReadModal card={{ ...d, cardId: id, instanceId: id, owner: 'p0', controller: 'p0', exhausted: false, faceDown: false }} definition={d} viewerId="p0" onClose={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: `${d.name}原始牌面` })).toHaveAttribute('src', `/cards/${id}.jpg`); mounted.unmount();
});
it.each(['p0','p1','p2','p3'])('reads only the currently revealed hand for %s and closes stale readers', you => {
  const shown={...testCard,instanceId:'public-hand-instance',name:'公开手牌',owner:'p2',controller:'p2'};
  const v={...testView, status:'playing', you, hand:[], revealedHands:[{playerId:'p2',cards:[shown]}], legalActions:[], players:[0,1,2,3].map(seat=>({...testView.players[0],id:`p${seat}`,seat,name:`玩家${seat}`,team:Math.floor(seat/2)}))};
  const m=render(<Table view={v} catalog={testCatalog} busy={false} onAction={vi.fn()}/>);
  const panel=screen.getByRole('region',{name:'公开展示的手牌'});fireEvent.click(within(panel).getByRole('button',{name:'放大阅读展示的公开手牌'}));
  expect(screen.getByRole('dialog',{name:'放大阅读公开手牌'})).toBeInTheDocument();
  m.rerender(<Table view={{...v,version:2,revealedHands:[]}} catalog={testCatalog} busy={false} onAction={vi.fn()}/>);
  expect(screen.queryByRole('region',{name:'公开展示的手牌'})).not.toBeInTheDocument();expect(screen.queryByRole('dialog',{name:'放大阅读公开手牌'})).not.toBeInTheDocument();m.unmount();
});
it.each(['revealed-hand-discard','optional-shuffle'])('submits explicit accept or decline for %s',kind=>{
  const submit=vi.fn();const option={id:kind==='optional-shuffle'?'shuffle':'hand-exact',label:'选择此项'};const choice={id:'choice',kind,title:'选择',description:'',playerId:'p0',options:[option],min:0,max:1,allowDecline:true};const action={id:'choose',kind:'choose',label:'确认'};
  const m=render(<ChoicePanel choice={choice} action={action} definitions={new Map()} busy={false} onSubmit={submit}/>);
  fireEvent.click(screen.getByRole('button',{name:'跳过此选择'}));expect(submit).toHaveBeenLastCalledWith({...action,choiceId:'choice',selected:[]});
  fireEvent.click(screen.getByRole('button',{name:/选择此项$/}));fireEvent.click(screen.getByRole('button',{name:'确认选择'}));expect(submit).toHaveBeenLastCalledWith({...action,choiceId:'choice',selected:[option.id]});m.unmount();
});
it('keeps exact discarded hand costs separate when grouping recovery targets',()=>{
  const actions=['hand-one','hand-two'].flatMap(cost=>['grave-one','grave-two'].map(target=>({id:cost+target,kind:'activate',cardId:'scholar',abilityId:'recover-grave-discard',targetId:target,costSelected:[cost],label:`金特·易卜拉欣：标准行动 → ${target}（费用：弃掉无知路人 [${cost}]）`})));
  const groups=groupObjectActions(actions);expect(groups).toHaveLength(2);expect(groups.map(g=>g.actions.length)).toEqual([2,2]);expect(groups[0].label).toContain('费用：弃掉');expect(groups[0].actions[0].costSelected).toEqual(['hand-one']);expect(groups[1].actions[0].costSelected).toEqual(['hand-two']);
});
