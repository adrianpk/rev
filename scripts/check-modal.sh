#!/usr/bin/env bash
# One modal journey owns its isolated stores, installed server and finite PTYs.
set -euo pipefail

if [[ ${1-} == --editor ]]; then
    shift
    printf 'REV_EDITOR_PID=%s\n' "$$"
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
fixture=$(mktemp -d -t rev-modal-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
command -v rust-analyzer >/dev/null
command -v rg >/dev/null
rust-analyzer --version
rustc --version
cargo --version
CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 2 --bin rev --example modal-acceptance --example source-check
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
source_args=()
while IFS= read -r source; do
    kind=--file
    [[ $source == src/lib.rs || $source == src/main.rs ]] && kind=--root
    source_args+=("$kind" "$source")
done < <(rg --files src -g '*.rs' | LC_ALL=C sort)
production_count=$((${#source_args[@]} / 2))
[[ $production_count -gt 2 ]] || { echo 'missing production source inventory' >&2; exit 1; }
source_args+=(--root examples/modal-acceptance.rs --root examples/source-check.rs)
while IFS= read -r source; do source_args+=(--file "$source"); done < <(rg --files examples/modal_acceptance -g '*.rs' | LC_ALL=C sort)
source_args+=(--file examples/completion_acceptance/pty.rs --file examples/completion_acceptance/support.rs)
"$target/release/examples/source-check" "${source_args[@]}" > "$fixture/source-quality.log"
tail -n 1 "$fixture/source-quality.log"
printf 'SOURCE_COVERAGE=%s production files, actual crate roots and modal/shared acceptance modules\n' "$production_count"
printf 'BUILD_PROFILE=normal release, jobs=2, target=%s, cpu_count=%s, clock_ticks_per_second=%s\n' "$(uname -m)" "$(getconf _NPROCESSORS_ONLN)" "$(getconf CLK_TCK)"
for owner in native pty; do
    mkdir -p "$fixture/$owner/home" "$fixture/$owner/config/rev"
done
cp docs/reference/config.toml "$fixture/pty/config/rev/config.toml"
mkdir -p "$fixture/workspace"
env HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config" \
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state" \
    XDG_CACHE_HOME="$fixture/native/cache" RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}" \
    TERM=xterm-256color NO_COLOR=1 \
    "$target/release/examples/modal-acceptance" "$fixture" "$target/release/rev" "$root/scripts/check-modal.sh"
