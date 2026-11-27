/* tslint:disable */
/* eslint-disable */

export class WasmTvc {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Loads a system or cartridge ROM. A `*DOS*` ROM name keeps its earlier
     * meaning and plugs an HBF card into slot 0.
     */
    addRom(name: string, data: Uint8Array | number[]): void;
    /**
     * Plugs a card into `slot` (0-3): `vtdos12` with its ROM in `data`, or
     * `hostfile`, backed by the files managed with `putHostFile`.
     */
    attachCard(slot: number, card: string, data: Uint8Array | number[]): void;
    audioSampleRate(): number;
    focusChange(has_focus: boolean): void;
    framebufferLen(): number;
    framebufferPtr(): number;
    getHostFile(name: string): Uint8Array | undefined;
    keyDown(code: number): boolean;
    keyPressText(text: string): void;
    keyUp(code: number): void;
    /**
     * Emulated seconds the last `runFrame` covered (one TV picture, about
     * 20.1 ms); the player paces by it instead of a fixed 1/50 s.
     */
    lastFrameSeconds(): number;
    listHostFiles(): string[];
    loadCartRom(data: Uint8Array | number[]): void;
    /**
     * Inserts a disk into drive 0/1 of the first HBF card.
     */
    loadDisk(drive: number, name: string, data: Uint8Array | number[]): void;
    loadSlotDisk(slot: number, drive: number, name: string, data: Uint8Array | number[]): void;
    loadSnapshot(data: Uint8Array | number[]): void;
    constructor(is_plus: boolean);
    putHostFile(name: string, data: Uint8Array | number[]): void;
    reset(): void;
    runFrame(): boolean;
    saveSnapshot(): Uint8Array;
    screenHeight(): number;
    screenWidth(): number;
    setVidModel(model: string): void;
    takeAudioSamples(): Float32Array;
    takeFrameComplete(): boolean;
    /**
     * Queues text to be typed one key at a time over the next frames, for
     * keyboards that deliver characters without a usable key code (phone
     * soft keyboards). `\r` is Return and `\x08` is Backspace.
     */
    typeText(text: string): void;
    vidModel(): string;
}

export class WebHandle {
    free(): void;
    [Symbol.dispose](): void;
    constructor();
    start(canvas_id: string): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmtvc_free: (a: number, b: number) => void;
    readonly __wbg_webhandle_free: (a: number, b: number) => void;
    readonly wasmtvc_addRom: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmtvc_attachCard: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly wasmtvc_audioSampleRate: (a: number) => number;
    readonly wasmtvc_focusChange: (a: number, b: number) => void;
    readonly wasmtvc_framebufferLen: (a: number) => number;
    readonly wasmtvc_framebufferPtr: (a: number) => number;
    readonly wasmtvc_getHostFile: (a: number, b: number, c: number, d: number) => void;
    readonly wasmtvc_keyDown: (a: number, b: number) => number;
    readonly wasmtvc_keyPressText: (a: number, b: number, c: number) => void;
    readonly wasmtvc_keyUp: (a: number, b: number) => void;
    readonly wasmtvc_lastFrameSeconds: (a: number) => number;
    readonly wasmtvc_listHostFiles: (a: number, b: number) => void;
    readonly wasmtvc_loadCartRom: (a: number, b: number, c: number) => void;
    readonly wasmtvc_loadDisk: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wasmtvc_loadSlotDisk: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly wasmtvc_loadSnapshot: (a: number, b: number, c: number, d: number) => void;
    readonly wasmtvc_new: (a: number) => number;
    readonly wasmtvc_putHostFile: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wasmtvc_reset: (a: number) => void;
    readonly wasmtvc_runFrame: (a: number) => number;
    readonly wasmtvc_saveSnapshot: (a: number, b: number) => void;
    readonly wasmtvc_screenHeight: (a: number) => number;
    readonly wasmtvc_screenWidth: (a: number) => number;
    readonly wasmtvc_setVidModel: (a: number, b: number, c: number, d: number) => void;
    readonly wasmtvc_takeAudioSamples: (a: number, b: number) => void;
    readonly wasmtvc_takeFrameComplete: (a: number) => number;
    readonly wasmtvc_typeText: (a: number, b: number, c: number) => void;
    readonly wasmtvc_vidModel: (a: number, b: number) => void;
    readonly webhandle_new: () => number;
    readonly webhandle_start: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_5129: (a: number, b: number, c: number, d: any) => void;
    readonly __wasm_bindgen_func_elem_5151: (a: number, b: number, c: any, d: any) => void;
    readonly __wasm_bindgen_func_elem_1789: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_1789_33: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_1791: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_1562: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export4: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_export5: (a: number) => void;
    readonly __wbindgen_export6: (a: number, b: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export7: (a: number) => void;
    readonly __wbindgen_export8: (a: number, b: number) => void;
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
