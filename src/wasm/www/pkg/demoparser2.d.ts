/* tslint:disable */
/* eslint-disable */
/**
* @param {Uint8Array} file
* @param {string | undefined} event_name
* @param {any[] | undefined} wanted_player_props
* @param {any[] | undefined} wanted_other_props
* @returns {any}
*/
export function parseEvent(file: Uint8Array, event_name?: string, wanted_player_props?: any[], wanted_other_props?: any[]): any;
/**
* @param {Uint8Array} file
* @param {any[] | undefined} wanted_props
* @param {Int32Array | undefined} wanted_ticks
* @param {boolean | undefined} struct_of_arrays
* @returns {any}
*/
export function parseTicks(file: Uint8Array, wanted_props?: any[], wanted_ticks?: Int32Array, struct_of_arrays?: boolean): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly parseEvent: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
  readonly parseTicks: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
  readonly memory: WebAssembly.Memory;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_thread_destroy: (a: number, b: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
* @param {WebAssembly.Memory} maybe_memory
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput, maybe_memory?: WebAssembly.Memory): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
* @param {WebAssembly.Memory} maybe_memory
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>, maybe_memory?: WebAssembly.Memory): Promise<InitOutput>;
