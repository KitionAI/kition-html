#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SDK="${KITION_SDK_DIR:-$ROOT/../kition-sdk}"
cd "$ROOT"
node "$SDK/tools/check-manifest.mjs" kit.json --module engine
node "$SDK/tools/check-sdk.mjs" --dir engine --module kition_html_kit \
  --class HtmlDocument --name fixture.html \
  --edit '{"command":"file.setSource","params":{"source":"<!doctype html><html><body style=\"margin:0;background:#5645d4\"></body></html>"}}'
