import { RoomStore } from './store.mjs';
import { retryOriginalCommand } from './storage-errors.mjs';

export class HttpError extends Error {
  constructor(status, message, view, code) { super(message); this.status = status; this.view = view; this.code = code; }
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
const commandFields = new Set(['kind','cardId','targetId','region','option','choiceId','selected','top','bottom','allocations','abilityId','costSelected','deckDraft']);
const sessionFields = {
  game: ['kind', 'action'], beginResponse: ['kind', 'windowId', 'intentId'],
  passResponse: ['kind', 'windowId'], cancelAndPass: ['kind', 'windowId', 'intentId'],
  submitResponse: ['kind', 'windowId', 'intentId', 'action'],
};
export function normalizedAction(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value) || typeof value.kind !== 'string') bad('行动格式不正确');
  const fields = Object.hasOwn(sessionFields, value.kind) ? sessionFields[value.kind] : null;
  if (fields) {
    if (Object.keys(value).some(key => !fields.includes(key))) bad('响应意图含未知字段');
    const result = Object.fromEntries(Object.entries(value).filter(([, v]) => v !== null));
    if (fields.includes('action')) {
      if (!value.action || Object.hasOwn(sessionFields, value.action.kind)) bad('请提交一个完整游戏动作');
      result.action = normalizedAction(value.action);
    }
    return result;
  }
  if (Object.keys(value).some(key => !commandFields.has(key))) bad('行动含未知字段');
  if (value.deckDraft != null && value.kind !== 'deck') bad('只有大厅更换牌组可提交构筑草稿');
  return Object.fromEntries(Object.entries(value).filter(([, v]) => v !== null));
}
function deckSelection(body) {
  if (body.deckDraft === undefined) return { deckId: body.deckId };
  if (body.deckId !== undefined) bad('请选择一个预组或一个自定义牌组');
  if (!body.deckDraft || typeof body.deckDraft !== 'object' || Array.isArray(body.deckDraft)) bad('自定义牌组格式不正确');
  return { deckDraft: body.deckDraft };
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
  constructor(db, kernel, now = Date.now) { this.store = new RoomStore(db); this.kernel = kernel; this.now = now; }
  view(state, seat) { return JSON.parse(this.kernel.view(state, seat)); }
  async refreshedRoom(id, seat) {
    for (let attempt = 0; attempt < 5; attempt++) {
      const room = await this.store.room(id);
      if (!room) throw new HttpError(404, '房间不存在');
      if (!this.kernel.supportsPacing?.(room.state)) return { room, view: null };
      const next = call(() => transition(this.kernel.pollRoom(room.state, seat, String(this.now()))));
      if (!next.changed) return { room, view: next.view };
      if (next.version !== room.version + 1) throw new Error('Unexpected clock revision');
      const committed = await this.store.system({ id, expectedVersion: room.version, state: next.state,
        version: next.version, nonce: secure(16), entry: JSON.stringify({ SessionEvents: { events: next.journal } }) });
      if (committed) return { room: { ...room, state: next.state, version: next.version }, view: next.view };
    }
    throw new HttpError(409, '牌桌正在变化，请重新同步');
  }
  async entryKey(body, values) {
    // Old clients without a request key remain compatible, but cannot recover a lost lobby ACK.
    const key = body.requestId ?? secure(32);
    if (typeof key !== 'string' || !/^[a-zA-Z0-9-]{32,128}$/.test(key)) bad('请求标识格式不正确');
    return { requestHash: await digest(key), intentHash: await intent(values) };
  }
  async recoverEntry(key) {
    const previous = recovered(await this.store.entryReceipt(key.requestHash), key.intentHash);
    if (previous && this.kernel.assertSupported) {
      const room = await this.store.room(previous.roomId);
      if (!room) throw new HttpError(404, '房间不存在');
      this.kernel.assertSupported(room.state);
    }
    return previous;
  }
  async create(body) {
    const values = { kind: 'create', name: name(body.name), mode: body.mode, ...deckSelection(body) };
    const key = await this.entryKey(body, values);
    const previous = await this.recoverEntry(key);
    if (previous) return previous;
    const id = secure(12), invite = secure(6).toUpperCase(), token = secure(32), nonce = secure(16);
    const seed = BigInt('0x' + secure(8)).toString();
    const next = call(() => transition(values.deckDraft
      ? this.kernel.newGameWithDeck(id, invite, values.mode, values.name, JSON.stringify(values.deckDraft), seed)
      : this.kernel.newGame(id, invite, values.mode, values.name, values.deckId, seed)));
    const response = { roomId: id, inviteCode: invite, token, seat: 0, view: next.view };
    try {
      await this.store.create({ id, invite, state: next.state, nonce, tokenHash: await digest(token), ...key, response: JSON.stringify(response) });
    } catch (error) {
      const concurrent = await this.recoverEntry(key);
      if (concurrent) return concurrent;
      throw error;
    }
    return response;
  }
  async join(body) {
    if (typeof body.inviteCode !== 'string') bad('请输入邀请码');
    const values = { kind: 'join', inviteCode: body.inviteCode.trim().toUpperCase(), name: name(body.name), ...deckSelection(body) };
    const key = await this.entryKey(body, values);
    for (let attempt = 0; attempt < 5; attempt++) {
      const previous = await this.recoverEntry(key);
      if (previous) return previous;
      const room = await this.store.byInvite(values.inviteCode);
      if (!room) throw new HttpError(404, '房间不存在');
      let next;
      try { next = call(() => transition(values.deckDraft
        ? this.kernel.joinGameWithDeck(room.state, values.name, JSON.stringify(values.deckDraft))
        : this.kernel.joinGame(room.state, values.name, values.deckId))); }
      catch (error) {
        const concurrent = await this.recoverEntry(key);
        if (concurrent) return concurrent;
        throw error;
      }
      if (next.version !== room.version + 1) throw new Error('Unexpected join version');
      const token = secure(32), nonce = secure(16);
      const response = { roomId: room.id, inviteCode: room.invite, token, seat: next.seat, view: next.view };
      const committed = await this.store.join({ id: room.id, expectedVersion: room.version, state: next.state, version: next.version, nonce,
        seat: next.seat, tokenHash: await digest(token), ...key, response: JSON.stringify(response),
        entry: JSON.stringify(values.deckDraft
          ? { JoinWithDeck: { name: values.name, deck_draft: values.deckDraft } }
          : { Join: { name: values.name, deck_id: values.deckId } }) });
      if (committed) return response;
    }
    const previous = await this.recoverEntry(key);
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
    // A version poll must evaluate the persisted deadline even at an unchanged revision.
    const { room, view } = await this.refreshedRoom(id, seat);
    if (afterVersion !== null && afterVersion === String(room.version)) return null;
    return view || this.view(room.state, seat);
  }
  async catalog(id, authorization) {
    await this.authenticated(id, authorization);
    const room = await this.store.room(id);
    if (!room) throw new HttpError(404, '房间不存在');
    return JSON.parse(this.kernel.catalog(room.state));
  }
  async command(id, authorization, body) {
    // The whole operation repeats its original actor, ID, version and intent.
    // Receipt lookup precedes CAS, so even an unknown commit outcome is safe.
    return retryOriginalCommand(() => this.commandAttempt(id, authorization, body));
  }
  async commandAttempt(id, authorization, body) {
    const seat = await this.authenticated(id, authorization);
    if (typeof body.commandId !== 'string' || !body.commandId || body.commandId.length > 128) bad('命令标识格式不正确');
    if (!Number.isSafeInteger(body.expectedVersion) || body.expectedVersion < 0) bad('版本格式不正确');
    const action = normalizedAction(body.action);
    const hash = await intent({ seat, expectedVersion: body.expectedVersion, action });
    const room = await this.store.room(id);
    if (!room) throw new HttpError(404, '房间不存在');
    this.kernel.assertSupported?.(room.state);
    const previous = recovered(await this.store.receipt(id, body.commandId), hash);
    if (previous) return previous;
    if (this.kernel.supportsPacing?.(room.state)) {
      return this.pacedCommand(id, seat, room, body, action, hash);
    }
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
  async pacedCommand(id, seat, initialRoom, body, action, hash) {
    let room = initialRoom;
    for (let attempt = 0; attempt < 5; attempt++) {
      // Original receipt always wins over deadlines and a newer room revision.
      const previous = recovered(await this.store.receipt(id, body.commandId), hash);
      if (previous) return previous;
      const command = { commandId: body.commandId, expectedVersion: body.expectedVersion, action };
      const next = call(() => transition(this.kernel.applyRoom(room.state, seat, JSON.stringify(command), String(this.now()))));
      if (!['accepted', 'rejected'].includes(next.outcome) || typeof next.changed !== 'boolean' || !Array.isArray(next.journal)) throw new Error('Unsafe session transition');
      if (!next.changed) {
        const concurrent = recovered(await this.store.receipt(id, body.commandId), hash);
        if (concurrent) return concurrent;
        this.rejectSession(next);
        throw new Error('Accepted command did not advance revision');
      }
      if (next.version !== room.version + 1) throw new Error('Unexpected session revision');
      const values = { id, expectedVersion: room.version, state: next.state, version: next.version, nonce: secure(16),
        entry: JSON.stringify({ SessionEvents: { events: next.journal } }) };
      const accepted = next.outcome === 'accepted';
      const committed = accepted
        ? await this.store.command({ ...values, seat, commandId: body.commandId, intentHash: hash, response: JSON.stringify(next.view) })
        : await this.store.system(values);
      if (committed) {
        if (accepted) return next.view;
        this.rejectSession(next);
      }
      const concurrent = recovered(await this.store.receipt(id, body.commandId), hash);
      if (concurrent) return concurrent;
      // The reducer itself permits only same-window Begin/Pass/Cancel to use a newer
      // CAS base; the original ID, actor, expectedVersion and action remain identical.
      room = await this.store.room(id);
      if (!room) throw new HttpError(404, '房间不存在');
    }
    throw new HttpError(409, '牌桌正在变化，请使用原请求重试', this.view(room.state, seat));
  }
  rejectSession(next) {
    const code = next.errorCode || 'invalid_action';
    throw new HttpError(code === 'invalid_action' ? 400 : 409, next.errorMessage || '当前响应窗口已变化', next.view, code);
  }
  async quote(id, authorization, body) {
    const seat = await this.authenticated(id, authorization);
    if (typeof body.windowId !== 'string' || typeof body.intentId !== 'string'
      || Object.keys(body).some(key => !['windowId', 'intentId', 'draft'].includes(key))) bad('响应报价格式不正确');
    const { room } = await this.refreshedRoom(id, seat);
    if (!this.kernel.supportsPacing?.(room.state)) bad('旧牌桌保留原响应规则');
    const request = { windowId: body.windowId, intentId: body.intentId,
      ...(body.draft == null ? {} : { draft: normalizedAction(body.draft) }) };
    if (request.draft && Object.hasOwn(sessionFields, request.draft.kind)) bad('报价只接受游戏动作');
    return call(() => JSON.parse(this.kernel.quoteRoom(room.state, seat, JSON.stringify(request))));
  }
}
