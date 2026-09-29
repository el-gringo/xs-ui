#!/usr/bin/env sh

# Serves one of the examples in this crate with the dev server.
#
#   ./serve.sh webgl
#
# The CLI injects the wasm loader and the hot reload client, so this crate does
# not have to know where its wasm ends up.

if [ -z "$1" ]; then
    echo "usage: ./serve.sh <example>"
    exit 1
fi

exec cargo run -p xs-cli -- serve --package xs-js-sys --example "$1" html/index.html
