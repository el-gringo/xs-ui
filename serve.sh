#!/usr/bin/env sh

# Serves an app from examples/ with the dev server.
#
#   ./serve.sh peertube
#
# Everything the page needs (the wasm loader, the hot reload client) is injected
# by the CLI, so this is only a shortcut for `xs serve`.

if [ -z "$1" ]; then
    echo "usage: ./serve.sh <package>"
    exit 1
fi

exec cargo run -p xs-cli -- serve "$1"
