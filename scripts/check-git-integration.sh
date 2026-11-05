#!/usr/bin/env bash
# One selected Git acceptance entry; historical feature gates are independent.
set -euo pipefail
if [[ ${1-} == --editor ]]; then
    shift
    printf 'REV_EDITOR_PID=%s\n' "$$"
    printf 'REV_EDITOR_SIZE=24 100\n'
    printf '%s\n' "$$" > "$(dirname "$2")/../pty/editor.pid"
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
mode=${1:---all}
case "$mode" in --all|--native|--resources|--lifetime|--pty|--build) ;; *) echo 'unknown Git acceptance mode' >&2; exit 1 ;; esac
fixture=$(mktemp -d -t rev-git-acceptance-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
for required in git timeout script stty kill tar stat rg; do command -v "$required" >/dev/null; done
available=$(df -B1 --output=avail . | tail -n 1)
((available >= 2147483648)) || { echo 'Git candidate and baseline builds require 2 GiB free' >&2; exit 1; }
export TMPDIR="$fixture" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_CONFIG_COUNT GIT_CONFIG_PARAMETERS
phase() {
    local name=$1 started=$SECONDS
    shift
    "$@"
    printf 'MEASURE phase=%s elapsed_seconds=%s\n' "$name" "$((SECONDS-started))"
}
baseline=772051f7a70b4be66d86ccf4fdea27f9b6c2da68
# This set adds no dependencies. Compare actual manifest/lock bytes, including
# an uncommitted candidate, rather than treating build provenance as proof.
git diff --exit-code "$baseline" -- Cargo.toml Cargo.lock
phase candidate-build env CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 1 --bin rev --example git-integration-acceptance --example source-check
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
source_args=(--root examples/git-integration-acceptance.rs --file examples/completion_acceptance/terminal.rs
    --file tests/configuration/fixture.rs --file tests/git_integration/support.rs
    --file examples/shell_command_acceptance/metrics.rs)
while IFS= read -r source; do source_args+=(--file "$source"); done < <(rg --files examples/git_integration_acceptance -g '*.rs' | LC_ALL=C sort)
"$target/release/examples/source-check" "${source_args[@]}" > "$fixture/source-quality.log"
tail -n 1 "$fixture/source-quality.log"
printf 'SOURCE_COVERAGE=Git acceptance entry, journeys, resources, lifetime, routing and shared fixture/PTY owners; dependency additions=0\n'
if [[ $mode == --all || $mode == --build ]]; then
    # Fresh immutable source is owned by this invocation. The baseline Cargo
    # target is separate from the candidate and belongs to this slice worktree.
    mkdir "$fixture/baseline"
    git archive "$baseline" | tar -x -C "$fixture/baseline"
    phase baseline-build env CARGO_TARGET_DIR="$target/git-baseline-$baseline" CARGO_NET_OFFLINE=true \
        cargo build --locked --release --jobs 1 --manifest-path "$fixture/baseline/Cargo.toml" --bin rev
    baseline_binary="$target/git-baseline-$baseline/release/rev"
    baseline_bytes=$(stat -c %s "$baseline_binary")
    candidate_bytes=$(stat -c %s "$target/release/rev")
    printf 'MEASURE Git delivery binary delta: baseline=%s baseline_bytes=%s candidate_bytes=%s delta_bytes=%s\n' \
        "$baseline" "$baseline_bytes" "$candidate_bytes" "$((candidate_bytes-baseline_bytes))"
    printf 'BUILD_PROFILE=release, locked offline dependencies, jobs=1, separate targets; '
    rustc --version
fi
mkdir -p "$fixture/home" "$fixture/config"
isolation=(env HOME="$fixture/home" XDG_CONFIG_HOME="$fixture/config" XDG_DATA_HOME="$fixture/data" \
    XDG_STATE_HOME="$fixture/state" XDG_CACHE_HOME="$fixture/cache" \
    TERM=xterm-256color LC_ALL=C NO_COLOR=1)
driver=("$target/release/examples/git-integration-acceptance")
if [[ $mode == --all || $mode == --native ]]; then
    phase native "${isolation[@]}" timeout --kill-after=1 60 "${driver[@]}" --native
fi
if [[ $mode == --all || $mode == --resources ]]; then
    phase resources "${isolation[@]}" timeout --kill-after=1 60 "${driver[@]}" --resources
fi
if [[ $mode == --all || $mode == --lifetime ]]; then
    phase lifetime "${isolation[@]}" timeout --kill-after=1 20 "${driver[@]}" --lifetime
fi
if [[ $mode == --all ]]; then
    phase baseline-startup "${isolation[@]}" timeout --kill-after=1 20 "${driver[@]}" --startup "$baseline_binary" "$root/scripts/check-git-integration.sh"
fi
if [[ $mode == --all || $mode == --pty ]]; then
    phase pty "${isolation[@]}" timeout --kill-after=1 20 "${driver[@]}" --pty "$target/release/rev" "$root/scripts/check-git-integration.sh"
fi
printf 'PASS Git acceptance mode=%s\n' "$mode"
