#!/usr/bin/env sh

# Builds an app and everything the page needs into ./dist, ready to be served
# as static files.
#
#   ./build.sh [package] [--release|--debug] [--dist <dir>]
#
# `xs build` injects the wasm loader into the html and copies the assets, so
# there is no patching step here.

exec cargo run -p xs-cli -- build "${@:-peertube}"
