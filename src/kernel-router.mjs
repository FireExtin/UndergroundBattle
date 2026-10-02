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
    joinGame: (state, ...args) => select(state).joinGame(state, ...args),
    apply: (state, ...args) => select(state).apply(state, ...args),
    view: (state, ...args) => select(state).view(state, ...args),
  };
}
