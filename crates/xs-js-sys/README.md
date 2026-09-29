# JS/WASM bindings

## Run example

```shell
./serve.sh <example>   # a shortcut for: xs serve -p xs-js-sys -e <example> html/index.html
```

`xs` builds the example with cargo watch, serves it on :8080 and injects the
script that loads the wasm into `html/index.html`. In dev the injected script
also connects to the dev server websocket and reloads the page on every rebuild.
The wasm flags (`target-cpu=mvp`, the rebuilt std) come from the repository's
`.cargo/config.toml`, not from the tool.

## The html is plain, the tool injects

`html/index.html` carries no xs-ui markup: no wasm url, no loader, no hot reload
code. `xs` reads it on every request and replaces its

```html
<link rel="rust" />
```

with the script that boots the wasm, appending it at the end of the body when a
page has no marker (`tools/cli/src/html.rs`). The file stays plain html and the
tool owns everything that has to run in the page. `xs build` does the same,
without the websocket part, which is what `dist/` ends up containing.

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

`dom` is a small todo list and the reference for `Element` in `src/element.rs`
(attributes, classes, inline styles, inner html, dimensions, `remove`, `focus`)
and `Node` in `src/node.rs` (`query_selector`, `append_child`, `insert_before`,
`remove_child`, `replace_child`). It adds a row with `insert_before` (newest
first), toggles one done with `replace_child`, deletes one with `remove_child`,
and reports `classList` and coordinates from an `EventMouse`.

Strings coming back from the DOM are written in wasm memory as null terminated c
strings, so reading one with `CStr::from_ptr` is what `attribute`,
`inner_html` and `style` do. Those buffers come from the heap allocator and are
not freed.

The js side mirrors the rust side: `bindings/node.js` holds the tree operations
and the query, `bindings/element.js` the element properties. `boot.js` requires
both.
