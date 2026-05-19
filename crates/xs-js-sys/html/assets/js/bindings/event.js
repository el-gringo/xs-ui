/**
 * @import { App } from "./types.js";
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
      const key = e.key;
      const keyResult = utf8.toWasm(key);
      const view = new DataView(wasm.memory.buffer);
      view.setUint32(bufPtr, e.keyCode, true);
      let mods = 0;
      if (e.shiftKey) mods |= 1;
      if (e.ctrlKey) mods |= 2;
      if (e.altKey) mods |= 4;
      if (e.metaKey) mods |= 8;
      view.setUint32(bufPtr + 4, mods, true);
      view.setUint32(bufPtr + 8, keyResult.ptr, true);
      view.setUint32(bufPtr + 12, keyResult.len, true);
    },
  };
}

define(initializeEventBindings);
