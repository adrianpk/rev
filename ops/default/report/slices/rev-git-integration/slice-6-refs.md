# Slice 6: Branches and Both Tag Kinds

Status: delivered
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-06-refs`
PR: `#263`

## Purpose

Deliver GIT08 and its GIT17-GIT19 integration through the approved T6.1-T6.3
[plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

T6.1 adds exact ref inspection and local branch requests:

- b or y in Status opens native Refs in both profiles. Local branches,
  remote-tracking refs, symbolic refs, tags, configured remote names and
  available full upstream/tracking facts retain exact byte identities.
  Annotated tag objects and their peeled targets remain separate.
- HEAD stays explicitly symbolic, unborn or detached, including read-only
  bare-repository observations. Ref and remote count limits produce a visible
  truncated state that cannot authorize mutation. Remote URL values are not
  retained in public ref observations.
- n collects a new local name at the selected exact commit, S switches the
  selected local branch, R collects its replacement name and D requests local
  deletion. A scrollable native review shows the full target and names. Only
  explicit y accepts; Return is not acceptance, n or C-g cancels.
- Git validates names. Creation does not enable tracking implicitly. Existing
  names, stale refs/HEAD, current-branch deletion and unmerged deletion reject.
  Git retains its non-forced deletion and linked-worktree protections.
- Switch admission rejects incomplete or dirty index/worktree observations and
  modified repository-backed buffers. Source stamps are checked again at
  confirmation. Started changes refresh compatible Status and Refs together;
  source file observations use the existing automatic-or-ask reload owner.
- q returns from Refs to Status and the retained source layout. View exit
  retains started work. Failed observation leaves confirmed facts visible and
  stale; g can establish a fresh compatible observation.

T6.2 adds local lightweight and annotated tags:

- t / `git-tag-create` collects a name, an explicit kind with lightweight as
  the default, and one full exact object. Installed Git validates names,
  existing-name absence, object/type and applicable signing policy before review.
- Lightweight creation requires explicit y at a scrollable full-target review.
  Annotated creation uses the existing retained native message editor. Return
  edits text, C-Enter or C-c C-c opens exact review, and those aliases accept it.
  Both profiles retain literal text, source owners and native cancellation.
- Git supplies tagger identity/date and configured signing. Lightweight creation
  rejects `tag.gpgSign` rather than changing kind or disabling signing. Annotated
  signing honors both `tag.gpgSign` and `tag.forceSignAnnotated`. Unsigned
  verbatim annotations preserve entered bytes; signed review includes the final
  newline required to keep Git's signature delimiter recognizable, within the
  unchanged 65536-byte bound.
- Failed or cancelled annotation text remains session-only and can be recovered
  through an explicitly prepared annotated retry. Actual child diagnostics remain
  available after post-effect refresh. Native failure feedback includes safe
  retained diagnostic text.
- T / `git-tag-delete` reviews the local full ref, tag object and peeled target.
  Git's expected-old-object ref update rejects a moved deletion target. Neither
  tag kind nor deletion invokes publication or changes HEAD/index.

T6.3 completes focused acceptance and native inspection:

- Real linked-worktree fixtures retain Git's switch/deletion protection.
  Invalid/existing branch names leave refs unchanged before any changing child.
- A fixture moves a tag after preflight but before Git's expected-object lock.
  The actual deletion child fails once with retained lock diagnostics; its
  replacement ref survives and the request is never replayed by name.
- Hostile mirror/follow-tag/push settings do not publish either local tag kind.
  The local bare publication fixture remains empty after each creation and
  deletion; HEAD and index remain unchanged.
- Both profiles retain modified source while composing tags. One native route
  verifies signed annotation review and actual Git signature verification;
  the other preserves unsigned verbatim bytes. Signed review exposes its
  accepted final newline before submission.
- Tab shows scrollable full ref/object/type/symbolic/peeled/upstream facts.
  Configured remotes remain reachable after long ref lists without exposing
  URL values. Returning from review invalidates old painted list coordinates.
- Ref operations queue an ordinary owned read to refresh retained History.
  Unchanged HEAD retains pinned admitted pages; changed HEAD starts current
  ancestry. A branch switch also reloads clean visiting source through the
  existing file policy. Background history does not replace ref-failure feedback.

## Implementation Notes

The existing repository/process owner runs structured installed-Git argv,
serializes shared storage for mutation and retains changing-child diagnostics
across refresh and verification. No command reparses a displayed ref name or
abbreviated object. A complete captured ref set and HEAD are checked before
execution; index/worktree cleanliness is rechecked for switch. A postcondition
checks the accepted ref/object and HEAD relation, with uncertain effects
remaining explicit instead of replaying the child.

Ref metadata fields retain the 4096-byte bound. Raw output, parsed candidates,
source stamps, review text and public observations use the existing shared
32 MiB admission. Counts use `git.max_refs` and `git.max_remotes`; observation
and changing-action deadlines remain the configured existing limits.

## Contracts Added or Changed

- `git-refs`, `git-branch-create`, `git-branch-switch`, `git-branch-rename` and
  `git-branch-delete` have native handlers, catalog/reference entries and
  both-profile contextual bindings.
- `keys.git_refs` has an independent map and complete active sample. n owns
  branch creation; navigation uses Up/Down or C-p/C-n.
- Ref refresh publishes Status and Refs with one compatible observation;
  incompatible old targets cannot supply changing authority.
- `git-tag-create` and `git-tag-delete` have native handlers, complete command
  inventory/reference entries and active sample bindings. Annotated text shares
  the existing `keys.git_commit` owner; tag review keeps explicit submit aliases.
- No dependency or persistent repository store was added.

## Files of Interest

- `src/git/refs.rs`, `src/git/work/refs.rs`, their tag modules: exact observations,
  bounded reads, branch/tag preconditions, signing, execution and postconditions.
- `src/session/git/refs.rs`, its tag module and the shared native message owner:
  exact selection, full detail, review, source protection and history refresh.
- `src/input/keymap/defaults.rs`, `docs/reference/config.toml`: actual factory
  bindings and complete configuration sample.
- `tests/git_integration/refs.rs`, `refs_presentation.rs`: real object/ref effects,
  stale and dirty-state rejection, and both-profile native interaction.

## Validation

T6.1 development validation passed. Final exact-HEAD validation will
run through the controller after T6.2-T6.3 and PR-reference follow-up.
The focused ref fixtures use serial native execution, isolated installed Git,
neutral identity/configuration and the existing unchanged five-second fixture
wait. The first six tests passed in 17.70 seconds after the new worktree's
initial build. A missing process-presentation enum arm found by the first
compile was corrected before that successful test run.

Successful development commands:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs:: -- --test-threads=1` passed all six tests in 17.15 seconds after scrollable review integration.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs::presentation::native_branches -- --test-threads=1` passed after adding full SHA-256 review and review-scrolling assertions in both profiles.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1` passed 21 tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` passed: 508 commands, 267 configuration entries, 76 reachable pages and 485 links.
- `git diff --check`

A focused interleaving fixture then reproduced a HEAD move after the Status
HEAD check but before ref collection: the old implementation published Status
for main together with Refs for other. Ref collection now checks both the full
HEAD ref and object against its Status owner before admission, in addition to
the existing before/after ref-read check. The fixture performs a real checkout
through isolated installed Git and retains the unchanged five-second wait.
Native branch review also invalidates the preceding painted list so that its
coordinates cannot authorize selection after confirmation closes.
The complete focused refs selection passed all seven cases in 29.52 seconds
after the HEAD compatibility correction. Strict focused Clippy passed again.
No assertion, deadline or resource limit was weakened.

T6.2 development checks passed the complete 13-case refs selection, including
both-profile creation/review/cancellation and real tag objects, signing-policy
conflicts, actual signer failure and retained drafts. Strict focused Clippy and
documentation checks also passed (510 commands, 267 configuration entries).
The first compile exposed visibility/type inference mistakes, corrected before
successful tests. An early fixture entered its launch repository instead of the
owned fixture; it now enters through its owned visiting file. The unsigned
verbatim check was corrected to preserve actual Git bytes without an invented
final newline. Existing fixture deadlines and acceptance assertions remain intact.

The extended signing fixture then found that a signed verbatim message lacking
its final newline attached Git's signature marker to the text. Git could not
recognize that signature. Exact native review and the checked request now use
one shared final-newline rule when signing, within the existing byte ceiling.
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs::tags::signed_annotation -- --test-threads=1` passed after correction: both signing policies verify through installed Git, including non-newline-terminated input, while the lightweight tag still points directly to its commit.

T6.3 development command:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs:: -- --test-threads=1` passed all 17 tests in 47.35 seconds. The final candidate also checks the shared 90-test Git baseline, command/configuration references, documentation, strict Clippy and whitespace through the controller.

T6.2's pre-commit formatter detected one late unformatted review line, but the
shell sequence still committed it. T6.3 formats that line; subsequent delivery
commands use checked subprocess results and the final controller receipt must
establish the actual candidate's formatter result.

Controller receipt `QSHMVEJLZ3ZZKQNKLG7SBKTXAU` at
`ad28099225bdf6102d0cacb87525577a288815d8` passed formatter, then stopped
at Clippy on a sibling-module visibility error in background history feedback.
The existing refs failure flag is now visible only within its parent Git session
module. Behavioral checks had not started; the corrected head requires a fresh
ordered receipt. No behavior assertion or resource limit changed.

Final ordered contract, executed through the controller after this PR-reference
follow-up commit:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration -- --skip refs:: --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The [canonical PR #263](https://forge.adrianpk.com/adrianpk/rev/pulls/263)
Validation list and controller receipt record the exact clean-HEAD results.
Receipt `P6AQRLEPYKAAT3MWRQKSXFWWQG` passed all seven ordered checks at
`bef97e57c7953c9ab463c59b52b15ebcea22ac06`: 107 Git tests, 21 reference
tests and 510-command documentation coverage, plus formatter, Clippy and
whitespace checks.

## Risks and Follow-ups

PR #263 merged through controller-owned Rebase + Fast-forward into dev at
`bef97e57c7953c9ab463c59b52b15ebcea22ac06`. Slice 6 of 14 completed.
Combined native PTY/resource
acceptance remains Slice 13; the complete User Guide remains Slice 14.
Revalidation detects changed Git facts and does not claim an exclusive
transaction against external Git clients.
