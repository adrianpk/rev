#!/usr/bin/env bash
# Native joint acceptance; fixtures and subprocesses belong only to this run.
set -euo pipefail

if [[ ${1-} == --pty-child ]]; then
    shift
    stty rows 24 cols 120
    before=$(stty -g)
    printf 'REV_PTY_TTY=%s\n' "$(tty)"
    result=0
    timeout --foreground --kill-after=1 8 "$1" "$2" || result=$?
    [[ $(stty -g) == "$before" ]] || { echo 'terminal attributes changed' >&2; exit 1; }
    printf '\nREV_TTY_RESTORED\n'
    exit "$result"
fi

cd "$(dirname "${BASH_SOURCE[0]}")/.."
root=$PWD
fixture=$(mktemp -d -t rev-wrap-navigation-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export HOME="$fixture/home" XDG_CONFIG_HOME="$fixture/config"
export XDG_DATA_HOME="$fixture/data" XDG_STATE_HOME="$fixture/state"
export XDG_CACHE_HOME="$fixture/cache" TERM=xterm-256color NO_COLOR=1
mkdir -p "$HOME" "$XDG_CONFIG_HOME/rev" "$fixture/workspace/child"
cp docs/reference/config.toml "$XDG_CONFIG_HOME/rev/config.toml"
printf 'fn main() { let message = "one two three four five six seven eight nine ten eleven twelve NEEDLE"; }\n' > "$fixture/workspace/alpha.rs"
printf '* A heading with several words that wraps across the narrow editing pane\nHIDDEN body\n** Child\nhidden child\n* Peer\n| alpha beta gamma | delta |\n#+begin_src text\nliteral source with many words\n#+end_src\n' > "$fixture/workspace/bravo.org"
printf 'unobserved descendant\n' > "$fixture/workspace/child/nested.txt"

cargo build --locked --release --bin rev --example wrap-navigation --example wrap-performance
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
"$target/release/examples/wrap-navigation" "$fixture" "$target/release/rev" "$root/scripts/check-wrap-navigation.sh"
"$target/release/examples/wrap-performance"
