#!/usr/bin/env bash
# One native/resource journey and two owned PTYs; no Python prerequisite.
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
fixture=$(mktemp -d -t rev-org-indentation-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
for required in rg script stty kill timeout; do command -v "$required" >/dev/null; done
rustc --version
cargo --version
CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 1 --bin rev --example org-indentation-acceptance --example source-check
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
source_args=(--root src/lib.rs --root src/main.rs --root examples/org-indentation-acceptance.rs --root examples/source-check.rs)
while IFS= read -r source; do source_args+=(--file "$source"); done < <(rg --files examples/org_indentation_acceptance -g '*.rs' | LC_ALL=C sort)
source_args+=(--file examples/org_heading_acceptance/support.rs --file examples/completion_acceptance/terminal.rs --file src/view/wrap/cache.rs)
"$target/release/examples/source-check" "${source_args[@]}" > "$fixture/source-quality.log"
tail -n 1 "$fixture/source-quality.log"
printf 'SOURCE_COVERAGE=crate roots, indentation acceptance, shared PTY/native helpers and actual cache-retention owner\n'
printf 'BUILD_PROFILE=normal release, jobs=1, target=%s, clock_ticks_per_second=%s\n' "$(uname -m)" "$(getconf CLK_TCK)"
for owner in native pty; do
    mkdir -p "$fixture/$owner/home" "$fixture/$owner/config/rev"
done
mkdir -p "$fixture/workspace"
driver=("$target/release/examples/org-indentation-acceptance")
isolation=(env HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
    XDG_CACHE_HOME="$fixture/native/cache" TERM=xterm-256color NO_COLOR=1)
printf 'CACHE_ASSERTIONS_PROFILE=low-debug test, jobs=1\n'
CARGO_NET_OFFLINE=true "${isolation[@]}" RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}" CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}" CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib view::wrap:: -- --nocapture --test-threads=1
timeout --kill-after=1 20 "${isolation[@]}" "${driver[@]}" --native "$fixture"
timeout --kill-after=1 30 "${isolation[@]}" "${driver[@]}" --resources "$fixture"
timeout --kill-after=1 20 "${isolation[@]}" "${driver[@]}" --pty "$fixture" "$target/release/rev" "$root/scripts/check-org-indentation.sh"
printf 'PASS native Org indentation acceptance\n'
