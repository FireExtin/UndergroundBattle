import { useCallback, useEffect, useRef, useState } from 'react';
import { actionPayload, ApiError, createRoom, getCatalog, getState, joinRoom, newCommandId, pollState, readPending, readSession, savePending, saveSession, sendCommand, type PendingCommand } from './api';
import type { Action, Catalog, SavedSession, Session, View } from './types';

export function newerView(current: View | null, next: View): View {
  return current && current.roomId === next.roomId && current.version > next.version ? current : next;
}

export function useGame() {
  const [catalog, setCatalog] = useState<Catalog | null>(null);
  const [session, setSession] = useState<SavedSession | null>(readSession);
  const [view, setView] = useState<View | null>(null);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [connection, setConnection] = useState<'connecting' | 'online' | 'offline'>('connecting');
  const [catalogRetry, setCatalogRetry] = useState(0);
  const commandLock = useRef(false);
  const pending = useRef<PendingCommand | null>(readPending());
  const [uncertain, setUncertain] = useState(!!pending.current);
  const activeRoom = useRef(session?.roomId);
  const acceptedVersion = useRef(0);
  activeRoom.current = session?.roomId;
  const accept = useCallback((next: View) => {
    if (next.roomId === activeRoom.current) {
      acceptedVersion.current = Math.max(acceptedVersion.current, next.version);
      setView(current => newerView(current, next));
    }
  }, []);

  useEffect(() => {
    const controller = new AbortController();
    getCatalog(controller.signal).then(setCatalog).catch(e => { if (!controller.signal.aborted) setError(e.message); });
    return () => controller.abort();
  }, [catalogRetry]);

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
        if (controller.signal.aborted) return;
        if (state) accept(state);
        setConnection('online'); attempts = 0; initialized = true;
        if (pending.current?.roomId === session.roomId && !commandLock.current) {
          await resolvePending(session);
        }
        if (!controller.signal.aborted) timer = setTimeout(reconnect, document.visibilityState === 'hidden' ? 12000 : 1500);
        return;
      } catch (e) {
        if (controller.signal.aborted) return;
        if (e instanceof ApiError && (e.status === 401 || e.status === 403 || e.status === 404)) {
          saveSession(null); savePending(null); pending.current = null; setUncertain(false); setSession(null); setView(null);
          setError('保存的座位已无法恢复。请使用邀请码重新加入牌桌。'); return;
        }
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
    commandLock.current = true; setBusy(true); setError('');
    try {
      const next = await task();
      pending.current = null; savePending(null); setUncertain(false);
      saveSession(next); activeRoom.current = next.roomId;
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
    if (!command || command.roomId !== currentSession.roomId || commandLock.current) return;
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
      if (activeRoom.current !== command.roomId) return;
      confirmed = e instanceof ApiError && e.status >= 400 && e.status < 500;
      if (e instanceof ApiError && e.view) accept(e.view);
      else { try { accept(await getState(currentSession)); } catch { /* Keep the last confirmed table visible. */ } }
      setError(e instanceof ApiError && e.status === 409
        ? '牌桌刚刚发生了变化，已同步最新状态。请查看当前可用行动后重新选择。'
        : !confirmed ? '行动结果暂未确认，已尝试同步牌桌。请确认上一行动后继续；重试不会重复执行。'
        : e instanceof Error ? `操作未执行：${e.message}` : '操作未执行，请重试。');
    } finally {
      if (activeRoom.current === command.roomId) {
        if (confirmed) { pending.current = null; savePending(null); }
        setUncertain(!confirmed);
      }
      commandLock.current = false; setBusy(false);
    }
  };
  const act = async (action: Action) => {
    if (!session || !view || commandLock.current) return;
    if (pending.current) { await resolvePending(session); return; }
    pending.current = { roomId: session.roomId, commandId: newCommandId(), expectedVersion: view.version, action: actionPayload(action) };
    savePending(pending.current);
    await resolvePending(session);
  };

  return {
    catalog, session, view, error, busy, uncertain, connection, act,
    create: (name: string, mode: 'duel' | 'teams', deckId: string) => enter(() => createRoom(name, mode, deckId)),
    join: (inviteCode: string, name: string, deckId: string) => enter(() => joinRoom(inviteCode, name, deckId)),
    dismissError: () => setError(''),
    retryCatalog: () => { setError(''); setCatalogRetry(n => n + 1); },
    retryPending: () => { if (session) void resolvePending(session); },
    resumeAvailable: !session && !!readSession(),
    resume: () => { const saved = readSession(); if (saved) { activeRoom.current = saved.roomId; acceptedVersion.current = 0; setSession(saved); setUncertain(!!pending.current); setError(''); } },
    // Return to the lobby without destroying the only credential for an occupied seat.
    leave: () => { activeRoom.current = undefined; setSession(null); setView(null); setError(''); setUncertain(false); },
  };
}
