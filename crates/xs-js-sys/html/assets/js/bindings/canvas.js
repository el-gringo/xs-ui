/**
 * @import { App } from "../types.js";
 *
 *
 * @param {App} app
 */
function initializeCanvasBindings(app) {
  const { heap } = app;

  return {
    // Asking a canvas for a context is also how the app says which canvas it
    // renders into: the gl bindings read `app.gl` and the canvas sizing reads
    // `app.canvas`, so nothing has to guess which canvas the page has.
    canvas_get_context(canvasPtr, contextTypePtr, contextTypeLen) {
      const { heap, utf8 } = app;
      const canvas = heap.get(canvasPtr);
      const contextType = utf8.fromWasm(contextTypePtr, contextTypeLen);
      app.canvas = canvas;
      const gl = canvas.getContext(contextType);
      if (gl === null) {
        // `app.gl` keeps the placeholder the gl bindings installed, so a call
        // still explains itself; this line says why the context is missing.
        console.error(`canvas_get_context: no ${contextType} on this canvas`);
        return -1;
      }
      // The objects, not their heap indexes: the JsValue returned here is
      // dropped as soon as the caller's binding goes out of scope, and a freed
      // index would be a recycled slot.
      app.gl = gl;
      return heap.add(gl);
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
        wasm.load_image_callback(handle, imagePtr, image.naturalWidth, image.naturalHeight);
      };
      image.src = src;
    },
  };
}

define(initializeCanvasBindings);
