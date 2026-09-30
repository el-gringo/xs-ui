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
 * @typedef {(tag: string, ptr: number, len: number) => boolean} memoryInRange
 *
 * @typedef {Object} Memory
 * @property {memoryInRange} inRange
 * @property {(type: Function, ptr: number, len: number, tag?: string) => any} view
 *
 * @typedef {Object} WasmBridge
 * @property {Memory} memory
 * @property {Utf8} utf8
 * @property {Bytes} bytes
 *
 * @typedef {Object} App
 * @property {Heap} heap
 * @property {Memory} memory
 * @property {Utf8} utf8
 * @property {Bytes} bytes
 * @property {any} gl the WebGL2 context, resolved on first read
 * @property {{ __xs_malloc: (count: number, align: number ) => number, main: () => void }} wasm
 */
