import { act,fireEvent,render,screen } from '@testing-library/react';
import { afterEach,expect,it,vi } from 'vitest';
import { Table } from './Table';
import { isRoomView } from './api';
import { testCard,testCatalog,testView } from './testFixtures';
import type { Catalog,View } from './types';
const catalog:Catalog={...testCatalog,cards:testCatalog.cards.map(c=>({...c,abilities:[{key:'preview-fast',label:'能力',timing:'fast',triggered:false}]}))};
const hand={...testCard,instanceId:'preview-hand'};
const hidden={...testCard,instanceId:'preview-hidden',faceDown:true};
const enemy={...hidden,instanceId:'preview-enemy',owner:'p1',controller:'p1',name:'秘密身份'};
const view:View={...testView,status:'playing',serverNowMs:1000,hand:[hand],stack:[{id:'preview-top',label:'待结算',controller:'p1'}],
  players:[...testView.players,{...testView.players[0],id:'p1',seat:1,team:1,name:'对手'}],
  regions:[{id:'preview-region',index:0,cardId:'JC001',name:'地区',threshold:3,points:1,influence:[0,0],characters:[hidden,enemy]}],
  legalActions:[{id:'pass',kind:'pass',label:'让过'}],
  responseWindow:{id:'preview-window',stackTopId:'preview-top',holderTeam:0,canBegin:true,
    members:[{playerId:'p0',status:'undecided',deadlineMs:6000}],
    previewSources:[{kind:'activate',cardId:hand.instanceId,abilityId:'preview-fast'},{kind:'reveal',cardId:hidden.instanceId}]}};
const tile=(container:HTMLElement,id:string)=>container.querySelector<HTMLElement>(`[data-card-instance="${id}"]`)!;
const props={catalog,busy:false,connection:'online' as const};
afterEach(()=>vi.useRealTimers());

it('shows hand and own-hidden response sources before beginning, without creating an intent or submitting summaries',()=>{
  const submit=vi.fn();const app=render(<Table {...props} view={view} onAction={submit}/>);
  expect(tile(app.container,hand.instanceId)).toHaveTextContent('候选·快速');
  expect(tile(app.container,hidden.instanceId)).toHaveTextContent('候选·快速');
  expect(tile(app.container,enemy.instanceId)).not.toHaveTextContent('候选');
  const dock=screen.getByRole('region',{name:'当前窗口与候选行动'});
  expect(dock).toHaveTextContent('响应候选：快速行动；先选择“连锁”再提交');
  fireEvent.click(tile(app.container,hidden.instanceId));
  expect(submit).not.toHaveBeenCalled();
  expect(screen.queryByRole('button',{name:'现身',exact:true})).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'连锁',exact:true}));
  expect(submit).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({kind:'beginResponse',windowId:'preview-window'}));
  expect(view.responseWindow!.myIntentId).toBeUndefined();
});

it('allows a direct pass while preview sources exist and preserves its original payload',()=>{
  const submit=vi.fn();render(<Table {...props} view={view} onAction={submit}/>);
  fireEvent.click(screen.getByRole('button',{name:'不连锁，让过',exact:true}));
  expect(submit).toHaveBeenCalledExactlyOnceWith({kind:'passResponse',windowId:'preview-window'});
});

it('removes preview at the same trusted deadline without posting or extending it',()=>{
  vi.useFakeTimers();const submit=vi.fn();const app=render(<Table {...props} view={view} onAction={submit}/>);
  act(()=>vi.advanceTimersByTime(5000));
  expect(tile(app.container,hidden.instanceId)).not.toHaveTextContent('候选');
  expect(screen.getByRole('region',{name:'当前窗口与候选行动'})).toHaveTextContent('响应决定时间已到');
  expect(submit).not.toHaveBeenCalled();
  expect(view.responseWindow!.members[0].deadlineMs).toBe(6000);
});

it('keeps frozen-kernel fallback and never treats stale or foreign summaries as response permissions',()=>{
  const submit=vi.fn();const app=render(<Table {...props} view={{...view,responseWindow:{...view.responseWindow!,previewSources:undefined}}} onAction={submit}/>);
  expect(tile(app.container,hidden.instanceId)).not.toHaveTextContent('候选');
  expect(screen.getByRole('region',{name:'当前窗口与候选行动'})).toHaveTextContent('先选择“连锁”');
  for(const responseWindow of [{...view.responseWindow!,stackTopId:'old-top'},
    {...view.responseWindow!,members:[{playerId:'p0',status:'passed' as const}]},
    {...view.responseWindow!,previewSources:[{kind:'reveal',cardId:enemy.instanceId}]}]) {
    app.rerender(<Table {...props} view={{...view,responseWindow}} onAction={submit}/>);
    expect(tile(app.container,hidden.instanceId)).not.toHaveTextContent('候选');
    expect(tile(app.container,enemy.instanceId)).not.toHaveTextContent('候选');
    expect(screen.queryByText('秘密身份',{exact:true})).not.toBeInTheDocument();
  }
  expect(submit).not.toHaveBeenCalled();
});

it.each(['busy','uncertain','offline','pause','waiting'] as const)('suspends display-only preview while %s',state=>{
  const app=render(<Table {...props} busy={state==='busy'} uncertain={state==='uncertain'} connection={state==='offline'?'offline':'online'}
    view={{...view,...(state==='pause'?{pause:{pausedAtMs:1000,pausedBy:0}}:{}),...(state==='waiting'?{waitingChoice:{playerId:'p1',kind:'target',title:'选择'}}:{})}} onAction={vi.fn()}/>);
  expect(tile(app.container,hand.instanceId)).not.toHaveTextContent('候选');
});

it('accepts legacy or bounded source summaries but rejects action/target payloads in the preview wire field',()=>{
  const session={roomId:view.roomId,seat:0};
  expect(isRoomView(view,session as Parameters<typeof isRoomView>[1])).toBe(true);
  for(const previewSources of [null,[{kind:'reveal',cardId:hidden.instanceId,targetId:enemy.instanceId}],[{kind:'reveal',cardId:2}]]) {
    expect(isRoomView({...view,responseWindow:{...view.responseWindow!,previewSources}},session as Parameters<typeof isRoomView>[1])).toBe(false);
  }
});
