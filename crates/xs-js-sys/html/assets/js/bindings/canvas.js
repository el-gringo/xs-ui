/**
 * @import { App } from "./types.js";
 *
 *
 * @param {App} app
 */
function initializeCanvasBindings(app) {
  return {
    get_context(canvasPtr, contextTypePtr, contextTypeLen) {
      const { heap, utf8 } = app;
      const contextType = utf8.fromWasm(contextTypePtr, contextTypeLen);
      const ptr = heap.get(canvasPtr).getContext(contextType);
      const context = heap.add(ptr);
      return context;
    },
  };
}

define(initializeCanvasBindings);
