function App(wasm) {
  const heap = WasmHeap();
  const string = WasmString(wasm);

}

function load(wasmUrl) {
  new Promise(function () {
    if (window["WasmString"] && window["WasmHeap"]) {
      return Promise.resolve();
    }
    return Promise.all([addScript("heap.js"), addScript("string.js")]);
  }).then(() => {
    fetch(wasmUrl)
      .then((response) => response.arrayBuffer())
      .then((bytes) =>
        WebAssembly.instantiate(bytes, { env }),
      )
      .then((res) => {

        wasm = res.instance.exports;

        wasm.main();
      });
  });
}
