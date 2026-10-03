// Purpose: actual hook + controlled AutoPass under a fake clock and synthetic fetch; no live rooms.
import { act, render } from '@testing-library/react';
import { expect, vi } from 'vitest';
import { AutoPass } from './AutoPass';
import { saveSession } from './api';
import { playerStorage } from './playerStorage';
import { testCatalog, testView } from './testFixtures';
import { useGame } from './useGame';

export const pass = { id: 'synthetic-pass', kind: 'pass', label: '让过' };
export const playing = { ...testView, status: 'playing', legalActions: [] };
export const session = { roomId: testView.roomId, inviteCode: 'SYNTHETIC', token: 'synthetic-not-a-real-token', seat: 0 };
export const json = (body, status = 200) => new Response(JSON.stringify(body), { status });
export const advance = ms => act(async () => { await vi.advanceTimersByTimeAsync(ms); });
export const changeVisibility = (h, visible) => act(() => {
  h.visibility.mockReturnValue(visible ? 'visible' : 'hidden');
  document.dispatchEvent(new Event('visibilitychange'));
});

export async function mountSynthetic({ view = playing, enabled = false, hidden = true, active = true, command, read, ready = true } = {}) {
  const visibility = vi.spyOn(document, 'visibilityState', 'get').mockReturnValue(hidden ? 'hidden' : 'visible');
  if (active) saveSession(session);
  playerStorage().setItem('hegemony.autoPass.v1', String(enabled));
  const trace = []; const requests = []; const server = { view };
  vi.stubGlobal('fetch', vi.fn(async (url, init) => {
    requests.push({ url, init });
    const after = new URL(url, 'http://synthetic.invalid').searchParams.get('afterVersion');
    const authorization = new Headers(init?.headers).get('Authorization');
    if (url.endsWith('/catalog')) return json(testCatalog);
    if (url.endsWith('/commands')) {
      const body = JSON.parse(String(init?.body));
      trace.push({ kind: 'command', timeMs: Date.now(), authorization, ...body });
      if (command) return command(body, server);
      server.view = { ...server.view, version: server.view.version + 1, legalActions: [] };
      return json(server.view);
    }
    if (!url.includes('/state')) throw new Error(`Unexpected synthetic route: ${url}`);
    trace.push({ kind: after === null ? 'initial-state' : 'poll', timeMs: Date.now(), authorization,
      afterVersion: after === null ? null : Number(after) });
    if (read) {
      const response = await read({ url, init, afterVersion: after === null ? null : Number(after) }, server);
      if (response !== undefined) return response;
    }
    return after !== null && server.view.version <= Number(after) ? new Response(null, { status: 204 }) : json(server.view);
  }));
  let current;
  function Harness() {
    current = useGame();
    return current.view && current.view.status !== 'lobby'
      ? <AutoPass view={current.view} busy={current.busy} uncertain={current.uncertain} connection={current.connection}
        onAction={current.act} enabled={current.autoPassEnabled} onEnabledChange={current.setAutoPassEnabled} /> : null;
  }
  let mounted = render(<Harness />);
  await advance(0);
  if (active && ready) expect(current.view?.version).toBe(view.version);
  return { trace, requests, server, visibility, current: () => current,
    unmount: () => mounted.unmount(),
    remount: async () => { mounted.unmount(); mounted = render(<Harness />); await advance(0); },
    polls: () => trace.filter(event => event.kind === 'poll'),
    commands: () => trace.filter(event => event.kind === 'command'),
    initialReads: () => trace.filter(event => event.kind === 'initial-state'),
  };
}
