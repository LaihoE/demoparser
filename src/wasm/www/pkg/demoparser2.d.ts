declare namespace wasm_bindgen {
	/* tslint:disable */
	/* eslint-disable */
	export function parseEvent(file: Uint8Array, event_name?: string | null, wanted_player_props?: any[] | null, wanted_other_props?: any[] | null): any;
	export function parseEvents(file: Uint8Array, event_names?: any[] | null, wanted_player_props?: any[] | null, wanted_other_props?: any[] | null): any;
	export function listGameEvents(fileBytes: Uint8Array): any;
	export function listUpdatedFields(fileBytes: Uint8Array): any;
	export function parseTicks(file: Uint8Array, wanted_props?: any[] | null, wanted_ticks?: Int32Array | null, wanted_players?: any[] | null, struct_of_arrays?: boolean | null): any;
	export function parseGrenades(file: Uint8Array): any;
	export function parseHeader(file: Uint8Array): any;
	
}

declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly parseEvent: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number];
  readonly parseEvents: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number];
  readonly listGameEvents: (a: number, b: number) => [number, number, number];
  readonly listUpdatedFields: (a: number, b: number) => [number, number, number];
  readonly parseTicks: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number];
  readonly parseGrenades: (a: number, b: number) => [number, number, number];
  readonly parseHeader: (a: number, b: number) => [number, number, number];
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_4: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
declare function wasm_bindgen (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
