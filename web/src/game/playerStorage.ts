/** A namespace selects browser storage; it never authenticates a seat or goes to the server. */
const MODE_KEY = 'hegemony.playerSession.v1';
export type PlayerMode = 'ordinary' | 'independent';
function namespace(): string | null {
  const id = sessionStorage.getItem(MODE_KEY);
  if (id === null) return null;
  if (!/^[a-zA-Z0-9-]{16,64}$/.test(id)) throw new Error('独立玩家会话存储已损坏，请显式切回普通玩家会话。');
  return id;
}
export function readPlayerMode(): PlayerMode {
  try { return namespace() ? 'independent' : 'ordinary'; }
  catch { return 'independent'; } // Do not silently read ordinary credentials on a storage failure.
}
export function playerStorage() {
  const id = namespace();
  const storage = id ? sessionStorage : localStorage;
  const keyFor = (key: string) => id ? `hegemony.player.${id}.${key}` : key;
  return { scope: id || 'ordinary',
    getItem: (key: string) => storage.getItem(keyFor(key)),
    setItem: (key: string, value: string) => storage.setItem(keyFor(key), value),
    removeItem: (key: string) => storage.removeItem(keyFor(key)),
  };
}
export function selectPlayerMode(mode: PlayerMode, id?: string) {
  try {
    if (mode === 'independent') {
      if (!id || !/^[a-zA-Z0-9-]{16,64}$/.test(id)) throw new Error('invalid namespace');
      sessionStorage.setItem(MODE_KEY, id);
      if (namespace() !== id) throw new Error('tab storage did not persist');
    } else {
      sessionStorage.removeItem(MODE_KEY);
      if (namespace() !== null) throw new Error('tab storage did not clear');
    }
  } catch { throw new Error('无法保存独立玩家会话设置，请检查当前标签是否允许浏览器存储。原座位未更改。'); }
}
