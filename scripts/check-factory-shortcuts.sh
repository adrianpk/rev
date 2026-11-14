#!/usr/bin/env bash
# Reuse native regression batches; keep physical journeys representative.
set -euo pipefail

if [[ ${1-} == --editor ]]; then
    shift
    printf 'REV_EDITOR_PID=%s\nREV_EDITOR_SIZE=24 80\n' "$$"
    exec "$1" "$2"
fi
if [[ ${1-} == --pty-child ]]; then
    shift
    stty rows 24 cols 80
    before=$(stty -g)
    result=0
    timeout --foreground --kill-after=1 8 bash "$0" --editor "$1" "$2" || result=$?
    [[ $(stty -g) == "$before" ]] || { echo 'terminal attributes changed' >&2; exit 1; }
    printf '\nREV_TTY_RESTORED\n'
    exit "$result"
fi

cd "$(dirname "${BASH_SOURCE[0]}")/.."

root=$PWD
for required in script stty kill timeout git sha256sum; do command -v "$required" >/dev/null; done
catalog=ops/default/report/rev-factory-shortcuts/catalog.json
manifest=ops/default/report/rev-factory-shortcuts/mapping-proposal.json
[[ $(sha256sum "$catalog" | cut -d ' ' -f 1) == bd694628253ddd3338e83aa5b79b1821429d50f5a1f6256e8490e0d99a715e3a ]]
[[ $(sha256sum "$manifest" | cut -d ' ' -f 1) == e578368195b2e46acb0c7e7e1a8f3b289ddf0595667f64f890a8cb418d8dd6cf ]]
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
cargo test --locked --test commands -- factory_shortcuts keymap org_contexts command_sequences --test-threads=1
cargo test --locked --test configuration keymaps -- --test-threads=1
cargo test --locked --test modal keymaps -- --test-threads=1
cargo build --locked --bin rev --example factory-shortcuts-acceptance
target=$(cargo metadata --locked --no-deps --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
[[ -n $target ]] || { echo 'missing Cargo target directory' >&2; exit 1; }
fixture=$(mktemp -d -t rev-factory-shortcuts-XXXXXXXX)
trap 'rm -rf -- "$fixture"' EXIT
mkdir -p "$fixture/native/home" "$fixture/native/config"
isolation=(env HOME="$fixture/native/home" XDG_CONFIG_HOME="$fixture/native/config"
    XDG_DATA_HOME="$fixture/native/data" XDG_STATE_HOME="$fixture/native/state"
    XDG_CACHE_HOME="$fixture/native/cache" TERM=xterm-256color NO_COLOR=1)
timeout --kill-after=1 30 "${isolation[@]}" "$target/debug/examples/factory-shortcuts-acceptance" --native "$fixture/native"
for profile in non-modal modal; do
    timeout --kill-after=1 30 "${isolation[@]}" "$target/debug/examples/factory-shortcuts-acceptance" \
        --journey "$fixture/$profile" "$target/debug/rev" "$root/scripts/check-factory-shortcuts.sh" "$profile"
done
timeout --kill-after=1 30 "${isolation[@]}" "$target/debug/examples/factory-shortcuts-acceptance" \
    --git "$target/debug/rev" "$root/scripts/check-factory-shortcuts.sh"
printf 'PASS factory shortcuts: native effects/accepted maps and three Linux PTYs; owned fixtures cleaned on exit\n'
