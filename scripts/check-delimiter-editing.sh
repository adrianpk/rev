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

batch pairs ''
batch configuration 'pairs::'
batch editing 'regions::' 'prefix_arguments::' 'typing_group' 'typing_views' 'prepared_history'
batch commands 'paste::'
batch macros 'recording::' 'runner::' 'capture::' 'regions::' 'counters::'
batch modal 'objects::delimited_objects' 'local_targets::structural_targets'
batch org_headings 'continuity::' 'geometry::'
batch markdown 'continuity::' 'presentation::'

sha256sum --check <<'DIGESTS'
bd694628253ddd3338e83aa5b79b1821429d50f5a1f6256e8490e0d99a715e3a  ops/default/report/rev-factory-shortcuts/catalog.json
e578368195b2e46acb0c7e7e1a8f3b289ddf0595667f64f890a8cb418d8dd6cf  ops/default/report/rev-factory-shortcuts/mapping-proposal.json
DIGESTS
cargo build --locked --bin rev --example factory-shortcuts-acceptance --example delimiter-editing-acceptance
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
fixture=$(mktemp -d -t rev-delimiter-editing-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
mkdir -p "$fixture/native/home" "$fixture/native/config"
isolation=(env HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
    XDG_CACHE_HOME="$fixture/native/cache" TERM=xterm-256color NO_COLOR=1)
timeout --kill-after=1 30 "${isolation[@]}" "$target/debug/examples/factory-shortcuts-acceptance" --native "$fixture/native"
for profile in non-modal modal; do
    timeout --kill-after=1 30 "${isolation[@]}" "$target/debug/examples/delimiter-editing-acceptance" \
        "$fixture/$profile" "$target/debug/rev" "$root/scripts/check-delimiter-editing.sh" "$profile"
done
printf 'PASS automatic delimiter editing: native bounds/shared owners and two Linux PTYs; owned fixtures cleaned on exit\n'
