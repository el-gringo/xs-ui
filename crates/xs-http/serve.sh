#!/usr/bin/env sh

EXAMPLE=$1
TARGET=target/wasm32-unknown-unknown/debug/examples/

cargo watch \
  --package xs-http \
  --exec "build --package xs-http --target wasm32-unknown-unknown --target-dir ./target --example $EXAMPLE" \
  --shell "
    cp index.html $TARGET && \
    sed -i \"s/{wasm-file}/$EXAMPLE/\" $TARGET/index.html && \
    python3 -m http.server -d $TARGET
  "
