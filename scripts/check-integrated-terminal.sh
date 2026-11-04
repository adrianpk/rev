#!/usr/bin/env bash
# All processes, repositories, stores and streams belong to this finite fixture.
set -euo pipefail
if [[ ${1-} == --editor ]]; then
    shift
    printf 'REV_EDITOR_PID=%s\n' "$$"
    printf 'REV_EDITOR_SIZE=%s\n' "$(stty size)"
    printf '%s\n' "$$" > "$(dirname "$2")/editor.pid"
    exec "$1" "$2"
fi
if [[ ${1-} == --pty-child || ${1-} == --error-child ]]; then
    kind=$1
    shift
    stty rows 40 cols 100
    before=$(stty -g)
    result=0
    if [[ $kind == --error-child ]]; then
        timeout --foreground --kill-after=1 8 "$1" --host-error "$2" || result=$?
    else
        timeout --foreground --kill-after=1 8 bash "$0" --editor "$1" "$2" || result=$?
    fi
    [[ $(stty -g) == "$before" ]] || { echo 'terminal attributes changed' >&2; exit 1; }
    printf '\nREV_TTY_RESTORED\n'
    exit "$result"
fi
cd "$(dirname "${BASH_SOURCE[0]}")/.."
root=$PWD
mode=${1:---all}
fixture=$(mktemp -d -t rev-terminal-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
for required in bash git lazygit rg script stty timeout; do command -v "$required" >/dev/null; done
available=$(df -B1 --output=avail . | tail -n 1)
((available >= 1073741824)) || { echo 'terminal build requires 1 GiB free' >&2; exit 1; }
CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 1 --bin rev --example integrated-terminal-acceptance --example source-check
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
source_args=(--root examples/integrated-terminal-acceptance.rs)
while IFS= read -r source; do source_args+=(--file "$source"); done < <(rg --files examples/integrated_terminal_acceptance -g '*.rs' | LC_ALL=C sort)
"$target/release/examples/source-check" "${source_args[@]}" > "$fixture/source-quality.log"
tail -n 1 "$fixture/source-quality.log"
mkdir -p "$fixture/native/home" "$fixture/native/config" "$fixture/workspace"
isolation=(env -u GIT_DIR -u GIT_WORK_TREE -u GIT_INDEX_FILE -u GIT_CONFIG_COUNT
    HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
    XDG_CACHE_HOME="$fixture/native/cache" GIT_CONFIG_NOSYSTEM=1
    GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_TERMINAL_PROMPT=0
    TERM=xterm-256color LC_ALL=C NO_COLOR=1)
case "$mode" in
    --all|--native) timeout --kill-after=1 20 "${isolation[@]}" "$target/release/examples/integrated-terminal-acceptance" --native "$fixture" ;;&
    --all|--pty) timeout --kill-after=1 25 "${isolation[@]}" "$target/release/examples/integrated-terminal-acceptance" --pty "$fixture" "$target/release/rev" "$root/scripts/check-integrated-terminal.sh" ;;&
    --all|--native|--pty) ;;
    *) echo 'expected --all, --native or --pty' >&2; exit 1 ;;
esac
printf 'PASS integrated terminal acceptance mode=%s\n' "$mode"
