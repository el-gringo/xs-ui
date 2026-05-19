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
  };
}

define(initializeDocumentBindings);
