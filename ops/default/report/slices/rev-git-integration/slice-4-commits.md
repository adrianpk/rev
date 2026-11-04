# Slice 4: Native Commit Composition

Status: reviewing
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-04-commits`
PR: pending

## Purpose

Deliver native normal, amend and fixup composition under GIT06 and GIT16-GIT19.
The [approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md) define T4.1-T4.3.

## Delivered Behavior

T4.1 delivers normal composition:

- One c in Status opens a native transient subject/body buffer. Following c is
  literal in both profiles; Return inserts a newline. Ordinary native motions,
  deletion, undo, selection, transfer and text search use the existing owners.
- C-Enter or C-c C-c explicitly submits; C-g or C-c C-k returns to Status and
  retains the session draft. Source publication and workspace changes are
  protected while composition owns the view.
- Submission uses the checked staged delta and exact HEAD, with full semantic
  index snapshots rechecked before Git. No source save or staging is implicit.
  Installed Git supplies identity, hooks and signing policy.
- Success requires actual commit parent, message bytes and tree to match the
  accepted request, with unchanged index entries. Git's verbatim message input
  preserves the accepted final-newline state. Changing-child diagnostics survive
  refresh; failed hooks retain the draft, and post-effect failures remain
  indeterminate without automatic replay.
- Returning preserves the retained source view, modified text, point, split
  identities and prior tree/terminal focus.

T4.2 delivers amend and fixup composition:

- `git-commit-amend` reads the exact observed HEAD, parent set and existing
  message before native editing. Submission uses Git's amend semantics while
  preserving its author identity and current committer/signing/hook policy.
- `git-commit-fixup` collects one revision and resolves it once to a full
  immutable commit. Native composition prepares the standard `fixup!` subject
  from Git's actual target subject; edits retain that target subject and may
  supply a body. The resulting commit is an ordinary child of checked HEAD.
- The first explicit submission for either prepared mode opens a read-only
  exact review. Complete target IDs, checked HEAD, accepted message byte count
  and final-newline state remain inspectable with wrapped vertical scrolling.
  Plain Return cannot accept; C-Enter or C-c C-c accepts, q or C-g retains the
  cancelled draft. Prompt cancellation preserves its outer composition owner.
- Worker preflight repeats checked index/HEAD observations after preparation.
  Moved HEAD, changed index, missing target and unsupported message metadata
  cannot silently retarget a submission. Full object formats remain intact.
- Fixup never starts autosquash or rewrites existing history. Installed Git
  rejects `--fixup` together with `--file`; native sealed message input instead
  supplies the standard complete fixup message. An independent installed-Git
  `--fixup` fixture proves the actual resulting message bytes.

T4.3 delivers failure and ownership proofs:

- Both submit aliases and both cancellation aliases have native cases in both
  profiles. Prompt cancellation retains the outer message owner, and UTF-8
  byte-limit rejection preserves the buffer revision, history and draft.
- Native editing rejects oversized clipboard transfers before materializing
  strings or rectangle padding. The canonical edit owner retains final message
  and batch limits; ordinary source buffers keep their existing policy.
- Real terminal panel/pane fixtures retain the same live process and source
  split identities. Composer bytes never reach terminal input; normal commit
  submission or cancellation restores prior focus, text and point.
- Failed hooks expose their actual native Process diagnostic and retain the
  exact message for a separately explicit fresh submission. Configured signing
  is attempted and its failure is reported without disabling the policy.
- A selected changing child can be cancelled with bounded group cleanup and a
  retained draft. Zero-exit hooks that alter message or index remain
  indeterminate; possible effects are never replayed automatically.
- Full-index admission overflow and foreign observation owners reject before
  mutation. Real Git proves merge-parent preservation and a verified no-op
  amend that retains the same object ID.

## Implementation Notes

The Git worker shares existing capture, cancellation, process-group cleanup,
deadline and session admission. Normal composition immediately captures current
complete Status; worker preflight compares index-layer facts independently of
unstaged changes, then checks full staged entries before the sole mutator.
External Git clients remain external; postconditions detect changed authority
without claiming an exclusive cross-process transaction.

Messages admit at most 65536 bytes per repository context. One active native
composer reserves 8 MiB from the existing shared 32 MiB Git budget for text,
rope fragments, history, edit candidates and presentation. Its own history is
limited to 16 groups with a 128 KiB payload target; each edit admits at most 64
ranges and 16 points before candidate allocation. The buffer's byte bound is
checked by the canonical edit owner before publication. Ordinary source buffers
retain their existing policy. Cancelled/failed text has a separate bounded
session lease; verified success clears it.

Single text replacement and clipboard reads check the transient payload bound
before cloning input. Rectangle insertion checks each row and cumulative bytes
before extending retained strings, including generated padding and newlines;
its bounded draft also rejects excess distinct ranges before retaining them.

Read-only commit preparation shares the existing observation worker and deadline.
Its message/parent metadata is admitted against the same session budget before
retention. Prepared results are generation-bound; source view acquisition also
requires current observation authority. Submission requires the exact native
observation owner, preventing equal numeric IDs from another repository engine
from supplying authority. A no-op amend may retain the same object identity if
Git's actual parent/message/tree postconditions all hold.

## Contracts Added or Changed

- `git-commit`, `git-commit-amend`, `git-commit-fixup`, `git-commit-submit`,
  `git-commit-cancel` and the `keys.git_commit` editor overlay are implemented
  and documented together. The Git navigation map also admits the two explicit
  submission aliases while exact review owns input.
- The complete Git index evidence uses existing configured output/path-count
  limits; incomplete or unsupported entries reject before a changing child.
- No new dependency, persistent application state or process framework.

## Files of Interest

- `src/git/commit.rs`, `src/git/work/commit.rs`: message ownership, preconditions
  and actual Git object/index proof.
- `src/session/git/commit.rs`: retained native view and explicit submit/cancel.
- `src/input/keymap.rs`, `src/buffer/edit.rs`: native input context and bounded
  draft editing through existing owners.
- `tests/git_integration/commits.rs`, `commits_presentation.rs`: real-Git and
  native profile proofs.
- `src/session/git/commit/`, `tests/git_integration/commits_modes.rs`: immutable
  preparation, explicit review and real amend/fixup graph/identity proofs.
- `tests/git_integration/commits_acceptance.rs`, `commits_effects.rs`: native
  terminal, draft and capacity cases, real hook/signer/cancellation evidence,
  semantic-effect mismatches and complete object authority.

## Validation

Development checks, before final exact-HEAD controller validation:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings` passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration commits:: -- --test-threads=1` passed ten tests: SHA-1/SHA-256 and unborn commits, exact message/tree/index, stale authority, hook failure, post-effect uncertainty, both profiles, all composition aliases, amend author/parents, immutable fixup target/message oracle and native exact review.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration commits::acceptance:: -- --test-threads=1` passed four native tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration commits::effects:: -- --test-threads=1` passed five real-Git effect/authority tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration commits::acceptance::bounded_native_editing -- --test-threads=1` passed after the transfer admission changes.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration commits::acceptance::terminal_source_retention -- --test-threads=1` passed with panel and pane placement.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration reference -- --test-threads=1` passed 11 tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --quiet --locked --example documentation-check` passed: 495 commands, 264 configuration entries and 76 reachable pages.
- A message proof initially expected Git to add a final newline; installed Git
  demonstrated verbatim preservation. Verification and assertions were
  corrected to require exact accepted bytes; the focused suite then passed.

The canonical PR Validation list and controller receipt provide the final
ordered results for its clean HEAD after the PR-reference follow-up commit.

## Risks and Follow-ups

Combined PTY/resource acceptance remains Slice 13; the full User Guide remains
Slice 14. External hook side effects remain Git/user policy; unverified effects
require inspection before a fresh request.
