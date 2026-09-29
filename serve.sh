#!/usr/bin/env sh

EXAMPLE=$1
TARGET=target/wasm32-unknown-unknown/debug/
# One directory per example: each run rewrites its own index.html and serves
# its own wasm, so several examples can be watched at once.
SERVE=$TARGET$EXAMPLE/
PORT=8421

if [ -z "$EXAMPLE" ]; then
    cargo run --bin
    exit 0
fi

# A server that cannot bind is silent about it, and the browser then keeps
# being served by whichever example got the port first.
if ss -tln 2>/dev/null | grep -q ":$PORT "; then
    echo "error: port $PORT is already in use, stop the other serve.sh first" >&2
    exit 1
fi

# We need to target-cpu=mvp because older chrome from SmartTv do not support some features
# This requires to rebuild the Rust's std
RUSTFLAGS="-Ctarget-cpu=mvp" cargo watch \
  --exec "build -Zbuild-std=panic_abort,std --target wasm32-unknown-unknown --target-dir ./target --bin $EXAMPLE" \
  --shell "
    mkdir -p $SERVE && \
    cp -vr crates/xs-js-sys/html/* $SERVE && \
    cp -f $TARGET$EXAMPLE.wasm $SERVE/ && \
    sed -i \"s/{wasm-pathname}/${EXAMPLE}/\" $SERVE/index.html && \
    echo \"Serving the '${EXAMPLE}' example on http://localhost:$PORT/ (from $SERVE)\" && \
    python3 -m http.server -d $SERVE $PORT
  "
