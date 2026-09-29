/**
 * @typedef {Object} Heap
 * @property {(obj: any) => number} add
 * @property {(idx: number) => number} drop
 * @property {(idx: number) => any} get
 * @property {(idx: number) => any} take
 *
 * @typedef {(ptr: number, len: number) => string} stringFromWasm
 * @typedef {(str: string) => { ptr: number, len: number }} stringToWasm
 *
 * @typedef {Object} Utf8
 * @property {stringFromWasm} fromWasm
 * @property {stringToWasm} toWasm
 *
 *
 * @typedef {(ptr: number, len: number) => Uint8Array} bytesFromWasm
 * @typedef {(buffer: Array | Uint8Array | Uint8ClampedArray) => number} bytesToWasm
 * @typedef {(value: string) => number} bytesToCString
 *
 * @typedef {Object} Bytes
 * @property {bytesFromWasm} fromWasm
 * @property {bytesToWasm} toWasm
 * @property {bytesToCString} toCString
 *
 * @typedef {Object} WasmBridge
 * @property {Utf8} utf8
 * @property {Bytes} bytes
 *
 * @typedef {Object} App
 * @property {Heap} heap
 * @property {Utf8} utf8
 * @property {Bytes} bytes
 * @property {{ __xs_malloc: (count: number, align: number ) => number, main: () => void }} wasm
 */
