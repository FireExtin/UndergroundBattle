// Test-only production Game::new bridge. Keep the authoritative state opaque in JS.
import { readFileSync } from 'node:fs';

const [bindingsPath, wasmPath] = process.argv.slice(2);
const request = JSON.parse(readFileSync(0, 'utf8'));
if (typeof request.seedDecimal !== 'string') throw new Error('Seed must remain a decimal string');
const bindings = await import('data:text/javascript;base64,' + readFileSync(bindingsPath).toString('base64'));
bindings.initSync({ module: readFileSync(wasmPath) });
process.stdout.write(bindings.newGame(request.roomId, request.inviteCode, request.mode,
  request.name, request.deckId, request.seedDecimal));
