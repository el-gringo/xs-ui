/**
 * @import { Heap } from "./types.js";
 *
 * @returns {Heap}
 */
function Heap() {
  const heap = Array.from({ length: 128 }).fill(undefined);
  heap.push(undefined, null, true, false);
  let heapNext = heap.length;

  function add(obj) {
    if (heapNext === heap.length) heap.push(heap.length + 1);
    const idx = heapNext;
    heapNext = heap[idx];
    heap[idx] = obj;
    return idx;
  }

  function drop(idx) {
    if (idx < 132) return;
    heap[idx] = heapNext;
    heapNext = idx;
  }

  function get(idx) {
    return heap[idx];
  }

  function take(idx) {
    const ret = get(idx);
    drop(idx);
    return ret;
  }

  return {
    add,
    drop,
    get,
    take,
  };
}
define(Heap);
