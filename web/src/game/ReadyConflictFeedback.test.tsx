import { act, cleanup, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { readPending, saveSession } from './api';
import { testCatalog, testView } from './testFixtures';
import type { Action, View } from './types';
import { useGame } from './useGame';

const seat = { roomId: testView.roomId, seat: 0, token: 'feedback-token', inviteCode: 'INVITE' };
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); vi.useFakeTimers(); saveSession(seat); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });
async function conflict(action: Action, latest?: View, code = 'version_conflict') {
  const commands: unknown[] = [];
  vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
    if (url.endsWith('/commands')) { commands.push(JSON.parse(String(init?.body))); return json({ error: code, ...(latest ? { view: latest } : {}) }, 409); }
    if (url.endsWith('/catalog')) return json(testCatalog);
    if (commands.length && !latest) return json({ message: 'sync unavailable' }, 503);
    return json({ ...testView, serverNowMs: 1000 });
  }));
  const hook = renderHook(useGame); await act(async () => {});
  await act(async () => { await hook.result.current.act(action); });
  expect(commands).toHaveLength(1); // Feedback never creates a new retry identity.
  expect(readPending()).toBeNull();
  expect(hook.result.current.uncertain).toBe(false);
  return hook.result.current;
}
describe('lobby ready conflict feedback without rebased retries', () => {
  it('explains that the seat is still unready and leaves the ready button for a manual click', async () => {
    const state = await conflict({ kind: 'ready' }, { ...testView, version: 2 });
    expect(state.error).toContain('你当前尚未准备'); expect(state.error).toContain('再点“准备”');
  });
  it('reports already-ready state without falsely saying the rejected toggle executed', async () => {
    const state = await conflict({ kind: 'ready' }, { ...testView, version: 2, players: [{ ...testView.players[0], ready: true }] });
    expect(state.error).toContain('你当前已准备'); expect(state.error).toContain('取消准备');
  });
  it.each(['playing', 'finished'] as const)('uses the actual %s room state instead of requesting readiness', async status => {
    const state = await conflict({ kind: 'ready' }, { ...testView, version: 2, status });
    expect(state.error).toContain(status === 'playing' ? '牌桌已开始' : '牌桌已结束');
    expect(state.error).not.toContain('再点“准备”'); expect(state.view?.status).toBe(status);
  });
  it('does not claim successful synchronization when the conflict has no view and synchronization fails', async () => {
    const state = await conflict({ kind: 'ready' });
    expect(state.error).toContain('查看最新牌桌'); expect(state.error).not.toContain('已同步');
  });
  it('keeps a different game action on the existing generic conflict path', async () => {
    const state = await conflict({ kind: 'play', cardId: 'paid-source' }, { ...testView, version: 2 });
    expect(state.error).toContain('当前可用行动'); expect(state.error).not.toContain('准备状态');
  });
  it('keeps other rejection codes on the existing conflict path even for a ready-shaped action', async () => {
    const state = await conflict({ kind: 'ready' }, { ...testView, version: 2 }, 'room_paused');
    expect(state.error).toContain('当前可用行动'); expect(state.error).not.toContain('准备状态');
  });
});
