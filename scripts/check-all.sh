#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
python3 scripts/check-source.py
python3 scripts/check-pty.py --suite acceptance
python3 scripts/check-performance.py
python3 scripts/check-acceptance.py
git diff --check
