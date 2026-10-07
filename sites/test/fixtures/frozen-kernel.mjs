// Test-only historical ABIs; never copied to generated/ or imported by the Worker.
import { existsSync, readFileSync } from 'node:fs';
const root = existsSync(new URL('../../../rust-game-wasm/Cargo.toml', import.meta.url))
  ? new URL('../../../rust-game-wasm/', import.meta.url)
  : new URL('../../rust-game-wasm/', import.meta.url);
export async function frozenKernel(folder) {
  const abi = await import(new URL(folder + '/hegemony_wasm.js', root));
  abi.initSync({ module: readFileSync(new URL(folder + '/hegemony_wasm_bg.wasm', root)) });
  return abi;
}
