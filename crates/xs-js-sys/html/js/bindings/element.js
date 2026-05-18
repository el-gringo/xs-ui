/**
 * @import { App } from "./types.js";
 *
 *
 * @param {App} app
 */
function initializeElementBindings(app) {
  return {
    element_query_selector(elementPtr, selectorPtr, selectorLen) {
      const { heap, utf8 } = app;
      const selector = utf8.fromWasm(selectorPtr, selectorLen);
      const obj = heap.get(elementPtr);
      const element = obj.querySelector(selector);
      const ptr = heap.add(element);
      return ptr;
    },
    element_offset_width(elementPtr) {
      const { heap } = app;
      return heap.get(elementPtr).offsetWidth;
    },
    element_offset_height(elementPtr) {
      const { heap } = app;
      return heap.get(elementPtr).offsetHeight;
    },
  };
}

define(initializeElementBindings);
