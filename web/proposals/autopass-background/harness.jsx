// Purpose: exercise the released hook and AutoPass with a synthetic server and fake clock.
// No live URL, real room, browser credential, DOM/state injection into a public game, or core fixture.
import { act, render } from '@testing-library/react';
import { expect, vi } from 'vitest';
import { AutoPass } from '../../src/game/AutoPass';
import { saveSession } from '../../src/game/api';
import { testCatalog, testView } from '../../src/game/testFixtures';
import { useGame } from '../../src/game/useGame';

export const pass = { id: 'synthetic-pass', kind: 'pass', label: '让过' };
export const playing = { ...testView, status: 'playing', legalActions: [] };
const session = { roomId: testView.roomId, inviteCode: 'SYNTHETIC', token: 'synthetic-not-a-real-token', seat: 0 };
const json = body => new Response(JSON.stringify(body), { status: 200 });
export const advance = ms => act(async () => { await vi.advanceTimersByTimeAsync(ms); });

export async function mountSynthetic({ view = playing, enabled = false, hidden = true, active = true, command, poll } = {}) {
  const visibility = vi.spyOn(document, 'visibilityState', 'get').mockReturnValue(hidden ? 'hidden' : 'visible');
  if (active) saveSession(session);
  localStorage.setItem('hegemony.autoPass.v1', String(enabled));
  const trace = [];
  const server = { view };
  vi.stubGlobal('fetch', vi.fn(async (url, init) => {
    if (url.endsWith('/catalog')) return json(testCatalog);
    const timeMs = Date.now();
    if (url.endsWith('/commands')) {
      const body = JSON.parse(String(init?.body));
      trace.push({ kind: 'command', timeMs, ...body });
      if (command) return command(body, server);
      server.view = { ...server.view, version: server.view.version + 1, legalActions: [] };
      return json(server.view);
    }
    const after = new URL(url, 'http://synthetic.invalid').searchParams.get('afterVersion');
    trace.push({ kind: after === null ? 'initial-state' : 'poll', timeMs, afterVersion: after === null ? null : Number(after) });
    if (poll && after !== null) return poll(server, Number(after));
    return after !== null && server.view.version <= Number(after) ? new Response(null, { status: 204 }) : json(server.view);
  }));
  let current;
  function Harness() {
    current = useGame();
    return current.view && current.view.status !== 'lobby'
      ? <AutoPass view={current.view} busy={current.busy} uncertain={current.uncertain} connection={current.connection} onAction={current.act} />
      : null;
  }
  const mounted = render(<Harness />);
  await advance(0);
  if (active) expect(current.view?.version).toBe(view.version);
  return {
    ...mounted, trace, server, visibility, current: () => current,
    polls: () => trace.filter(event => event.kind === 'poll'),
    commands: () => trace.filter(event => event.kind === 'command'),
    json,
  };
}
