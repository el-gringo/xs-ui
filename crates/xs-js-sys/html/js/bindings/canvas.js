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
    canvas_width(canvasPtr) {
      const { heap } = app;
      console.log("canvas_width(canvasPtr)", canvasPtr, heap.get(canvasPtr));
      return heap.get(canvasPtr).width;
    },
    canvas_height(canvasPtr) {
      const { heap } = app;
      return heap.get(canvasPtr).height;
    },
  };
}

define(initializeCanvasBindings);
