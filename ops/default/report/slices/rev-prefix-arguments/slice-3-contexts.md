# Slice 3: Contextual argument behavior

Status: reviewing
Delivery set: rev-prefix-arguments
Plan: [`ops/default/plan/rev-prefix-arguments.md`](../../../plan/rev-prefix-arguments.md)
Tracker: [`ops/default/tracker/rev-prefix-arguments.md`](../../../tracker/rev-prefix-arguments.md)
Branch: `feat/rev-prefix-arguments-03-contexts`
PR: pending

## Purpose

Complete T3.1, T3.2 and T3.3 positioning, scrolling, loaded-tree and shell policies
while preserving native input ownership and bounded execution. Canonical
integration and the delivery-set gate remain controller-owned.

## Delivered Behavior

- Line start/end arguments traverse N-1 ordinary visual/logical rows before
  choosing that row's native boundary. Signed and zero values use the same
  row and source boundary rules as ordinary movement.
- Supplied beginning/end-of-buffer values choose N tenths from the requested
  end, clamp to source bounds and snap to complete grapheme/CRLF boundaries.
  Positive supplied goto-line values select the one-based logical line directly;
  nonpositive values refuse, while absent input retains the prompt.
- Supplied page scrolling counts signed ordinary rows. Recenter arguments
  choose a top-relative or negative bottom-relative content-row offset,
  clamped to pane geometry. Absent page/recenter behavior remains native.
- Tree next/previous and their contextual row aliases count signed distances
  through already loaded preorder rows, including the parent. They clamp at
  endpoints without expansion, directory reads or source edits.
- A prefix reaching named shell-command selects the existing insertion variant
  once; named shell-command-on-region selects the checked filter once. Numeric
  values do not repeat process launches. Existing C-u shell continuations keep
  precedence in ordinary and modal Insert input.
- Editor prefixes and native modal counts/operators refuse ambiguous composition
  in either order, including an explicitly named helper after a modal count.
  Normal/Visual C-u keeps half-page ownership; Insert/Replace inherit editor
  entry. Escape retires editor prefix input through native modal ownership.
- Source, prompt, structural and raw PTY owners retain their admitted input
  paths. Returning from a PTY does not leave an editor count behind. Unknown
  policies consume the argument and execute the resolved command once.
- Large row/distance plans retain disposable native views, service cancellation
  after at most 256 steps or the existing 4 ms cooperative target, and recheck
  source/role identity before further preparation. Source changes revoke the
  target. Existing entry and generated UTF-8 bounds remain enforced without
  partial source/history publication or a permanent count worker.

## Implementation Notes

The existing movement plan handles signed ordinary rows and final row
boundaries. Absolute positions use the same checked coordinate/view owner.
Supplied recentering reuses the native logical/wrapped alignment path; ordinary
cycling calls that shared path with its existing alignment state.

Loaded tree selection uses one saturating arithmetic operation in the native
Tree owner. Shell arguments select existing compiled variants before the
ordinary shell owner captures its source and launches a single process.

Input retains only a transient conflict bit while M-x borrows a pending modal
composition. Resolving an editor helper refuses that ambiguity; another resolved
command or cancellation clears it. No numeric multiplication between modal and
editor owners, new persistent state, preference, dependency or generic command
repeat loop is introduced.

Commit `5b264dee4a5f250830b25f6c07f1d4bf9f20553f` closes Slice 2 records.
Commit `bc1301031337d2a27dd6bdab04558d371efd1796` implements T3.1 and
T3.2 and the native acceptance portion of T3.3. This report, current command
reference and cohesive Unreleased outcome complete T3.3.

The preceding Slice 2 report/tracker entries are closed after canonical PR #315
integration at `57afa0a31ffcb56ee9bff6da0135e7e75a628248`. The controller selected
this slice; Slice 4 and the combined gate remain pending.

## Contracts Added or Changed

- Native movement preparation now covers all approved positioning and source
  scrolling policies, preserving checked source, geometry and row ownership.
- Tree gains bounded loaded-row selection and keeps its ordinary one-step API.
- Native view alignment accepts a supplied pane-relative row offset while
  retaining the ordinary recenter cycle.
- Named shell prefixes select the already existing insertion/filter contracts.
- Native input refuses mixed editor/modal counts across direct bindings and M-x.

## Files of Interest

- `src/session/arguments.rs`, `src/session/arguments/position.rs`: policy dispatch,
  native destination preparation, source invalidation and tree selection.
- `src/view/scroll.rs`, `src/tree/navigation.rs`: native alignment and loaded rows.
- `src/input/{argument,dispatch,grammar,palette}.rs`: modal conflict ownership.
- `tests/commands/prefix_arguments.rs`: positioning, scroll, tree, transient/raw
  ownership and bounded stale work acceptance alongside the retained cases.
- `tests/modal/prefix_arguments.rs`: four native profile/conflict/cancel cases.
- `tests/shell_commands/prefix_arguments.rs`: three real isolated shell cases
  across both profiles, including one-execution counters and native undo.
- `docs/reference/commands.md`, `CHANGELOG.md`: complete delivered policy reference
  and cohesive user-visible Unreleased outcome.

## Validation

The exact required slice commands are recorded before PR publication. The final
controller receipt binds their ordered execution to the clean PR-reference HEAD.
These checks do not claim the integrated delivery-set gate.

- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- Passed, 32 tests including 24 prefix cases: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments context_isolation transient_isolation prompt_reset workspace_reset tree_up_directory tree_actions tree_transitions tree_open_sequence --test-threads=1`
- Passed, 28 tests including four prefix cases: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal -- count operator navigation prefix_arguments --test-threads=1`
- Passed, three real shell tests across both profiles: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands prefix_arguments -- --test-threads=1`
- Passed, 628 commands, 291 settings, 91 pages and 629 links: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- Passed: `git diff --check`

## Risks and Follow-ups

History retains the approved progressive cancellation behavior from Slice 2.
The complete joint acceptance sweep, final reference reconciliation and User
Guide build belong to approved Slice 4. No successor or integrated gate has been
started by this worker.
