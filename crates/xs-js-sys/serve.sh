#!/usr/bin/env sh

EXAMPLE=$1
TARGET=target/wasm32-unknown-unknown/debug/examples/

cargo watch \
  --package xs-js-sys \
  --exec "build --package xs-js-sys --target wasm32-unknown-unknown --target-dir ./target --example $EXAMPLE" \
  --shell "
    cp -rv html/* $TARGET && \
    sed -i \"s/{wasm-pathname}/$EXAMPLE/\" $TARGET/index.html && \
    python3 -m http.server -d $TARGET 8421
  "
