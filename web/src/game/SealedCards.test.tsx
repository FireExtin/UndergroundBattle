import { fireEvent, render, screen, within } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { Table } from './Table';
import { testCard, testCatalog, testView } from './testFixtures';
import native47 from './sealedNative47Test.fixture.json';
import type { Catalog, CardDefinition, SealedCard, View } from './types';

// Deliberately populated printed fallbacks prove that the current sealed state
// cannot accidentally regain printed mechanics through the catalog or a card.
const dream: CardDefinition = { id:'XQ41', name:'启迪之梦', kind:'character', type:'梦境', cost:3, loyalty:['紫','紫'], magic:'星辰', text:'被封印触发：抓一张牌', defense:1,
  permanentIcons:{investigation:1,combat:0,influence:0}, temporaryIcons:{investigation:0,combat:0,influence:1} };
const catalog = {...testCatalog, cards:[...testCatalog.cards,dream]};
const host = {...testCard,instanceId:'host-exact',owner:'p1',controller:'p0',region:0};
const sealed:SealedCard = {...testCard,instanceId:'sealed-exact',cardId:'XQ41',name:'启迪之梦',kind:'sealed',owner:'p1',controller:'p1',hostId:host.instanceId,cost:3,text:dream.text,magic:'星辰'};
const view:View = {...testView,status:'playing',hand:[],legalActions:[],players:[...testView.players,{...testView.players[0],id:'p1',seat:1,name:'乙',team:1}],
  regions:[{id:'region-exact',index:0,cardId:'DQJC112',name:'纽约',threshold:4,points:4,influence:[0,0],characters:[host]}],sealedCards:[sealed]};

it('reads every public sealed card as blank and outside play without catalog fallback or action', () => {
  const act=vi.fn();
  const {rerender}=render(<Table view={view} catalog={catalog} busy={false} onAction={act} />);
  fireEvent.click(screen.getByText('封印牌 · 1'));
  const region=screen.getByRole('region',{name:'场外封印牌'});
  expect(within(region).getByText('乙 拥有 · 已封印')).toBeInTheDocument();
  const reader=within(region).getByRole('button',{name:'阅读封印牌启迪之梦'});
  expect(reader).toHaveAttribute('data-sealed-host',host.instanceId);
  expect(reader).not.toHaveAttribute('data-card-targeted');
  fireEvent.click(reader);
  const modal=screen.getByRole('dialog',{name:'放大阅读启迪之梦'});
  expect(within(modal).getByText('已封印 · 场外空白牌')).toBeInTheDocument();
  expect(within(modal).queryByText(dream.text)).not.toBeInTheDocument();
  expect(within(modal).queryByText(/防御 1|忠诚|星辰|白底图标/)).not.toBeInTheDocument();
  expect(modal.querySelector('.hg-cost')).toBeNull();expect(modal.querySelector('.hg-icons')).toBeNull();
  fireEvent.click(within(modal).getByRole('button',{name:'原始牌面'}));
  expect(within(modal).getByRole('img',{name:'启迪之梦原始牌面'})).toHaveAttribute('src','/cards/XQ41.jpg');
  expect(within(modal).getByText('原图中的印刷能力和数值在封印期间不生效。')).toBeInTheDocument();
  expect(act).not.toHaveBeenCalled();
  // Payload returns under a fresh identity; an open sealed reader must close.
  rerender(<Table view={{...view,version:2,sealedCards:[],regions:[{...view.regions[0],characters:[]}],hand:[]}} catalog={catalog} busy={false} onAction={act} />);
  expect(screen.queryByRole('dialog',{name:'放大阅读启迪之梦'})).not.toBeInTheDocument();
  expect(screen.queryByRole('region',{name:'场外封印牌'})).not.toBeInTheDocument();
});

it('retains exact carrier identity across movement/control and never names a replacement as carrier', () => {
  const act=vi.fn();const {rerender}=render(<Table view={view} catalog={catalog} busy={false} onAction={act} />);
  fireEvent.click(screen.getByText('封印牌 · 1'));
  const original=screen.getByRole('button',{name:'阅读封印牌启迪之梦'});
  expect(original).toHaveTextContent('甲 操控 · 地区 1');
  const moved:View={...view,version:2,regions:[{...view.regions[0],index:1,characters:[{...host,region:1,controller:'p1'}]}]};
  rerender(<Table view={moved} catalog={catalog} busy={false} onAction={act} />);
  expect(screen.getByRole('button',{name:'阅读封印牌启迪之梦'})).toHaveTextContent('乙 操控 · 地区 2');
  rerender(<Table view={{...moved,version:3,regions:[{...moved.regions[0],characters:[{...host,instanceId:'replacement-host'}]}]}} catalog={catalog} busy={false} onAction={act} />);
  expect(screen.getByRole('button',{name:'阅读封印牌启迪之梦'})).toHaveTextContent('原载体未在当前视图中显示');
  expect(act).not.toHaveBeenCalled();
});

it('requires one actual hand instance for the private seal choice and displays no sealed target action', () => {
  const act=vi.fn();const held={...sealed,kind:'character',instanceId:'hand-before-seal',controller:'p0'};
  const choice={id:'seal-private-choice',kind:'handSeal',title:'选择一张手牌封印在目标角色上',description:'必须选择一张手牌',playerId:'p0',min:1,max:1,options:[{id:held.instanceId,label:held.name,card:held}]};
  render(<Table view={{...view,hand:[held],sealedCards:[],pendingChoice:choice,legalActions:[{id:'choice-command',kind:'choose',choiceId:choice.id,label:'确认选择'}]}} catalog={catalog} busy={false} onAction={act} />);
  const modal=screen.getByRole('dialog',{name:'待完成的选择'});
  expect(within(modal).getByRole('button',{name:'确认选择'})).toBeDisabled();
  expect(within(modal).queryByRole('button',{name:'跳过此选择'})).not.toBeInTheDocument();
  fireEvent.click(modal.querySelector('[data-choice-option="hand-before-seal"]')!);
  fireEvent.click(within(modal).getByRole('button',{name:'确认选择'}));
  expect(act).toHaveBeenCalledExactlyOnceWith({id:'choice-command',kind:'choose',choiceId:choice.id,label:'确认选择',selected:['hand-before-seal']});
});

const native = native47 as unknown as { catalog:Catalog; cases:Record<string,{views:View[]}> };

it('uses actual Native private projections: only the hand holder receives seal options', () => {
  const views=native.cases['private-hand-seal'].views;const act=vi.fn();
  const {rerender}=render(<Table view={views[1]} catalog={native.catalog} busy={false} onAction={act} />);
  expect(views[1].pendingChoice).toBeNull();
  expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();
  rerender(<Table view={views[0]} catalog={native.catalog} busy={false} onAction={act} />);
  const choice=views[0].pendingChoice!;expect(choice.kind).toBe('handSeal');expect(choice.min).toBe(1);
  const modal=screen.getByRole('dialog',{name:'待完成的选择'});
  expect(within(modal).getByRole('button',{name:'确认选择'})).toBeDisabled();
  fireEvent.click(modal.querySelector(`[data-choice-option="${choice.options[0].id}"]`)!);
  fireEvent.click(within(modal).getByRole('button',{name:'确认选择'}));
  expect(act.mock.calls[0][0].selected).toEqual([choice.options[0].id]);
});

it('shows actual Native XQ41 sealed projection publicly and closes its reader on owner return', () => {
  const publicView=native.cases['sealed-trigger'].views[2];const act=vi.fn();
  const {rerender}=render(<Table view={publicView} catalog={native.catalog} busy={false} onAction={act} />);
  expect(publicView.sealedCards![0].owner).toBe('p3');
  fireEvent.click(screen.getByText('封印牌 · 1'));
  fireEvent.click(screen.getByRole('button',{name:'阅读封印牌启迪之梦'}));
  const modal=screen.getByRole('dialog',{name:'放大阅读启迪之梦'});
  expect(within(modal).queryByText('被封印触发：抓一张牌')).not.toBeInTheDocument();
  expect(modal.querySelector('.hg-icons')).toBeNull();expect(modal.querySelector('.hg-cost')).toBeNull();
  const returned=native.cases['owner-return'].views[2];
  rerender(<Table view={returned} catalog={native.catalog} busy={false} onAction={act} />);
  expect(screen.queryByRole('dialog',{name:'放大阅读启迪之梦'})).not.toBeInTheDocument();
  expect(returned.hand.some(card=>card.cardId==='XQ41')).toBe(false);
  expect(native.cases['owner-return'].views[3].hand.some(card=>card.cardId==='XQ41')).toBe(true);
  expect(act).not.toHaveBeenCalled();
});
