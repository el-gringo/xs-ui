/**
 * @import { App } from "./types.js";
 *
 *
 * @param {App} app
 */
function initializeConsoleBindings({ utf8 }) {
  return {
    console_log: function (ptr, len) {
      console.log(utf8.fromWasm(ptr, len));
    },
    console_debug: function (ptr, len) {
      console.debug(utf8.fromWasm(ptr, len));
    },
    console_info: function (ptr, len) {
      console.info(utf8.fromWasm(ptr, len));
    },
    console_error: function (ptr, len) {
      console.error(utf8.fromWasm(ptr, len));
    },
    console_warn: function (ptr, len) {
      console.warn(utf8.fromWasm(ptr, len));
    },
  };
}

define(initializeConsoleBindings);
