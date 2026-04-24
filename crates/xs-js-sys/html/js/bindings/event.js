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
  };
}

define(initializeEventBindings);
