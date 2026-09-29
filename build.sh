#!/usr/bin/env sh

mode=${1:-release}
package=peertube

if [ "$mode" = "dev" ]; then
    cargo watch \
      --why \
      -p $package \
      -x "build --target wasm32-unknown-unknown" \
      -s "cp -v target/wasm32-unknown-unknown/debug/$package.wasm dist/app.wasm"
elif [ "$mode" = "release" ]; then
    patch_index_html() {
        index_html_path=dist/index.html
        sed 's/{wasm-pathname}/app/' -i $index_html_path
        sed 's/MODE = "development"/MODE = "production"/' -i $index_html_path
        echo "Patched ${index_html_path}"
    }
    mkdir -pv dist
    cargo build --package $package --target wasm32-unknown-unknown --release || exit 1
    cp -v target/wasm32-unknown-unknown/release/$package.wasm dist/app.wasm
    cp -rv crates/xs-js-sys/html/* dist/
    patch_index_html
fi
