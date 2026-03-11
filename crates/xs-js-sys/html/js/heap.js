function WasmHeap() {
  const heap = Array.from({ length: 128 }).fill(undefined);
  heap.push(undefined, null, true, false);
  let heapNext = heap.length;

  function add(obj) {
    if (heapNext === heap.length) heap.push(heap.length + 1);
    const id = heapNext;
    heapNext = heap[id];
    heap[id] = obj;
    return id;
  }

  function drop(id) {
    if (id < 132) return;
    heap[id] = heapNext;
    heapNext = id;
  }

  function get(id) {
    return heap[id];
  }

  function take(id) {
    const ret = get(id);
    drop(id);
    return ret;
  }

  return {
    add,
    drop,
    get,
    take
  };
}
window.WasmHeap = WasmHeap;
