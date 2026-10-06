// Instantiate a frozen core only when its ABI is actually used.
export function lazyKernel(abi, module, catalogIdentity) {
  const identity = Object.freeze({ ...catalogIdentity });
  let catalog;
  const initialize = () => {
    if (catalog !== undefined) return;
    abi.initSync({ module });
    const serialized = abi.catalog();
    const actual = JSON.parse(serialized);
    for (const key of ['rulesVersion', 'cardPoolVersion', 'engineVersion']) {
      if (actual[key] !== identity[key]) throw new Error('Frozen kernel identity mismatch');
    }
    catalog = serialized;
  };
  const kernel = { catalogIdentity: identity };
  for (const [name, fn] of Object.entries(abi)) {
    if (typeof fn !== 'function' || name === 'default' || name === 'initSync') continue;
    kernel[name] = (...args) => {
      initialize();
      return name === 'catalog' ? catalog : fn(...args);
    };
  }
  return kernel;
}
