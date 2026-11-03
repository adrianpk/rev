#!/usr/bin/env bash
# Native heading presentation: isolated stores, one release job and owned PTYs.
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
fixture=$(mktemp -d -t rev-org-headings-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
for required in rg script stty kill timeout; do command -v "$required" >/dev/null; done
rustc --version
cargo --version
CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 1 --bin rev --example org-heading-acceptance --example source-check
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
source_args=(--root src/lib.rs --root src/main.rs)
for source in src/config/headings.rs src/display/headings.rs src/view/headings.rs src/session/outline/headings.rs; do
    source_args+=(--file "$source")
done
source_args+=(--root examples/org-heading-acceptance.rs --root examples/source-check.rs)
while IFS= read -r source; do source_args+=(--file "$source"); done < <(rg --files examples/org_heading_acceptance -g '*.rs' | LC_ALL=C sort)
source_args+=(--file examples/completion_acceptance/terminal.rs)
"$target/release/examples/source-check" "${source_args[@]}" > "$fixture/source-quality.log"
tail -n 1 "$fixture/source-quality.log"
printf 'SOURCE_COVERAGE=heading policy/projection/view/control owners, crate roots and heading/shared acceptance modules\n'
printf 'BUILD_PROFILE=normal release, jobs=1, target=%s, cpu_count=%s, clock_ticks_per_second=%s\n' "$(uname -m)" "$(getconf _NPROCESSORS_ONLN)" "$(getconf CLK_TCK)"
for owner in native pty; do
    mkdir -p "$fixture/$owner/home" "$fixture/$owner/config/rev"
done
mkdir -p "$fixture/workspace"
driver=("$target/release/examples/org-heading-acceptance")
isolation=(env HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
    XDG_CACHE_HOME="$fixture/native/cache" TERM=xterm-256color NO_COLOR=1)
timeout --kill-after=1 20 "${isolation[@]}" "${driver[@]}" --native "$fixture"
timeout --kill-after=1 20 "${isolation[@]}" "${driver[@]}" --resources "$fixture"
timeout --kill-after=1 35 "${isolation[@]}" "${driver[@]}" --pty "$fixture" "$target/release/rev" "$root/scripts/check-org-headings.sh"
printf 'PASS native Org heading acceptance\n'
