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
case "$mode" in --all|--native|--pty) ;; *) echo 'expected --native or --pty' >&2; exit 1 ;; esac
fixture=$(mktemp -d -t rev-git-acceptance-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
for required in git timeout script stty kill; do command -v "$required" >/dev/null; done
export TMPDIR="$fixture" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_CONFIG_COUNT GIT_CONFIG_PARAMETERS
CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 1 --bin rev --example git-integration-acceptance
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
mkdir -p "$fixture/home" "$fixture/config"
isolation=(env HOME="$fixture/home" XDG_CONFIG_HOME="$fixture/config" XDG_DATA_HOME="$fixture/data" \
    XDG_STATE_HOME="$fixture/state" XDG_CACHE_HOME="$fixture/cache" \
    TERM=xterm-256color LC_ALL=C NO_COLOR=1)
driver=("$target/release/examples/git-integration-acceptance")
if [[ $mode == --all || $mode == --native ]]; then
    "${isolation[@]}" timeout --kill-after=1 60 "${driver[@]}" --native
fi
if [[ $mode == --all || $mode == --pty ]]; then
    "${isolation[@]}" timeout --kill-after=1 20 "${driver[@]}" --pty "$target/release/rev" "$root/scripts/check-git-integration.sh"
fi
printf 'PASS Git acceptance mode=%s\n' "$mode"
