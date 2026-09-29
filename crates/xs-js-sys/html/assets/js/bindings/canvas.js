/**
 * @import { App } from "../types.js";
 *
 *
 * @param {App} app
 */
function initializeCanvasBindings(app) {
  return {
    canvas_get_context(canvasPtr, contextTypePtr, contextTypeLen) {
      const { heap, utf8 } = app;
      const contextType = utf8.fromWasm(contextTypePtr, contextTypeLen);
      const ptr = heap.get(canvasPtr).getContext(contextType);
      const context = heap.add(ptr);
      return context;
    },

    canvas_get_width(canvasPtr) {
      const { heap } = app;
      return heap.get(canvasPtr).width;
    },
    canvas_get_height(canvasPtr) {
      const { heap } = app;
      return heap.get(canvasPtr).height;
    },
    canvas_set_width(canvasPtr, width) {
      const { heap } = app;
      heap.get(canvasPtr).width = width;
    },
    canvas_set_height(canvasPtr, height) {
      const { heap } = app;
      heap.get(canvasPtr).height = height;
    },

    load_image(handle, ptrSrc, lenSrc) {
      const { wasm, utf8, heap } = app;
      const src = utf8.fromWasm(ptrSrc, lenSrc);
      const image = new Image();
      image.crossOrigin = "anonymous";
      image.onload = () => {
        const imagePtr = heap.add(image);
        wasm.load_image_callback(
          handle,
          imagePtr,
          image.naturalWidth,
          image.naturalHeight,
        );
      };
      image.src = src;
    },
  };
}

define(initializeCanvasBindings);
