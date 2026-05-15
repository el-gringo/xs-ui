#!/usr/bin/env sh

EXAMPLE=$1
TARGET=target/wasm32-unknown-unknown/debug/

if [ -z "$EXAMPLE" ]; then
    cargo run --bin
    exit 0
fi

cargo watch \
  --exec "build --target wasm32-unknown-unknown --target-dir ./target --bin $EXAMPLE" \
  --shell "
    cp -vr tools/cli/src/html/* $TARGET && \
    sed -i \"s/app.wasm/${EXAMPLE}.wasm/\" $TARGET/index.html && \
    python3 -m http.server -d $TARGET 8421
  "
