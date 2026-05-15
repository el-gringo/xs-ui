#!/usr/bin/env sh

EXAMPLE=$1
TARGET=target/wasm32-unknown-unknown/debug/

if [ -z "$EXAMPLE" ]; then
    cargo run --bin
    exit 0
fi

# We need to target-cpu=mvp because older chrome from SmartTv do not support some features
# This requires to rebuild the Rust's std
RUSTFLAGS="-Ctarget-cpu=mvp" cargo watch \
  --exec "build -Zbuild-std=panic_abort,std --target wasm32-unknown-unknown --target-dir ./target --bin $EXAMPLE" \
  --shell "
    cp -vr tools/cli/src/html/* $TARGET && \
    sed -i \"s/app.wasm/${EXAMPLE}.wasm/\" $TARGET/index.html && \
    python3 -m http.server -d $TARGET 8421
  "
