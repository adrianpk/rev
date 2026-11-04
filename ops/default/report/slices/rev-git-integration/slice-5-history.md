# Slice 5: History Ranges Reflog and Blame

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-05-history`
PR: pending

## Purpose

Deliver GIT07, the read-only reflog/blame portion of GIT13 and their native
GIT17-GIT18 integration. The [approved plan](../../../plan/rev-git-integration.md)
and [tracker](../../../tracker/rev-git-integration.md) define T5.1-T5.3.

## Delivered Behavior

T5.1 delivers ancestry and revision inspection:

- h or l l in Status opens bounded History. Detached or bare entry collects
  an explicit root, resolved once by installed Git to a full commit identity.
  Unborn History has an explicit empty state.
- f continues topological ancestry from the pinned full root and admitted count.
  Each page contains at most `git.history_page_size` revisions; at most
  `git.max_history_pages` pages remain retained. Completion and reached page
  limits remain explicit. Refresh verifies HEAD and root existence, retaining
  the same immutable pages, continuation and compatible selected object.
  Revalidation checks the complete admitted ancestry prefix before and after
  reading; changed traversal, including a moved shallow boundary, rejects.
- Tab inspects full revision metadata, complete parent identities, exact changed
  paths and bounded read-only diffs. Historical renames retain both exact paths.
  `git-show` in detail selects a complete parent ID from the revision's own
  parent set, including either parent of a merge. Root commits compare to the
  empty tree. Truncated path or patch detail is labeled explicitly.
- q returns from revision detail to History, then Status and the original live
  editor context. Both profiles share native list keys and named commands.
  Pointer selection requires the current observation and painted viewport.
- A prepares amend of checked HEAD; F prepares fixup of the selected complete
  History revision without a revision-text prompt. Existing native composition
  and explicit exact review own message editing and submission.
- Missing objects, changed HEAD and stale native owners reject without replacing
  confirmed history. Inspection never checks out a file or changes source
  buffers, index or HEAD. Native source text, point and split identities survive.

T5.2 delivers ranges, attribution and historical source:

- * sets one full History range base; Return compares that base and the selected
  endpoint in their explicit order. a/U appends/removes independent ordered
  revision marks. Base changes retain order; sequence changes retain the base.
  Duplicate, excess and incompatible marks reject without partial updates.
- l r opens bounded HEAD Reflog. Entry selectors remain volatile display
  metadata; revision inspection uses admitted full commit objects. Refresh
  retains a compatible object/subject selection when selectors move.
- B b or B B attributes the selected or visiting repository source at checked
  HEAD. Each line retains its originating object, exact filename and source
  location, including historical renames and non-UTF-8/quoted path bytes.
- Return in inspection opens exact parents, revisions or historical file bytes;
  v inspects the selected reflog/blame revision. Deleted paths read their exact
  old-side object. Historical blobs remain typed read-only resources, with
  escaped binary bytes, explicit truncation and wrapped source display.
- `git-visit-file` visits the current canonical source through the existing
  file owner. Modified buffers remain intact; historical path absence is
  explicit. Blame keeps its current path separately from its historical origin.
- `keys.git_inspect` gives both profiles independent inspection navigation.
  q returns to the correct History, Reflog or Blame origin without checkout,
  saving, staging or creating a writable historical buffer.

## Implementation Notes

The existing repository worker owns observation deadlines, cancellation,
process groups, aggregate output and diagnostics. History uses installed Git's
explicit topological order, with a complete immutable root and ancestry-count
continuation. It never pages a moving ref or substitutes a subject or displayed
abbreviation for an object argument. Read-only object and diff commands disable
replacement objects, external diff helpers and text conversion.

Immutable pages retain separate admission leases and are shared across refresh
and continuation. New pages, metadata, exact paths, parser candidates, detail
line indexes and presentation participate in the existing 32 MiB Git budget.
Metadata/control fields remain limited to 4096 bytes; parser scratch, configured
output/detail/path bounds and the shared worker deadline remain enforced.

Historical detail has a separate typed owner from current Status comparisons.
It cannot become an index mutation target, writable source buffer or save
transaction. Parent selection accepts only a complete actual parent of the
admitted revision. A current native observation owner is required at admission
and publication; current repository identity and HEAD are checked by Git before
and after inspection. A changed HEAD requires Status refresh and explicit
History reentry.

History input uses `keys.git_history`; Status keeps `keys.git` and composition
keeps `keys.git_commit`. Entering review from History transfers to the existing
commit review map so both explicit submission aliases remain available.

## Contracts Added or Changed

- `git-log`, `git-show` and `git-log-next-page` have working native catalog,
  command-interface, keymap and documentation paths.
- `keys.git_history` has a complete active sample and schema/default reference.
- Historical observations expose full format-aware commit IDs, metadata,
  parents, exact ancestry continuation and independently admitted pages.
- No dependency, persistent application state or process framework was added.
- `git-show-range`, `git-sequence-mark`, `git-sequence-unmark`, `git-reflog`
  and `git-blame` have working typed targets, native keys and complete reference
  entries. `keys.git_inspect` is included in the complete sample and schema.

## Files of Interest

- `src/git/history.rs`, `src/git/work/history.rs`: typed history and revision
  results, installed-Git reads, exact parents, finite parsing and admission.
- `src/session/git/history.rs`: retained selection, native navigation and
  read-only revision presentation.
- `src/input/keymap.rs`, `src/input/keymap/defaults.rs`: independent history map
  and complete default bindings.
- `tests/git_integration/history.rs`, `history_presentation.rs`: actual Git
  object, ancestry, rename, immutability and both-profile native proofs.

## Validation

Development checks for T5.1:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration history:: -- --test-threads=1` passed six tests, including changed ancestry rejection, native fixup review acceptance and failed-refresh target protection.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` passed: 498 commands, 265 configuration entries and 76 reachable pages.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings` passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1` passed 21 tests.
- `git diff --check` passed.

The shallow-boundary fixture initially queried the last completed record instead
of its returned operation ID. It now requires the exact operation's retained
ancestry-change diagnostic. The focused case and complete six-test history batch
then passed; the rejection and retained-page assertions remain unchanged.

Development checks for T5.2:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration history::inspection:: -- --test-threads=1` passed five real-Git/native tests in 60 seconds, including both-profile range/mark roles and immutable source visits.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings` passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` passed: 503 commands, 266 configuration entries and 76 reachable pages.
- `git diff --check` passed.

T5.3 adds focused limit/failure cases. Its 257-commit mark fixture uses one
installed-Git fast-import process to bound setup cost while preserving the real
256-mark application boundary. The selected slice runner remains serial with
one build job; the configured full gate is unchanged.

Final exact-HEAD controller validation remains pending after T5.3 and the
canonical PR-reference follow-up.

## Risks and Follow-ups

T5.3 remains pending: the remaining finite-admission, historical-source and
attribution edge cases and slice regressions. Combined PTY/resource acceptance
remains Slice 13; the full User Guide remains Slice 14.
