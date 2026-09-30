/**
 * @import { App } from "../types.js";
 *
 *
 * @param {App} app
 */
function initializeEventBindings(app) {
  return {
    add_event_listener(
      obj_idx,
      event_name_ptr,
      event_name_len,
      closure_handle,
    ) {
      const { wasm, heap, utf8 } = app;
      const obj = heap.get(obj_idx);
      const event_name = utf8.fromWasm(event_name_ptr, event_name_len);
      function handleEvent(event) {
        const event_ptr = heap.add(event);
        wasm.callback_event_handler(closure_handle, event_ptr);
      }
      obj.addEventListener(event_name, handleEvent);
    },

    event_key_down(eventPtr, bufPtr) {
      const { wasm, heap, utf8 } = app;
      const e = heap.get(eventPtr);
      // `code` is the physical key, `key` would be the character it produces
      // and would follow the keyboard layout.
      const code = utf8.toWasm(e.code);
      const view = new DataView(wasm.memory.buffer);
      let mods = 0;
      if (e.shiftKey) mods |= 1;
      if (e.ctrlKey) mods |= 2;
      if (e.altKey) mods |= 4;
      if (e.metaKey) mods |= 8;
      view.setUint32(bufPtr, mods, true);
      view.setUint32(bufPtr + 4, code.ptr, true);
      view.setUint32(bufPtr + 8, code.len, true);
      view.setUint32(bufPtr + 12, e.keyCode, true);
    },
    event_mouse(eventPtr, bufPtr) {
      const { wasm, heap } = app;
      const e = heap.get(eventPtr);
      const view = new DataView(wasm.memory.buffer);
      view.setFloat64(bufPtr, e.clientX, true);
      view.setFloat64(bufPtr + 8, e.clientY, true);
      view.setFloat64(bufPtr + 16, e.offsetX, true);
      view.setFloat64(bufPtr + 24, e.offsetY, true);
      view.setUint16(bufPtr + 32, e.button, true);
      view.setUint16(bufPtr + 34, e.buttons, true);
      let mods = 0;
      if (e.shiftKey) mods |= 1;
      if (e.ctrlKey) mods |= 2;
      if (e.altKey) mods |= 4;
      if (e.metaKey) mods |= 8;
      view.setUint8(bufPtr + 36, mods);
    },

    event_target(eventPtr) {
      const { heap } = app;
      return heap.add(heap.get(eventPtr).target);
    },
  };
}

define(initializeEventBindings);
