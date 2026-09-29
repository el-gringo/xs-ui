/**
 * @import { App } from "../types.js";
 *
 *
 * @param {App} app
 */
function initializeWebGlBindings(app) {
  const { heap, utf8, bytes } = app;

  // The bindings are built before the wasm runs, so the canvas the app creates
  // does not exist yet: resolve the context on first use.
  let gl = null;
  function context() {
    if (gl === null) {
      const canvas = document.querySelector("canvas");
      if (canvas === null) {
        throw new Error(
          "no canvas in the document, create one before using webgl",
        );
      }
      gl = canvas.getContext("webgl2");
      if (gl === null) {
        throw new Error("WebGL2 is not available");
      }
    }
    return gl;
  }

  // Cached views over the wasm memory. They are dropped whenever the memory
  // grows, which detaches the previous buffer.
  let cachedBuffer = null;
  const views = new Map();

  function viewOf(Type, ptr, len) {
    const buffer = app.wasm.memory.buffer;
    if (cachedBuffer !== buffer) {
      cachedBuffer = buffer;
      views.clear();
    }
    let view = views.get(Type);
    if (view === undefined) {
      view = new Type(buffer);
      views.set(Type, view);
    }
    return view.subarray(ptr, ptr + len);
  }

  function floats(ptr, count) {
    return viewOf(Float32Array, ptr, count);
  }

  function shorts(ptr, count) {
    return viewOf(Uint16Array, ptr, count);
  }

  function uints(ptr, count) {
    return viewOf(Uint32Array, ptr, count);
  }

  function bytesOf(ptr, len) {
    return viewOf(Uint8Array, ptr, len);
  }

  // Uniform locations are opaque objects: they are stored in the heap and
  // unwrapped again by the glUniform* bindings. -1 means "not found".
  function uniformLocationOf(program, namePtr, nameLen) {
    const location = context().getUniformLocation(
      heap.get(program),
      utf8.fromWasm(namePtr, nameLen),
    );
    if (location === null) {
      return -1;
    }
    return heap.add(location);
  }

  // Attribute locations are plain integers, they are passed as is to
  // glEnableVertexAttribArray and glVertexAttribPointer. -1 means "not found".
  function attribLocationOf(program, namePtr, nameLen) {
    const location = context().getAttribLocation(
      heap.get(program),
      utf8.fromWasm(namePtr, nameLen),
    );
    if (location === null) {
      return -1;
    }
    return location;
  }

  return {
    glActiveTexture(texture) {
      context().activeTexture(texture);
    },
    glClear(mask) {
      context().clear(mask);
    },
    glClearColor(red, green, blue, alpha) {
      context().clearColor(red, green, blue, alpha);
    },
    glColorMask(red, green, blue, alpha) {
      context().colorMask(red, green, blue, alpha);
    },
    glViewport(x, y, width, height) {
      context().viewport(x, y, width, height);
    },
    glBlendFunc(sfactor, dfactor) {
      context().blendFunc(sfactor, dfactor);
    },
    glVertexAttribPointer(index, size, type_, normalized, stride, offset) {
      context().vertexAttribPointer(
        index,
        size,
        type_,
        !!normalized,
        stride,
        offset,
      );
    },
    glDrawArrays(mode, first, count) {
      context().drawArrays(mode, first, count);
    },
    glDrawElements(mode, count, type_, indices, indice_count) {
      const indexArray =
        type_ === context().UNSIGNED_INT
          ? uints(indices, indice_count)
          : shorts(indices, indice_count);
      context().drawElements(mode, count, type_, indexArray);
    },
    glEnableVertexAttribArray(index) {
      context().enableVertexAttribArray(index);
    },
    glBufferData(target, size, data, usage) {
      context().bufferData(target, data === 0 ? size : bytesOf(data, size), usage);
    },
    glEnable(cap) {
      context().enable(cap);
    },
    glDisable(cap) {
      context().disable(cap);
    },
    glBindBuffer(target, buffer) {
      context().bindBuffer(target, heap.get(buffer));
    },
    glUniform1i(location, v0) {
      context().uniform1i(heap.get(location), v0);
    },
    glUniform1f(location, v0) {
      context().uniform1f(heap.get(location), v0);
    },
    glUniform2f(location, v0, v1) {
      context().uniform2f(heap.get(location), v0, v1);
    },
    glUniform3f(location, v0, v1, v2) {
      context().uniform3f(heap.get(location), v0, v1, v2);
    },
    glUniform4f(location, v0, v1, v2, v3) {
      context().uniform4f(heap.get(location), v0, v1, v2, v3);
    },
    glUniform4fv(location, count, value) {
      context().uniform4fv(heap.get(location), floats(value, 4 * count));
    },
    glUniformMatrix3fv(location, count, transpose, value) {
      context().uniformMatrix3fv(heap.get(location), !!transpose, floats(value, 9 * count));
    },
    glUniformMatrix4fv(location, count, transpose, value) {
      context().uniformMatrix4fv(heap.get(location), !!transpose, floats(value, 16 * count));
    },
    glUseProgram(program) {
      context().useProgram(heap.get(program));
    },
    glGetUniformLocation(program, namePtr, nameLen) {
      return uniformLocationOf(program, namePtr, nameLen);
    },
    glCreateBuffer() {
      return heap.add(context().createBuffer());
    },
    glCreateProgram() {
      return heap.add(context().createProgram());
    },
    glAttachShader(program, shader) {
      context().attachShader(heap.get(program), heap.get(shader));
    },
    glLinkProgram(program) {
      context().linkProgram(heap.get(program));
    },
    glCreateShader(type_) {
      return heap.add(context().createShader(type_));
    },
    glShaderSource(shader, source, length) {
      context().shaderSource(heap.get(shader), utf8.fromWasm(source, length));
    },
    glCompileShader(shader) {
      context().compileShader(heap.get(shader));
    },
    glGetAttribLocation(program, namePtr, nameLen) {
      return attribLocationOf(program, namePtr, nameLen);
    },
    glTexParameteri(target, param, pname) {
      context().texParameteri(target, param, pname);
    },
    glTexImage2DByPointer(
      target,
      level,
      internalFormat,
      width,
      height,
      border,
      format,
      type_,
      source,
    ) {
      context().texImage2D(
        target,
        level,
        internalFormat,
        width,
        height,
        border,
        format,
        type_,
        heap.get(source),
      );
    },
    glTexImage2DByBytes(
      target,
      level,
      internalFormat,
      width,
      height,
      border,
      format,
      type_,
      ptr_data,
      len_data,
    ) {
      context().texImage2D(
        target,
        level,
        internalFormat,
        width,
        height,
        border,
        format,
        type_,
        bytesOf(ptr_data, len_data),
      );
    },
    glGenerateMipmap(target) {
      context().generateMipmap(target);
    },
    glCreateTexture() {
      return heap.add(context().createTexture());
    },
    glBindTexture(target, texture) {
      context().bindTexture(target, heap.get(texture));
    },
    glFlush() {
      context().flush();
    },
    glPixelStorei(pname, param) {
      context().pixelStorei(pname, param);
    },
    glGetProgramParameter(program, pname) {
      return context().getProgramParameter(heap.get(program), pname);
    },
    glGetProgramInfoLog(program) {
      return bytes.toCString(context().getProgramInfoLog(heap.get(program)));
    },
    glGetShaderParameter(shader, pname) {
      return context().getShaderParameter(heap.get(shader), pname);
    },
    glGetShaderInfoLog(shader) {
      return bytes.toCString(context().getShaderInfoLog(heap.get(shader)));
    },
  };
}

define(initializeWebGlBindings);
