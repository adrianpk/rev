#!/usr/bin/env bash
# One isolated management journey; real installed servers and controlled assets.
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
repo_root=$PWD
started=$SECONDS
fixture=$(mktemp -d -t rev-lsp-installation-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
for tool in rust-analyzer gopls go gzip sha256sum script timeout; do command -v "$tool" >/dev/null; done
export REV_ACCEPTANCE_RUST
REV_ACCEPTANCE_RUST=$(command -v rust-analyzer)
export REV_ACCEPTANCE_GO
REV_ACCEPTANCE_GO=$(command -v gopls)
rust-analyzer --version
gopls version
go version
rustc --version
cargo --version
CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 2 --bin rev --example lsp-installation-acceptance --example source-check
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
source_args=(--root examples/lsp-installation-acceptance.rs)
while IFS= read -r source; do source_args+=(--file "$source"); done < <(rg --files examples/lsp_installation_acceptance -g '*.rs' | LC_ALL=C sort)
source_args+=(--file examples/completion_acceptance/terminal.rs)
"$target/release/examples/source-check" "${source_args[@]}" > "$fixture/source-quality.log"
tail -n 1 "$fixture/source-quality.log"
export REV_INSTALLATION_FIXTURE="$fixture/acquisition"
"$target/release/examples/lsp-installation-acceptance" --fixture "$REV_INSTALLATION_FIXTURE"
export PATH="$REV_INSTALLATION_FIXTURE/bin:$PATH"
export CARGO_NET_OFFLINE=true GOTOOLCHAIN=local GOTELEMETRY=off CARGO_BUILD_JOBS=2
for profile in non-modal modal; do
    native="$fixture/$profile-native"
    mkdir -p "$native/home" "$native/config/rev" "$native/workspace" "$native/target"
    env REV_ACCEPTANCE_PROFILE="$profile" HOME="$native/home" \
        XDG_CONFIG_HOME="$native/config" XDG_DATA_HOME="$native/data" \
        XDG_STATE_HOME="$native/state" XDG_CACHE_HOME="$native/cache" \
        CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}" RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}" \
        CARGO_TARGET_DIR="$native/target" \
        timeout --kill-after=2 90 "$target/release/examples/lsp-installation-acceptance" --native "$native"
    terminal="$fixture/$profile-terminal"
    mkdir -p "$terminal/pty/home" "$terminal/pty/config/rev"
    env REV_ACCEPTANCE_PROFILE="$profile" TERM=xterm-256color NO_COLOR=1 \
        RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}" CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}" \
        "$target/release/examples/lsp-installation-acceptance" --pty "$terminal" "$target/release/rev" "$repo_root/scripts/check-lsp-installation.sh"
done
printf 'MEASURE editor_bytes=%s fixture_disk_bytes=%s helper_calls=%s; release jobs=2; server payloads separate\n' \
    "$(stat -c %s "$target/release/rev")" "$(du -sb "$fixture" | cut -f1)" "$(wc -l < "$fixture/acquisition/calls")"
printf 'MEASURE management_total_seconds=%s including build; cpu_count=%s clock_ticks_per_second=%s\n' "$((SECONDS - started))" "$(getconf _NPROCESSORS_ONLN)" "$(getconf CLK_TCK)"
echo 'PASS guided server management: both profiles, real Rust/Go, custom language, local managed acquisition, lifecycle and restored PTYs'
