#!/usr/bin/env bash
# Build the browser bundle into ./dist for Cloudflare Workers Static Assets.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

if ! rustup target list --installed | grep -qx 'wasm32-unknown-unknown'; then
  rustup target add wasm32-unknown-unknown
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen is not installed. Install 0.2.129 to match Cargo.lock:" >&2
  echo "  cargo install wasm-bindgen-cli --version 0.2.129 --locked" >&2
  exit 1
fi

cargo build --profile wasm --target wasm32-unknown-unknown

rm -rf dist
mkdir -p dist/assets
wasm-bindgen \
  --no-typescript \
  --target web \
  --out-dir dist \
  --out-name color-bomber \
  target/wasm32-unknown-unknown/wasm/color-bomber.wasm

if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -Oz --all-features dist/color-bomber_bg.wasm -o dist/color-bomber_bg.opt.wasm
  mv dist/color-bomber_bg.opt.wasm dist/color-bomber_bg.wasm
else
  echo "wasm-opt not found; shipping the unoptimized wasm" >&2
fi

cp web/index.html dist/index.html
cp -a assets/sprites assets/fonts dist/assets/

bytes=$(wc -c < dist/color-bomber_bg.wasm)
echo "color-bomber_bg.wasm is ${bytes} bytes"
if (( bytes > 25 * 1024 * 1024 )); then
  echo "wasm exceeds the 25 MiB Workers static asset limit" >&2
  exit 1
fi
