import { afterEach, describe, expect, it, vi } from 'vitest';
import { actionPayload, ApiError, getState, readSession, saveSession, sendCommand, streamEvents } from './api';
import { testView } from './testFixtures';

const session = { roomId: 'room-test', inviteCode: 'INVITE', token: 'opaque-seat-token', seat: 0 };
afterEach(() => { vi.unstubAllGlobals(); localStorage.clear(); });
describe('cloud table API', () => {
  it('stores an opaque seat credential locally without storing a hidden table snapshot', () => {
    saveSession({ ...session, view: testView } as typeof session);
    expect(readSession()).toEqual(session);
    expect(localStorage.getItem('hegemony.session.v1')).not.toContain('无知路人');
    localStorage.setItem('hegemony.session.v1', '{corrupt');
    expect(readSession()).toBeNull();
  });
  it('sends a bearer token and canonical authoritative action with command identity and version', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify(testView), { status: 200 }));
    vi.stubGlobal('fetch', fetchMock);
    await sendCommand(session, 7, actionPayload({ id: 'ui-action', label: '派遣', kind: 'deploy', cardId: 'instance-a', region: 2 }), 'stable-command');
    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe('/api/rooms/room-test/commands'); expect(url).not.toContain(session.token);
    expect(init.headers.Authorization).toBe(`Bearer ${session.token}`);
    expect(JSON.parse(init.body)).toEqual({ commandId: 'stable-command', expectedVersion: 7, action: { kind: 'deploy', cardId: 'instance-a', region: 2 } });
  });
  it('preserves the reconciliation view on version conflict', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(JSON.stringify({ error: 'stale', view: testView }), { status: 409 })));
    await expect(getState(session)).rejects.toMatchObject({ status: 409, view: testView });
  });
  it('reads split UTF-8 and CRLF SSE frames through authenticated fetch', async () => {
    const bytes = new TextEncoder().encode(`: keepalive\r\n\r\ndata: ${JSON.stringify(testView)}\r\n\r\n`);
    const frames = [bytes.slice(0, 12), bytes.slice(12, bytes.length - 3), bytes.slice(bytes.length - 3, bytes.length - 1), bytes.slice(bytes.length - 1)];
    const body = new ReadableStream({ start(controller) { frames.forEach(frame => controller.enqueue(frame)); controller.close(); } });
    const fetchMock = vi.fn().mockResolvedValue(new Response(body, { status: 200 }));
    vi.stubGlobal('fetch', fetchMock); const receive = vi.fn();
    await streamEvents(session, new AbortController().signal, receive);
    expect(receive).toHaveBeenCalledExactlyOnceWith(testView);
    expect(fetchMock.mock.calls[0][1].headers.Authorization).toBe(`Bearer ${session.token}`);
    expect(fetchMock.mock.calls[0][0]).not.toContain('token');
  });
  it('distinguishes lost transport from a confirmed rejection', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('lost acknowledgement')));
    await expect(sendCommand(session, 1, { kind: 'pass' }, 'same-command')).rejects.toBeInstanceOf(ApiError);
    await expect(sendCommand(session, 1, { kind: 'pass' }, 'same-command')).rejects.toMatchObject({ status: 0 });
  });
});
