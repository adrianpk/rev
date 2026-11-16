#!/usr/bin/env bash
# Selected native regressions, independent frozen oracles and exactly two PTYs.
set -euo pipefail

if [[ ${1-} == --editor ]]; then
    shift
    printf 'REV_EDITOR_PID=%s\nREV_EDITOR_SIZE=24 100\n' "$$"
    exec "$1" "$2"
fi
if [[ ${1-} == --pty-child ]]; then
    shift
    stty rows 24 cols 100
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

# libtest returns success for an empty selection. Check every declared filter
# before running each native batch, including full-suite and library selections.
batch() {
    local kind=$1 suite=$2 listing selected filter
    shift 2
    local target=("--$kind" "$suite")
    [[ $kind != lib ]] || target=(--lib)
    listing=$(cargo test --locked "${target[@]}" -- --list)
    for filter in "$@"; do
        selected=$(awk -v filter="$filter" 'index($0, filter) && /: test$/ {n++} END {print n+0}' <<<"$listing")
        [[ $selected -gt 0 ]] || { printf 'empty selection: %s %s %s\n' "$kind" "$suite" "$filter" >&2; exit 1; }
        printf 'COVERAGE %s %s filter=%s cases=%s\n' "$kind" "$suite" "$filter" "$selected"
    done
    cargo test --locked "${target[@]}" -- "$@" --nocapture --test-threads=1
}

sha256sum --check <<'DIGESTS'
8a0ccc8e6a9ac4ebb24f4445c2f314b472045f6a426014b53ced065f477da771  ops/default/report/rev-delimiter-presentation/acceptance-contract.json
bd694628253ddd3338e83aa5b79b1821429d50f5a1f6256e8490e0d99a715e3a  ops/default/report/rev-factory-shortcuts/catalog.json
e578368195b2e46acb0c7e7e1a8f3b289ddf0595667f64f890a8cb418d8dd6cf  ops/default/report/rev-factory-shortcuts/mapping-proposal.json
b11986ff19227d0a30442325b7be95b4074b17b2649ad428bd0d4c717a1e3fd3  ops/default/report/rev-delimiter-navigation/reference-navigation.json
3becd84cbe82297735b2a916556a4499cd3c284be9f072579160e93b8e70e724  ops/default/report/rev-delimiter-navigation/reference-navigation.el
a92c78e4ebf8127b82793cc0bd0703d7340ee7e80149f96abf6d678fd6479a8a  ops/default/report/rev-delimiter-navigation/mate-contract.json
DIGESTS

# rendering::catalog_and_maps is the single complete independent 719-command /
# 28-factory-map oracle. Do not repeat a historical factory acceptance journey.
batch test pair_presentation ''
batch test pairs ''
batch test pair_navigation ''
batch test configuration 'pairs::' 'keymaps::' 'merging::' 'org_palette::' 'markdown::leaf_inheritance' 'markdown::candidate_rejection'
batch test display 'palette::'
batch test macros 'recording::native_text' 'recording::effective_bindings' 'capture::named_prefixes' 'runner::source_invalidation' 'runner::cancellation' 'runner::modal_entry' 'runner::native_stops'
batch test org_headings 'continuity::'
batch test markdown 'continuity::'
batch lib '' 'pair_presentation_continuity'
batch example completion-acceptance 'every_completed_update'

cargo build --locked --bin rev --example delimiter-presentation-acceptance
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
fixture=$(mktemp -d -t rev-delimiter-presentation-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
for profile in non-modal modal; do
    timeout --kill-after=1 15 env -u NO_COLOR TERM=xterm-256color COLORTERM=truecolor \
        "$target/debug/examples/delimiter-presentation-acceptance" \
        "$fixture/$profile" "$target/debug/rev" "$root/scripts/check-delimiter-presentation.sh" "$profile"
done
printf 'PASS delimiter presentation: selected native batches, frozen digests, one catalog/map oracle and two restored/reaped Linux PTYs\n'
