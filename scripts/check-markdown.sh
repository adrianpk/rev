#!/usr/bin/env bash
# One finite Markdown fixture, isolated stores and the existing bounded PTY owner.
set -euo pipefail

if [[ ${1-} == --editor ]]; then
    shift
    printf 'REV_EDITOR_PID=%s\n' "$$"
    printf '%s\n' "$$" > "$(dirname "$2")/../pty/editor.pid"
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
fixture=$(mktemp -d -t rev-markdown-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
for required in rg script stty kill timeout; do command -v "$required" >/dev/null; done
rustc --version
cargo --version
CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 1 --bin rev --example markdown-acceptance --example source-check
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
source_args=(--root examples/markdown-acceptance.rs --root examples/source-check.rs)
while IFS= read -r source; do source_args+=(--source "$source"); done < <(rg --files examples/markdown_acceptance -g '*.rs' | LC_ALL=C sort)
source_args+=(--source examples/completion_acceptance/terminal.rs --source tests/support/surface.rs)
"$target/release/examples/source-check" "${source_args[@]}" > "$fixture/source-quality.log"
tail -n 1 "$fixture/source-quality.log"
printf 'SOURCE_COVERAGE=Markdown acceptance crate, fixture owners and shared terminal/cell decoder\n'
printf 'BUILD_PROFILE=normal release, jobs=1, target=%s, cpu_count=%s, clock_ticks_per_second=%s\n' "$(uname -m)" "$(getconf _NPROCESSORS_ONLN)" "$(getconf CLK_TCK)"
for owner in native pty; do
    mkdir -p "$fixture/$owner/home" "$fixture/$owner/config/rev"
done
mkdir -p "$fixture/workspace"
driver=("$target/release/examples/markdown-acceptance")
isolation=(env HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
    XDG_CACHE_HOME="$fixture/native/cache" TERM=xterm-256color)
for colors in color monochrome; do
    if [[ $colors == color ]]; then color_env=(env -u NO_COLOR); else color_env=(env NO_COLOR=1); fi
    timeout --kill-after=1 20 "${color_env[@]}" "${isolation[@]}" "${driver[@]}" --native "$fixture"
    timeout --kill-after=1 35 "${color_env[@]}" "${isolation[@]}" "${driver[@]}" --pty "$fixture" "$target/release/rev" "$root/scripts/check-markdown.sh"
done
printf 'PASS native Markdown acceptance\n'
