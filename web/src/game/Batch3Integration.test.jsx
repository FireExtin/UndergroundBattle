import {fireEvent,render,screen,within} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import {CardTile,visibleCard} from './CardTile';
import {Table} from './Table';
import {actionForRoom} from './api';
import {groupObjectActions} from './objectActions';
import {testCard,testView} from './testFixtures';
import {catalog as testCatalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
const definition=(id)=>testCatalog.cards.find(c=>c.id===id);
it('admits the complete twenty-card local candidate and excludes deferred cards',()=>{
  // Recheck the third cohort against the sole current kernel; later private
  // definitions and the new20 admissions are checked independently.
  expect(testCatalog.engineVersion).toBe('rust-v0.2.112-shared-keyword-rewards');
  expect(testCatalog.cards.filter(c=>c.supported===false)).toHaveLength(39);
  expect(testCatalog.cards.filter(c=>c.supported!==false)).toHaveLength(233);
  expect(definition('CQ07').supported).toBe(true);
  expect(definition('CQ11')).toBeUndefined();expect(definition('TK008')).toBeUndefined();
  expect(definition('BQ026').additionalKinds).toEqual(['attachment']);
});
it('renders the replacement Phoebe with her actual subtitle and whole printed rule text',()=>{
  const d=definition('CQ07');
  expect(d.unique).toBe(true);expect(d.subtitle).toBe('巡查使');
  render(<CardTile card={{...testCard,instanceId:'phoebe',cardId:d.id,name:d.name,kind:'character',faceDown:false,region:0,text:d.text,icons:d.temporaryIcons,defense:1}} definition={d} viewerId="p0" />);
  expect(screen.getAllByText(/菲碧·π/).length).toBeGreaterThan(0);
  expect(screen.getAllByText(/本方吸血鬼或法师角色/).length).toBeGreaterThan(0);
});
it('renders attached bats as both printed types with current character stats and markers',()=>{
  const d=definition('BQ026');const card={...testCard,instanceId:'bat',cardId:d.id,name:d.name,kind:'attachment',faceDown:false,region:0,icons:{investigation:0,combat:2,influence:0},defense:2,permanentCombatMarkers:1};
  render(<CardTile card={card} definition={d} viewerId="p0" />);
  expect(screen.getByText(/角色／附属/)).toBeInTheDocument();expect(screen.getByText('当前防御 2 · 印刷防御 1')).toBeInTheDocument();expect(screen.getByText('永久战斗图标标志 1')).toBeInTheDocument();
});
it('shows society warrant naming and barrier without requiring a region',()=>{
  const d=definition('XQ26');const card={...testCard,instanceId:'warrant',cardId:d.id,name:d.name,kind:'attachment',faceDown:false,region:undefined,namedCharacter:'穿刺公弗拉德三世',currentBarrier:true};
  render(<CardTile card={card} definition={d} viewerId="p0" />);expect(screen.getByText('屏障')).toBeInTheDocument();expect(screen.getByText('通缉：穿刺公弗拉德三世')).toBeInTheDocument();
});
it('strips new dynamic fields from hidden and asset projections including the controller view',()=>{
  for(const kind of ['hidden','asset'])for(const viewer of ['p0','p1']){
    const projected=visibleCard({...testCard,kind,faceDown:kind==='hidden',controller:'p0',namedCharacter:'private name',permanentDefenseMarkers:3,currentEnemySpellProtection:true},viewer);
    expect(projected.namedCharacter).toBeUndefined();expect(projected.permanentDefenseMarkers).toBeUndefined();expect(projected.currentEnemySpellProtection).toBeUndefined();
  }
});
it('displays independent permanent defense and spell-only target protection',()=>{
  const d=definition('BQ079');render(<CardTile card={{...testCard,cardId:d.id,name:d.name,kind:'character',faceDown:false,region:0,permanentDefenseMarkers:2,currentEnemySpellProtection:true}} definition={d} viewerId="p0" />);
  expect(screen.getByText('永久防御标志 2')).toBeInTheDocument();expect(screen.getByText('不能成为敌方事务目标')).toBeInTheDocument();
});
it('submits a real inspection before showing a front and reads remembered information without another command',()=>{
  const monitor={...testCard,instanceId:'monitor',cardId:'JC057',name:'监控员',controller:'p0',region:0,faceDown:false};
  const hidden={...testCard,instanceId:'hidden',cardId:undefined,name:'暗藏者',controller:'p1',owner:'p1',kind:'hidden',region:0,faceDown:true,text:undefined,icons:undefined};
  const inspect={id:'inspect',kind:'inspect-hidden',cardId:'monitor',targetId:'hidden',label:'监控员：检视暗藏者 [hidden]'};
  const view={...testView,status:'playing',hand:[],assets:[],pendingChoice:null,waitingChoice:null,privateInspections:undefined,regions:[{id:'region:0',index:0,cardId:'DQJC115',name:'死者之城',threshold:3,points:1,influence:[0,0],characters:[monitor,hidden]}],legalActions:[inspect]};
  const submit=vi.fn();const {rerender}=render(<Table view={view} catalog={testCatalog} busy={false} connection="online" onAction={submit} />);
  fireEvent.click(screen.getByRole('button',{name:inspect.label}));expect(submit).toHaveBeenCalledTimes(1);expect(submit).toHaveBeenCalledWith(inspect);expect(screen.queryByText('袖中毒物')).not.toBeInTheDocument();
  const d=definition('JZ26');const front={...hidden,cardId:d.id,name:d.name,faceDown:false,region:undefined,text:d.text};
  rerender(<Table view={{...view,version:view.version+1,privateInspections:[{sourceInstanceId:'monitor',region:0,cards:[front]}]}} catalog={testCatalog} busy={false} connection="online" onAction={submit} />);
  fireEvent.click(screen.getByRole('button',{name:/查看已检视暗藏者 .*监控员/}));expect(within(screen.getByRole('dialog')).getByText('袖中毒物')).toBeInTheDocument();expect(submit).toHaveBeenCalledTimes(1);
});
it('sends inspection as an information command in every response decision state',()=>{
  for(const status of ['undecided','composing','passed']){
    const view={...testView,serverNowMs:0,responseWindow:{id:'window',stackTopId:'top',holderTeam:0,members:[{playerId:testView.you,status}],canBegin:true,myIntentId:status==='composing'?'intent':undefined}};
    const action={kind:'inspect-hidden',cardId:'monitor',targetId:'hidden'};expect(actionForRoom(view,action)).toEqual({kind:'game',action});
  }
});
it('keeps normal and extra-paid guard plays visibly distinct while preserving every option payload',()=>{
  const actions=[0,1].flatMap(region=>[false,true].map(extra=>({id:`${region}-${extra}`,kind:'deploy',cardId:'guard',region,...(extra?{option:'pay-extra-1'}:{}),label:`派遣 赤鳞近卫 → 地区${region}${extra?'（额外支付1）':''}`})));
  const groups=groupObjectActions(actions);expect(groups).toHaveLength(2);expect(groups.map(g=>g.label)).toEqual(['派遣 赤鳞近卫','派遣 赤鳞近卫（额外支付1）']);expect(groups[1].actions.every(a=>a.option==='pay-extra-1')).toBe(true);
});
