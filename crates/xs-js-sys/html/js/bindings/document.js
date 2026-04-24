/**
 * @import { App } from "./types.js";
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
    document_query_selector(documentPtr, selectorPtr, selectorLen) {
      const { heap, utf8 } = app;
      const selector = utf8.fromWasm(selectorPtr, selectorLen);
      /** @type {Document} */
      const doc = heap.get(documentPtr);

      const element = doc.querySelector(selector);
      const ptr = heap.add(element);
      return ptr;
    },
  };
}

define(initializeDocumentBindings);
