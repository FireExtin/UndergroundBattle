import { useCallback, useEffect, useRef, useState } from 'react';
import { actionForRoom, ApiError, clearPending, createRoom, createRoomWithDeck, forgetSavedSeat, getCatalog, getState, joinRoom, joinRoomWithDeck, newCommandId, pollState, readActiveSession, readPendings, readSavedSeats, readSession, retireSeatPendings, returnToLobby, savePending, saveSession, sendCommand, type PendingCommand } from './api';
import type { Action, Catalog, SavedSession, Session, View } from './types';
import type { DeckDraft } from './deckLibrary';
import { playerStorage, readPlayerMode, selectPlayerMode, type PlayerMode } from './playerStorage';

export function newerView(current: View | null, next: View): View {
  return current && current.roomId === next.roomId && current.you === next.you && current.version > next.version ? current : next;
}
const commandFor = (command: PendingCommand | null, session: SavedSession | null) => !!command && !!session
  && command.roomId === session.roomId && command.seat === session.seat;
const invalidSeatToken = (error: unknown) => error instanceof ApiError && (error.code === 'invalid_seat_token'
  || (error.code === 'unauthorized' && error.message === '需要此房间的座位令牌'));
const needsSiteLogin = (error: unknown) => error instanceof ApiError && (error.status === 401 || error.status === 403) && !invalidSeatToken(error);
const loginMessage = '访问牌桌需要重新登录。座位和未确认行动已保留，请登录后刷新页面继续。';
const unboundMessage = '旧行动缺少原座位记录，无法安全自动恢复。原记录已保留，当前仅同步牌桌状态。';
const unbound = (command: PendingCommand | null) => !!command && command.seat === undefined;
const AUTO_PASS_KEY = 'hegemony.autoPass.v1';
const readAutoPassPreference = () => {
  try { return playerStorage().getItem(AUTO_PASS_KEY) === 'true'; }
  catch { return false; }
};

export function useGame() {
  const [playerMode, setPlayerMode] = useState(readPlayerMode);
  const [catalog, setCatalog] = useState<Catalog | null>(null);
  const [session, setSession] = useState<SavedSession | null>(readActiveSession);
  const [savedSeats, setSavedSeats] = useState<SavedSession[]>(readSavedSeats);
  const locallyConfirmed = useRef(new Set<string>());
  const confirmationKey = (command: PendingCommand, storage?: ReturnType<typeof playerStorage>) => {
    let scope = 'unavailable';
    try { scope = (storage || playerStorage()).scope; } catch { /* No persistent records can be read in this scope. */ }
    return JSON.stringify([scope, command.roomId, command.seat, command.commandId, command.expectedVersion, command.action]);
  };
  const storedPending = (target?: SavedSession | null, storage?: ReturnType<typeof playerStorage>) => readPendings(storage).find(command =>
    !locallyConfirmed.current.has(confirmationKey(command, storage)) && (unbound(command) || !target || commandFor(command, target))) || null;
  const pending = useRef<PendingCommand | null>(storedPending(session) || storedPending());
  const [view, setView] = useState<View | null>(null);
  const [error, setError] = useState(unbound(pending.current) ? unboundMessage : '');
  const [busy, setBusy] = useState(false);
  const [connection, setConnection] = useState<'connecting' | 'online' | 'offline'>('connecting');
  const [catalogRetry, setCatalogRetry] = useState(0);
  const [autoPassEnabled, setAutoPassState] = useState(readAutoPassPreference);
  const autoPassPreference = useRef(autoPassEnabled);
  const acceptedView = useRef<View | null>(null);
  const rearmPolling = useRef<(() => void) | null>(null);
  const refreshPolling = useRef<(() => void) | null>(null);
  const commandLock = useRef(false);
  const [uncertain, setUncertain] = useState(unbound(pending.current) || commandFor(pending.current, session));
  const activeSession = useRef(session);
  const acceptedVersion = useRef(0);
  activeSession.current = session;
  const isActive = (captured: SavedSession) => activeSession.current?.roomId === captured.roomId
    && activeSession.current.seat === captured.seat && activeSession.current.token === captured.token;
  const retireActor = (captured: SavedSession) => {
    retireSeatPendings(captured).forEach(command => locallyConfirmed.current.add(confirmationKey(command)));
    if (commandFor(pending.current, captured)) {
      locallyConfirmed.current.add(confirmationKey(pending.current!)); pending.current = null;
    }
  };
  const retireInvalidSeat = (captured: SavedSession, failure: unknown) => {
    if (!isActive(captured) || !(invalidSeatToken(failure) || failure instanceof ApiError && failure.code === 'room_not_found')) return false;
    retireActor(captured);
    forgetSavedSeat(captured); setSavedSeats(readSavedSeats()); returnToLobby();
    activeSession.current = null; acceptedView.current = null; acceptedVersion.current = 0;
    setUncertain(unbound(pending.current)); setSession(null); setView(null);
    setError(unbound(pending.current) ? unboundMessage : '保存的座位已无法恢复。请使用邀请码重新加入牌桌。');
    return true;
  };
  const retireUnsupportedRoom = (captured: SavedSession, failure: unknown) => {
    if (!(failure instanceof ApiError) || failure.code !== 'unsupported_room_version' || !isActive(captured)) return false;
    // The server definitively rejects this tuple. Keep its saved seat for a future
    // rollback, but stop polling/retrying an obsolete room and allow a new game.
    retireActor(captured);
    returnToLobby(); activeSession.current = null; acceptedView.current = null; acceptedVersion.current = 0;
    setSession(null); setView(null); setUncertain(unbound(pending.current)); setConnection('connecting');
    setError(unbound(pending.current) ? unboundMessage : failure.message); return true;
  };
  const accept = useCallback((next: View) => {
    if (next.roomId === activeSession.current?.roomId && next.you === `p${activeSession.current.seat}`
      && Number.isSafeInteger(next.version) && next.version >= 0) {
      acceptedVersion.current = Math.max(acceptedVersion.current, next.version);
      acceptedView.current = newerView(acceptedView.current, next);
      setView(acceptedView.current);
      rearmPolling.current?.();
      return true;
    }
    return false;
  }, []);
  const setAutoPassEnabled = useCallback((enabled: boolean) => {
    autoPassPreference.current = enabled; setAutoPassState(enabled);
    try { playerStorage().setItem(AUTO_PASS_KEY, String(enabled)); } catch { /* Keep this player's in-memory preference. */ }
    rearmPolling.current?.();
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
    let scheduledDelay: number | null = null;
    const delay = () => document.visibilityState === 'hidden'
      ? autoPassPreference.current && acceptedView.current?.status === 'playing' ? 3000 : 12000
      : 1500;
    const schedule = (ms: number, healthy: boolean) => {
      if (timer !== undefined) clearTimeout(timer);
      scheduledDelay = healthy ? ms : null;
      timer = setTimeout(() => { timer = undefined; scheduledDelay = null; void reconnect(); }, ms);
    };
    const rearm = () => {
      // Preference/status changes only rearm an idle healthy timer, never an in-flight read or backoff.
      if (controller.signal.aborted || !isActive(session) || polling) return;
      if (acceptedView.current?.pause) {
        if (timer !== undefined) clearTimeout(timer);
        timer = undefined; scheduledDelay = null;
        return;
      }
      if (timer === undefined) {
        if (initialized && attempts === 0) schedule(delay(), true);
        return;
      }
      if (scheduledDelay === null) return;
      const nextDelay = delay();
      if (nextDelay !== scheduledDelay) schedule(nextDelay, true);
    };
    rearmPolling.current = rearm;
    const refresh = () => { if (!controller.signal.aborted && isActive(session)) schedule(0, false); };
    refreshPolling.current = refresh;
    const reconnect = async () => {
      if (controller.signal.aborted || polling) return;
      polling = true;
      if (!initialized) setConnection('connecting');
      try {
        const state = initialized ? await pollState(session, acceptedVersion.current, controller.signal) : await getState(session, controller.signal);
        if (controller.signal.aborted || !isActive(session)) return;
        if (state) accept(state);
        setConnection('online'); attempts = 0; initialized = true;
        if (unbound(pending.current)) { setError(unboundMessage); setUncertain(true); }
        if (commandFor(pending.current, session) && !commandLock.current) {
          await resolvePending(session);
        }
        if (!controller.signal.aborted && isActive(session) && !acceptedView.current?.pause) schedule(delay(), true);
        return;
      } catch (e) {
        if (controller.signal.aborted || !isActive(session)) return;
        if (retireUnsupportedRoom(session, e)) return;
        if (retireInvalidSeat(session, e)) return;
        if (needsSiteLogin(e)) setError(loginMessage);
      } finally { polling = false; }
      if (!controller.signal.aborted && isActive(session)) {
        setConnection('offline'); attempts++;
        schedule(Math.min(1000 * 2 ** Math.min(attempts, 4), 15000), false);
      }
    };
    const visible = () => {
      if (document.visibilityState === 'visible' && !controller.signal.aborted && !acceptedView.current?.pause) {
        schedule(0, false);
      }
    };
    document.addEventListener('visibilitychange', visible);
    void reconnect();
    return () => {
      controller.abort(); if (timer !== undefined) clearTimeout(timer);
      if (rearmPolling.current === rearm) rearmPolling.current = null;
      if (refreshPolling.current === refresh) refreshPolling.current = null;
      document.removeEventListener('visibilitychange', visible);
    };
  }, [session, accept]);

  const enter = async (task: () => Promise<Session>) => {
    if (commandLock.current) return;
    pending.current ||= storedPending();
    if (pending.current) { setError(unbound(pending.current) ? unboundMessage : '旧牌桌还有待确认行动，请先回到该牌桌确认，再新建或加入另一桌。'); return; }
    commandLock.current = true; setBusy(true); setError('');
    try {
      const next = await task();
      pending.current = null; setUncertain(false);
      saveSession(next); setSavedSeats(readSavedSeats()); activeSession.current = next;
      acceptedVersion.current = next.view.version;
      acceptedView.current = next.view;
      setSession({ roomId: next.roomId, inviteCode: next.inviteCode, token: next.token, seat: next.seat });
      setView(next.view);
      // Invitations contain only a room code; seat credentials are stored locally.
      if (new URL(location.href).searchParams.has('invite')) history.replaceState({}, '', location.pathname);
    } catch (e) { setError(e instanceof Error ? e.message : '加入失败，请重试。'); }
    finally { commandLock.current = false; setBusy(false); }
  };

  const resolvePending = async (currentSession: SavedSession) => {
    const command = pending.current;
    if (unbound(command)) { setError(unboundMessage); setUncertain(true); refreshPolling.current?.(); return; }
    if (!commandFor(command, currentSession) || !command || commandLock.current) return;
    let capturedStorage: ReturnType<typeof playerStorage> | undefined;
    try { capturedStorage = playerStorage(); } catch { /* Continue with the in-memory original if storage is unavailable. */ }
    commandLock.current = true; setBusy(true); setError('');
    savePending(command, capturedStorage); // Preserve a legacy receipt separately before attempting recovery.
    let confirmed = false;
    try {
      let result: View;
      try { result = await sendCommand(currentSession, command.expectedVersion, command.action, command.commandId); }
      catch (firstError) {
        if (!isActive(currentSession)) throw firstError;
        if (!(firstError instanceof ApiError) || (firstError.status !== 0 && firstError.status < 500)) throw firstError;
        // A lost acknowledgement can hide a committed action. Retry only its original identity.
        result = await sendCommand(currentSession, command.expectedVersion, command.action, command.commandId);
      }
      accept(result); confirmed = true;
    }
    catch (e) {
      if (!isActive(currentSession)) return;
      if (retireUnsupportedRoom(currentSession, e)) return;
      if (retireInvalidSeat(currentSession, e)) return;
      confirmed = e instanceof ApiError && e.status >= 400 && e.status < 500 && !needsSiteLogin(e);
      let synced = false;
      if (e instanceof ApiError && e.view) synced = accept(e.view);
      else { try { synced = accept(await getState(currentSession)); } catch { /* Keep the last confirmed table visible. */ } }
      const readyConflict = e instanceof ApiError && e.status === 409 && e.code === 'version_conflict'
        && (command.action.kind === 'ready' || command.action.kind === 'game' && command.action.action?.kind === 'ready');
      const latest = acceptedView.current;
      const readyMessage = !synced || !latest ? '准备状态已变化，请查看最新牌桌后再确认。'
        : latest.status === 'playing' ? '牌桌已开始，已同步当前状态。'
        : latest.status === 'finished' ? '牌桌已结束，已同步当前状态。'
        : latest.players.find(player => player.id === latest.you)?.ready
          ? '准备状态已同步；你当前已准备。如需取消，请再点“取消准备”。'
          : '准备状态已同步；你当前尚未准备。需要准备时，请再点“准备”。';
      setError(needsSiteLogin(e) ? loginMessage : e instanceof ApiError && e.status === 409
        ? readyConflict ? readyMessage : '牌桌刚刚发生了变化，已同步最新状态。请查看当前可用行动后重新选择。'
        : !confirmed ? '行动结果暂未确认，已尝试同步牌桌。请确认上一行动后继续；重试不会重复执行。'
        : e instanceof Error ? `操作未执行：${e.message}` : '操作未执行，请重试。');
    } finally {
      if (isActive(currentSession)) {
        if (confirmed) {
          locallyConfirmed.current.add(confirmationKey(command, capturedStorage));
          clearPending(command, capturedStorage); pending.current = storedPending(currentSession, capturedStorage);
        }
        if (unbound(pending.current)) setError(unboundMessage);
        setUncertain(!confirmed || unbound(pending.current) || commandFor(pending.current, currentSession));
      }
      commandLock.current = false; setBusy(false);
    }
  };
  const act = async (action: Action) => {
    if (!session || !view || commandLock.current) return;
    pending.current ||= storedPending(session);
    if (pending.current) { await resolvePending(session); return; }
    if (view.pause && action.kind !== 'resumeRoom') { setError('此桌已暂停，请先恢复对局。'); return; }
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
    pending.current ||= storedPending(saved) || storedPending();
    if (pending.current && !unbound(pending.current) && !commandFor(pending.current, saved)) { setError('请先恢复待确认行动所在的原席位，再切换牌桌或席位。'); return; }
    saveSession(saved); setSavedSeats(readSavedSeats()); activeSession.current = saved; acceptedVersion.current = 0; acceptedView.current = null;
    setView(null); setSession(saved); setUncertain(!!pending.current); setError(unbound(pending.current) ? unboundMessage : '');
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
    pending.current ||= storedPending();
    if (pending.current) { setError(unbound(pending.current) ? unboundMessage : '当前会话还有待确认行动，请先恢复原席确认，再切换玩家会话。'); return; }
    try { selectPlayerMode(mode, mode === 'independent' ? newCommandId() : undefined); }
    catch (e) { setError(e instanceof Error ? e.message : '无法切换玩家会话。'); return; }
    const next = readActiveSession();
    pending.current = storedPending(next) || storedPending(); activeSession.current = next; acceptedVersion.current = 0; acceptedView.current = null;
    autoPassPreference.current = readAutoPassPreference(); setAutoPassState(autoPassPreference.current);
    setPlayerMode(mode); setSavedSeats(readSavedSeats()); setSession(next); setView(null);
    setUncertain(unbound(pending.current) || commandFor(pending.current, next)); setError(unbound(pending.current) ? unboundMessage : ''); setConnection('connecting');
    setCatalog(null); setCatalogRetry(n => n + 1);
  };

  return {
    catalog, session, view, error, busy, uncertain, connection, act, playerMode, autoPassEnabled, setAutoPassEnabled,
    legacyRecoveryBlocked: unbound(pending.current),
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
    refresh: () => refreshPolling.current?.(),
    savedSeats,
    resumeAvailable: !session && savedSeats.length > 0,
    resume,
    // Return to the lobby without destroying the only credential for an occupied seat.
    leave: () => { if (commandLock.current) return; returnToLobby(); activeSession.current = null; acceptedView.current = null; acceptedVersion.current = 0; setSession(null); setView(null); setError(unbound(pending.current) ? unboundMessage : ''); setUncertain(unbound(pending.current)); },
  };
}
