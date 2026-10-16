// Exact published runtime for preserved ABI77 Native/D1 oracle inputs.
import { existsSync } from 'node:fs';
import { pathToFileURL, fileURLToPath } from 'node:url';
const project = fileURLToPath(new URL('../', import.meta.url));
const root = existsSync(project + 'web/package.json') ? project : fileURLToPath(new URL('../../', import.meta.url));
const { frozenAbi77 } = await import(pathToFileURL(root + 'tools/frozen-kernels.mjs').href);
const retained = frozenAbi77();
export const moduleBytes = retained.wasm;
export const kernel = await import(pathToFileURL(retained.jsPath).href);
kernel.initSync({ module: moduleBytes });
for (const key of ['rulesVersion', 'cardPoolVersion', 'engineVersion']) {
  if (JSON.parse(kernel.catalog())[key] !== retained.identity.catalogIdentity[key]) throw Error('Published ABI77 catalog mismatch');
}
