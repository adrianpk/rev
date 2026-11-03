#!/usr/bin/env bash
# One finite shell journey owns its sources, private stores and subprocesses.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
fixture=$(mktemp -d -t rev-shell-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
for required in sh sort tr cat sleep rg timeout; do command -v "$required" >/dev/null; done
available=$(df -B1 --output=avail . | tail -n 1)
((available >= 1073741824)) || { echo 'shell build requires 1 GiB free' >&2; exit 1; }
CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 1 --bin rev --example shell-command-acceptance --example source-check
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
source_args=(--root src/lib.rs --root src/main.rs --root examples/shell-command-acceptance.rs --root examples/source-check.rs)
for source in src/shell.rs src/process.rs src/config/shell.rs src/input/shell.rs src/session/shell.rs src/session/status.rs src/display/activity.rs; do
    source_args+=(--file "$source")
done
while IFS= read -r source; do source_args+=(--file "$source"); done < <(rg --files src/shell src/process src/session/shell examples/shell_command_acceptance -g '*.rs' | LC_ALL=C sort)
"$target/release/examples/source-check" "${source_args[@]}" > "$fixture/source-quality.log"
tail -n 1 "$fixture/source-quality.log"
printf 'SOURCE_COVERAGE=shell/configuration/prompt/publication/process/status owners, crate roots and shell acceptance modules\n'
printf 'BUILD_PROFILE=release, jobs=1, no incremental/dev/test debug, clock_ticks_per_second=%s\n' "$(getconf CLK_TCK)"
mkdir -p "$fixture/native/home" "$fixture/native/config" "$fixture/workspace"
timeout --kill-after=1 20 env HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config" \
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state" \
    XDG_CACHE_HOME="$fixture/native/cache" TERM=xterm-256color LC_ALL=C NO_COLOR=1 \
    "$target/release/examples/shell-command-acceptance" --native "$fixture"
