#!/usr/bin/env bash
set -euo pipefail
umask 077

repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/rev-hook-tests.XXXXXXXX")
owner=''
cleanup() {
    if [[ -n "$owner" ]]; then
        : > "$scratch/release"
        wait "$owner" || true
    fi
    rm -rf -- "$scratch"
}
trap cleanup EXIT
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_PREFIX CARGO_TARGET_DIR
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_OPTIONAL_LOCKS=0
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
fixture="$scratch/repository space"
mkdir -p "$fixture"/src "$fixture"/sdk/src "$fixture"/examples "$fixture"/scripts "$fixture"/.githooks
cp -- "$repo/.githooks/pre-commit" "$fixture/.githooks/"
cp -- "$repo/scripts/check-quality.sh" "$repo/scripts/install-hooks.sh" "$fixture/scripts/"
cp -- "$repo/examples/source-check.rs" "$fixture/examples/"
cp -R -- "$repo/examples/source_check" "$fixture/examples/"
cp -- "$repo/clippy.toml" "$fixture/"
{
    printf '[workspace]\nmembers=["sdk"]\nresolver="3"\n'
    sed -n '/^\[workspace.lints.rust\]/,/^\[lints\]/{ /^\[lints\]/!p; }' "$repo/Cargo.toml"
    printf '[package]\nname="rev"\nversion="0.1.0"\nedition="2024"\n[lints]\nworkspace=true\n[dependencies]\n'
    # Use only existing parser dependencies and their pinned versions. The real
    # native driver/checker run here; no compiler or source-policy mock is used.
    sed -n '/^serde = /p; /^serde_json = /p; /^tree-sitter = /p; /^tree-sitter-rust = /p' "$repo/Cargo.toml"
} > "$fixture/Cargo.toml"
printf '[package]\nname="hook-sdk"\nversion="0.1.0"\nedition="2024"\n[lints]\nworkspace=true\n' > "$fixture/sdk/Cargo.toml"
printf '#![forbid(unsafe_code)]\npub fn sdk() -> u8 {\n    1\n}\n' > "$fixture/sdk/src/lib.rs"
printf '# Hook fixture\n' > "$fixture/README.md"
printf '/target/\n' > "$fixture/.gitignore"
cd -- "$fixture"

source_text() { printf '#![forbid(unsafe_code)]\npub fn answer() -> u8 {\n    %s\n}\n' "$1" > src/lib.rs; }
source_text 1
git -c init.defaultBranch=dev init --quiet
git config user.name 'Hook Fixture'
git config user.email 'fixture@example.invalid'
git config commit.gpgsign false
cargo generate-lockfile --offline --quiet
git add .
git commit --quiet -m 'test: seed owned hook fixture'
bash scripts/install-hooks.sh > "$scratch/install.log"
[[ $(git config --local --get core.hooksPath) == .githooks ]]
mkdir target
printf 'ignored build data\n' > target/untouched
printf 'untracked user data\n' > untracked.txt
index=$(git rev-parse --path-format=absolute --git-path index)
cache="$(git rev-parse --absolute-git-dir)/rev-commit-check"

fingerprint() {
    sha256sum -- "$index" target/untouched untracked.txt
    # Even observations of restored file metadata must not refresh the index
    # being measured. Keep the fixture's own Git queries on a separate copy.
    cp -- "$index" "$scratch/observation-index"
    GIT_INDEX_FILE="$scratch/observation-index" git ls-files --stage -z
    GIT_INDEX_FILE="$scratch/observation-index" git diff --no-ext-diff --binary
    GIT_INDEX_FILE="$scratch/observation-index" git status --porcelain=v1 -z --untracked-files=all
}

checked() {
    local name=$1 expected=$2 pattern=$3 before after status=0
    fingerprint > "$scratch/before"
    before=$(sha256sum < "$scratch/before")
    bash "$repo/.githooks/pre-commit" > "$scratch/check.log" 2>&1 || status=$?
    fingerprint > "$scratch/after"
    after=$(sha256sum < "$scratch/after")
    if [[ "$before" != "$after" ]] || { [[ "$expected" == pass ]] && [[ $status != 0 ]]; } || { [[ "$expected" == fail ]] && [[ $status == 0 ]]; }; then
        cat "$scratch/check.log" >&2
        tr '\0' '\n' < "$scratch/before" > "$scratch/before.txt"
        tr '\0' '\n' < "$scratch/after" > "$scratch/after.txt"
        diff -u -- "$scratch/before.txt" "$scratch/after.txt" >&2 || true
        if [[ -f "$cache/run/original" ]]; then sha256sum -- "$cache/run/original" >&2; fi
        printf 'Hook fixture failed: %s (status %s, preserved=%s)\n' "$name" "$status" "$([[ "$before" == "$after" ]] && printf yes || printf no)" >&2
        exit 1
    fi
    if ! grep -Eq -- "$pattern" "$scratch/check.log"; then
        cat "$scratch/check.log" >&2
        printf 'Missing expected diagnostic for %s: %s\n' "$name" "$pattern" >&2
        exit 1
    fi
    printf 'Hook fixture passed: %s\n' "$name"
}

restore() { git restore --source=HEAD --staged --worktree .; }

# The hook must compile staged source in both directions of partial staging.
source_text 2
git add src/lib.rs
source_text 'invalid Rust !'
start=$SECONDS
checked 'valid index with unstaged error (cold)' pass 'Native source policy: passed'
printf 'Hook cold elapsed: %ss\n' "$((SECONDS - start))"
source_text missing
git add src/lib.rs
source_text 1
checked 'invalid index with unstaged repair' fail 'cannot find value'
restore
source_text 2
git add src/lib.rs
start=$SECONDS
checked 'warm complete workspace' pass 'Clippy \(full workspace\): passed'
printf 'Hook warm elapsed: %ss\n' "$((SECONDS - start))"
[[ -d "$cache/target" && -d "$cache/source" ]]
timestamp=$(stat -c %y -- "$cache/source/src/lib.rs")
checked 'unchanged snapshot preserves source timestamp' pass 'Native source policy: passed'
[[ $(stat -c %y -- "$cache/source/src/lib.rs") == "$timestamp" ]]

# A real commit through the installed hook must leave an unstaged edit intact.
source_text 3
git add src/lib.rs
source_text 4
git commit --quiet -m 'test: commit exact staged fixture' > "$scratch/commit.log" 2>&1
git show HEAD:src/lib.rs | grep -Eq '    3'
grep -Eq '    4' src/lib.rs
restore

# Docs-only checks must not invoke Cargo or replace the previous source export.
exported=$(stat -c %i -- "$cache/source")
printf '\nA documentation edit.\n' >> README.md
git add README.md
checked 'docs-only fast path' pass 'docs/data-only'
[[ $(stat -c %i -- "$cache/source") == "$exported" ]]
restore
printf 'Trailing whitespace. \n' >> README.md
git add README.md
checked 'docs-only whitespace failure' fail 'trailing whitespace'
restore

mv .githooks/pre-commit "$scratch/saved-hook"
ln -s -- "$fixture/README.md" .githooks/pre-commit
checked 'symlinked working hook' fail 'unsafe hook path'
rm .githooks/pre-commit
mv "$scratch/saved-hook" .githooks/pre-commit

# The native checker must see a six-parameter violation even when Rust compiles.
printf '#![forbid(unsafe_code)]\npub fn many(_a: u8, _b: u8, _c: u8, _d: u8, _e: u8, _f: u8) {}\n' > src/lib.rs
rustfmt --edition 2024 src/lib.rs
git add src/lib.rs
checked 'native source bounds' fail '6 parameters exceeds 5'
restore

# An SDK diagnostic cannot be bypassed by an editor-only source change.
printf '#![forbid(unsafe_code)]\npub fn sdk() -> bool {\n    1 == 1\n}\n' > sdk/src/lib.rs
git add sdk/src/lib.rs
source_text 5
git add src/lib.rs
checked 'complete member coverage' fail 'equal expressions'
restore

printf 'not valid TOML\n' > Cargo.toml
git add Cargo.toml
git show HEAD:Cargo.toml > Cargo.toml
checked 'staged manifest with unstaged repair' fail 'expected'
restore

printf '#![forbid(unsafe_code)]\npub fn leak(value: String) {\n    std::mem::forget(value);\n}\n' > src/lib.rs
git add src/lib.rs
checked 'disallowed ownership escape' fail 'disallowed method'
restore

printf '#![forbid(unsafe_code)]\n#[expect(clippy::redundant_clone, reason = "Deliberately stale fixture expectation.")]\npub fn clean() {}\n' > src/lib.rs
rustfmt --edition 2024 src/lib.rs
git add src/lib.rs
checked 'unfulfilled local expectation' fail 'unfulfilled'
restore

printf 'disallowed-methods = ["std::mem::drop"]\n' > clippy.toml
printf '#![forbid(unsafe_code)]\npub fn release(value: String) {\n    std::mem::drop(value);\n}\n' > src/lib.rs
git add clippy.toml src/lib.rs
git show HEAD:clippy.toml > clippy.toml
checked 'staged lint policy with unstaged repair' fail 'disallowed method'
restore

printf '#!/usr/bin/env bash\nprintf "staged policy executed\\n"\nexit 37\n' > scripts/check-quality.sh
git add scripts/check-quality.sh
git show HEAD:scripts/check-quality.sh > scripts/check-quality.sh
checked 'staged tooling with unstaged repair' fail 'staged policy executed'
restore
printf '#!/usr/bin/env bash\nprintf "changed export\\n" > src/lib.rs\n' > scripts/check-quality.sh
git add scripts/check-quality.sh
checked 'changed exported bytes' fail 'check changed exported source'
restore
printf '#!/usr/bin/env bash\nchmod +x src/lib.rs\n' > scripts/check-quality.sh
git add scripts/check-quality.sh
checked 'changed exported mode' fail 'check changed exported mode'
restore

source_text missing
git add src/lib.rs
printf 'src/lib.rs filter=repair\n' > .gitattributes
git config filter.repair.smudge 'sed s/missing/3/'
git add .gitattributes
checked 'checkout smudge cannot repair staged source' fail 'cannot find value'
restore
git config --unset filter.repair.smudge
printf 'pub fn many(_a: u8, _b: u8, _c: u8, _d: u8, _e: u8, _f: u8) {}\n' > src/excluded.rs
printf 'src/excluded.rs export-ignore\n' > .gitattributes
git add src/excluded.rs .gitattributes
checked 'export-ignore cannot hide a source violation' fail '6 parameters exceeds 5'
restore

# Includes old/new paths for renames and deletes instead of dropping them from
# classification. The marker is a valid non-root Rust source in the workspace.
printf 'pub fn marker() {}\n' > src/marker.rs
git add src/marker.rs
git commit --quiet -m 'test: add owned rename fixture' > "$scratch/rename.log" 2>&1
git mv src/marker.rs src/renamed.rs
checked 'staged Rust rename' pass 'Native source policy: passed'
restore
git rm --quiet src/marker.rs
checked 'staged Rust deletion' pass 'Native source policy: passed'
restore

blob=$(git rev-parse HEAD:src/lib.rs)
git update-index --force-remove src/lib.rs
printf '100644 %s 1\tsrc/lib.rs\n100644 %s 2\tsrc/lib.rs\n100644 %s 3\tsrc/lib.rs\n' "$blob" "$blob" "$blob" | git update-index --index-info
checked 'unmerged index' fail 'resolve index conflicts'
restore

# Controlled pause fixtures test ownership and change-during-check, after the
# real compiler/source-policy cases above. They do not replace lint evidence.
gated_driver() {
    {
        printf '#!/usr/bin/env bash\nprintf ready > %q\n' "$scratch/ready"
        printf 'for ((attempt=0; attempt<300; attempt++)); do\n    [[ ! -f %q ]] || exit 0\n    sleep 0.05\ndone\nexit 1\n' "$scratch/release"
    } > scripts/check-quality.sh
    git add scripts/check-quality.sh
    bash "$repo/.githooks/pre-commit" > "$scratch/owner.log" 2>&1 &
    owner=$!
    for ((attempt=0; attempt<200; attempt++)); do
        [[ ! -f "$scratch/ready" ]] || return 0
        sleep 0.05
    done
    printf 'Timed out waiting for owned hook fixture\n' >&2
    exit 1
}

release_driver() {
    local status=0
    : > "$scratch/release"
    wait "$owner" || status=$?
    owner=''
    return "$status"
}

gated_driver
checked 'concurrent check refuses owned cache' fail 'another commit check owns'
release_driver
rm -- "$scratch/ready" "$scratch/release"
restore

gated_driver
printf '\nNew staged candidate.\n' >> README.md
git add README.md
before=$(fingerprint | sha256sum)
status=0
release_driver || status=$?
[[ $status != 0 && "$before" == "$(fingerprint | sha256sum)" ]]
grep -Eq 'index changed during validation' "$scratch/owner.log"
printf 'Hook fixture passed: changed index is rejected without rollback\n'
rm -- "$scratch/ready" "$scratch/release"
restore

# Symlink and owner checks must fail before touching their destination.
mv -- "$cache/source" "$cache/saved-source"
ln -s -- "$scratch" "$cache/source"
checked 'symlinked cache source' fail 'unsafe cache directory'
rm -- "$cache/source"
mv -- "$cache/saved-source" "$cache/source"
printf 'other owner\n' > "$cache/owner"
checked 'foreign cache owner' fail 'unrecognized cache owner'
git rev-parse --absolute-git-dir > "$cache/owner"
printf 'outside data\n' > "$scratch/outside"
ln -s -- "$scratch/outside" src/link.rs
git add src/link.rs
checked 'staged source symlink' fail 'symlinked check input'
restore
[[ $(cat "$scratch/outside") == 'outside data' ]]

# Install refusal and linked-worktree lookup exercise real local Git config.
git config --local core.hooksPath custom-hooks
if bash scripts/install-hooks.sh > "$scratch/conflict.log" 2>&1; then exit 1; fi
[[ $(git config --local --get core.hooksPath) == custom-hooks ]]
grep -Eq 'Preserving conflicting' "$scratch/conflict.log"
git config --local core.hooksPath .githooks
# Existing default hooks, including symlinks, are also installation conflicts.
git config --local --unset core.hooksPath
ln -s -- "$scratch/outside" "$(git rev-parse --absolute-git-dir)/hooks/pre-commit"
if bash scripts/install-hooks.sh > "$scratch/default-conflict.log" 2>&1; then exit 1; fi
grep -Eq 'Preserving existing hooks' "$scratch/default-conflict.log"
rm -- "$(git rev-parse --absolute-git-dir)/hooks/pre-commit"
bash scripts/install-hooks.sh > "$scratch/reinstall.log"
linked="$scratch/linked worktree"
git worktree add --quiet -b linked-fixture "$linked" HEAD
cd -- "$linked"
printf '/target/\n' > .gitignore
source_text 6
git add src/lib.rs
git commit --quiet -m 'test: linked worktree uses its own staged checks' > "$scratch/linked.log" 2>&1
linked_cache="$(git rev-parse --absolute-git-dir)/rev-commit-check"
[[ "$linked_cache" != "$cache" && -d "$linked_cache/target" && -d "$cache/target" ]]
git show HEAD:src/lib.rs | grep -Eq '    6'
printf 'Hook fixture passed: installed linked-worktree hook and separate cache\n'

cd -- "$fixture"
gated_driver
cd -- "$linked"
source_text 7
git add src/lib.rs
git commit --quiet -m 'test: independent cache during another worktree check' > "$scratch/parallel-linked.log" 2>&1
git show HEAD:src/lib.rs | grep -Eq '    7'
cd -- "$fixture"
release_driver
rm -- "$scratch/ready" "$scratch/release"
restore
printf 'Hook fixture passed: independent linked-worktree concurrent owner\n'
printf 'Hook acceptance passed.\n'
