import { useEffect, useState } from 'react';
import type { SavedSession } from './types';

const CHANNEL = 'hegemony.seatPresence.v1';
const HEARTBEAT_MS = 15_000;
// Background tabs commonly run timers only once a minute. Keep them visible to
// peers across that delay, while eventually forgetting crashed/frozen pages.
const PEER_TIMEOUT_MS = 120_000;
type SeatIdentity = Pick<SavedSession, 'roomId' | 'seat'>;
type Presence = SeatIdentity & { kind: 'hello' | 'alive' | 'bye'; tabId: string };
type Channel = { send: (message: Presence) => void; close: () => void };

function isPresence(value: unknown): value is Presence {
  if (!value || typeof value !== 'object') return false;
  const message = value as Partial<Presence>;
  return (message.kind === 'hello' || message.kind === 'alive' || message.kind === 'bye')
    && typeof message.tabId === 'string' && message.tabId.length > 0 && message.tabId.length <= 128
    && typeof message.roomId === 'string' && message.roomId.length > 0
    && Number.isSafeInteger(message.seat) && message.seat! >= 0;
}

function openChannel(receive: (message: unknown) => void): Channel | null {
  try {
    if (typeof BroadcastChannel !== 'undefined') {
      const channel = new BroadcastChannel(CHANNEL);
      channel.addEventListener('message', event => receive(event.data));
      return {
        send: message => { try { channel.postMessage(message); } catch { /* Presence is advisory. */ } },
        close: () => channel.close(),
      };
    }
  } catch { /* Older/restricted browsers can still exchange storage events. */ }
  try {
    const storage = window.localStorage;
    const onStorage = (event: StorageEvent) => {
      if (event.storageArea !== storage || event.key !== CHANNEL || !event.newValue) return;
      try { receive(JSON.parse(event.newValue)); } catch { /* Ignore damaged or unrelated messages. */ }
    };
    window.addEventListener('storage', onStorage);
    return {
      send: message => {
        try {
          storage.setItem(CHANNEL, JSON.stringify(message));
          storage.removeItem(CHANNEL); // No durable presence/identity record or stale lease to recover.
        } catch { /* Storage denial must not block play. */ }
      },
      close: () => window.removeEventListener('storage', onStorage),
    };
  } catch { return null; }
}

/**
 * An advisory for matching active seats in reachable same-origin tabs. Storage
 * namespaces do not change seat identity: duplicated independent tabs can share it.
 * It is neither a command lock nor a cross-device login check. Suspended pages
 * may disappear after the grace period and are rediscovered when they resume.
 */
export function useSeatConflict(session: SeatIdentity | null): boolean {
  const roomId = session?.roomId;
  const seat = session?.seat;
  const identity = roomId !== undefined && seat !== undefined ? JSON.stringify([roomId, seat]) : null;
  // Keep this nonce in memory: browsers copy sessionStorage when duplicating a tab.
  const [tabId] = useState(() => globalThis.crypto?.randomUUID?.()
    || `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`);
  const [conflictFor, setConflictFor] = useState<string | null>(null);

  useEffect(() => {
    setConflictFor(null);
    if (roomId === undefined || seat === undefined || identity === null) return;
    const peers = new Map<string, number>();
    let disposed = false;
    let present = true;
    let channel: Channel | null = null;
    const report = () => setConflictFor(peers.size ? identity : null);
    const announce = (kind: Presence['kind']) => channel?.send({ kind, tabId, roomId, seat });
    const prune = () => {
      const now = Date.now();
      for (const [peer, seen] of peers) if (now - seen > PEER_TIMEOUT_MS) peers.delete(peer);
      report();
    };
    channel = openChannel(value => {
      if (disposed || !present || !isPresence(value) || value.tabId === tabId
        || value.roomId !== roomId || value.seat !== seat) return;
      if (value.kind === 'bye') peers.delete(value.tabId);
      else peers.set(value.tabId, Date.now());
      report();
      if (value.kind === 'hello') announce('alive');
    });
    if (!channel) return;
    const heartbeat = setInterval(() => {
      if (!present) return;
      prune(); announce('alive');
    }, HEARTBEAT_MS);
    const hide = () => {
      if (!present) return;
      present = false; peers.clear(); report(); announce('bye');
    };
    const show = () => { present = true; prune(); announce('hello'); };
    const visible = () => { if (present && document.visibilityState === 'visible') show(); };
    window.addEventListener('pagehide', hide);
    window.addEventListener('pageshow', show);
    document.addEventListener('visibilitychange', visible);
    announce('hello');
    return () => {
      disposed = true;
      if (present) announce('bye');
      clearInterval(heartbeat);
      window.removeEventListener('pagehide', hide);
      window.removeEventListener('pageshow', show);
      document.removeEventListener('visibilitychange', visible);
      channel.close();
    };
  }, [roomId, seat, identity, tabId]);

  return identity !== null && conflictFor === identity;
}
