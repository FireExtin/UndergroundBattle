import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { GameApp } from './GameApp';
import { testCatalog, testView } from './testFixtures';
import type { SavedSession } from './types';

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
let seats: SavedSession[];
let posts: { url: string; body: Record<string, unknown>; authorization?: string }[];
let reads: { url: string; authorization?: string }[];
let rejectEntry: boolean;
let rejectCommand: boolean;
let playing: boolean;
beforeEach(() => {
  localStorage.clear(); sessionStorage.clear(); history.replaceState({}, '', '/');
  seats = []; posts = []; reads = []; rejectEntry = false; rejectCommand = false; playing = false;
  vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
    const authorization = new Headers(init?.headers).get('Authorization') || undefined;
    if (init?.method === 'POST') {
      const body = JSON.parse(String(init.body)); posts.push({ url, body, authorization });
      if (url.endsWith('/commands')) return rejectCommand ? json({ message: 'lost acknowledgement' }, 503) : json(viewFor(authorization));
      if (rejectEntry) return json({ message: 'entry temporarily unavailable' }, 503);
      const code = String(body.inviteCode || 'INVITE').toUpperCase();
      const seat = seats.filter(s => s.inviteCode === code).length;
      const saved = { roomId: `room-${code}`, inviteCode: code, token: `test-opaque-${seats.length}`, seat };
      seats.push(saved); return json({ ...saved, view: viewFor(`Bearer ${saved.token}`) });
    }
    reads.push({ url, authorization });
    return url.endsWith('/catalog') ? json({ ...testCatalog, entryIdempotency: true }) : json(viewFor(authorization));
  }));
});
afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });
function viewFor(authorization?: string) {
  const saved = seats.find(s => authorization === `Bearer ${s.token}`) || seats.at(-1);
  return { ...testView, status: playing ? 'playing' : 'lobby', legalActions: playing ? [] : testView.legalActions,
    roomId: saved?.roomId || 'room-INVITE', inviteCode: saved?.inviteCode || 'INVITE',
    you: `p${saved?.seat || 0}`, players: seats.filter(s => s.roomId === saved?.roomId).map(s => ({ ...testView.players[0], id: `p${s.seat}`, seat: s.seat })) };
}
async function lobbyReady() { await waitFor(() => expect(screen.getByRole('button', { name: '创建牌桌 →' })).toBeInTheDocument()); }
async function create() {
  await lobbyReady(); fireEvent.change(screen.getByLabelText('你的称呼'), { target: { value: '测试玩家' } });
  fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
  if (playing) await screen.findByRole('switch', { name: '无可用行动时自动让过' });
  else await screen.findByText('等待秘社集结');
}
async function lobby() { fireEvent.click(screen.getByRole('button', { name: '返回大厅 / 新建牌桌' })); await lobbyReady(); }
async function independent() { fireEvent.click(screen.getByRole('button', { name: '开始新的独立玩家会话' })); await screen.findByText('独立玩家会话 · 当前标签'); }
async function join(code = 'INVITE') {
  fireEvent.click(screen.getByRole('button', { name: '邀请码加入' }));
  fireEvent.change(screen.getByLabelText('你的称呼'), { target: { value: '测试玩家' } });
  fireEvent.change(screen.getByLabelText('邀请码'), { target: { value: code } });
  await waitFor(() => expect(screen.getByRole('button', { name: '加入牌桌 →' })).toBeEnabled());
  fireEvent.click(screen.getByRole('button', { name: '加入牌桌 →' }));
  if (playing) await screen.findByRole('switch', { name: '无可用行动时自动让过' });
  else await screen.findByText('等待秘社集结');
}
const entries = () => posts.filter(p => !p.url.endsWith('/commands'));

describe('explicit independent player sessions through normal UI', () => {
  it('starts a new tab’s independent seat directly from the ordinary table and preserves the ordinary reload screen', async () => {
    playing = true;
    let app = render(<GameApp />); await create();
    const ordinary = localStorage.getItem('hegemony.session.v1');
    const history = localStorage.getItem('hegemony.seats.v1');
    expect(localStorage.getItem('hegemony.screen.v1')).toBe('table');
    app.unmount();
    // A fresh second tab sees the same ordinary localStorage and its own empty page session.
    app = render(<GameApp />); await screen.findByRole('switch', { name: '无可用行动时自动让过' });
    expect(screen.getByRole('button', { name: '开始新的独立玩家会话' })).toBeEnabled();
    expect(screen.getByText('只切换当前标签；普通玩家的座位和牌桌恢复方式保留。')).toBeInTheDocument();
    await independent(); await lobbyReady();
    expect(localStorage.getItem('hegemony.screen.v1')).toBe('table');
    await join();
    expect(seats.map(s => s.seat)).toEqual([0, 1]);
    expect(localStorage.getItem('hegemony.session.v1')).toBe(ordinary);
    expect(localStorage.getItem('hegemony.seats.v1')).toBe(history);
    expect(localStorage.getItem('hegemony.screen.v1')).toBe('table');
    expect(posts.filter(p => p.url.endsWith('/commands'))).toHaveLength(0);
    app.unmount(); sessionStorage.clear(); // Model returning to the ordinary tab’s page session.
    app = render(<GameApp />); await screen.findByRole('switch', { name: '无可用行动时自动让过' });
    await waitFor(() => expect(reads.at(-1)?.authorization).toBe(`Bearer ${seats[0].token}`));
    expect(screen.queryByRole('button', { name: '创建牌桌 →' })).not.toBeInTheDocument();
    expect(entries()).toHaveLength(2);
  });
  it('locks the direct ordinary-table switch while a command is pending and keeps that lock after lobby return', async () => {
    render(<GameApp />); await create(); rejectCommand = true;
    fireEvent.click(screen.getByRole('button', { name: '准备' }));
    const tableSwitch = screen.getByRole('button', { name: '开始新的独立玩家会话' });
    expect(tableSwitch).toBeDisabled();
    await screen.findByRole('button', { name: '确认上一行动' });
    await waitFor(() => expect(screen.getByRole('button', { name: '返回大厅 / 新建牌桌' })).toBeEnabled());
    expect(tableSwitch).toBeDisabled();
    const pending = localStorage.getItem('hegemony.pending.v1');
    await lobby();
    const lobbySwitch = screen.getByRole('button', { name: '开始新的独立玩家会话' });
    expect(lobbySwitch).toBeDisabled(); fireEvent.click(lobbySwitch);
    expect(localStorage.getItem('hegemony.pending.v1')).toBe(pending);
    expect(localStorage.getItem('hegemony.screen.v1')).toBe('lobby');
    expect(sessionStorage.getItem('hegemony.playerSession.v1')).toBeNull();
    expect(entries()).toHaveLength(1);
  });
  it('warns before replacing an independent session or switching back that this seat may become unrecoverable', async () => {
    render(<GameApp />); await lobbyReady(); await independent(); await join(); await lobby();
    expect(screen.getByText('开始新会话或切回普通后，此独立席位可能无法恢复；需要多席请另开标签。')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '开始新的独立玩家会话' })).toBeEnabled();
    expect(screen.getByRole('button', { name: '切回普通玩家会话' })).toBeEnabled();
    fireEvent.click(screen.getByRole('button', { name: '切回普通玩家会话' }));
    expect(screen.queryByRole('button', { name: '回到牌桌 INVITE · 席位 1' })).not.toBeInTheDocument();
    expect(entries()).toHaveLength(1);
  });
  it('keeps automatic pass opt-in separate from the ordinary seat and restores the ordinary preference', async () => {
    playing = true; render(<GameApp />); await lobbyReady();
    fireEvent.change(screen.getByLabelText('你的称呼'), { target: { value: '测试玩家' } });
    fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
    const ordinarySwitch = await screen.findByRole('switch', { name: '无可用行动时自动让过' });
    fireEvent.click(ordinarySwitch); expect(ordinarySwitch).toBeChecked();
    await lobby(); await independent();
    fireEvent.click(screen.getByRole('button', { name: '邀请码加入' }));
    fireEvent.change(screen.getByLabelText('你的称呼'), { target: { value: '测试玩家' } });
    fireEvent.change(screen.getByLabelText('邀请码'), { target: { value: 'INVITE' } });
    await waitFor(() => expect(screen.getByRole('button', { name: '加入牌桌 →' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: '加入牌桌 →' }));
    expect(await screen.findByRole('switch', { name: '无可用行动时自动让过' })).not.toBeChecked();
    await lobby(); fireEvent.click(screen.getByRole('button', { name: '切回普通玩家会话' }));
    fireEvent.click(await screen.findByRole('button', { name: '回到牌桌 INVITE · 席位 1' }));
    expect(await screen.findByRole('switch', { name: '无可用行动时自动让过' })).toBeChecked();
    expect(posts.filter(p => p.url.endsWith('/commands'))).toHaveLength(0);
  });
  it('isolates same-invite seats, refreshes the current identity and returns to the ordinary saved seat', async () => {
    let app = render(<GameApp />); await create(); const ordinary = localStorage.getItem('hegemony.session.v1');
    await lobby(); await independent(); expect(entries()).toHaveLength(1);
    await join(); expect(seats.map(s => s.seat)).toEqual([0, 1]);
    expect(localStorage.getItem('hegemony.session.v1')).toBe(ordinary);
    app.unmount(); app = render(<GameApp />); await screen.findByText('等待秘社集结');
    await waitFor(() => expect(reads.at(-1)?.authorization).toBe(`Bearer ${seats[1].token}`));
    await lobby(); await join(' invite '); expect(entries()).toHaveLength(2);
    await lobby(); fireEvent.click(screen.getByRole('button', { name: '切回普通玩家会话' }));
    fireEvent.click(await screen.findByRole('button', { name: '回到牌桌 INVITE · 席位 1' }));
    await screen.findByText('等待秘社集结');
    await waitFor(() => expect(reads.at(-1)?.authorization).toBe(`Bearer ${seats[0].token}`));
    expect(entries()).toHaveLength(2);
    expect(reads.every(r => !r.url.includes('test-opaque') && !r.url.includes('player-session'))).toBe(true);
  });
  it('allocates no identity on refresh or lobby return; only an explicit new session can join another seat', async () => {
    let app = render(<GameApp />); await lobbyReady(); await independent(); await join();
    app.unmount(); app = render(<GameApp />); await screen.findByText('等待秘社集结');
    expect(entries()).toHaveLength(1);
    await lobby(); await join(); expect(entries()).toHaveLength(1);
    await lobby(); await independent(); expect(entries()).toHaveLength(1); await join();
    expect(entries()).toHaveLength(2); expect(seats.map(s => s.seat)).toEqual([0, 1]);
  });
  it('uses a fresh entry receipt in a new scope and keeps identical retries inside that scope', async () => {
    render(<GameApp />); await lobbyReady(); rejectEntry = true;
    fireEvent.change(screen.getByLabelText('你的称呼'), { target: { value: '测试玩家' } });
    fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' })); await screen.findByRole('alert');
    const ordinaryId = entries()[0].body.requestId; expect(entries()[1].body.requestId).toBe(ordinaryId);
    await independent(); await lobbyReady();
    fireEvent.change(screen.getByLabelText('你的称呼'), { target: { value: '测试玩家' } });
    await waitFor(() => expect(screen.getByRole('button', { name: '创建牌桌 →' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' })); await screen.findByRole('alert');
    expect(entries()[2].body.requestId).not.toBe(ordinaryId); expect(entries()[3].body).toEqual(entries()[2].body);
    rejectEntry = false; fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
    await screen.findByText('等待秘社集结'); expect(entries()[4].body).toEqual(entries()[2].body);
  });
  it('blocks switching scopes while an original paid/action command remains unconfirmed, including in the lobby', async () => {
    render(<GameApp />); await create(); rejectCommand = true;
    fireEvent.click(screen.getByRole('button', { name: '准备' }));
    await screen.findByRole('button', { name: '确认上一行动' });
    await waitFor(() => expect(screen.getByRole('button', { name: '返回大厅 / 新建牌桌' })).toBeEnabled());
    const pending = localStorage.getItem('hegemony.pending.v1'); expect(pending).not.toBeNull();
    await lobby(); fireEvent.click(screen.getByRole('button', { name: '开始新的独立玩家会话' }));
    expect(screen.queryByText('独立玩家会话 · 当前标签')).not.toBeInTheDocument();
    expect(localStorage.getItem('hegemony.pending.v1')).toBe(pending); expect(entries()).toHaveLength(1);
  });
  it('refreshes an independent unconfirmed command with the original actor and command ID', async () => {
    let app = render(<GameApp />); await lobbyReady(); await independent(); await join();
    rejectCommand = true; fireEvent.click(screen.getByRole('button', { name: '准备' }));
    await screen.findByRole('button', { name: '确认上一行动' });
    await waitFor(() => expect(screen.getByRole('button', { name: '返回大厅 / 新建牌桌' })).toBeEnabled());
    const originals = posts.filter(p => p.url.endsWith('/commands'));
    expect(originals).toHaveLength(2); expect(originals[1]).toEqual(originals[0]);
    expect(localStorage.getItem('hegemony.pending.v1')).toBeNull();
    app.unmount(); rejectCommand = false; app = render(<GameApp />);
    await screen.findByText('等待秘社集结');
    await waitFor(() => expect(posts.filter(p => p.url.endsWith('/commands'))).toHaveLength(3));
    expect(posts.filter(p => p.url.endsWith('/commands'))[2]).toEqual(originals[0]);
    await waitFor(() => expect(screen.queryByRole('button', { name: '确认上一行动' })).not.toBeInTheDocument());
    expect(entries()).toHaveLength(1);
  });
  it('ignores other-tab storage events and joins a different invitation only with its own credentials', async () => {
    render(<GameApp />); await create(); await lobby(); await independent(); await join();
    const count = reads.length;
    window.dispatchEvent(new StorageEvent('storage', { key: 'hegemony.session.v1', newValue: 'another-tab-event', storageArea: localStorage }));
    await lobby(); await join('OTHER');
    expect(entries()).toHaveLength(3); expect(seats.at(-1)?.inviteCode).toBe('OTHER');
    expect(reads.slice(count).filter(r => r.authorization).every(r => r.authorization !== `Bearer ${seats[0].token}`)).toBe(true);
  });
  it('reports unavailable tab storage without copying or clearing ordinary credentials', async () => {
    render(<GameApp />); await create(); await lobby(); const ordinary = localStorage.getItem('hegemony.session.v1');
    const original = Storage.prototype.setItem;
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (this: Storage, key, value) {
      if (this === sessionStorage) throw new DOMException('blocked', 'SecurityError'); original.call(this, key, value);
    });
    fireEvent.click(screen.getByRole('button', { name: '开始新的独立玩家会话' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('独立玩家会话');
    expect(localStorage.getItem('hegemony.session.v1')).toBe(ordinary); expect(entries()).toHaveLength(1);
  });
  it('makes closing the independent page session lose its recovery while ordinary seats survive', async () => {
    const app = render(<GameApp />); await create(); await lobby(); await independent(); await join();
    expect(screen.getByText(/关闭标签可能失去此独立座位的恢复方式/)).toBeInTheDocument();
    app.unmount(); sessionStorage.clear(); // Model the browser ending the page session, never seed credentials.
    render(<GameApp />); await lobbyReady();
    expect(await screen.findByRole('button', { name: '回到牌桌 INVITE · 席位 1' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '回到牌桌 INVITE · 席位 2' })).not.toBeInTheDocument();
    expect(entries()).toHaveLength(2);
  });
});
