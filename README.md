# XS-UI

An extra small, Rust GPU accelerated UI toolkit with minimal dependencies. Targeting STB / SmartTVs

Inspired by [GPUI](https://github.com/zed-industries/gpui) and [Iced](https://github.com/iced-rs/iced). With a builder API.

Highly experimental. Not production ready.

## Features

- Minimal dependencies - compiles in seconds
- Linux Wayland & WASM
- OpenGl / WebGl renderer
- 

## TODO

[] MSDF text rendering (currently using alpha atlas)
[] E2E Testing
[] UI Components set
[] Storybook
[] Inspector


## Crates

- `xs` - Application + re-export
- `xs-backend-browser` - Browser/WASM backend
- `xs-backend-wayland` - Wayland backend
- `xs-renderer` - Rendering abstractions
- `xs-renderer-opengl` - OpenGL renderer
- `xs-http` - Http server / client
- `xs-js` - Javascript bindings
- `xs-layout` - Flex layout system
- `xs-math` - Math primitives
- `xs-websocket` - WebSocket client

## Examples

```rust

```

```bash
# Run the example app
cargo run --example app
```

## License

GPL 3
