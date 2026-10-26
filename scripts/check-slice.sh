#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
python3 scripts/check-source.py
if [ "$#" -gt 0 ]; then
    "$@"
fi
git diff --check
