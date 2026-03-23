#!/usr/bin/env bash
# Clean-build mruby and recompile rrcad — simulates a CI build locally.
#
# Usage:
#   ./scripts/clean-build.sh
#
# Run this whenever you change build.rs, mruby_configs/, or any mruby
# build configuration to verify the full from-scratch build works before
# pushing.

set -euo pipefail

MRUBY_LIB="vendor/mruby/build/host/lib/libmruby.a"

echo "==> Removing cached libmruby.a to force a clean mruby build..."
rm -f "$MRUBY_LIB"

echo "==> Running cargo build..."
cargo build

echo "==> Clean build succeeded."
