import { HttpError } from './service.mjs';

const unsupported = () => new HttpError(410,
  '旧牌桌使用的规则已停止支持。请返回大厅新建牌桌；旧牌桌数据仍保留。',
  undefined, 'unsupported_room_version');

// Check only Rust's small identity envelope; private state stays opaque.
export function routeKernels(current) {
  const catalog = JSON.parse(current.catalog());
  const matches = (version, catalog) => version.rules === catalog.rulesVersion
    && version.cardPool === catalog.cardPoolVersion && version.engine === catalog.engineVersion;
  const select = state => {
    let identity;
    try { identity = JSON.parse(current.stateIdentity(state)); }
    catch (error) { if (error === 'Unsupported persisted state schema') throw unsupported(); throw error; }
    if (identity.state_schema !== 3 || !matches(identity.versions, catalog)) throw unsupported();
    return current;
  };
  return {
    assertSupported: state => { select(state); },
    catalog: state => (state === undefined ? current : select(state)).catalog(),
    newGame: (...args) => current.newGame(...args),
    newGameWithDeck: (...args) => current.newGameWithDeck(...args),
    joinGame: (state, ...args) => select(state).joinGame(state, ...args),
    joinGameWithDeck: (state, ...args) => select(state).joinGameWithDeck(state, ...args),
    view: (state, ...args) => select(state).view(state, ...args),
    applyRoom: (state, ...args) => select(state).applyRoom(state, ...args),
    pollRoom: (state, ...args) => select(state).pollRoom(state, ...args),
    quoteRoom: (state, ...args) => select(state).quoteRoom(state, ...args),
  };
}
