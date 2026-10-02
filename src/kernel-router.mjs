// Routing reads only Rust's small identity envelope; private state stays opaque.
export function routeKernels(current, previous) {
  const currentVersions = JSON.parse(current.catalog());
  const previousVersions = JSON.parse(previous.catalog());
  const matches = (version, catalog) => version.rules === catalog.rulesVersion
    && version.cardPool === catalog.cardPoolVersion && version.engine === catalog.engineVersion;
  const select = state => {
    const identity = JSON.parse(current.stateIdentity(state));
    if (identity.state_schema !== 2) throw 'Unsupported persisted state schema';
    if (matches(identity.versions, currentVersions)) return current;
    if (matches(identity.versions, previousVersions)) return previous;
    throw 'Unsupported persisted rules/card-pool/engine version';
  };
  return {
    catalog: () => current.catalog(),
    newGame: (...args) => current.newGame(...args),
    joinGame: (state, ...args) => select(state).joinGame(state, ...args),
    apply: (state, ...args) => select(state).apply(state, ...args),
    view: (state, ...args) => select(state).view(state, ...args),
  };
}
