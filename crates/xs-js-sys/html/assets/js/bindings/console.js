/**
 * @import { App } from "../types.js";
 *
 *
 * @param {App} app
 */
function initializeConsoleBindings({ utf8 }) {
  return {
    console_log(ptr, len) {
      console.log(utf8.fromWasm(ptr, len));
    },
    console_debug2(ptrArg1, lenArg1, ptrArg2, lenArg2) {
      console.debug(
        utf8.fromWasm(ptrArg1, lenArg1),
        utf8.fromWasm(ptrArg2, lenArg2),
      );
    },
    console_info2(ptrArg1, lenArg1, ptrArg2, lenArg2) {
      console.info(
        utf8.fromWasm(ptrArg1, lenArg1),
        utf8.fromWasm(ptrArg2, lenArg2),
      );
    },
    console_error2(ptrArg1, lenArg1, ptrArg2, lenArg2) {
      console.error(
        utf8.fromWasm(ptrArg1, lenArg1),
        utf8.fromWasm(ptrArg2, lenArg2),
      );
    },
    console_trace2(ptrArg1, lenArg1, ptrArg2, lenArg2) {
      console.trace(
        utf8.fromWasm(ptrArg1, lenArg1),
        utf8.fromWasm(ptrArg2, lenArg2),
      );
    },
    console_warn2(ptrArg1, lenArg1, ptrArg2, lenArg2) {
      console.warn(
        utf8.fromWasm(ptrArg1, lenArg1),
        utf8.fromWasm(ptrArg2, lenArg2),
      );
    },
  };
}

define(initializeConsoleBindings);
