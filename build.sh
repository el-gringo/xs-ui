#!/usr/bin/env sh

mode=${1:-release}

if [ "$mode" = "dev" ]; then
    cargo watch \
      --why \
      -p app \
      -x "build --target wasm32-unknown-unknown" \
      -s "cp -v target/wasm32-unknown-unknown/release/app.wasm www/main.wasm"
elif [ "$mode" = "release" ]; then
    patch_index_html() {
        index_html_path=dist/index.html
        sed 's/assets\/script.js/script.js/' -i $index_html_path
        sed 's/\/wasm32-unknown-unknown\/debug\/app.wasm/app.wasm/' -i $index_html_path
        sed 's/MODE = "development"/MODE = "production"/' -i $index_html_path
        echo "Patched ${index_html_path}"
    }
    mkdir -pv dist
    cargo build --package app --target wasm32-unknown-unknown --release
    cp -v target/wasm32-unknown-unknown/release/app.wasm dist/
    cp -v tools/cli/src/assets/* dist/
    patch_index_html
fi
