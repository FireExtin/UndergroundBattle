// Current catalog and newly created rooms. Recorded views in consumers are
// fixture UI inputs; they are not current Native/WASM replay evidence.
import { kernel, catalog } from '../../../rust-game-wasm/tests/current-kernel.mjs';

export { kernel, catalog };
export const definitions = new Map(catalog.cards.map(c => [c.id, c]));
