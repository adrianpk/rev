#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

cargo run --quiet --example documentation-check
cargo test --test commands reference
cargo test --test configuration reference
