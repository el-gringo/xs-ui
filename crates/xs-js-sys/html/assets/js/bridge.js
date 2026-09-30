/**
 * @import { App, Utf8, Bytes, Memory, WasmBridge } from "./types.js";
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

  let WASM_VECTOR_LEN = 0;

  /**
   * The only place that reads and writes the wasm memory. The bindings must go
   * through it: a view is only valid until the memory grows, and Chrome 63 (the
   * webview the older TVs ship) grows it *in place* - `memory.buffer` keeps the
   * same object while the old views are detached and turn into empty ones. A
   * view cached by buffer identity therefore hands gl a short array, so views
   * are built on every call over the live memory, which costs the same as the
   * subarray it replaces.
   *
   * The range is checked first as well. A null pointer or an out of bounds one
   * is a RangeError on a current V8, but Chrome 63 hits a DCHECK and takes the
   * whole renderer down with it, so it is reported and skipped here.
   * @type Memory
   */
  function Memory() {
    function inRange(tag, ptr, len) {
      const wasm = app.wasm;
      if (wasm === undefined) {
        console.error(`${tag}: the wasm module is not instantiated yet`);
        return false;
      }
      if (ptr === 0) {
        console.error(`${tag}: null pointer into the wasm memory, skipped`);
        return false;
      }
      const size = wasm.memory.buffer.byteLength;
      if (ptr < 0 || len < 0 || ptr + len > size) {
        console.error(
          `${tag}: [${ptr}, ${ptr + len}) is outside the ${size} bytes of the ` +
            `wasm memory, skipped`,
        );
        return false;
      }
      return true;
    }

    /**
     * A view of `len` elements of `Type` at `ptr`. Empty when the range is not
     * usable, after reporting why.
     */
    function view(Type, ptr, len, tag = "view") {
      if (!inRange(tag, ptr, len)) {
        return new Type(0);
      }
      return new Type(app.wasm.memory.buffer, ptr, len);
    }

    return { inRange, view };
  }

  const memory = Memory();

  function Utf8() {
    /** @type stringFromWasm */
    function fromWasm(ptr, len) {
      return cachedTextDecoder.decode(
        memory.view(Uint8Array, ptr, len, "utf8.fromWasm"),
      );
    }

    /** @type stringToWasm */
    function toWasm(str) {
      const { __xs_malloc: malloc, __xs_realloc: realloc } = app.wasm;
      if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(str);
        const ptr = malloc(buf.length, 1) >>> 0;
        memory
          .view(Uint8Array, ptr, buf.length, "utf8.toWasm")
          .set(buf);
        WASM_VECTOR_LEN = buf.length;
        const len = WASM_VECTOR_LEN;
        return { ptr, len };
      }

      let len = str.length;
      let ptr = malloc(len, 1) >>> 0;
      const mem = memory.view(Uint8Array, ptr, len, "utf8.toWasm");

      // The ascii part is written by hand, only the tail needs the encoder.
      let offset = 0;
      for (; offset < len; offset++) {
        const code = str.charCodeAt(offset);
        if (code > 0x7f) break;
        mem[offset] = code;
      }
      if (offset !== len) {
        if (offset !== 0) {
          str = str.slice(offset);
        }
        ptr = realloc(ptr, len, (len = offset + str.length * 3), 1) >>> 0;
        // `TextEncoder.encodeInto` is Chrome 73, the encoding is done by hand so
        // the same code runs on the old webviews.
        const encoded = cachedTextEncoder.encode(str);
        memory
          .view(Uint8Array, ptr, encoded.length, "utf8.toWasm")
          .set(encoded);
        offset += encoded.length;
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
      return memory.view(Uint8Array, ptr, len, "bytes.fromWasm");
    }

    /** @type bytesToWasm */
    function toWasm(buffer) {
      const { __xs_malloc: malloc } = app.wasm;
      let bytes = new Uint8Array(buffer);
      const ptr = malloc(bytes.byteLength, 1) >>> 0;
      if (!memory.inRange("bytes.toWasm", ptr, bytes.byteLength)) {
        return { ptr: 0, len: 0 };
      }
      memory
        .view(Uint8Array, ptr, bytes.byteLength, "bytes.toWasm")
        .set(bytes);
      const len = (WASM_VECTOR_LEN = bytes.byteLength);
      return { ptr, len };
    }

    /**
     * Writes a js string in wasm memory as a null terminated c string, for the
     * bindings that hand a string back to be read as a `CStr`. The buffer comes
     * from the heap allocator and is not freed.
     * @type bytesToCString
     */
    function toCString(value) {
      return toWasm(cachedTextEncoder.encode(`${value}\0`)).ptr;
    }

    return { fromWasm, toWasm, toCString };
  }

  return {
    memory,
    utf8: Utf8(),
    bytes: Bytes(),
  };
}
window.WasmBridge = WasmBridge;
define(WasmBridge);
