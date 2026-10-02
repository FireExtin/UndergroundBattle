import { RoomStore } from './store.mjs';

export class HttpError extends Error {
  constructor(status, message, view) { super(message); this.status = status; this.view = view; }
}
const bad = message => { throw new HttpError(400, message); };
export function secure(bytes) {
  return Array.from(crypto.getRandomValues(new Uint8Array(bytes)), b => b.toString(16).padStart(2, '0')).join('');
}
export async function digest(value) {
  const bytes = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(value));
  return Array.from(new Uint8Array(bytes), b => b.toString(16).padStart(2, '0')).join('');
}
function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') return Object.fromEntries(Object.keys(value).sort().map(k => [k, canonical(value[k])]));
  return value;
}
const intent = value => digest(JSON.stringify(canonical(value)));
const name = value => {
  if (typeof value !== 'string') bad('请输入昵称');
  const result = value.trim();
  if (!result || Array.from(result).length > 40) bad('昵称长度须为1至40字');
  return result;
};
const commandFields = new Set(['kind','cardId','targetId','region','option','choiceId','selected','top','bottom','allocations','abilityId','costSelected']);
export function normalizedAction(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value) || typeof value.kind !== 'string') bad('行动格式不正确');
  if (Object.keys(value).some(key => !commandFields.has(key))) bad('行动含未知字段');
  return Object.fromEntries(Object.entries(value).filter(([, v]) => v !== null));
}
function transition(raw) {
  const next = JSON.parse(raw);
  if (typeof next.state !== 'string' || !Number.isSafeInteger(next.version) || !Number.isSafeInteger(next.seat)) throw new Error('Unsafe kernel envelope');
  // The full state stays opaque. In particular, never parse its u64 seed/PRNG.
  return next;
}
function recovered(receipt, hash) {
  if (!receipt) return null;
  if (receipt.intent_hash !== hash) throw new HttpError(409, '同一请求标识已用于不同操作，请使用原操作重试。');
  return JSON.parse(receipt.response);
}
function call(operation) {
  try { return operation(); }
  catch (error) { if (error instanceof HttpError) throw error; throw new HttpError(400, typeof error === 'string' ? error : '此行动当前不可执行'); }
}
export class RoomService {
  constructor(db, kernel) { this.store = new RoomStore(db); this.kernel = kernel; }
  view(state, seat) { return JSON.parse(this.kernel.view(state, seat)); }
  async entryKey(body, values) {
    // Old clients without a request key remain compatible, but cannot recover a lost lobby ACK.
    const key = body.requestId ?? secure(32);
    if (typeof key !== 'string' || !/^[a-zA-Z0-9-]{32,128}$/.test(key)) bad('请求标识格式不正确');
    return { requestHash: await digest(key), intentHash: await intent(values) };
  }
  async create(body) {
    const values = { kind: 'create', name: name(body.name), mode: body.mode, deckId: body.deckId };
    const key = await this.entryKey(body, values);
    const previous = recovered(await this.store.entryReceipt(key.requestHash), key.intentHash);
    if (previous) return previous;
    const id = secure(12), invite = secure(6).toUpperCase(), token = secure(32), nonce = secure(16);
    const seed = BigInt('0x' + secure(8)).toString();
    const next = call(() => transition(this.kernel.newGame(id, invite, values.mode, values.name, values.deckId, seed)));
    const response = { roomId: id, inviteCode: invite, token, seat: 0, view: next.view };
    try {
      await this.store.create({ id, invite, state: next.state, nonce, tokenHash: await digest(token), ...key, response: JSON.stringify(response) });
    } catch (error) {
      const concurrent = recovered(await this.store.entryReceipt(key.requestHash), key.intentHash);
      if (concurrent) return concurrent;
      throw error;
    }
    return response;
  }
  async join(body) {
    if (typeof body.inviteCode !== 'string') bad('请输入邀请码');
    const values = { kind: 'join', inviteCode: body.inviteCode.trim().toUpperCase(), name: name(body.name), deckId: body.deckId };
    const key = await this.entryKey(body, values);
    for (let attempt = 0; attempt < 5; attempt++) {
      const previous = recovered(await this.store.entryReceipt(key.requestHash), key.intentHash);
      if (previous) return previous;
      const room = await this.store.byInvite(values.inviteCode);
      if (!room) throw new HttpError(404, '房间不存在');
      let next;
      try { next = call(() => transition(this.kernel.joinGame(room.state, values.name, values.deckId))); }
      catch (error) {
        const concurrent = recovered(await this.store.entryReceipt(key.requestHash), key.intentHash);
        if (concurrent) return concurrent;
        throw error;
      }
      if (next.version !== room.version + 1) throw new Error('Unexpected join version');
      const token = secure(32), nonce = secure(16);
      const response = { roomId: room.id, inviteCode: room.invite, token, seat: next.seat, view: next.view };
      const committed = await this.store.join({ id: room.id, expectedVersion: room.version, state: next.state, version: next.version, nonce,
        seat: next.seat, tokenHash: await digest(token), ...key, response: JSON.stringify(response),
        entry: JSON.stringify({ Join: { name: values.name, deck_id: values.deckId } }) });
      if (committed) return response;
    }
    const previous = recovered(await this.store.entryReceipt(key.requestHash), key.intentHash);
    if (previous) return previous;
    throw new HttpError(409, '房间正在变化，请使用原请求重试加入。');
  }
  async authenticated(id, authorization) {
    const token = authorization?.match(/^Bearer ([a-f0-9]{64})$/)?.[1];
    if (!token) throw new HttpError(401, '需要此房间的座位令牌');
    const seat = await this.store.seat(id, await digest(token));
    if (!seat) throw new HttpError(401, '需要此房间的座位令牌');
    return seat.seat;
  }
  async state(id, authorization, afterVersion) {
    const seat = await this.authenticated(id, authorization);
    if (afterVersion !== null) {
      const known = await this.store.version(id);
      if (known && afterVersion === String(known.version)) return null;
    }
    const room = await this.store.room(id);
    if (!room) throw new HttpError(404, '房间不存在');
    if (afterVersion !== null && afterVersion === String(room.version)) return null;
    return this.view(room.state, seat);
  }
  async command(id, authorization, body) {
    const seat = await this.authenticated(id, authorization);
    if (typeof body.commandId !== 'string' || !body.commandId || body.commandId.length > 128) bad('命令标识格式不正确');
    if (!Number.isSafeInteger(body.expectedVersion) || body.expectedVersion < 0) bad('版本格式不正确');
    const action = normalizedAction(body.action);
    const hash = await intent({ seat, expectedVersion: body.expectedVersion, action });
    const previous = recovered(await this.store.receipt(id, body.commandId), hash);
    if (previous) return previous;
    const room = await this.store.room(id);
    if (!room) throw new HttpError(404, '房间不存在');
    if (room.version !== body.expectedVersion) {
      // A concurrent identical request may have committed after the first lookup.
      const concurrent = recovered(await this.store.receipt(id, body.commandId), hash);
      if (concurrent) return concurrent;
      throw new HttpError(409, '牌桌版本已变化', this.view(room.state, seat));
    }
    const next = call(() => transition(this.kernel.apply(room.state, seat, JSON.stringify(action))));
    if (next.version !== room.version + 1) throw new Error('Unexpected command version');
    const command = { commandId: body.commandId, expectedVersion: body.expectedVersion, action };
    const committed = await this.store.command({ id, seat, commandId: body.commandId, intentHash: hash,
      expectedVersion: body.expectedVersion, state: next.state, version: next.version, nonce: secure(16),
      response: JSON.stringify(next.view), entry: JSON.stringify({ Command: { seat, command } }) });
    if (committed) return next.view;
    const concurrent = recovered(await this.store.receipt(id, body.commandId), hash);
    if (concurrent) return concurrent;
    const current = await this.store.room(id);
    throw new HttpError(409, '牌桌版本已变化', this.view(current.state, seat));
  }
}
