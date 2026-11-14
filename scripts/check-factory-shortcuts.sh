#!/usr/bin/env bash
# Reuse native regression batches; keep physical journeys representative.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

catalog=ops/default/report/rev-factory-shortcuts/catalog.json
manifest=ops/default/report/rev-factory-shortcuts/mapping-proposal.json
[[ $(sha256sum "$catalog" | cut -d ' ' -f 1) == bd694628253ddd3338e83aa5b79b1821429d50f5a1f6256e8490e0d99a715e3a ]]
[[ $(sha256sum "$manifest" | cut -d ' ' -f 1) == e578368195b2e46acb0c7e7e1a8f3b289ddf0595667f64f890a8cb418d8dd6cf ]]
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
cargo test --locked --test commands -- factory_shortcuts keymap org_contexts command_sequences --test-threads=1
cargo test --locked --test configuration keymaps -- --test-threads=1
cargo test --locked --test modal keymaps -- --test-threads=1
cargo build --locked --bin rev --example factory-shortcuts-acceptance
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
fixture=$(mktemp -d -t rev-factory-shortcuts-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
mkdir -p "$fixture/native/home" "$fixture/native/config"
isolation=(env HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
    XDG_CACHE_HOME="$fixture/native/cache" TERM=xterm-256color NO_COLOR=1)
timeout --kill-after=1 30 "${isolation[@]}" "$target/debug/examples/factory-shortcuts-acceptance" --native "$fixture/native"
printf 'PASS factory shortcut native acceptance; owned fixtures cleaned on exit\n'
