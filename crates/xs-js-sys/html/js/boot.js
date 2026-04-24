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

function loadWasm(wasmUrl) {
  return require(["js/heap.js", "js/bridge.js"]).then((exports) => {
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
    require([
      "js/bindings/canvas.js",
      "js/bindings/console.js",
      "js/bindings/document.js",
      "js/bindings/event.js",
      "js/bindings/webgl.js",
      "js/bindings/window.js",
    ]).then((wasmBindings) => {
      for (let bindings of wasmBindings) {
        Object.assign(env, bindings(wasmUtils));
      }
      fetch(wasmUrl)
        .then((response) => response.arrayBuffer())
        .then((bytes) => WebAssembly.instantiate(bytes, { env }))
        .then((res) => {
          const wasm = res.instance.exports;
          wasmUtils.wasm = wasm;
          wasm.main();
        });
    });
  });
}
window.loadWasm = loadWasm;
