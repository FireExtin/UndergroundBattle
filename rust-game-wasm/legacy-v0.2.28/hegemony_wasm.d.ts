/* tslint:disable */
/* eslint-disable */
export function stateIdentity(state: string): string;
export function catalog(): string;
export function newGame(room_id: string, invite_code: string, mode: string, name: string, deck_id: string, seed_decimal: string): string;
export function joinGame(state: string, name: string, deck_id: string): string;
export function newGameWithDeck(room_id: string, invite_code: string, mode: string, name: string, deck_json: string, seed_decimal: string): string;
export function joinGameWithDeck(state: string, name: string, deck_json: string): string;
export function roomCatalog(state: string, seat: number): string;
export function apply(_state: string, _seat: number, _action_json: string): string;
export function applyRoom(state: string, seat: number, command_json: string, server_now_decimal: string): string;
export function pollRoom(state: string, seat: number, server_now_decimal: string): string;
export function quoteRoom(state: string, seat: number, request_json: string): string;
export function view(state: string, seat: number): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly stateIdentity: (a: number, b: number) => [number, number, number, number];
  readonly catalog: () => [number, number, number, number];
  readonly newGame: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => [number, number, number, number];
  readonly joinGame: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
  readonly newGameWithDeck: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => [number, number, number, number];
  readonly joinGameWithDeck: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
  readonly roomCatalog: (a: number, b: number, c: number) => [number, number, number, number];
  readonly apply: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
  readonly applyRoom: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
  readonly pollRoom: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
  readonly quoteRoom: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
  readonly view: (a: number, b: number, c: number) => [number, number, number, number];
  readonly __wbindgen_export_0: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
