#!/bin/bash
set -e
wasm-pack build ../ggl_wasm --target web --out-dir ../ggl_cli/static/pkg --no-typescript
