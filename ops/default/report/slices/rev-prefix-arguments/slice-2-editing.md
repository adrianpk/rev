# Slice 2: Counted edits and history

Status: delivered
Delivery set: rev-prefix-arguments
Plan: [`ops/default/plan/rev-prefix-arguments.md`](../../../plan/rev-prefix-arguments.md)
Tracker: [`ops/default/tracker/rev-prefix-arguments.md`](../../../tracker/rev-prefix-arguments.md)
Branch: `feat/rev-prefix-arguments-02-editing`
PR: [#315](https://forge.adrianpk.com/adrianpk/rev/pulls/315)
Integrated: `57afa0a31ffcb56ee9bff6da0135e7e75a628248`

## Purpose

Apply checked prefix arguments to native source edits, kills, yank and history.
T2.1, T2.2 and T2.3 are implemented and canonically integrated through PR #315
(Rebase + Fast-forward). The delivery-set gate has since passed; the
[Slice 4 closure](slice-4-acceptance.md#delivery-set-closure) retains its evidence.

## Delivered Behavior

- Literal characters and admitted paste insert N UTF-8 copies, replacing an
  active region once. Newline repeats the source's LF/CRLF separator; tab
  composes the native indentation policy at each resulting display column.
  Zero is inert and negative insertion counts refuse. Checked multiplication
  and aggregate admission bound generated text to 8 MiB before materialization.
- Each counted edit publishes one native transaction and one undo group,
  separated from neighboring ordinary or modal Insert/Replace typing.
  Completion is scheduled once after a counted text publication.
- Signed character deletion traverses complete native graphemes and CRLF
  separators, reversing direction for negative counts. A live region takes
  precedence once; zero preserves the selection.
- Word kills prepare one combined signed span. Supplied line kills count logical
  lines independently of wrapping, including positive line endings, zero to
  the current line start and negative counts to preceding line starts.
  Existing directional kill-chain append/prepend behavior remains intact.
- Positive numeric yank selects the Nth newest retained entry once. Plain-prefix
  yank selects the newest and prepares exchanged point/mark orientation in the
  same source transaction. Signed yank-pop rotates its exact captured insertion
  span; zero preserves the capture. Nonpositive numeric yank refuses.
- Nonnegative undo/redo counts replay existing native groups progressively.
  Zero is inert; negative counts refuse. Boundaries report the native stop,
  retaining completed groups. Cancellation also retains completed replays.
- Pending distance plans yield after at most 256 native steps or the existing
  4 ms cooperative target. History uses the same bounds between native group
  replays. The existing deferred-input FIFO owns cancellation and source/point
  invalidation; canceled or stale atomic plans publish no text or ring entry.

## Implementation Notes

Input emits one typed TextInvocation for admitted literal/paste input with an
argument. Prompt editing and named command selection retain their existing
owners. Ordinary text without an argument keeps its prior input path.

The existing layout/input owner retains finite text, distance or history work.
Counted text materializes a checked candidate once and uses the ordinary source
replacement transaction. Deletion and kill extend the existing disposable
movement plan, then publish one range through native edit/ring admission.
Kill entries are composed before source mutation and published only on success.

History calls the native group replay owner, without a synthetic transaction or
a selective undo engine. Plain-prefix yank admits its resulting mark on the
private Prepared candidate, so source, point, mark and undo data publish
coherently. No dependency, preference, persistent argument state or generic
command repetition loop is introduced.

Commit `665e5f956ba41ac52703785e85617177db481aba` closes the verified Slice 1 records and activates this
controller-selected slice. Commit
`8a34b5ae72b9ef09a0fcebdaa8c245dd9444d9aa` implements T2.1 and T2.2 and the
native acceptance portion of T2.3. Commit
`a59daebb930f7dcd5b8c2ffbc6754919d610c6e5` introduces this report and completes
T2.3 with the current command reference and cohesive Unreleased outcome.

## Contracts Added or Changed

- Public TextInvocation carries one admitted source payload, checked argument
  and captured input/source origin; it has no serialized form.
- Native Prepared admits a resulting mark before publication. Ordinary edit
  source/position/read-only guards and reversible history remain authoritative.
- Ring entry lookup and signed rotation retain the existing bounded ring order.
- Deferred input now owns counted atomic edits and progressive native history
  as well as movement. It retains no count work after completion or retirement.

## Files of Interest

- `src/input/argument.rs`, `src/input/paste.rs`: counted source entry transport.
- `src/session/arguments.rs`, `src/session/arguments/{text,history}.rs`,
  `src/session/layout.rs`: preparation, publication and bounded native replay.
- `src/session/yank.rs`, `src/kill.rs`, `src/buffer/edit.rs`: checked ring and
  transactional point/mark behavior.
- `tests/editing/prefix_arguments.rs`: fourteen real native editing/ring/history
  cases, including read-only admission, modal entry, cancellation and stale work.
- `tests/commands/prefix_arguments.rs`: named edit/history usage and terminated
  literal count journeys, alongside the retained Slice 1 acceptance.
- `docs/reference/commands.md`, `CHANGELOG.md`: current delivered behavior.

## Validation

These are local slice checks, not the integrated delivery-set gate. The final
controller validation receipt retains the exact clean candidate and ordered
command results outside the repository.

- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- Passed, 24 tests including 16 prefix cases: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments context_isolation transient_isolation prompt_reset workspace_reset tree_up_directory tree_actions tree_transitions tree_open_sequence --test-threads=1`
- Passed, 110 tests including 14 prefix cases: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test editing -- --test-threads=1`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- Passed: `git diff --check`

## Risks and Follow-ups

History is explicitly progressive and cancellation does not roll it back.
The remaining positioning, scrolling, tree, shell/context/modal policies were
subsequently integrated in [Slice 3](slice-3-contexts.md). Joint acceptance and
guidance are recorded in [Slice 4](slice-4-acceptance.md), now integrated through
canonical PR #317. The controller passed the integrated delivery-set gate at
`7cfef9be78fad975040ccd78dcb9cfebe32b28cd`; [Slice 4 closure](slice-4-acceptance.md#delivery-set-closure)
records acceptance and the completed documentation-only closure.
