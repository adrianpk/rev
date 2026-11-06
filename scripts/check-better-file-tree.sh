#!/usr/bin/env bash
# Native variants and one bounded owned PTY per interaction profile.
set -euo pipefail

if [[ ${1-} == --editor ]]; then
    shift
    printf 'REV_EDITOR_PID=%s\n' "$$"
    exec "$1" "$2"
fi
if [[ ${1-} == --pty-child ]]; then
    shift
    read -r columns rows < "$(dirname "$2")/../pty/size"
    stty rows "$rows" cols "$columns"
    before=$(stty -g)
    result=0
    timeout --foreground --kill-after=1 8 bash "$0" --editor "$1" "$2" || result=$?
    [[ $(stty -g) == "$before" ]] || { echo 'terminal attributes changed' >&2; exit 1; }
    printf '\nREV_TTY_RESTORED\n'
    exit "$result"
fi

cd "$(dirname "${BASH_SOURCE[0]}")/.."
root=$PWD
fixture=$(mktemp -d -t rev-better-file-tree-XXXXXXXX)
cleanup() {
    rm -rf -- "$fixture"
    [[ ! -e $fixture ]] || { echo 'owned fixture cleanup failed' >&2; return 1; }
}
trap cleanup EXIT
for required in script stty kill timeout; do command -v "$required" >/dev/null; done
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
CARGO_NET_OFFLINE=true cargo build --locked --bin rev --example better-file-tree-acceptance
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
mkdir -p "$fixture/native/home" "$fixture/native/config" "$fixture/workspace"
isolation=(env HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
    XDG_CACHE_HOME="$fixture/native/cache" TERM=xterm-256color NO_COLOR=1)
driver="$target/debug/examples/better-file-tree-acceptance"
timeout --kill-after=1 20 "${isolation[@]}" "$driver" --native "$fixture"
timeout --kill-after=1 20 "${isolation[@]}" "$driver" --no-color "$fixture"
timeout --kill-after=1 20 "${isolation[@]}" "$driver" --pty "$fixture" "$target/debug/rev" "$root/scripts/check-better-file-tree.sh"
printf 'PASS Better File Tree acceptance\n'
