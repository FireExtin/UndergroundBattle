import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { Lobby } from './Lobby';
import { RoomLobby } from './Table';
import { BotMatchNotice } from './BotMatchNotice';
import { RoomManagement } from './RoomManagement';
import { testCatalog, testView } from './testFixtures';
import type { Catalog, View } from './types';
const catalog: Catalog = {...testCatalog,botPractice:{difficulties:['easy','normal','hard'],hardAdvantages:['peekHumanHands']}};
const open = () => fireEvent.click(within(screen.getByRole('navigation',{name:'大厅导航'})).getByRole('button',{name:'入席'}));
beforeEach(()=>{localStorage.clear();history.replaceState({},'','/');});
afterEach(()=>{localStorage.clear();history.replaceState({},'','/');});
it.each(['easy','normal','hard'])('explicit user selection sends %s only for Bot practice',difficulty=>{
 const create=vi.fn();render(<Lobby catalog={catalog} busy={false} onCreate={create} onJoin={vi.fn()} retry={vi.fn()}/>);open();
 fireEvent.change(screen.getByRole('combobox',{name:'对局类型'}),{target:{value:'bot'}});
 fireEvent.change(screen.getByRole('combobox',{name:'Bot难度'}),{target:{value:difficulty}});
 if(difficulty==='hard')expect(screen.getByRole('note')).toHaveTextContent('读取你的当前手牌');
 fireEvent.click(screen.getByRole('button',{name:'创建Bot练习 →'}));
 expect(create).toHaveBeenCalledWith(expect.any(String),'duel',expect.any(String),{roomKind:'bot',difficulty});
});
it('returning to a human room omits every Bot option',()=>{
 const create=vi.fn();render(<Lobby catalog={catalog} busy={false} onCreate={create} onJoin={vi.fn()} retry={vi.fn()}/>);open();
 fireEvent.change(screen.getByRole('combobox',{name:'对局类型'}),{target:{value:'bot'}});fireEvent.change(screen.getByRole('combobox',{name:'Bot难度'}),{target:{value:'hard'}});
 fireEvent.change(screen.getByRole('combobox',{name:'对局类型'}),{target:{value:'human'}});fireEvent.click(screen.getByRole('button',{name:'创建牌桌 →'}));
 expect(create.mock.calls[0]).toHaveLength(3);expect(screen.queryByRole('combobox',{name:'Bot难度'})).toBeNull();
});
it('unsupported service and invite-only entry do not offer Bot creation',()=>{
 const a=render(<Lobby catalog={testCatalog} busy={false} onCreate={vi.fn()} onJoin={vi.fn()} retry={vi.fn()}/>);open();expect(screen.queryByRole('combobox',{name:'Bot难度'})).toBeNull();a.unmount();
 history.replaceState({},'','/?invite=EXISTING');render(<Lobby catalog={catalog} busy={false} onCreate={vi.fn()} onJoin={vi.fn()} retry={vi.fn()}/>);expect(screen.queryByRole('combobox',{name:'对局类型'})).toBeNull();
});
it('capability removal blocks submission instead of silently creating a human room',()=>{
 const create=vi.fn();const app=render(<Lobby catalog={catalog} busy={false} onCreate={create} onJoin={vi.fn()} retry={vi.fn()}/>);open();fireEvent.change(screen.getByRole('combobox',{name:'对局类型'}),{target:{value:'bot'}});
 app.rerender(<Lobby catalog={testCatalog} busy={false} onCreate={create} onJoin={vi.fn()} retry={vi.fn()}/>);expect(screen.getByRole('button',{name:'创建Bot练习 →'})).toBeDisabled();expect(create).not.toHaveBeenCalled();
});
const botView: View={...testView,botMatch:{roomKind:'bot',difficulty:'hard',advantages:['peekHumanHands'],description:'困难Bot可以读取你的当前手牌，用于选择合法行动。',enabled:true}};
it('server-confirmed difficulty and exact advantage remain visible after view recovery',()=>{
 const app=render(<BotMatchNotice view={JSON.parse(JSON.stringify(botView))}/>);expect(screen.getByRole('region',{name:'Bot对局规则'})).toHaveTextContent('困难');expect(screen.getByText(/读取你的当前手牌/)).toBeVisible();
 app.rerender(<BotMatchNotice view={{...testView}}/>);expect(screen.queryByRole('region',{name:'Bot对局规则'})).toBeNull();
});
it('Bot lobby removes invitations and reports a disabled scheduler without private data',()=>{
 render(<><BotMatchNotice view={{...botView,botMatch:{...botView.botMatch!,enabled:false}}}/><RoomLobby view={{...botView,status:'lobby'}} catalog={catalog} busy={false} onAction={vi.fn()}/></>);
 expect(screen.queryByRole('button',{name:/复制邀请链接/})).toBeNull();expect(screen.getByRole('status')).toHaveTextContent('关闭Bot调度');expect(screen.queryByText(/9007199254740993/)).toBeNull();
});
it('an active Bot match offers local seat recovery and pause without an unusable invite',()=>{
 render(<RoomManagement view={{...botView,canPause:true,pause:{pausedAtMs:1,pausedBy:0}}} blocked={false} onAction={vi.fn()} refresh={vi.fn()}/>);
 expect(screen.queryByRole('button',{name:'复制此桌邀请链接'})).toBeNull();
 expect(screen.getByText(/Bot房不能通过邀请码加入/)).toBeVisible();
 expect(screen.getByRole('button',{name:'恢复对局'})).toBeEnabled();
});
