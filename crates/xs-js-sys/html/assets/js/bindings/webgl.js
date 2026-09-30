/**
 * @import { App } from "../types.js";
 *
 *
 * @param {App} app
 */
function initializeWebGlBindings(app) {
  const { heap, memory, utf8, bytes } = app;

  // `app.gl` is a plain property, but the bindings are built before the app has
  // created its canvas, so it is empty until `canvas_get_context` fills it.
  // Standing in for it with an object that throws says why, where reading a
  // method off undefined would only say "cannot read properties of undefined".
  if (app.gl === undefined) {
    app.gl = missingContext();
  }

  // Uniform locations are opaque objects: they are stored in the heap and
  // unwrapped again by the glUniform* bindings. -1 means "not found".
  function uniformLocationOf(program, namePtr, nameLen) {
    const location = app.gl.getUniformLocation(heap.get(program), utf8.fromWasm(namePtr, nameLen));
    if (location === null) {
      return -1;
    }
    return heap.add(location);
  }

  // Attribute locations are plain integers, they are passed as is to
  // glEnableVertexAttribArray and glVertexAttribPointer. -1 means "not found".
  function attribLocationOf(program, namePtr, nameLen) {
    const location = app.gl.getAttribLocation(heap.get(program), utf8.fromWasm(namePtr, nameLen));
    if (location === null) {
      return -1;
    }
    return location;
  }

  return {
    glActiveTexture(texture) {
      app.gl.activeTexture(texture);
    },
    glClear(mask) {
      app.gl.clear(mask);
    },
    glClearColor(red, green, blue, alpha) {
      app.gl.clearColor(red, green, blue, alpha);
    },
    glColorMask(red, green, blue, alpha) {
      app.gl.colorMask(red, green, blue, alpha);
    },
    glViewport(x, y, width, height) {
      app.gl.viewport(x, y, width, height);
    },
    glBlendFunc(sfactor, dfactor) {
      app.gl.blendFunc(sfactor, dfactor);
    },
    glVertexAttribPointer(index, size, type_, normalized, stride, offset) {
      app.gl.vertexAttribPointer(index, size, type_, !!normalized, stride, offset);
    },
    glDrawArrays(mode, first, count) {
      app.gl.drawArrays(mode, first, count);
    },
    glDrawElements(mode, count, type_, indices, indice_count) {
      const Type = type_ === app.gl.UNSIGNED_INT ? Uint32Array : Uint16Array;
      const indexArray = memory.view(Type, indices, indice_count, "glDrawElements");
      app.gl.drawElements(mode, count, type_, indexArray);
    },
    glEnableVertexAttribArray(index) {
      app.gl.enableVertexAttribArray(index);
    },
    glBufferData(target, size, data, usage) {
      app.gl.bufferData(
        target,
        data === 0 ? size : memory.view(Uint8Array, data, size, "glBufferData"),
        usage,
      );
    },
    glEnable(cap) {
      app.gl.enable(cap);
    },
    glDisable(cap) {
      app.gl.disable(cap);
    },
    glBindBuffer(target, buffer) {
      app.gl.bindBuffer(target, heap.get(buffer));
    },
    glUniform1i(location, v0) {
      app.gl.uniform1i(heap.get(location), v0);
    },
    glUniform1f(location, v0) {
      app.gl.uniform1f(heap.get(location), v0);
    },
    glUniform2f(location, v0, v1) {
      app.gl.uniform2f(heap.get(location), v0, v1);
    },
    glUniform3f(location, v0, v1, v2) {
      app.gl.uniform3f(heap.get(location), v0, v1, v2);
    },
    glUniform4f(location, v0, v1, v2, v3) {
      app.gl.uniform4f(heap.get(location), v0, v1, v2, v3);
    },
    glUniform4fv(location, count, value) {
      app.gl.uniform4fv(
        heap.get(location),
        memory.view(Float32Array, value, 4 * count, "glUniform4fv"),
      );
    },
    glUniformMatrix3fv(location, count, transpose, value) {
      app.gl.uniformMatrix3fv(
        heap.get(location),
        !!transpose,
        memory.view(Float32Array, value, 9 * count, "glUniformMatrix3fv"),
      );
    },
    glUniformMatrix4fv(location, count, transpose, value) {
      app.gl.uniformMatrix4fv(
        heap.get(location),
        !!transpose,
        memory.view(Float32Array, value, 16 * count, "glUniformMatrix4fv"),
      );
    },
    glUseProgram(program) {
      app.gl.useProgram(heap.get(program));
    },
    glGetUniformLocation(program, namePtr, nameLen) {
      return uniformLocationOf(program, namePtr, nameLen);
    },
    glCreateBuffer() {
      return heap.add(app.gl.createBuffer());
    },
    glCreateProgram() {
      return heap.add(app.gl.createProgram());
    },
    glAttachShader(program, shader) {
      app.gl.attachShader(heap.get(program), heap.get(shader));
    },
    glLinkProgram(program) {
      app.gl.linkProgram(heap.get(program));
    },
    glCreateShader(type_) {
      return heap.add(app.gl.createShader(type_));
    },
    glShaderSource(shader, source, length) {
      app.gl.shaderSource(heap.get(shader), utf8.fromWasm(source, length));
    },
    glCompileShader(shader) {
      app.gl.compileShader(heap.get(shader));
    },
    glGetAttribLocation(program, namePtr, nameLen) {
      return attribLocationOf(program, namePtr, nameLen);
    },
    glTexParameteri(target, param, pname) {
      app.gl.texParameteri(target, param, pname);
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
      app.gl.texImage2D(
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
      app.gl.texImage2D(
        target,
        level,
        internalFormat,
        width,
        height,
        border,
        format,
        type_,
        memory.view(Uint8Array, ptr_data, len_data, "glTexImage2DByBytes"),
      );
    },
    glGenerateMipmap(target) {
      app.gl.generateMipmap(target);
    },
    glCreateTexture() {
      return heap.add(app.gl.createTexture());
    },
    glBindTexture(target, texture) {
      app.gl.bindTexture(target, heap.get(texture));
    },
    glFlush() {
      app.gl.flush();
    },
    glPixelStorei(pname, param) {
      app.gl.pixelStorei(pname, param);
    },
    glGetProgramParameter(program, pname) {
      return app.gl.getProgramParameter(heap.get(program), pname);
    },
    glGetProgramInfoLog(program) {
      return bytes.toCString(app.gl.getProgramInfoLog(heap.get(program)));
    },
    glGetShaderParameter(shader, pname) {
      return app.gl.getShaderParameter(heap.get(shader), pname);
    },
    glGetShaderInfoLog(shader) {
      return bytes.toCString(app.gl.getShaderInfoLog(heap.get(shader)));
    },
  };
}

/**
 * Stands in for `app.gl` until the app has asked a canvas for a context.
 * Reading anything on it throws, so a gl call made too early names the missing
 * step instead of failing on `undefined`.
 */
function missingContext() {
  return new Proxy(
    {},
    {
      get(_target, property) {
        throw new Error(
          `webgl: no context yet, get one with ` +
            `canvas.get_context("webgl2") before calling the gl bindings ` +
            `(tried to read "${String(property)}")`,
        );
      },
    },
  );
}

define(initializeWebGlBindings);
