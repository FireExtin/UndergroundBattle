// Routing reads only Rust's small identity envelope; private state stays opaque.
export function routeKernels(current, previous) {
  const kernels = [current, ...(Array.isArray(previous) ? previous : [previous])]
    .map(kernel => ({ kernel, catalog: JSON.parse(kernel.catalog()) }));
  const matches = (version, catalog) => version.rules === catalog.rulesVersion
    && version.cardPool === catalog.cardPoolVersion && version.engine === catalog.engineVersion;
  const select = state => {
    const identity = JSON.parse(current.stateIdentity(state));
    if (identity.state_schema !== 2) throw 'Unsupported persisted state schema';
    const selected = kernels.find(({ catalog }) => matches(identity.versions, catalog));
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
  };
}
