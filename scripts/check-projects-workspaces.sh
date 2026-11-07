#!/usr/bin/env bash
# Native capacity/resource probes and one bounded serial journey per profile.
set -euo pipefail

if [[ ${1-} == --editor ]]; then
    shift
    printf 'REV_EDITOR_PID=%s\nREV_EDITOR_SIZE=30 120\n' "$$"
    exec "$@"
fi
if [[ ${1-} == --pty-child ]]; then
    shift
    stty rows 30 cols 120
    before=$(stty -g)
    result=0
    timeout --foreground --kill-after=1 8 bash "$0" --editor "$@" || result=$?
    [[ $(stty -g) == "$before" ]] || { echo 'terminal attributes changed' >&2; exit 1; }
    printf '\nREV_TTY_RESTORED\n'
    exit "$result"
fi

cd "$(dirname "${BASH_SOURCE[0]}")/.."
root=$PWD
fixture=$(mktemp -d -t rev-projects-workspaces-XXXXXXXX)
cleanup() {
    rm -rf -- "$fixture"
    [[ ! -e $fixture ]] || { echo 'owned fixture cleanup failed' >&2; return 1; }
}
trap cleanup EXIT
for required in script stty kill timeout; do command -v "$required" >/dev/null; done
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
CARGO_NET_OFFLINE=true cargo build --locked --bin rev --example projects-workspaces-acceptance
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
mkdir -p "$fixture/native/home" "$fixture/native/config"
isolation=(env HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
    XDG_CACHE_HOME="$fixture/native/cache" TERM=xterm-256color)
driver="$target/debug/examples/projects-workspaces-acceptance"
timeout --kill-after=1 60 "${isolation[@]}" "$driver" --native "$fixture/native"
for profile in non-modal modal; do
    if [[ $profile == modal ]]; then color=(env NO_COLOR=1); else color=(env -u NO_COLOR); fi
    timeout --kill-after=1 30 "${isolation[@]}" "${color[@]}" "$driver" --journey "$fixture/$profile" "$target/debug/rev" "$root/scripts/check-projects-workspaces.sh" "$profile"
done
printf 'PASS Projects and Workspaces acceptance; owned fixtures cleaned on exit\n'
