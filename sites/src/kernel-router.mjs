// Routing reads only Rust's small identity envelope; private state stays opaque.
export function routeKernels(current, previous) {
  const kernels = [current, ...(Array.isArray(previous) ? previous : [previous])]
    .map(kernel => ({ kernel, catalog: kernel.catalogIdentity ?? JSON.parse(kernel.catalog()) }));
  const matches = (version, catalog) => version.rules === catalog.rulesVersion
    && version.cardPool === catalog.cardPoolVersion && version.engine === catalog.engineVersion;
  const select = state => {
    const identity = JSON.parse(current.stateIdentity(state));
    if (![2, 3].includes(identity.state_schema)) throw 'Unsupported persisted state schema';
    const selected = kernels.find(({ kernel, catalog }) => matches(identity.versions, catalog)
      && identity.state_schema === (kernel.pollRoom ? 3 : 2));
    if (selected) return selected.kernel;
    throw 'Unsupported persisted rules/card-pool/engine version';
  };
  return {
    catalog: state => (state === undefined ? current : select(state)).catalog(),
    newGame: (...args) => current.newGame(...args),
    newGameWithDeck: (...args) => {
      if (!current.newGameWithDeck) throw '此版本尚未支持自定义牌组';
      return current.newGameWithDeck(...args);
    },
    joinGame: (state, ...args) => select(state).joinGame(state, ...args),
    joinGameWithDeck: (state, ...args) => {
      const selected = select(state);
      if (!selected.joinGameWithDeck) throw '这张旧牌桌保留原规则，只能选择其原有预组';
      return selected.joinGameWithDeck(state, ...args);
    },
    apply: (state, seat, action) => {
      const selected = select(state);
      if (JSON.parse(action).deckDraft != null && !selected.joinGameWithDeck) throw '这张旧牌桌保留原规则，只能选择其原有预组';
      return selected.apply(state, seat, action);
    },
    view: (state, ...args) => select(state).view(state, ...args),
    supportsPacing: state => !!select(state).pollRoom,
    applyRoom: (state, ...args) => {
      const selected = select(state);
      if (!selected.applyRoom) throw '旧牌桌保留原响应规则';
      return selected.applyRoom(state, ...args);
    },
    pollRoom: (state, ...args) => {
      const selected = select(state);
      if (!selected.pollRoom) throw '旧牌桌保留原响应规则';
      return selected.pollRoom(state, ...args);
    },
    quoteRoom: (state, ...args) => {
      const selected = select(state);
      if (!selected.quoteRoom) throw '旧牌桌尚不支持响应报价';
      return selected.quoteRoom(state, ...args);
    },
  };
}
