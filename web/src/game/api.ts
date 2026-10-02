import type { Action, Catalog, SavedSession, Session, View } from './types';

const STORAGE_KEY = 'hegemony.session.v1';
const PENDING_KEY = 'hegemony.pending.v1';
const ENTRY_KEY = 'hegemony.entry.v1';
export function newCommandId(): string {
  if (typeof crypto.randomUUID === 'function') return crypto.randomUUID();
  // getRandomValues also works on HTTP development hosts without randomUUID support.
  return Array.from(crypto.getRandomValues(new Uint8Array(16)), byte => byte.toString(16).padStart(2, '0')).join('');
}
export type PendingCommand = { roomId: string; commandId: string; expectedVersion: number; action: Action };
export function readPending(): PendingCommand | null {
  try {
    const value = JSON.parse(localStorage.getItem(PENDING_KEY) || 'null');
    if (value && typeof value.roomId === 'string' && typeof value.commandId === 'string' && typeof value.expectedVersion === 'number' && typeof value.action?.kind === 'string') return value;
  } catch { /* Browser storage is optional. */ }
  return null;
}
export function savePending(command: PendingCommand | null) {
  try {
    if (command) localStorage.setItem(PENDING_KEY, JSON.stringify(command));
    else localStorage.removeItem(PENDING_KEY);
  } catch { /* Preserve the in-memory command while this tab is open. */ }
}

export class ApiError extends Error {
  status: number;
  view?: View;
  constructor(status: number, message: string, view?: View) {
    super(message); this.status = status; this.view = view;
  }
}

export function readSession(): SavedSession | null {
  try {
    const value = JSON.parse(localStorage.getItem(STORAGE_KEY) || 'null');
    if (value && typeof value.roomId === 'string' && typeof value.token === 'string'
      && typeof value.inviteCode === 'string' && Number.isInteger(value.seat)) return value;
  } catch { /* A corrupted or unavailable storage must not block the lobby. */ }
  return null;
}
export function saveSession(session: SavedSession | null) {
  try {
    if (session) localStorage.setItem(STORAGE_KEY, JSON.stringify({
      roomId: session.roomId, inviteCode: session.inviteCode, token: session.token, seat: session.seat,
    }));
    else localStorage.removeItem(STORAGE_KEY);
  } catch { /* The current session remains usable if browser storage is disabled. */ }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let response: Response;
  try { response = await fetch(path, { ...init, signal: timedSignal(init?.signal) }); }
  catch { throw new ApiError(0, '暂时无法连接牌桌服务，请检查连接后重试。'); }
  const body = await response.json().catch(() => ({}));
  if (!response.ok) throw new ApiError(response.status, body.message || body.error || '操作未完成，请重试。', body.view);
  return body as T;
}
function timedSignal(signal?: AbortSignal | null): AbortSignal {
  const timeout = AbortSignal.timeout(12000);
  return signal ? AbortSignal.any([signal, timeout]) : timeout;
}
const headers = (session: SavedSession) => ({ Authorization: `Bearer ${session.token}` });
const roomPath = (session: SavedSession) => `/api/rooms/${encodeURIComponent(session.roomId)}`;
let supportsEntryReceipts = false;
export const getCatalog = async (signal?: AbortSignal) => {
  const catalog = await request<Catalog & { entryIdempotency?: boolean }>('/api/catalog', { signal });
  supportsEntryReceipts = catalog.entryIdempotency === true;
  return catalog;
};
export const getState = (session: SavedSession, signal?: AbortSignal) => request<View>(`${roomPath(session)}/state`, { headers: headers(session), signal });
let entryMemory: { intent: string; requestId: string } | null = null;
async function enterRoom(path: string, values: Record<string, string>): Promise<Session> {
  const intent = JSON.stringify([path, values]);
  try { entryMemory = JSON.parse(localStorage.getItem(ENTRY_KEY) || 'null') || entryMemory; } catch { /* Keep an in-memory recovery key. */ }
  if (!entryMemory || entryMemory.intent !== intent || typeof entryMemory.requestId !== 'string') entryMemory = { intent, requestId: newCommandId() };
  try { localStorage.setItem(ENTRY_KEY, JSON.stringify(entryMemory)); } catch { /* Storage may be disabled. */ }
  const init = { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ ...values, requestId: entryMemory.requestId }) };
  try {
    let session: Session;
    try { session = await request<Session>(path, init); }
    catch (error) {
      if (!supportsEntryReceipts || !(error instanceof ApiError) || (error.status !== 0 && error.status < 500)) throw error;
      session = await request<Session>(path, init);
    }
    entryMemory = null;
    try { localStorage.removeItem(ENTRY_KEY); } catch { /* No persistent storage. */ }
    return session;
  } catch (error) { throw error; /* Keep exactly the original request key for an explicit retry. */ }
}
export const createRoom = (name: string, mode: 'duel' | 'teams', deckId: string) => enterRoom('/api/rooms', { name, mode, deckId });
export const joinRoom = (inviteCode: string, name: string, deckId: string) => enterRoom('/api/rooms/join', { inviteCode, name, deckId });

/** One authenticated version poll. No token in URLs, no full hidden state. */
export async function pollState(session: SavedSession, version: number, signal: AbortSignal): Promise<View | null> {
  let response: Response;
  try { response = await fetch(`${roomPath(session)}/state?afterVersion=${version}`, { headers: headers(session), signal: timedSignal(signal), cache: 'no-store' }); }
  catch { throw new ApiError(0, '暂时无法连接牌桌服务，请检查连接后重试。'); }
  if (response.status === 204) return null;
  const body = await response.json().catch(() => ({}));
  if (!response.ok) throw new ApiError(response.status, body.message || '同步暂不可用。', body.view);
  if (body.roomId !== session.roomId || !Number.isSafeInteger(body.version)) throw new ApiError(0, '同步返回格式不正确。');
  return body as View;
}

export function actionPayload(action: Action | (Action & { id: string; label: string; description?: string })): Action {
  const { kind, cardId, targetId, region, option, abilityId, costSelected, choiceId, selected, top, bottom, allocations } = action;
  return { kind, cardId, targetId, region, option, abilityId, costSelected, choiceId, selected, top, bottom, allocations };
}
export const sendCommand = (session: SavedSession, version: number, action: Action, commandId: string = newCommandId()) => request<View>(`${roomPath(session)}/commands`, {
  method: 'POST', headers: { ...headers(session), 'Content-Type': 'application/json' },
  body: JSON.stringify({ commandId, expectedVersion: version, action: actionPayload(action) }),
});

/** SSE over authenticated fetch keeps the seat credential out of URLs and browser history. */
export async function streamEvents(session: SavedSession, signal: AbortSignal, onView: (view: View) => void) {
  const response = await fetch(`${roomPath(session)}/events`, { headers: { ...headers(session), Accept: 'text/event-stream' }, signal });
  if (!response.ok) throw new ApiError(response.status, '实时连接暂不可用。');
  if (!response.body) throw new Error('实时连接没有响应流');
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = '';
  try {
    while (!signal.aborted) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer = (buffer + decoder.decode(value, { stream: true })).replace(/\r\n/g, '\n');
      let boundary: number;
      while ((boundary = buffer.indexOf('\n\n')) >= 0) {
        const event = buffer.slice(0, boundary); buffer = buffer.slice(boundary + 2);
        const data = event.split('\n').filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
        if (data) {
          const view = JSON.parse(data) as View;
          if (view.roomId === session.roomId && typeof view.version === 'number') onView(view);
        }
      }
    }
  } finally { reader.releaseLock(); }
}
