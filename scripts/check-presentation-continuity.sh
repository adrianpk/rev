#!/usr/bin/env bash
# Narrow continuity matrix over the existing native drivers and owned PTY stream.
set -euo pipefail

if [[ ${1-} == --editor || ${1-} == --pty-child ]]; then
    exec bash "$(dirname "${BASH_SOURCE[0]}")/check-markdown.sh" "$@"
fi

cd "$(dirname "${BASH_SOURCE[0]}")/.."
root=$PWD
fixture=$(mktemp -d -t rev-presentation-continuity-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
for required in script stty kill timeout; do command -v "$required" >/dev/null; done
rustc --version
cargo --version
cargo test --locked --example org-heading-acceptance terminal::tests -- --test-threads=1
CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 1 --bin rev --example org-heading-acceptance --example markdown-acceptance
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
for owner in native pty; do
    mkdir -p "$fixture/$owner/home" "$fixture/$owner/config/rev"
done
mkdir -p "$fixture/workspace"
isolation=(env HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
    XDG_CACHE_HOME="$fixture/native/cache" TERM=xterm-256color)
printf 'BUILD_PROFILE=normal release, jobs=1, target=%s, cpu_count=%s, clock_ticks_per_second=%s\n' "$(uname -m)" "$(getconf _NPROCESSORS_ONLN)" "$(getconf CLK_TCK)"
for driver in org-heading-acceptance markdown-acceptance; do
    timeout --kill-after=1 120 "${isolation[@]}" "$target/release/examples/$driver" --continuity-native "$fixture"
    for colors in enabled disabled monochrome; do
        if [[ $colors == monochrome ]]; then color_env=(env NO_COLOR=1); else color_env=(env -u NO_COLOR); fi
        timeout --kill-after=1 45 "${color_env[@]}" "${isolation[@]}" REV_CONTINUITY_COLORS="$colors" \
            "$target/release/examples/$driver" --continuity-pty "$fixture" "$target/release/rev" "$root/scripts/check-presentation-continuity.sh"
    done
done
printf 'PASS native and Linux PTY presentation continuity; graphical acceptance not claimed\n'
