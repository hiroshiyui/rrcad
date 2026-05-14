#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

rm -f vendor/mruby/build/host/lib/libmruby.a
./scripts/observe.sh cargo build
./scripts/observe.sh cargo test test_mcp_vm_no_file_read -- --ignored
