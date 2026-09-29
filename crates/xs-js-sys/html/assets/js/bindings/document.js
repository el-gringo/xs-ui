/**
 * @import { App } from "../types.js";
 *
 *
 * @param {App} app
 */
function initializeDocumentBindings(app) {
  return {
    document() {
      const { heap } = app;
      const ptr = heap.add(window.document);
      return ptr;
    },

    document_create_element(tagPtr, tagLen) {
      const { heap, utf8 } = app;
      const tag = utf8.fromWasm(tagPtr, tagLen);
      return heap.add(window.document.createElement(tag));
    },
  };
}

define(initializeDocumentBindings);
