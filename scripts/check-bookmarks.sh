#!/usr/bin/env bash
# One owned native/PTY fixture; no installed binary or user store is used.
set -euo pipefail

if [[ ${1-} == --pty-child ]]; then
    shift
    stty rows 24 cols 100
    before=$(stty -g)
    trap 'stty "$before"' EXIT
    printf 'REV_PTY_TTY=%s\n' "$(tty)"
    result=0
    timeout --foreground --kill-after=1 8 "$1" "$2" || result=$?
    [[ $(stty -g) == "$before" ]] || { echo 'terminal attributes changed' >&2; exit 1; }
    printf '\nREV_TTY_RESTORED\n'
    exit "$result"
fi

cd "$(dirname "${BASH_SOURCE[0]}")/.."
root=$PWD
fixture=$(mktemp -d -t rev-bookmarks-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
export HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
export XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
export XDG_CACHE_HOME="$fixture/native/cache" TERM=xterm-256color NO_COLOR=1
for owner in native pty; do
    mkdir -p "$fixture/$owner/home" "$fixture/$owner/config/rev"
    cp docs/reference/config.toml "$fixture/$owner/config/rev/config.toml"
done
mkdir -p "$fixture/workspace"
printf '%s\n' 'origin remains unchanged' > "$fixture/workspace/origin.txt"
printf '* Parent\nHIDDEN bookmark body with words that wrap across a narrow pane\n* Peer\n' > "$fixture/workspace/notes.org"

cargo build --locked --release --bin rev --example bookmark-acceptance
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
"$target/release/examples/bookmark-acceptance" "$fixture" "$target/release/rev" "$root/scripts/check-bookmarks.sh"
