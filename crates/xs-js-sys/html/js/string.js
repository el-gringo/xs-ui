function WasmString(wasm) {
  const cachedTextDecoder = new TextDecoder("utf-8", {
    ignoreBOM: true,
    fatal: true,
  });

  let cachedUint8Memory = null;
  function getUint8Memory0() {
    if (cachedUint8Memory === null || cachedUint8Memory.byteLength === 0) {
      cachedUint8Memory = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory;
  }

  function fromWasm(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
  }

  let WASM_VECTOR_LEN = 0;
  const cachedTextEncoder = new TextEncoder("utf-8");

  let cachedUint8ArrayMemory = null;
  function getUint8ArrayMemory() {
    if (
      cachedUint8ArrayMemory === null ||
      cachedUint8ArrayMemory.byteLength === 0
    ) {
      cachedUint8ArrayMemory = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory;
  }

  /* oxlint-disable-next-line: no-unused-vars */
  function toWasm(arg, malloc, realloc) {
    if (realloc === undefined) {
      const buf = cachedTextEncoder.encode(arg);
      const ptr = malloc(buf.length, 1) >>> 0;
      getUint8ArrayMemory()
        .subarray(ptr, ptr + buf.length)
        .set(buf);
      mem.offset = buf.length;
      return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory();

    let offset = 0;

    for (; offset < len; offset++) {
      const code = arg.charCodeAt(offset);
      if (code > 0x7f) break;
      mem[ptr + offset] = code;
    }
    if (offset !== len) {
      if (offset !== 0) {
        arg = arg.slice(offset);
      }
      ptr = realloc(ptr, len, (len = offset + arg.length * 3), 1) >>> 0;
      const view = getUint8ArrayMemory().subarray(ptr + offset, ptr + len);
      const ret = cachedTextEncoder.encodeInto(arg, view);

      offset += ret.written;
      ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return [ptr, WASM_VECTOR_LEN];
  }

  function getOffset() {
    return WASM_VECTOR_LEN;
  }

  return {
    fromWasm,
    toWasm,
    getOffset,
  };
}
window.WasmString = WasmString;
