import type { Action, Catalog, SavedSession, Session, View } from './types';
import type { DeckDraft } from './deckLibrary';
import { playerStorage } from './playerStorage';

const STORAGE_KEY = 'hegemony.session.v1';
const PENDING_KEY = 'hegemony.pending.v1';
const PENDING_PREFIX = 'hegemony.pending.v2.';
const LEGACY_CONFIRMED_PREFIX = 'hegemony.pending.legacyConfirmed.v2.';
const ENTRY_KEY = 'hegemony.entry.v1';
const SEATS_KEY = 'hegemony.seats.v1';
const SCREEN_KEY = 'hegemony.screen.v1';
export function newCommandId(): string {
  if (typeof crypto.randomUUID === 'function') return crypto.randomUUID();
  // getRandomValues also works on HTTP development hosts without randomUUID support.
  return Array.from(crypto.getRandomValues(new Uint8Array(16)), byte => byte.toString(16).padStart(2, '0')).join('');
}
export type PendingCommand = { roomId: string; seat?: number; commandId: string; expectedVersion: number; action: Action };
function decodePending(raw: string | null): PendingCommand | null {
  const value = JSON.parse(raw || 'null');
  if (!value || typeof value.roomId !== 'string' || typeof value.commandId !== 'string'
    || !Number.isSafeInteger(value.expectedVersion) || value.expectedVersion < 0 || typeof value.action?.kind !== 'string') return null;
  // A mutable saved session is never evidence of an older command's original actor.
  return { roomId: value.roomId, commandId: value.commandId, expectedVersion: value.expectedVersion, action: value.action,
    seat: Number.isInteger(value.seat) && value.seat >= 0 && value.seat < 4 ? value.seat : undefined };
}
const pendingKey = (command: PendingCommand) => `${PENDING_PREFIX}${JSON.stringify([command.roomId, command.seat ?? null, command.commandId])}`;
const samePending = (a: PendingCommand | null, b: PendingCommand) => !!a && a.roomId === b.roomId && a.seat === b.seat
  && a.commandId === b.commandId && a.expectedVersion === b.expectedVersion && JSON.stringify(a.action) === JSON.stringify(b.action);
function sameLegacyPending(raw: string | null, command: PendingCommand): boolean {
  try { return command.seat !== undefined && samePending(decodePending(raw), command); } catch { return false; }
}
export function readPendings(capturedStorage?: ReturnType<typeof playerStorage>): PendingCommand[] {
  try {
    const storage = capturedStorage || playerStorage();
    // Each command is its own atomic storage item. No shared index can overwrite a peer's receipt.
    const keys = storage.keys();
    const originals: PendingCommand[] = [];
    for (const key of keys.filter(key => key.startsWith(PENDING_PREFIX)).sort()) {
      try {
        const command = decodePending(storage.getItem(key));
        if (command && key === pendingKey(command)) {
          originals.push(command);
        }
      } catch { /* One malformed record cannot hide other recoverable commands. */ }
    }
    const legacy = storage.getItem(PENDING_KEY);
    let command: PendingCommand | null = null;
    try { command = decodePending(legacy); } catch { /* Malformed legacy data cannot hide valid independent records. */ }
    if (command && command.seat === undefined) {
      // Unreleased e082443 could copy this ID under a guessed seat. Those copies do not prove ownership either.
      return [command, ...originals.filter(item => item.roomId !== command.roomId || item.commandId !== command.commandId)];
    }
    if (originals.some(command => sameLegacyPending(legacy, command))) return originals;
    for (const key of keys.filter(key => key.startsWith(LEGACY_CONFIRMED_PREFIX))) {
      try {
        const command = decodePending(storage.getItem(key));
        if (command && key === `${LEGACY_CONFIRMED_PREFIX}${pendingKey(command).slice(PENDING_PREFIX.length)}` && sameLegacyPending(legacy, command)) return originals;
      }
      catch { /* A malformed legacy marker cannot suppress a different command. */ }
    }
    return command ? [...originals, command] : originals;
  } catch { /* Browser storage is optional. */ }
  return [];
}
export function readPending(session?: SavedSession | null, capturedStorage?: ReturnType<typeof playerStorage>): PendingCommand | null {
  return readPendings(capturedStorage).find(command => command.seat === undefined || !session
    || command.roomId === session.roomId && command.seat === session.seat) || null;
}
export function savePending(command: PendingCommand, capturedStorage?: ReturnType<typeof playerStorage>) {
  try {
    const storage = capturedStorage || playerStorage();
    const normalized = decodePending(JSON.stringify(command));
    if (!normalized || normalized.seat === undefined) return;
    const key = pendingKey(normalized);
    const existing = decodePending(storage.getItem(key));
    if (!existing || samePending(existing, normalized)) storage.setItem(key, JSON.stringify(normalized));
  } catch { /* Preserve the in-memory command while this tab is open. */ }
}
export function clearPending(command: PendingCommand, capturedStorage?: ReturnType<typeof playerStorage>) {
  if (command.seat === undefined) return;
  try {
    const storage = capturedStorage || playerStorage();
    const key = pendingKey(command);
    // Never delete the shared legacy slot: an old page can replace it between comparison and removal.
    if (sameLegacyPending(storage.getItem(PENDING_KEY), command)) {
      storage.setItem(`${LEGACY_CONFIRMED_PREFIX}${key.slice(PENDING_PREFIX.length)}`, JSON.stringify(command));
    }
    if (samePending(decodePending(storage.getItem(key)), command)) storage.removeItem(key);
  } catch { /* A retained receipt can safely be confirmed again with its original identity. */ }
}
export function retireSeatPendings(session: SavedSession, capturedStorage?: ReturnType<typeof playerStorage>): PendingCommand[] {
  const records = readPendings(capturedStorage).filter(command => command.roomId === session.roomId && command.seat === session.seat);
  records.forEach(command => clearPending(command, capturedStorage));
  return records;
}

export class ApiError extends Error {
  status: number;
  view?: View;
  code?: string;
  constructor(status: number, message: string, view?: View, code?: string) {
    super(message); this.status = status; this.view = view; this.code = code;
  }
}

export function readSession(capturedStorage?: ReturnType<typeof playerStorage>): SavedSession | null {
  try {
    const value = JSON.parse((capturedStorage || playerStorage()).getItem(STORAGE_KEY) || 'null');
    if (value && typeof value.roomId === 'string' && typeof value.token === 'string'
      && typeof value.inviteCode === 'string' && Number.isInteger(value.seat)) return value;
  } catch { /* A corrupted or unavailable storage must not block the lobby. */ }
  return null;
}
export function readSavedSeats(): SavedSession[] {
  let seats: SavedSession[] = [];
  try {
    const values = JSON.parse(playerStorage().getItem(SEATS_KEY) || '[]');
    if (Array.isArray(values)) seats = values.filter(value => value && typeof value.roomId === 'string'
      && typeof value.token === 'string' && typeof value.inviteCode === 'string' && Number.isInteger(value.seat));
  } catch { /* A damaged history must not hide the legacy saved seat. */ }
  const current = readSession();
  if (current && !seats.some(seat => seat.roomId === current.roomId && seat.seat === current.seat)) seats.push(current);
  return seats;
}
export function returnToLobby() {
  try { playerStorage().setItem(SCREEN_KEY, 'lobby'); } catch { /* This tab can still return. */ }
}
export function readActiveSession(): SavedSession | null {
  try { if (playerStorage().getItem(SCREEN_KEY) === 'lobby') return null; } catch { /* Restore a saved seat when possible. */ }
  return readSession();
}
export function forgetSavedSeat(session: SavedSession) {
  const keep = readSavedSeats().filter(seat => seat.roomId !== session.roomId || seat.seat !== session.seat);
  try {
    playerStorage().setItem(SEATS_KEY, JSON.stringify(keep));
    const current = readSession();
    if (current?.roomId === session.roomId && current.seat === session.seat) playerStorage().removeItem(STORAGE_KEY);
  } catch { /* The UI also removes the invalid seat from its own state. */ }
}
export function saveSession(session: SavedSession | null) {
  try {
    if (session) {
      const saved = { roomId: session.roomId, inviteCode: session.inviteCode, token: session.token, seat: session.seat };
      const seats = readSavedSeats().filter(seat => seat.roomId !== saved.roomId || seat.seat !== saved.seat);
      playerStorage().setItem(SEATS_KEY, JSON.stringify([...seats, saved]));
      playerStorage().setItem(STORAGE_KEY, JSON.stringify(saved));
      playerStorage().setItem(SCREEN_KEY, 'table');
    }
    else playerStorage().removeItem(STORAGE_KEY);
  } catch { /* The current session remains usable if browser storage is disabled. */ }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let response: Response;
  try { response = await fetch(path, { ...init, signal: timedSignal(init?.signal) }); }
  catch { throw new ApiError(0, '暂时无法连接牌桌服务，请检查连接后重试。'); }
  const body = await response.json().catch(() => ({}));
  if (!response.ok) throw new ApiError(response.status, body.message || body.error || '操作未完成，请重试。', body.view, typeof body.error === 'string' ? body.error : undefined);
  return body as T;
}
function timedSignal(signal?: AbortSignal | null): AbortSignal {
  const timeout = AbortSignal.timeout(12000);
  return signal ? AbortSignal.any([signal, timeout]) : timeout;
}
const headers = (session: SavedSession) => ({ Authorization: `Bearer ${session.token}` });
const roomPath = (session: SavedSession) => `/api/rooms/${encodeURIComponent(session.roomId)}`;
let supportsEntryReceipts = false;
export const getCatalog = async (signal?: AbortSignal, session?: SavedSession | null) => {
  const catalog = await request<Catalog & { entryIdempotency?: boolean }>(session ? `${roomPath(session)}/catalog` : '/api/catalog', {
    signal, ...(session ? { headers: headers(session) } : {}),
  });
  supportsEntryReceipts = catalog.entryIdempotency === true;
  return catalog;
};
export const getState = async (session: SavedSession, signal?: AbortSignal) => {
  const view = await request<View>(`${roomPath(session)}/state`, { headers: headers(session), signal });
  if (view.roomId !== session.roomId || !Number.isSafeInteger(view.version)) throw new ApiError(0, '同步返回格式不正确，请登录后重新同步。');
  return view;
};
const entryMemory = new Map<string, { intent: string; requestId: string }>();
async function enterRoom(path: string, values: Record<string, unknown>): Promise<Session> {
  const intent = JSON.stringify([path, values]);
  // Capture the namespace before awaiting; a late acknowledgement cannot clear another scope's receipt.
  const storage = playerStorage();
  let receipt = entryMemory.get(storage.scope);
  try { receipt = JSON.parse(storage.getItem(ENTRY_KEY) || 'null') || receipt; } catch { /* Keep an in-memory recovery key. */ }
  if (!receipt || receipt.intent !== intent || typeof receipt.requestId !== 'string') receipt = { intent, requestId: newCommandId() };
  entryMemory.set(storage.scope, receipt);
  try { storage.setItem(ENTRY_KEY, JSON.stringify(receipt)); } catch { /* Storage may be disabled. */ }
  const init = { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ ...values, requestId: receipt.requestId }) };
  try {
    let session: Session;
    try { session = await request<Session>(path, init); }
    catch (error) {
      if (!supportsEntryReceipts || !(error instanceof ApiError) || (error.status !== 0 && error.status < 500)) throw error;
      session = await request<Session>(path, init);
    }
    entryMemory.delete(storage.scope);
    try { storage.removeItem(ENTRY_KEY); } catch { /* No persistent storage. */ }
    return session;
  } catch (error) {
    if (error instanceof ApiError && error.code === 'unsupported_room_version') {
      // This key identifies an obsolete room. A later explicit new-game request
      // needs a fresh key; never silently resend the rejected request.
      entryMemory.delete(storage.scope);
      try { storage.removeItem(ENTRY_KEY); } catch { /* Storage may be disabled. */ }
    }
    throw error; // Other failures retain the exact original key for recovery.
  }
}
export const createRoom = (name: string, mode: 'duel' | 'teams', deckId: string) => enterRoom('/api/rooms', { name, mode, deckId });
export const joinRoom = (inviteCode: string, name: string, deckId: string) => enterRoom('/api/rooms/join', { inviteCode, name, deckId });
export const createRoomWithDeck = (name: string, mode: 'duel' | 'teams', deckDraft: DeckDraft) => enterRoom('/api/rooms', { name, mode, deckDraft });
export const joinRoomWithDeck = (inviteCode: string, name: string, deckDraft: DeckDraft) => enterRoom('/api/rooms/join', { inviteCode, name, deckDraft });

/** One authenticated version poll. No token in URLs, no full hidden state. */
export async function pollState(session: SavedSession, version: number, signal: AbortSignal): Promise<View | null> {
  let response: Response;
  try { response = await fetch(`${roomPath(session)}/state?afterVersion=${version}`, { headers: headers(session), signal: timedSignal(signal), cache: 'no-store' }); }
  catch { throw new ApiError(0, '暂时无法连接牌桌服务，请检查连接后重试。'); }
  if (response.status === 204) return null;
  const body = await response.json().catch(() => ({}));
  if (!response.ok) throw new ApiError(response.status, body.message || '同步暂不可用。', body.view, typeof body.error === 'string' ? body.error : undefined);
  if (body.roomId !== session.roomId || !Number.isSafeInteger(body.version)) throw new ApiError(0, '同步返回格式不正确。');
  return body as View;
}

export function actionPayload(action: Action | (Action & { id: string; label: string; description?: string })): Action {
  const { kind, cardId, targetId, region, option, abilityId, costSelected, choiceId, selected, top, bottom, allocations, deckDraft, windowId, intentId } = action;
  return { kind, cardId, targetId, region, option, abilityId, costSelected, choiceId, selected, top, bottom, allocations, deckDraft, windowId, intentId,
    ...(action.action ? { action: actionPayload(action.action) } : {}) };
}
const sessionKinds = new Set(['game', 'beginResponse', 'passResponse', 'cancelAndPass', 'submitResponse', 'pauseRoom', 'resumeRoom']);
/** Only the new room envelope wraps actions; pinned older rooms keep their exact request contract. */
export function actionForRoom(view: View, action: Action): Action {
  const payload = actionPayload(action);
  if (view.serverNowMs === undefined || sessionKinds.has(payload.kind)) return payload;
  const window = view.responseWindow;
  if (!window || view.pendingChoice || view.waitingChoice) return { kind: 'game', action: payload };
  const member = window.members.find(item => item.playerId === view.you);
  if (payload.kind === 'pass') {
    if (member?.status === 'undecided') return { kind: 'passResponse', windowId: window.id };
    if (member?.status === 'composing' && window.myIntentId) return { kind: 'cancelAndPass', windowId: window.id, intentId: window.myIntentId };
  } else if (member?.status === 'composing' && window.myIntentId) {
    return { kind: 'submitResponse', windowId: window.id, intentId: window.myIntentId, action: payload };
  }
  throw new Error('请先选择连锁；已让过的窗口须等待牌桌继续。');
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
