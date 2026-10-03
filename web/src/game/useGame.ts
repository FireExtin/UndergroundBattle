import { useCallback, useEffect, useRef, useState } from 'react';
import { actionForRoom, ApiError, createRoom, createRoomWithDeck, forgetSavedSeat, getCatalog, getState, joinRoom, joinRoomWithDeck, newCommandId, pollState, readActiveSession, readPending, readSavedSeats, readSession, returnToLobby, savePending, saveSession, sendCommand, type PendingCommand } from './api';
import type { Action, Catalog, SavedSession, Session, View } from './types';
import type { DeckDraft } from './deckLibrary';
import { readPlayerMode, selectPlayerMode, type PlayerMode } from './playerStorage';

export function newerView(current: View | null, next: View): View {
  return current && current.roomId === next.roomId && current.you === next.you && current.version > next.version ? current : next;
}
const commandFor = (command: PendingCommand | null, session: SavedSession | null) => !!command && !!session
  && command.roomId === session.roomId && command.seat === session.seat;
const invalidSeatToken = (error: unknown) => error instanceof ApiError && (error.code === 'invalid_seat_token'
  || (error.code === 'unauthorized' && error.message === '需要此房间的座位令牌'));
const needsSiteLogin = (error: unknown) => error instanceof ApiError && (error.status === 401 || error.status === 403) && !invalidSeatToken(error);
const loginMessage = '访问牌桌需要重新登录。座位和未确认行动已保留，请登录后刷新页面继续。';

export function useGame() {
  const [playerMode, setPlayerMode] = useState(readPlayerMode);
  const [catalog, setCatalog] = useState<Catalog | null>(null);
  const [session, setSession] = useState<SavedSession | null>(readActiveSession);
  const [savedSeats, setSavedSeats] = useState<SavedSession[]>(readSavedSeats);
  const [view, setView] = useState<View | null>(null);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [connection, setConnection] = useState<'connecting' | 'online' | 'offline'>('connecting');
  const [catalogRetry, setCatalogRetry] = useState(0);
  const commandLock = useRef(false);
  const pending = useRef<PendingCommand | null>(readPending());
  const [uncertain, setUncertain] = useState(commandFor(pending.current, session));
  const activeSession = useRef(session);
  const acceptedVersion = useRef(0);
  activeSession.current = session;
  const isActive = (captured: SavedSession) => activeSession.current?.roomId === captured.roomId
    && activeSession.current.seat === captured.seat && activeSession.current.token === captured.token;
  const accept = useCallback((next: View) => {
    if (next.roomId === activeSession.current?.roomId && next.you === `p${activeSession.current.seat}`) {
      acceptedVersion.current = Math.max(acceptedVersion.current, next.version);
      setView(current => newerView(current, next));
    }
  }, []);

  useEffect(() => {
    const controller = new AbortController();
    const matchesSession = () => session ? isActive(session) : activeSession.current === null;
    setCatalog(null);
    getCatalog(controller.signal, session).then(next => {
      if (!controller.signal.aborted && matchesSession()) setCatalog(next);
    }).catch(e => { if (!controller.signal.aborted && matchesSession()) setError(e.message); });
    return () => controller.abort();
  }, [session, catalogRetry]);

  useEffect(() => {
    if (!session) return;
    const controller = new AbortController();
    let timer: ReturnType<typeof setTimeout> | undefined;
    let attempts = 0;
    let initialized = false;
    let polling = false;
    const reconnect = async () => {
      if (controller.signal.aborted || polling) return;
      polling = true;
      if (!initialized) setConnection('connecting');
      try {
        const state = initialized ? await pollState(session, acceptedVersion.current, controller.signal) : await getState(session, controller.signal);
        if (controller.signal.aborted || !isActive(session)) return;
        if (state) accept(state);
        setConnection('online'); attempts = 0; initialized = true;
        if (commandFor(pending.current, session) && !commandLock.current) {
          await resolvePending(session);
        }
        if (!controller.signal.aborted) timer = setTimeout(reconnect, document.visibilityState === 'hidden' ? 12000 : 1500);
        return;
      } catch (e) {
        if (controller.signal.aborted || !isActive(session)) return;
        if (invalidSeatToken(e) || (e instanceof ApiError && e.code === 'room_not_found')) {
          forgetSavedSeat(session); setSavedSeats(readSavedSeats()); returnToLobby();
          if (commandFor(pending.current, session)) { savePending(null); pending.current = null; }
          activeSession.current = null; setUncertain(false); setSession(null); setView(null);
          setError('保存的座位已无法恢复。请使用邀请码重新加入牌桌。'); return;
        }
        if (needsSiteLogin(e)) setError(loginMessage);
      } finally { polling = false; }
      if (!controller.signal.aborted) {
        setConnection('offline'); attempts++;
        timer = setTimeout(reconnect, Math.min(1000 * 2 ** Math.min(attempts, 4), 15000));
      }
    };
    const visible = () => {
      if (document.visibilityState === 'visible' && !controller.signal.aborted) {
        if (timer) clearTimeout(timer);
        timer = setTimeout(reconnect, 0);
      }
    };
    document.addEventListener('visibilitychange', visible);
    void reconnect();
    return () => { controller.abort(); if (timer) clearTimeout(timer); document.removeEventListener('visibilitychange', visible); };
  }, [session, accept]);

  const enter = async (task: () => Promise<Session>) => {
    if (commandLock.current) return;
    if (pending.current) { setError('旧牌桌还有待确认行动，请先回到该牌桌确认，再新建或加入另一桌。'); return; }
    commandLock.current = true; setBusy(true); setError('');
    try {
      const next = await task();
      pending.current = null; savePending(null); setUncertain(false);
      saveSession(next); setSavedSeats(readSavedSeats()); activeSession.current = next;
      acceptedVersion.current = next.view.version;
      setSession({ roomId: next.roomId, inviteCode: next.inviteCode, token: next.token, seat: next.seat });
      setView(next.view);
      // Invitations contain only a room code; seat credentials are stored locally.
      if (new URL(location.href).searchParams.has('invite')) history.replaceState({}, '', location.pathname);
    } catch (e) { setError(e instanceof Error ? e.message : '加入失败，请重试。'); }
    finally { commandLock.current = false; setBusy(false); }
  };

  const resolvePending = async (currentSession: SavedSession) => {
    const command = pending.current;
    if (!commandFor(command, currentSession) || !command || commandLock.current) return;
    commandLock.current = true; setBusy(true); setError('');
    let confirmed = false;
    try {
      let result: View;
      try { result = await sendCommand(currentSession, command.expectedVersion, command.action, command.commandId); }
      catch (firstError) {
        if (!(firstError instanceof ApiError) || (firstError.status !== 0 && firstError.status < 500)) throw firstError;
        // A lost acknowledgement can hide a committed action. Retry only its original identity.
        result = await sendCommand(currentSession, command.expectedVersion, command.action, command.commandId);
      }
      accept(result); confirmed = true;
    }
    catch (e) {
      if (!isActive(currentSession)) return;
      confirmed = e instanceof ApiError && e.status >= 400 && e.status < 500 && !needsSiteLogin(e);
      if (e instanceof ApiError && e.view) accept(e.view);
      else { try { accept(await getState(currentSession)); } catch { /* Keep the last confirmed table visible. */ } }
      setError(needsSiteLogin(e) ? loginMessage : e instanceof ApiError && e.status === 409
        ? '牌桌刚刚发生了变化，已同步最新状态。请查看当前可用行动后重新选择。'
        : !confirmed ? '行动结果暂未确认，已尝试同步牌桌。请确认上一行动后继续；重试不会重复执行。'
        : e instanceof Error ? `操作未执行：${e.message}` : '操作未执行，请重试。');
    } finally {
      if (isActive(currentSession)) {
        if (confirmed) { pending.current = null; savePending(null); }
        setUncertain(!confirmed);
      }
      commandLock.current = false; setBusy(false);
    }
  };
  const act = async (action: Action) => {
    if (!session || !view || commandLock.current) return;
    if (pending.current) { await resolvePending(session); return; }
    let payload: Action;
    try { payload = actionForRoom(view, action); }
    catch (error) { setError(error instanceof Error ? error.message : '当前响应窗口已变化，请重新选择。'); return; }
    pending.current = { roomId: session.roomId, seat: session.seat, commandId: newCommandId(), expectedVersion: view.version, action: payload };
    savePending(pending.current);
    await resolvePending(session);
  };

  const resume = (target?: SavedSession) => {
    if (commandLock.current) return;
    const saved = target || readSession() || savedSeats.at(-1);
    if (!saved) return;
    if (pending.current && !commandFor(pending.current, saved)) { setError('请先恢复待确认行动所在的原席位，再切换牌桌或席位。'); return; }
    saveSession(saved); setSavedSeats(readSavedSeats()); activeSession.current = saved; acceptedVersion.current = 0;
    setView(null); setSession(saved); setUncertain(!!pending.current); setError('');
    if (new URL(location.href).searchParams.has('invite')) history.replaceState({}, '', location.pathname);
  };
  const join = (inviteCode: string, task: () => Promise<Session>) => {
    const code = inviteCode.trim().toUpperCase();
    const matching = readSavedSeats().filter(seat => seat.inviteCode.trim().toUpperCase() === code);
    // The invite is a room locator. A stored token, never a display name, identifies its occupant.
    const saved = matching.find(seat => commandFor(pending.current, seat)) || matching.at(-1);
    if (saved) { resume(saved); return Promise.resolve(); }
    return enter(task);
  };

  const switchPlayerMode = (mode: PlayerMode) => {
    if (commandLock.current) return;
    if (pending.current) { setError('当前会话还有待确认行动，请先恢复原席确认，再切换玩家会话。'); return; }
    try { selectPlayerMode(mode, mode === 'independent' ? newCommandId() : undefined); }
    catch (e) { setError(e instanceof Error ? e.message : '无法切换玩家会话。'); return; }
    const next = readActiveSession();
    pending.current = readPending(); activeSession.current = next; acceptedVersion.current = 0;
    setPlayerMode(mode); setSavedSeats(readSavedSeats()); setSession(next); setView(null);
    setUncertain(commandFor(pending.current, next)); setError(''); setConnection('connecting');
    setCatalog(null); setCatalogRetry(n => n + 1);
  };

  return {
    catalog, session, view, error, busy, uncertain, connection, act, playerMode,
    playerModeLocked: busy || !!pending.current,
    startIndependentSession: () => switchPlayerMode('independent'),
    useOrdinarySession: () => switchPlayerMode('ordinary'),
    create: (name: string, mode: 'duel' | 'teams', deckId: string) => enter(() => createRoom(name, mode, deckId)),
    join: (inviteCode: string, name: string, deckId: string) => join(inviteCode, () => joinRoom(inviteCode, name, deckId)),
    createDraft: (name: string, mode: 'duel' | 'teams', draft: DeckDraft) => enter(() => createRoomWithDeck(name, mode, draft)),
    joinDraft: (inviteCode: string, name: string, draft: DeckDraft) => join(inviteCode, () => joinRoomWithDeck(inviteCode, name, draft)),
    dismissError: () => setError(''),
    retryCatalog: () => { setError(''); setCatalogRetry(n => n + 1); },
    retryPending: () => { if (session) void resolvePending(session); },
    savedSeats,
    resumeAvailable: !session && savedSeats.length > 0,
    resume,
    // Return to the lobby without destroying the only credential for an occupied seat.
    leave: () => { if (commandLock.current) return; returnToLobby(); activeSession.current = null; setSession(null); setView(null); setError(''); setUncertain(false); },
  };
}
