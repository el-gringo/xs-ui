/**
 * @import { App } from "../types.js";
 *
 *
 * @param {App} app
 */
function initializeNodeBindings(app) {
  return {
    node_query_selector(elementPtr, selectorPtr, selectorLen) {
      const { heap, utf8 } = app;
      const selector = utf8.fromWasm(selectorPtr, selectorLen);
      const element = heap.get(elementPtr).querySelector(selector);
      return heap.add(element);
    },

    node_append_child(parentPtr, childPtr) {
      const { heap } = app;
      heap.get(parentPtr).appendChild(heap.get(childPtr));
    },

    node_insert_before(parentPtr, nodePtr, referencePtr) {
      const { heap } = app;
      const parent = heap.get(parentPtr);
      parent.insertBefore(heap.get(nodePtr), heap.get(referencePtr));
    },

    node_remove_child(parentPtr, childPtr) {
      const { heap } = app;
      heap.get(parentPtr).removeChild(heap.get(childPtr));
    },

    node_replace_child(parentPtr, newChildPtr, oldChildPtr) {
      const { heap } = app;
      const parent = heap.get(parentPtr);
      parent.replaceChild(heap.get(newChildPtr), heap.get(oldChildPtr));
    },
  };
}

define(initializeNodeBindings);
