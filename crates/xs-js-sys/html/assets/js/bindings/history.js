/**
 * @import { App } from "../types.js";
 *
 *
 * @param {App} app
 */
function initializeHistoryBindings(app) {
  return {
    history_push_state(ptr, len) {
      var utf8 = app.utf8;
      window.history.pushState(null, "", utf8.fromWasm(ptr, len));
    },

    history_replace_state(ptr, len) {
      var utf8 = app.utf8;
      window.history.replaceState(null, "", utf8.fromWasm(ptr, len));
    },

    history_back() {
      window.history.back();
    },

    history_forward() {
      window.history.forward();
    },

    history_go(delta) {
      window.history.go(delta);
    },

    location_path() {
      var bytes = app.bytes;
      var location = window.location;
      return bytes.toCString(
        location.pathname + location.search + location.hash,
      );
    },
  };
}

define(initializeHistoryBindings);
