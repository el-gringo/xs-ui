# JS/WASM bindings

## Run example
This will run a cargo watch and start a python server on port 8421

```shell
./serve.sh <example>
```

| Example                    | What it covers                                              |
| -------------------------- | ----------------------------------------------------------- |
| `fetch`                    | `fetch` and its callback                                    |
| `request-animation-frame`  | the frame loop                                              |
| `console`                  | `console_log`                                               |
| `document`                 | `document().query_selector`                                 |
| `canvas`                   | creates a canvas, its events, `offset_width` / `offset_height` |
| `webgl`                    | canvas + the WebGL2 bindings, clears the canvas to a color  |
| `dom`                      | attributes, classes, inline styles, inner html, tree edits  |

## Bindings mirror the browser api

There is no app specific binding: `src/canvas.rs` exposes the
`HTMLCanvasElement` properties (`get_context`, `width`, `height`) and the sizing
and focus code lives in the caller, `BackendBrowser::run`:

```rust
canvas.set_width(width);
canvas.set_height(height);
canvas.set_style("width", &format!("{width}px"));
canvas.set_style("height", &format!("{height}px"));
canvas.focus();
canvas.add_event_listener("blur", Closure::new(|_event: Event| {
    document().query_selector("canvas").focus();
}));
```

## The page has no canvas

`html/index.html` ships without a `<canvas>`. The app creates the one it needs
(`BackendBrowser::run` does it, before the event loop initializes, because the
renderer builds its gl resources there) and so do the `canvas` and `webgl`
examples, through `Document::create_element` plus `Node::append_child`. The
`canvas` css rule in the page only styles whatever canvas ends up there, and
positions it out of the flow so it never pushes page content below the fold.

`webgl.js` therefore resolves the context lazily, on the first gl call: the
bindings are built before `wasm.main()` runs, so at that point the canvas the
app is about to create does not exist yet.

`webgl` builds with `xs-gl-sys` as a dev dependency and is the smoke test for
`html/assets/js/bindings/webgl.js`: it creates a canvas, gets a `webgl2` context
and clears it.

`dom` is the reference for the two traits: `Element` in `src/element.rs` (attributes,
classes, inline styles, inner html, dimensions, `remove`, `focus`) and `Node` in
`src/node.rs` (`query_selector`, `append_child`, `insert_before`, `remove_child`,
`replace_child`). It builds a list, reorders it with `insert_before`, swaps an item
with `replace_child` and exercises `remove_child` on a throwaway child, so every
demo node stays attached and can be targeted by pointer events later.

Strings coming back from the DOM are written in wasm memory as null terminated c
strings, so reading one with `CStr::from_ptr` is what `get_attribute`,
`get_inner_html` and `get_style` do. Those buffers come from the heap allocator
and are not freed.

The js side mirrors the rust side: `bindings/node.js` holds the tree operations
and the query, `bindings/element.js` the element properties. `boot.js` requires
both.
