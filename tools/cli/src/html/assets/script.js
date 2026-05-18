let wasm;
let max_render_time = 0;

const heap = Array.from({ length: 128 }).fill(undefined);

heap.push(undefined, null, true, false);

function get_object(idx) {
  return heap[idx];
}

let heap_next = heap.length;

function drop_object(idx) {
  if (idx < 132) return;
  heap[idx] = heap_next;
  heap_next = idx;
}

function _take_object(idx) {
  const ret = get_object(idx);
  drop_object(idx);
  return ret;
}

function add_heap_object(obj) {
  if (heap_next === heap.length) heap.push(heap.length + 1);
  const idx = heap_next;
  heap_next = heap[idx];

  heap[idx] = obj;
  return idx;
}

const cachedTextDecoder = new TextDecoder("utf-8", {
  ignoreBOM: true,
  fatal: true,
});

cachedTextDecoder.decode();

let cachedBuffer = null;
let cachedUint8Memory0 = null;

function getUint8Memory0() {
  const buf = wasm.memory.buffer;
  if (cachedUint8Memory0 === null || cachedBuffer !== buf) {
    cachedBuffer = buf;
    cachedUint8Memory0 = new Uint8Array(buf);
  }
  return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
  return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;
const cachedTextEncoder = new TextEncoder("utf-8");

let cachedBuffer2 = null;
let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
  const buf = wasm.memory.buffer;
  if (
    cachedUint8ArrayMemory0 === null ||
    cachedBuffer2 !== buf
  ) {
    cachedBuffer2 = buf;
    cachedUint8ArrayMemory0 = new Uint8Array(buf);
  }
  return cachedUint8ArrayMemory0;
}

/* oxlint-disable-next-line: no-unused-vars */
function passStringToWasm0(arg, malloc, realloc) {
  if (realloc === undefined) {
    const buf = cachedTextEncoder.encode(arg);
    const ptr = malloc(buf.length, 1) >>> 0;
    getUint8ArrayMemory0()
      .subarray(ptr, ptr + buf.length)
      .set(buf);
    WASM_VECTOR_LEN = buf.length;
    return ptr;
  }

  let len = arg.length;
  let ptr = malloc(len, 1) >>> 0;

  const mem = getUint8ArrayMemory0();

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
    const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
    const ret = cachedTextEncoder.encodeInto(arg, view);

    offset += ret.written;
    ptr = realloc(ptr, len, offset, 1) >>> 0;
  }

  WASM_VECTOR_LEN = offset;
  return ptr;
}

function passBytesToWasm0(buffer, malloc) {
  let bytes = new Uint8Array(buffer);
  const ptr = malloc(bytes.byteLength, 1) >>> 0;
  getUint8ArrayMemory0()
    .subarray(ptr, ptr + bytes.byteLength)
    .set(bytes);
  WASM_VECTOR_LEN = bytes.byteLength;
  return ptr;
}

function stringToUTF8(str, heap, outIdx, maxBytesToWrite) {
  var startIdx = outIdx;
  var endIdx = outIdx + maxBytesToWrite;
  for (var i = 0; i < str.length; ++i) {
    // Gotcha: charCodeAt returns a 16-bit word that is a UTF-16 encoded code unit, not a Unicode code point of the character! So decode UTF16->UTF32->UTF8.
    // See http://unicode.org/faq/utf_bom.html#utf16-3
    // For UTF8 byte structure, see http://en.wikipedia.org/wiki/UTF-8#Description and https://www.ietf.org/rfc/rfc2279.txt and https://tools.ietf.org/html/rfc3629
    var u = str.charCodeAt(i); // possibly a lead surrogate
    if (u >= 0xd800 && u <= 0xdfff) {
      var u1 = str.charCodeAt(++i);
      u = (0x10000 + ((u & 0x3ff) << 10)) | (u1 & 0x3ff);
    }
    if (u <= 0x7f) {
      if (outIdx >= endIdx) break;
      heap[outIdx++] = u;
    } else if (u <= 0x7ff) {
      if (outIdx + 1 >= endIdx) break;
      heap[outIdx++] = 0xc0 | (u >> 6);
      heap[outIdx++] = 0x80 | (u & 63);
    } else if (u <= 0xffff) {
      if (outIdx + 2 >= endIdx) break;
      heap[outIdx++] = 0xe0 | (u >> 12);
      heap[outIdx++] = 0x80 | ((u >> 6) & 63);
      heap[outIdx++] = 0x80 | (u & 63);
    } else {
      if (outIdx + 3 >= endIdx) break;

      if (u >= 0x200000)
        console.warn(
          "Invalid Unicode code point 0x" +
            u.toString(16) +
            " encountered when serializing a JS string to an UTF-8 string on the asm.js/wasm heap! (Valid unicode code points should be in range 0-0x1FFFFF).",
        );

      heap[outIdx++] = 0xf0 | (u >> 18);
      heap[outIdx++] = 0x80 | ((u >> 12) & 63);
      heap[outIdx++] = 0x80 | ((u >> 6) & 63);
      heap[outIdx++] = 0x80 | (u & 63);
    }
  }
  return outIdx - startIdx;
}

function getArray(ptr, arr, n) {
  if (!ptr) {
    throw new Error();
  }
  return new arr(wasm.memory.buffer, ptr, n);
}

const SAPP_MODIFIER_SHIFT = 1;
const SAPP_MODIFIER_CTRL = 2;
const SAPP_MODIFIER_ALT = 4;
//const SAPP_MODIFIER_SUPER = 8;

const canvas = document.getElementById("glcanvas");
/**
 * @type {WebGL2RenderingContext}
 */
const gl = canvas.getContext("webgl2");
canvas.focus();

const FPS = 60;
const FRAME_INTERVAL = 1000 / FPS;

let start = performance.now();
const env = {
  console_log: function (ptr, len) {
    console.log(getStringFromWasm0(ptr, len));
  },
  console_debug: function (ptr, len) {
    console.debug(getStringFromWasm0(ptr, len));
  },
  console_debug2(ptr_arg1, len_arg1, ptr_arg2, len_arg2) {
    console.debug(
      getStringFromWasm0(ptr_arg1, len_arg1),
      getStringFromWasm0(ptr_arg2, len_arg2),
    );
  },
  console_info: function (ptr, len) {
    console.info(getStringFromWasm0(ptr, len));
  },
  console_info2(ptr_arg1, len_arg1, ptr_arg2, len_arg2) {
    console.info(
      getStringFromWasm0(ptr_arg1, len_arg1),
      getStringFromWasm0(ptr_arg2, len_arg2),
    );
  },
  console_error: function (ptr, len) {
    console.error(getStringFromWasm0(ptr, len));
  },
  console_error2(ptr_arg1, len_arg1, ptr_arg2, len_arg2) {
    console.error(
      getStringFromWasm0(ptr_arg1, len_arg1),
      getStringFromWasm0(ptr_arg2, len_arg2),
    );
  },

  console_trace: function (ptr, len) {
    console.trace(getStringFromWasm0(ptr, len));
  },
  console_trace2(ptr_arg1, len_arg1, ptr_arg2, len_arg2) {
    console.warn(
      getStringFromWasm0(ptr_arg1, len_arg1),
      getStringFromWasm0(ptr_arg2, len_arg2),
    );
  },
  console_warn: function (ptr, len) {
    console.warn(getStringFromWasm0(ptr, len));
  },
  console_warn2(ptr_arg1, len_arg1, ptr_arg2, len_arg2) {
    console.warn(
      getStringFromWasm0(ptr_arg1, len_arg1),
      getStringFromWasm0(ptr_arg2, len_arg2),
    );
  },
  heap_drop(ptr) {
    drop_object(ptr);
  },
  rand: function () {
    return Math.floor(Math.random() * 2147483647);
  },
  now: function () {
    return Date.now() / 1000.0;
  },
  canvas_width: function () {
    return Math.floor(canvas.width);
  },
  canvas_height: function () {
    return Math.floor(canvas.height);
  },
  document() {
    const ptr = add_heap_object(window.document);
    return ptr;
  },
  new_string(ptr, len) {
    let str = getStringFromWasm0(ptr, len);
    return add_heap_object(str);
  },
  new_image(ptr, len, width) {
    let imageData = new ImageData(getArray(ptr, Uint8ClampedArray, len), width);
    let image_ptr = add_heap_object(imageData);
    return image_ptr;
  },
  initialize: function (width, height) {
    const pixelRatio = window.devicePixelRatio || 1.0;
    canvas.width = width;
    canvas.height = height;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    canvas.onkeydown = function onKeyDown(event) {
      const { keyCode, ctrlKey, shiftKey, altKey, repeat } = event;
      let modifiers = 0;
      if (ctrlKey) {
        modifiers |= SAPP_MODIFIER_CTRL;
      }
      if (shiftKey) {
        modifiers |= SAPP_MODIFIER_SHIFT;
      }
      if (altKey) {
        modifiers |= SAPP_MODIFIER_ALT;
      }
      //wasm.key_down(keyCode, modifiers, repeat);
    };
    start = performance.now();
    // for (let i = 0; i < 1000; ++i) {
    //   requestAnimationFrame(loop_frames);
    // }
  },
  element_query_selector(elementPtr, selectorPtr, selectorLen) {
    const selector = getStringFromWasm0(selectorPtr, selectorLen);
    const obj = get_object(elementPtr);
    const element = obj.querySelector(selector);
    const ptr = add_heap_object(element);
    return ptr;
  },
  add_event_listener(obj_idx, event_name_ptr, event_name_len, closure_handle) {
    const obj = get_object(obj_idx);
    const event_name = getStringFromWasm0(event_name_ptr, event_name_len);
    function handleEvent(event) {
      const event_ptr = add_heap_object(event);
      wasm.callback_event_handler(closure_handle, event_ptr);
    }
    obj.addEventListener(event_name, handleEvent);
  },

  event_key_down(eventPtr, bufPtr) {
    const event = get_object(eventPtr);
    const ptrKeyString = passStringToWasm0(event.key, wasm.__xs_malloc);
    const view = new DataView(wasm.memory.buffer);
    view.setUint32(bufPtr, event.keyCode, true);
    let mods = 0;
    if (event.shiftKey) mods |= 1;
    if (event.ctrlKey) mods |= 2;
    if (event.altKey) mods |= 4;
    if (event.metaKey) mods |= 8;
    view.setUint32(bufPtr + 4, mods, true);
    view.setUint32(bufPtr + 8, ptrKeyString, true);
    view.setUint32(bufPtr + 12, WASM_VECTOR_LEN, true);
  },
  load_image(closure, ptr_src, len_src) {
    let src = getStringFromWasm0(ptr_src, len_src);
    const image = new Image();
    image.onload = function onImageLoaded() {
      const ptr_img = add_heap_object(image);
      wasm.load_image_callback(
        closure,
        ptr_img,
        image.naturalWidth,
        image.naturalHeight,
      );
    };
    image.crossOrigin = true;
    image.src = src;
  },
  fetch(handle, ptr, len) {
    const url = getStringFromWasm0(ptr, len);
    fetch(url)
      .then((response) => response.arrayBuffer())
      .then((body) => {
        const ptr = passBytesToWasm0(body, wasm.__xs_malloc);
        const len = WASM_VECTOR_LEN;
        wasm.fetch_callback(handle, ptr, len);
      });
  },
  request_animation_frame(closure, vcall) {
    return requestAnimationFrame((now) => {
      wasm.callback_request_animation_frame(closure, vcall, now);
      const frame_duration = performance.now() - now;
      if (frame_duration > max_render_time) {
        max_render_time = frame_duration;
      }
      if (performance.now() - start > 1000) {
        console.log("max_render_time", max_render_time);
        max_render_time = 0;
        start = performance.now();
      }
    });
  },
  glClearDepthf: function (depth) {
    gl.clearDepth(depth);
  },
  glClearColor: function (r, g, b, a) {
    gl.clearColor(r, g, b, a);
  },
  glClearStencil: function (s) {
    gl.clearStencil(s);
  },
  glColorMask: function (red, green, blue, alpha) {
    gl.colorMask(red, green, blue, alpha);
  },
  glCreateBuffer: function () {
    const buffer = gl.createBuffer();
    return add_heap_object(buffer);
  },
  glCreateTexture: function () {
    const texture = gl.createTexture();
    return add_heap_object(texture);
  },
  glScissor: function (x, y, w, h) {
    gl.scissor(x, y, w, h);
  },
  glClear: function (mask) {
    gl.clear(mask);
  },
  glActiveTexture: function (texture_unit) {
    gl.activeTexture(texture_unit);
  },
  glBindTexture: function (target, texture) {
    gl.bindTexture(target, get_object(texture));
  },
  glTexParameteriv(target, pname, param) {
    gl.glTexParameteriv(target, pname, param);
  },
  glTexImage2DByPointer: function (
    target,
    level,
    internalFormat,
    width,
    height,
    border,
    format,
    type,
    image,
  ) {
    const source = get_object(image);
    gl.texImage2D(
      target,
      level,
      internalFormat,
      width,
      height,
      border,
      format,
      type,
      source,
    );
  },
  glTexImage2DByBytes: function (
    target,
    level,
    internalFormat,
    width,
    height,
    border,
    format,
    type,
    ptr_data,
    len_data,
  ) {
    let data = getArray(ptr_data, Uint8Array, len_data);
    gl.texImage2D(
      target,
      level,
      internalFormat,
      width,
      height,
      border,
      format,
      type,
      data,
    );
  },
  glTexSubImage2D: function (
    target,
    level,
    xoffset,
    yoffset,
    width,
    height,
    format,
    type,
    pixels,
  ) {
    gl.texSubImage2D(
      target,
      level,
      xoffset,
      yoffset,
      width,
      height,
      format,
      type,
      pixels
        ? getArray(pixels, Uint8Array, texture_size(format, width, height))
        : null,
    );
  },
  glReadPixels: function (x, y, width, height, format, type, pixels) {
    var pixelData = getArray(
      pixels,
      Uint8Array,
      texture_size(format, width, height),
    );
    gl.readPixels(x, y, width, height, format, type, pixelData);
  },
  glTexParameteri: function (target, pname, param) {
    gl.texParameteri(target, pname, param);
  },
  glUniform1fv: function (location, count, value) {
    var view = getArray(value, Float32Array, 1 * count);
    gl.uniform1fv(get_object(location), view);
  },
  glUniform2f: function (location, v0, v1) {
    gl.uniform2f(get_object(location), v0, v1);
  },
  glUniform2fv: function (location, count, value) {
    var view = getArray(value, Float32Array, 2 * count);
    gl.uniform2fv(get_object(location), view);
  },
  glUniform3fv: function (location, count, value) {
    var view = getArray(value, Float32Array, 3 * count);
    gl.uniform3fv(get_object(location), view);
  },
  glUniform4f: function (location, v0, v1, v2, v3) {
    gl.uniform4f(get_object(location), v0, v1, v2, v3);
  },
  glUniform4fv: function (location, count, value) {
    var view = getArray(value, Float32Array, 4 * count);
    gl.uniform4fv(get_object(location), view);
  },
  glUniform1f: function (location, value) {
    gl.uniform1f(get_object(location), value);
  },
  glUniform1i: function (location, value) {
    gl.uniform1i(get_object(location), value);
  },
  glUniform1iv: function (location, count, value) {
    const view = getArray(value, Int32Array, 1 * count);
    gl.uniform1iv(location, view);
  },
  glUniform2iv: function (location, count, value) {
    var view = getArray(value, Int32Array, 2 * count);
    gl.uniform2iv(get_object(location), view);
  },
  glUniform3iv: function (location, count, value) {
    var view = getArray(value, Int32Array, 3 * count);
    gl.uniform3iv(get_object(location), view);
  },
  glUniform4iv: function (location, count, value) {
    var view = getArray(value, Int32Array, 4 * count);
    gl.uniform4iv(get_object(location), view);
  },
  glBlendFunc: function (sfactor, dfactor) {
    gl.blendFunc(sfactor, dfactor);
  },
  glBlendEquationSeparate: function (modeRGB, modeAlpha) {
    gl.blendEquationSeparate(modeRGB, modeAlpha);
  },
  glDisable: function (cap) {
    gl.disable(cap);
  },
  glDrawElements: function (mode, count, type, indices) {
    gl.drawElements(mode, count, type, indices);
  },
  glGetIntegerv: function (name_, p) {
    _webglGet(name_, p, "EM_FUNC_SIG_PARAM_I");
  },
  glGetAttribLocation: function (program, name, len) {
    const fieldName = getStringFromWasm0(name, len);
    const location = gl.getAttribLocation(get_object(program), fieldName);
    if (location !== null) {
      const ptr = add_heap_object(location);
      return ptr;
    }
    return -1;
  },
  glEnableVertexAttribArray: function (attribLocation) {
    gl.enableVertexAttribArray(get_object(attribLocation));
  },
  glDisableVertexAttribArray: function (attribLocation) {
    gl.disableVertexAttribArray(get_object(attribLocation));
  },
  glVertexAttribPointer: function (
    attribLocation,
    size,
    type,
    normalized,
    stride,
    ptr,
  ) {
    gl.vertexAttribPointer(
      get_object(attribLocation),
      size,
      type,
      !!normalized,
      stride,
      ptr,
    );
  },
  glVertexAttribIPointer: function (attribLocation, size, type, stride, ptr) {
    gl.vertexAttribIPointer(
      get_object(attribLocation),
      size,
      type,
      stride,
      ptr,
    );
  },
  glGetUniformLocation: function (program, name, len) {
    const fieldName = getStringFromWasm0(name, len);
    const location = gl.getUniformLocation(get_object(program), fieldName);
    if (location !== null) {
      const ptr = add_heap_object(location);
      return ptr;
    }
    return -1;
  },
  glUniformMatrix3fv: function (location, count, transpose, value) {
    var view = getArray(value, Float32Array, 9 * count);
    gl.uniformMatrix3fv(get_object(location), !!transpose, view);
  },
  glUniformMatrix4fv: function (location, count, transpose, value) {
    var view = getArray(value, Float32Array, 12 * count);
    gl.uniformMatrix4fv(get_object(location), !!transpose, view);
  },
  glUseProgram: function (program) {
    gl.useProgram(get_object(program));
  },
  glBindFramebuffer: function (target, framebuffer) {
    gl.bindFramebuffer(target, get_object(framebuffer));
  },
  glBindBuffer: function (target, buffer) {
    gl.bindBuffer(target, get_object(buffer));
  },
  glBufferData: function (target, size, ptr, usage) {
    data = getArray(ptr, Float32Array, size);
    gl.bufferData(target, data, usage);
  },
  glBufferSubData: function (target, offset, size, data) {
    gl.bufferSubData(
      target,
      offset,
      data ? getArray(data, Uint8Array, size) : size,
    );
  },
  glEnable: function (cap) {
    gl.enable(cap);
  },
  glFlush: function () {
    gl.flush();
  },
  glFinish: function () {
    gl.finish();
  },
  glDepthFunc: function (func) {
    gl.depthFunc(func);
  },
  glBlendFuncSeparate: function (
    sfactorRGB,
    dfactorRGB,
    sfactorAlpha,
    dfactorAlpha,
  ) {
    gl.blendFuncSeparate(sfactorRGB, dfactorRGB, sfactorAlpha, dfactorAlpha);
  },
  glViewport: function (x, y, width, height) {
    gl.viewport(x, y, width, height);
  },
  glDrawArrays: function (mode, first, count) {
    gl.drawArrays(mode, first, count);
  },
  glDrawBuffers: function (n, bufs) {
    gl.drawBuffers(getArray(bufs, Int32Array, n));
  },
  glCreateProgram: function () {
    const program = gl.createProgram();
    return add_heap_object(program);
  },
  glAttachShader: function (program, shader) {
    gl.attachShader(get_object(program), get_object(shader));
  },
  glDetachShader: function (program, shader) {
    gl.detachShader(get_object(program), get_object(shader));
  },
  glLinkProgram: function (program) {
    gl.linkProgram(get_object(program));
    //GL.populateUniformTable(program);
  },
  glPixelStorei: function (pname, param) {
    gl.pixelStorei(pname, param);
  },
  glFramebufferTexture2D: function (
    target,
    attachment,
    textarget,
    texture,
    level,
  ) {
    gl.framebufferTexture2D(target, attachment, textarget, texture, level);
  },
  glGetProgramParameter: function (program, pname) {
    const ret = gl.getProgramParameter(get_object(program), pname);
    return ret;
  },
  glCreateShader: function (shaderType) {
    const shader = gl.createShader(shaderType);
    return add_heap_object(shader);
  },
  glStencilFuncSeparate: function (face, func, ref_, mask) {
    gl.stencilFuncSeparate(face, func, ref_, mask);
  },
  glStencilMaskSeparate: function (face, mask) {
    gl.stencilMaskSeparate(face, mask);
  },
  glStencilOpSeparate: function (face, fail, zfail, zpass) {
    gl.stencilOpSeparate(face, fail, zfail, zpass);
  },
  glFrontFace: function (mode) {
    gl.frontFace(mode);
  },
  glCullFace: function (mode) {
    gl.cullFace(mode);
  },
  glCopyTexImage2D: function (
    target,
    level,
    internalformat,
    x,
    y,
    width,
    height,
    border,
  ) {
    gl.copyTexImage2D(
      target,
      level,
      internalformat,
      x,
      y,
      width,
      height,
      border,
    );
  },

  glShaderSource: function (shader, ptr, length) {
    const source = getStringFromWasm0(ptr, length);
    gl.shaderSource(get_object(shader), source);
  },
  glGetProgramInfoLog: function (program) {
    var log = gl.getProgramInfoLog(get_object(program));
    const ptr = passStringToWasm0(log, wasm.__xs_malloc);
    return ptr;
  },
  glCompileShader: function (shader) {
    gl.compileShader(get_object(shader));
  },
  glGetShaderParameter: function (shader, pname) {
    const ret = gl.getShaderParameter(get_object(shader), pname);
    return ret;
  },
  glGetShaderInfoLog: function (shader) {
    var log = gl.getShaderInfoLog(get_object(shader));
    const ptr = passStringToWasm0(log, wasm.__xs_malloc);
    return ptr;
  },
  glVertexAttribDivisor: function (index, divisor) {
    gl.vertexAttribDivisor(index, divisor);
  },
  glDrawArraysInstanced: function (mode, first, count, primcount) {
    gl.drawArraysInstanced(mode, first, count, primcount);
  },
  glDrawElementsInstanced: function (mode, count, type, indices, primcount) {
    gl.drawElementsInstanced(mode, count, type, indices, primcount);
  },
  glDeleteShader: function (shader) {
    var id = get_object(shader);
    if (id == null) {
      return;
    }
    gl.deleteShader(id);
  },
  glDeleteProgram: function (program) {
    var id = get_object(program);
    if (id == null) {
      return;
    }
    gl.deleteProgram(id);
  },
  glDeleteBuffers: function (n, buffers) {
    for (var i = 0; i < n; i++) {
      var id = getArray(buffers + i * 4, Uint32Array, 1)[0];
      var buffer = get_object(id);

      // From spec: "glDeleteBuffers silently ignores 0's and names that do not
      // correspond to existing buffer objects."
      if (!buffer) continue;

      gl.deleteBuffer(buffer);
    }
  },
  glDeleteFramebuffers: function (n, buffers) {
    for (var i = 0; i < n; i++) {
      var id = getArray(buffers + i * 4, Uint32Array, 1)[0];
      var buffer = get_object(id);

      // From spec: "glDeleteFrameBuffers silently ignores 0's and names that do not
      // correspond to existing buffer objects."
      if (!buffer) continue;

      gl.deleteFramebuffer(buffer);
      buffer.name = 0;
    }
  },
  glDeleteTextures: function (n, textures) {
    for (var i = 0; i < n; i++) {
      var id = getArray(textures + i * 4, Uint32Array, 1)[0];
      var texture = get_object(id);
      if (!texture) continue; // GL spec: "glDeleteTextures silently ignores 0s and names that do not correspond to existing textures".
      gl.deleteTexture(texture);
    }
  },
  glGenerateMipmap: function (index) {
    gl.generateMipmap(index);
  },
};

function load(wasmUrl) {
  fetch(wasmUrl)
    .then((response) => response.arrayBuffer())
    .then((bytes) => WebAssembly.instantiate(bytes, { env }))
    .then((res) => {
      const canvas = document.querySelector("canvas");
      canvas.onblur = () => {
        canvas.focus();
      };
      wasm = res.instance.exports;
      cachedInt32Memory0 = null;
      cachedUint32Memory0 = null;
      cachedUint8Memory0 = null;
      wasm.main();
    });
}
