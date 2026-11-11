#!/usr/bin/env bash
set -euo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."

if cargo fmt --all -- --check; then
    printf '%s\n' 'Formatting: passed (all workspace packages).'
else
    printf '%s\n' 'Formatting: failed.' >&2
    exit 1
fi

# The Rust policy owner supplies the strict inherited profile. Every warning
# and selected ownership/representation diagnostic blocks the final workspace;
# scopes select native source review, never downgrade compiler enforcement.
profile="$(cargo run --locked --quiet -p rev --example source-check -- --print-clippy-flags)"
mapfile -t lint_flags <<< "$profile"
diagnostics="$(mktemp -d "${TMPDIR:-/tmp}/rev-quality.XXXXXXXX")"
trap 'rm -rf -- "$diagnostics"' EXIT

status=0
cargo clippy --locked --workspace --all-targets --all-features --message-format=json \
    -- "${lint_flags[@]}" > "$diagnostics/clippy.jsonl" || status=$?

cargo run --locked --quiet -p rev --example source-check -- \
    --workspace --clippy-json "$diagnostics/clippy.jsonl" --clippy-status "$status" "$@"
