/**
 * @import { App, Utf8, Bytes, WasmBridge } from "./types.js";
 */

let loaded = {};
function addScript(scriptUrl) {
  const loader = loaded[scriptUrl]
    ? Promise.resolve()
    : new Promise((resolve, reject) => {
        function onLoad() {
          resolve(window["__exports__"]);
          window["__exports__"] = null;
        }
        const scriptNode = document.createElement("script");
        scriptNode.type = "text/javascript";
        scriptNode.src = scriptUrl;
        scriptNode.addEventListener("load", onLoad);
        scriptNode.addEventListener("error", reject);
        document.head.appendChild(scriptNode);
      });
  loaded[scriptUrl] = true;
  return loader;
}

function require(requirements) {
  if (typeof requirements === "string") {
    requirements = [requirements];
  }
  return Promise.all(requirements.map(addScript));
}
window.require = require;

function define(exports) {
  window.__exports__ = exports;
}
window.define = define;

// Script urls are resolved against the document, not against this file.
const JS_DIR = "assets/js";

function loadWasm(wasmUrl) {
  return require([`${JS_DIR}/heap.js`, `${JS_DIR}/bridge.js`]).then(
    (exports) => {
      /** @type {[Heap, WasmBridge]} */
      const [Heap, WasmBridge] = exports;

      const wasmUtils = {};
      const heap = Heap();
      const { utf8, bytes } = WasmBridge(wasmUtils);
      Object.assign(wasmUtils, { heap, utf8, bytes });

      const env = {
        heap_drop(idx) {
          wasmUtils.heap.drop(idx);
        },
      };
      return require([
        `${JS_DIR}/bindings/canvas.js`,
        `${JS_DIR}/bindings/console.js`,
        `${JS_DIR}/bindings/document.js`,
        `${JS_DIR}/bindings/element.js`,
        `${JS_DIR}/bindings/event.js`,
        `${JS_DIR}/bindings/node.js`,
        `${JS_DIR}/bindings/webgl.js`,
        `${JS_DIR}/bindings/window.js`,
      ]).then((wasmBindings) => {
        for (let bindings of wasmBindings) {
          Object.assign(env, bindings(wasmUtils));
        }
        return fetch(wasmUrl)
          .then((response) => {
            if (!response.ok) {
              throw new Error(`${response.status} fetching ${wasmUrl}`);
            }
            return response.arrayBuffer();
          })
          .then((bytes) => WebAssembly.instantiate(bytes, { env }))
          .then((res) => {
            const wasm = res.instance.exports;
            wasmUtils.wasm = wasm;
            wasm.main();
          });
      });
    },
  ).catch((error) => {
    // A failure here used to be an unhandled rejection, which looks exactly
    // like an app that renders nothing.
    console.error(`xs-ui boot failed for ${wasmUrl}:`, error);
    throw error;
  });
}
window.loadWasm = loadWasm;
