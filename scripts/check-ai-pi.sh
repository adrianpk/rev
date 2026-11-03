#!/usr/bin/env bash
# Joint offline acceptance; live provider evidence is a separate one-shot mode.
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
: "${REV_PI_EXECUTABLE:?Pin the installed native Pi 1.3.0 executable; update wrappers are not admitted}"
for tool in script timeout stty; do command -v "$tool" >/dev/null; done
fixture=$(mktemp -d -t rev-ai-pi-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
unset REV_PI_LIVE
started=$SECONDS
CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 2 --bin rev --example ai-pi-acceptance
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
"$target/release/examples/ai-pi-acceptance" --version
for profile in non-modal modal; do
    native="$fixture/$profile-native"
    mkdir -p "$native/home" "$native/config/rev" "$native/workspace"
    env REV_ACCEPTANCE_PROFILE="$profile" HOME="$native/home" \
        XDG_CONFIG_HOME="$native/config" XDG_DATA_HOME="$native/data" \
        XDG_STATE_HOME="$native/state" XDG_CACHE_HOME="$native/cache" \
        PI_CODING_AGENT_DIR="$native/pi-agent" PI_OFFLINE=1 PI_TELEMETRY=0 \
        timeout --kill-after=2 35 "$target/release/examples/ai-pi-acceptance" --native "$native"
    terminal="$fixture/$profile-terminal"
    mkdir -p "$terminal/pty/home" "$terminal/pty/config/rev"
    env REV_ACCEPTANCE_PROFILE="$profile" TERM=xterm-256color NO_COLOR=1 \
        PI_CODING_AGENT_DIR="$terminal/pi-agent" PI_OFFLINE=1 PI_TELEMETRY=0 \
        "$target/release/examples/ai-pi-acceptance" --pty "$terminal" "$target/release/rev" "$repo_root/scripts/check-ai-pi.sh"
done
printf 'MEASURE joint_total_seconds=%s including build; editor_bytes=%s clock_ticks_per_second=%s\n' \
    "$((SECONDS - started))" "$(stat -c %s "$target/release/rev")" "$(getconf CLK_TCK)"
echo 'PASS joint offline acceptance: both profiles; actual Pi RPC/tools/history; native reload/apply/undo; bounded restored PTYs'
