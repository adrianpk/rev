# Slice 12: Prepared Repository Command Families

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-12-commands`
PR: pending

## Purpose

Deliver GIT14-GIT15 and their GIT16-GIT20 contracts through T12.1-T12.3 in the
[approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

- T12.1 first routes: Status C C, C s, C b and C m prepare regular, depth-one,
  bare and mirror clones; I I prepares init. Named clone/init commands also
  work from ordinary editing and chosen directories without a Git context.
- The existing bounded native text owner edits a JSON array of literal Git
  arguments. Empty source/destination parameters must be filled. The selected
  family and explicit option/parameter structure remain checked.
- C-Enter or C-c C-c first captures read-only preparation and opens exact
  executable/mode/cwd/argv/destination review. A separate submission executes
  only that canonical plan. Plain Return does not accept review; q/C-g cancels
  it. Ordinary source views and editing state return after cancellation.
- Clone/init require an absent explicit destination and checked canonical
  ordinary parent. Execution rechecks directory/parent identity, source HEAD,
  index, working bytes and untracked hashes. Stale plans refuse before launch.
- Successful creation verifies the actual repository identity and object
  format, full existing HEAD, unborn init, shallow state or bare/mirror kind.
  Empty regular/bare/mirror remotes retain an actual unborn identity rather
  than inventing a commit. Failures retain uncertainty and never delete a
  partially created destination, retry or claim rollback.
- i t and i p visit actual .gitignore and common Git info/exclude files through
  their ordinary native owners, preserving existing unsaved text and inserting
  no pattern or hidden save.
- T12.1 submodule/subtree routes and T12.2-T12.3 remain pending. No PR is claimed.

## Implementation Notes

Prepared commands use the existing installed-Git worker, process records,
configuration, cancellation, stream limits and retained cleanup owner. One
canonical plan and result stay under the existing Git admission budget.
Preparation waits for active Git work so unknown destination/common identity
cannot race another owned request. Compatible source observations refresh after
possible effects; hidden views do not release the process owner.

The native draft reuses the existing retained source-view and message-buffer
facilities. Commit/tag dispatch remains separate from argument interpretation.
JSON arguments are literal OS arguments and are not passed through a shell.
The existing common saved-state helper also supports an unborn source: exact
index entries plus the working diff from that index replace a nonexistent HEAD
baseline, with the same saved untracked hashes.

Bounds remain 32 MiB editor-wide Git admission, 1 MiB scratch, existing process
output/deadline/cleanup ceilings and five-second fixture waits. Prepared input
admits at most 64 arguments, 4096 bytes each, 64 KiB total and an absolute cwd
within 4096 bytes. Native draft text remains bounded to 64 KiB. Transport output
is withheld from public records; diagnostic display keeps existing redaction.
Init obeys the configured operation timeout; clones obey the transport timeout.
No runtime/development dependency or validation prerequisite was added.

## Contracts Added or Changed

- Seven working commands: git-clone, git-clone-shallow, git-clone-bare,
  git-clone-mirror, git-init, git-ignore and git-ignore-exclude.
- Status has 48 active defaults. The existing keys.git_commit context also
  supplies ordinary editing and explicit submission for prepared argument text.
- Live catalogs contain 568 commands and 271 configuration entries.

## Files of Interest

- `src/git/families.rs`, `src/git/work/families.rs`: typed plans, literal input,
  exact preconditions, installed capability checks and creation verification.
- `src/session/git/families.rs`, `src/session/git/commit.rs`: native draft,
  two-step submission, exact review and original source restoration.
- `src/git/work/repository.rs`: shared saved-state evidence for unborn sources.
- `tests/git_integration/families.rs`, `families_presentation.rs`: real local
  Git objects/files and both-profile native input, review and visiting owners.

## Validation

Development initially passed six real Git/native cases in 10.29 seconds.
Additional native outside-repository and unborn-source cases passed with the
combined eight-case run in 11.54 seconds. The empty-clone case passed separately
in 0.63 seconds. The final combined nine-case run passed in 16.41 seconds. Both object formats, raw destinations, shallow/bare/mirror
facts, literal shell-looking arguments, stale destination/parent/source/index/
HEAD, ignore-file visiting identity and explicit review are covered.

The unborn-source regression first failed because preflight diffed nonexistent
HEAD. Its correction retains exact staged objects, uses the working diff from
the index only for an unborn source and preserves saved untracked hashes. The
unchanged assertion proves changed saved bytes refuse before creating the
destination. Initial compile errors were a duplicate helper name and incomplete
new-operation/native dispatch wiring; these were corrected before validation.
No observable assertion, deadline or resource limit was weakened.

Three existing native commit cases passed in 8.77 seconds, two native tag cases
in 10.48 seconds and the SHA-1/SHA-256 stash saved-byte round trip in 15.37
seconds. All 21 command/configuration reference cases passed in 14.53 seconds.
Documentation passed at 568 commands, 271 configuration entries, 76 pages and
485 links. Formatter and strict focused Clippy passed during development. After the native
input-bound check, all three native family cases passed in 4.61 seconds. A real
Git wrapper delaying only init/clone verified the separate configured deadlines
in 2.63 seconds: init timed out with possible effects, and clone completed under
its transport limit. The wrapper delegates all Git output and repository work to
installed Git; fixture waits and production limits remain unchanged.

Successful development commands:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration families:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration families::empty_clone -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration commits::presentation:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs::tags::presentation:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration repository::saved_roundtrip -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration families::presentation:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration families::configured_deadlines -- --test-threads=1`
- `git diff --check`

These are focused development results. Final exact-HEAD controller validation
will follow complete T12.1-T12.3 implementation and the canonical PR-reference
follow-up commit. No full gate or final receipt is claimed.

## Risks and Follow-ups

Complete the remaining approved submodule/subtree, notes/patch/sparse and direct
Git/separate shell routes in this same slice. Combined PTY/resource acceptance
and learning-path guidance remain in their recorded later slices. The controller
owns integration.
