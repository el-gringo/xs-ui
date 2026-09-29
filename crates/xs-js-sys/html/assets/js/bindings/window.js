/**
 * @import { App } from "../types.js";
 *
 *
 * @param {App} app
 */
function initializeWindowBindings(app) {
  let start = performance.now();
  let maxRenderTime = 0;
  return {
    window() {
      const { heap } = app;
      return heap.add(window);
    },

    fetch(handle, ptr, len) {
      const { wasm, utf8, bytes } = app;
      const url = utf8.fromWasm(ptr, len);
      fetch(url)
        .then((response) => response.arrayBuffer())
        .then((body) => {
          const { ptr, len } = bytes.toWasm(body);
          wasm.fetch_callback(handle, ptr, len);
        });
    },

    reload() {
      window.location.reload();
    },

    // Microseconds, matching the `u32` the wasm side expects.
    performance_now() {
      return performance.now() * 1000;
    },

    time(ptr, len) {
      const { utf8 } = app;
      const label = utf8.fromWasm(ptr, len);
      console.time(label);
    },

    time_end(ptr, len) {
      const { utf8 } = app;
      const label = utf8.fromWasm(ptr, len);
      console.timeEnd(label);
    },

    request_animation_frame(closure, vcall) {
      const { wasm } = app;
      return requestAnimationFrame((now) => {
        wasm.callback_request_animation_frame(closure, vcall, now);
        const frameDuration = performance.now() - now;
        if (frameDuration > maxRenderTime) {
          maxRenderTime = frameDuration;
        }
        if (performance.now() - start > 1000) {
          console.log("max_render_time", maxRenderTime);
          maxRenderTime = 0;
          start = performance.now();
        }
      });
    },
  };
}

define(initializeWindowBindings);
