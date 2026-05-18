/**
 * @import { App, Utf8, Bytes, WasmBridge } from "./types.js";
 *
 * @param {App} app;
 * @returns {WasmBridge}
 */
function WasmBridge(app) {
  const cachedTextDecoder = new TextDecoder("utf-8", {
    ignoreBOM: true,
    fatal: true,
  });
  const cachedTextEncoder = new TextEncoder("utf-8");

  let cachedUint8Memory = null;
  let cachedUint8ArrayMemory = null;
  let WASM_VECTOR_LEN = 0;

  function Utf8() {
    /** @type stringFromWasm */
    function fromWasm(ptr, len) {
      return cachedTextDecoder.decode(
        getUint8Memory().subarray(ptr, ptr + len),
      );
    }

    /** @type stringToWasm */
    function toWasm(str) {
      const { __xs_malloc: malloc, __xs_realloc: realloc } = app.wasm;
      if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(str);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory()
          .subarray(ptr, ptr + buf.length)
          .set(buf);
        WASM_VECTOR_LEN = buf.length;
        const len = WASM_VECTOR_LEN;
        return { ptr, len };
      }

      let len = str.length;
      let ptr = malloc(len, 1) >>> 0;

      const mem = getUint8ArrayMemory();

      let offset = 0;

      for (; offset < len; offset++) {
        const code = str.charCodeAt(offset);
        if (code > 0x7f) break;
        mem[ptr + offset] = code;
      }
      if (offset !== len) {
        if (offset !== 0) {
          str = str.slice(offset);
        }
        ptr = realloc(ptr, len, (len = offset + str.length * 3), 1) >>> 0;
        const view = getUint8ArrayMemory().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(str, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
      }
      WASM_VECTOR_LEN = offset;
      return { ptr, len: WASM_VECTOR_LEN };
    }

    return {
      fromWasm,
      toWasm,
    };
  }

  function Bytes() {
    /** @type bytesFromWasm */
    function fromWasm(ptr, len) {
      return getUint8ArrayMemory().subarray(ptr, ptr + len);
    }

    /** @type bytesToWasm */
    function toWasm(buffer) {
      const { __xs_malloc: malloc } = app.wasm;
      let bytes = new Uint8Array(buffer);
      const ptr = malloc(bytes.byteLength, 1) >>> 0;
      getUint8ArrayMemory()
        .subarray(ptr, ptr + bytes.byteLength)
        .set(bytes);
      const len = (WASM_VECTOR_LEN = bytes.byteLength);
      return { ptr, len };
    }

    return { fromWasm, toWasm };
  }

  function getUint8Memory() {
    if (cachedUint8Memory === null || cachedUint8Memory.byteLength === 0) {
      cachedUint8Memory = new Uint8Array(app.wasm.memory.buffer);
    }
    return cachedUint8Memory;
  }

  function getUint8ArrayMemory() {
    if (
      cachedUint8ArrayMemory === null ||
      cachedUint8ArrayMemory.byteLength === 0
    ) {
      cachedUint8ArrayMemory = new Uint8Array(app.wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory;
  }

  return {
    utf8: Utf8(),
    bytes: Bytes(),
  };
}
window.WasmBridge = WasmBridge;
define(WasmBridge);
