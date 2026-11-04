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
    stty rows 24 cols 80
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
build_started=$SECONDS
CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 1 --bin rev --example integrated-terminal-acceptance --example source-check
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
printf 'BUILD_PROFILE=normal-release jobs=1 elapsed_seconds=%s clock_ticks_per_second=%s\n' "$((SECONDS-build_started))" "$(getconf CLK_TCK)"
source_args=(--root src/lib.rs --root src/main.rs --root examples/integrated-terminal-acceptance.rs --root examples/source-check.rs)
for source in src/term.rs src/session/terminals.rs examples/completion_acceptance/terminal.rs examples/shell_command_acceptance/metrics.rs; do source_args+=(--file "$source"); done
while IFS= read -r source; do source_args+=(--file "$source"); done < <(rg --files src/term src/session/terminals examples/integrated_terminal_acceptance -g '*.rs' | LC_ALL=C sort)
"$target/release/examples/source-check" "${source_args[@]}" > "$fixture/source-quality.log"
tail -n 1 "$fixture/source-quality.log"
printf 'SOURCE_COVERAGE=terminal engine/session/view/input/copy/shutdown owners, crate roots and shared/native/PTY acceptance modules\n'
mkdir -p "$fixture/native/home" "$fixture/native/config" "$fixture/workspace"
# Remove inherited Git selectors and injected configuration before creating the
# owned repository; then install only this fixture's explicit Git policy.
unset_git=()
while IFS= read -r -d '' entry; do
    name=${entry%%=*}
    [[ $name != GIT_* ]] || unset_git+=(-u "$name")
done < <(env -0)
isolation=(env "${unset_git[@]}"
    HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
    XDG_CACHE_HOME="$fixture/native/cache" GIT_CONFIG_NOSYSTEM=1
    GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_TERMINAL_PROMPT=0
    TERM=xterm-256color LC_ALL=C NO_COLOR=1)
case "$mode" in
    --all|--native) timeout --kill-after=1 20 "${isolation[@]}" "$target/release/examples/integrated-terminal-acceptance" --native "$fixture" ;;&
    --all|--pty) timeout --kill-after=1 45 "${isolation[@]}" "$target/release/examples/integrated-terminal-acceptance" --pty "$fixture" "$target/release/rev" "$root/scripts/check-integrated-terminal.sh" ;;&
    --all|--resources) timeout --kill-after=1 20 "${isolation[@]}" "$target/release/examples/integrated-terminal-acceptance" --resources "$fixture" "$target/release/rev" ;;&
    --all|--native|--pty|--resources) ;;
    *) echo 'expected --all, --native, --pty or --resources' >&2; exit 1 ;;
esac
printf 'PASS integrated terminal acceptance mode=%s\n' "$mode"
