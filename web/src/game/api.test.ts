import { afterEach, describe, expect, it, vi } from 'vitest';
import { actionPayload, ApiError, createRoom, createRoomWithDeck, getCatalog, getState, pollState, readSession, saveSession, sendCommand, streamEvents } from './api';
import { testCatalog, testView } from './testFixtures';

const session = { roomId: 'room-test', inviteCode: 'INVITE', token: 'opaque-seat-token', seat: 0 };
afterEach(() => { vi.unstubAllGlobals(); localStorage.clear(); });
describe('cloud table API', () => {
  it('reads the room catalog with the original seat credential outside the URL', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify(testCatalog), { status: 200 }));
    vi.stubGlobal('fetch', fetchMock);
    expect(await getCatalog(undefined, session)).toEqual(testCatalog);
    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe('/api/rooms/room-test/catalog');
    expect(url).not.toContain(session.token);
    expect(init.headers.Authorization).toBe(`Bearer ${session.token}`);
  });
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
  it('forwards an authoritative ability and sacrifice cost selection without display metadata', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify(testView), { status: 200 }));
    vi.stubGlobal('fetch', fetchMock);
    await sendCommand(session, 8, {
      id: 'fast-response', label: '牺牲角色并响应', description: '服务端合法费用', kind: 'activate',
      cardId: 'source-instance', abilityId: 'quick-response', targetId: 'target-instance', costSelected: ['sacrificed-instance'],
    } as Parameters<typeof actionPayload>[0], 'response-command');
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual({
      commandId: 'response-command', expectedVersion: 8,
      action: { kind: 'activate', cardId: 'source-instance', abilityId: 'quick-response', targetId: 'target-instance', costSelected: ['sacrificed-instance'] },
    });
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
  it('polls only an authenticated version and accepts an unchanged 204', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(null, { status: 204 }));
    vi.stubGlobal('fetch', fetchMock);
    const controller = new AbortController();
    expect(await pollState(session, 17, controller.signal)).toBeNull();
    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe('/api/rooms/room-test/state?afterVersion=17');
    expect(init.headers.Authorization).toBe(`Bearer ${session.token}`);
    expect(init.cache).toBe('no-store');
    controller.abort(); expect(init.signal.aborted).toBe(true);
  });
  it('rejects a poll carrying another room or an unsafe version', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(JSON.stringify({ ...testView, roomId: 'other' }), { status: 200 })));
    await expect(pollState(session, 1, new AbortController().signal)).rejects.toMatchObject({ status: 0 });
  });
  it('recovers a lost lobby acknowledgement with the exact original request key', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(new Response(JSON.stringify({ entryIdempotency: true }), { status: 200 }))
      .mockRejectedValueOnce(new Error('lost ACK')).mockResolvedValueOnce(new Response(JSON.stringify(session), { status: 200 }));
    vi.stubGlobal('fetch', fetchMock); await getCatalog();
    expect(await createRoom('甲', 'duel', 'responders')).toEqual(session);
    expect(fetchMock.mock.calls[1][1].body).toBe(fetchMock.mock.calls[2][1].body);
    expect(JSON.parse(fetchMock.mock.calls[1][1].body).requestId.length).toBeGreaterThanOrEqual(32);
    expect(localStorage.getItem('hegemony.entry.v1')).toBeNull();
  });
  it('keeps a failed entry key for an explicit retry and does not retry legacy servers automatically', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(new Response('{}', { status: 200 }))
      .mockRejectedValueOnce(new Error('lost ACK')).mockResolvedValueOnce(new Response(JSON.stringify(session), { status: 200 }));
    vi.stubGlobal('fetch', fetchMock); await getCatalog();
    await expect(createRoom('乙', 'duel', 'watchers')).rejects.toMatchObject({ status: 0 });
    expect(fetchMock).toHaveBeenCalledTimes(2);
    const recovery = localStorage.getItem('hegemony.entry.v1'); expect(recovery).not.toBeNull();
    await createRoom('乙', 'duel', 'watchers');
    expect(fetchMock.mock.calls[1][1].body).toBe(fetchMock.mock.calls[2][1].body);
  });
  it('recovers a custom deck entry with the original draft and request identity', async () => {
    const draft = { id: 'draft-a', name: '我的牌组', description: '', societyId: null,
      cards: [{ cardId: 'JC125', count: 50 }], rulesVersion: 'rules1', cardPoolVersion: 'pool1', engineVersion: 'engine1', updatedAt: '2026-10-02T16:00:00Z' };
    const fetchMock = vi.fn().mockResolvedValueOnce(new Response(JSON.stringify({ entryIdempotency: true }), { status: 200 }))
      .mockRejectedValueOnce(new Error('lost ACK')).mockResolvedValueOnce(new Response(JSON.stringify(session), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify(testView), { status: 200 }));
    vi.stubGlobal('fetch', fetchMock); await getCatalog();
    expect(await createRoomWithDeck('构筑玩家', 'teams', draft)).toEqual(session);
    expect(fetchMock.mock.calls[1][1].body).toBe(fetchMock.mock.calls[2][1].body);
    expect(JSON.parse(fetchMock.mock.calls[1][1].body)).toMatchObject({ name: '构筑玩家', mode: 'teams', deckDraft: draft });
    expect(JSON.parse(fetchMock.mock.calls[1][1].body)).not.toHaveProperty('deckId');
    expect(localStorage.getItem('hegemony.entry.v1')).toBeNull();
    await sendCommand(session, 7, { kind: 'deck', deckDraft: draft }, 'draft-command');
    expect(JSON.parse(fetchMock.mock.calls[3][1].body)).toEqual({ commandId: 'draft-command', expectedVersion: 7, action: { kind: 'deck', deckDraft: draft } });
  });
});
