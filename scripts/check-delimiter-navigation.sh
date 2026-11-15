#!/usr/bin/env bash
# Native shared-owner batches and exactly two physical-input journeys.
set -euo pipefail

if [[ ${1-} == --editor ]]; then
    shift
    printf 'REV_EDITOR_PID=%s\nREV_EDITOR_SIZE=24 80\n' "$$"
    exec "$1" "$2"
fi
if [[ ${1-} == --pty-child ]]; then
    shift
    stty rows 24 cols 80
    before=$(stty -g)
    result=0
    timeout --foreground --kill-after=1 8 bash "$0" --editor "$1" "$2" || result=$?
    [[ $(stty -g) == "$before" ]] || { echo 'terminal attributes changed' >&2; exit 1; }
    printf '\nREV_TTY_RESTORED\n'
    exit "$result"
fi

cd "$(dirname "${BASH_SOURCE[0]}")/.."
root=$PWD
for required in script stty timeout sha256sum; do command -v "$required" >/dev/null; done
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1

# Check each declared filter before executing the batch: libtest otherwise
# succeeds silently when a renamed filter selects no cases.
batch() {
    local suite=$1 listing selected filter
    shift
    listing=$(cargo test --locked --test "$suite" -- --list)
    for filter in "$@"; do
        selected=$(awk -v filter="$filter" 'index($0, filter) && /: test$/ {n++} END {print n+0}' <<<"$listing")
        [[ $selected -gt 0 ]] || { printf 'empty selection: %s %s\n' "$suite" "$filter" >&2; exit 1; }
        printf 'COVERAGE %s filter=%s cases=%s\n' "$suite" "$filter" "$selected"
    done
    cargo test --locked --test "$suite" -- "$@" --nocapture --test-threads=1
}

batch pair_navigation ''
batch pairs ''
batch configuration 'keymaps::' 'pairs::'
batch editing 'prefix_arguments::history_groups' 'prefix_arguments::stale_preparation' 'prefix_arguments::modal_entries'
batch macros 'recording::normalized_prefixes' 'recording::effective_bindings' 'capture::named_prefixes' 'runner::source_invalidation' 'runner::cancellation' 'runner::modal_entry' 'runner::native_stops' 'regions::boundaries' 'regions::unicode_wrap' 'regions::cancellation'
batch modal 'objects::delimited_objects' 'local_targets::structural_targets'
batch org_headings 'continuity::'
batch markdown 'continuity::'

listing=$(cargo test --locked --lib -- --list)
for filter in mate_stale groups_stale nesting_stale expressions_stale; do
    selected=$(awk -v filter="$filter" 'index($0, filter) && /: test$/ {n++} END {print n+0}' <<<"$listing")
    [[ $selected -gt 0 ]] || { printf 'empty library selection: %s\n' "$filter" >&2; exit 1; }
    printf 'COVERAGE library filter=%s cases=%s\n' "$filter" "$selected"
done
cargo test --locked --lib -- mate_stale groups_stale nesting_stale expressions_stale --nocapture --test-threads=1

sha256sum --check <<'DIGESTS'
bd694628253ddd3338e83aa5b79b1821429d50f5a1f6256e8490e0d99a715e3a  ops/default/report/rev-factory-shortcuts/catalog.json
e578368195b2e46acb0c7e7e1a8f3b289ddf0595667f64f890a8cb418d8dd6cf  ops/default/report/rev-factory-shortcuts/mapping-proposal.json
b11986ff19227d0a30442325b7be95b4074b17b2649ad428bd0d4c717a1e3fd3  ops/default/report/rev-delimiter-navigation/reference-navigation.json
3becd84cbe82297735b2a916556a4499cd3c284be9f072579160e93b8e70e724  ops/default/report/rev-delimiter-navigation/reference-navigation.el
a92c78e4ebf8127b82793cc0bd0703d7340ee7e80149f96abf6d678fd6479a8a  ops/default/report/rev-delimiter-navigation/mate-contract.json
DIGESTS
cargo build --locked --bin rev --example factory-shortcuts-acceptance --example delimiter-navigation-acceptance
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
fixture=$(mktemp -d -t rev-delimiter-navigation-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
mkdir -p "$fixture/native/home" "$fixture/native/config"
isolation=(env HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
    XDG_CACHE_HOME="$fixture/native/cache" TERM=xterm-256color NO_COLOR=1)
timeout --kill-after=1 30 "${isolation[@]}" "$target/debug/examples/factory-shortcuts-acceptance" --native "$fixture/native"
for profile in non-modal modal; do
    timeout --kill-after=1 30 "${isolation[@]}" "$target/debug/examples/delimiter-navigation-acceptance" \
        "$fixture/$profile" "$target/debug/rev" "$root/scripts/check-delimiter-navigation.sh" "$profile"
done
printf 'PASS delimiter navigation: native bounds/shared owners and two Linux PTYs; owned fixtures cleaned on exit\n'
