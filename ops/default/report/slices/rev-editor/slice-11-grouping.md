# Slice 11: Group typing and compound history

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-11-grouping`
PR: `#11`
Integrated: Rebase + Fast-forward into dev at `b81d43b0f43dbaca5718f4b21035890c7e9fbf37`.

## Purpose

Make adjacent typing one coherent undo unit while retaining complete compound
operations, reachable save boundaries and bounded per-buffer history.

## Delivered Behavior

- Adjacent ordinary Unicode insertion forms one undo group, including combining
  text and emoji components. Every accepted key still publishes a fresh text
  revision and logical state; undo restores the group's original point/text
  and redo restores its complete final text and point.
- Commands, prefix handling, cancellation, invalid input and resize break
  typing continuity. Enter, Tab and deletion remain separate operations.
  Printable repeats can extend a group; releases do not interrupt it.
- Changes of initiating point, other live-point positions, mark, access state
  or save acknowledgment prevent merging across contexts. A prepared join is
  rejected if continuity ends before publication, even with equal text stamps.
- Bulk and multi-range replacements remain complete single undo units next to
  typing. Typing after undo starts a fresh group and discards the redo future
  only after successful publication.
- Save acknowledgment preserves history and ends the current group, retaining
  a reachable clean logical state. Canceled or rejected private candidates
  cannot extend stored fragments or erase confirmed history.
- History retains at most 512 complete groups with a soft 64 MiB logical
  inserted/removed-payload target. Pruning removes oldest whole groups and
  always retains the newest coherent operation, even when it alone exceeds
  that target. Lost history cannot falsely make retained text clean.

## Implementation Notes

- T11.1: `daf03ad40d2393c7731448a9a92d6a422180de5b` adds checked typing
  preparation, context breaks, group publication and initial integration tests.
- T11.2: `3846a2971f434b88ec40bee545b24bba938b6e38` completes continuity,
  compound, retention and terminal coverage. `febba5045e033c8409c59c254dae486fa9b82cea`
  synchronizes the PTY assertion with complete Unicode text rather than an
  intermediate emoji component at the same display column.
- A typing candidate retains the immediate edit's publication snapshot and a
  separately prepared merged history record. Publication rechecks positions,
  mark, revision, access state and join continuity before replacing history.
- Joining extends one insertion fragment through borrowed rope chunks. The
  retained record keeps the original before-state, initiating positions and
  mark, while adopting the latest after-state and positions. It never flattens
  document text or stores an additional authoritative document.
- Group replacement subtracts the previous payload before accounting for the
  merged record. The same whole-group pruning path handles ordinary edits,
  joined typing and compound replacements; undo/redo transfers preserve budget.
- The newest-operation test publishes an actual two-range payload of
  64 MiB + 2 bytes, confirms older history is pruned and both ranges remain
  reversible together. Deep traversal also covers 512 retained two-key groups.
- Source checks pass safe Rust, five parameters, nesting at most three and
  cyclomatic paths at most ten.

## Contracts Added or Changed

- `Buffer::prepare_typing` permits eligible single-scalar insertions to extend
  a checked group. Other replacement shapes remain standalone operations.
- `Buffer::break_typing` explicitly ends continuity. Confirmed save, mark and
  access metadata operations also end it; failed or canceled preparation does
  not mutate confirmed continuity or retained fragments.
- `Prepared` carries any merged record privately. A save or context break
  between preparation and publication invalidates the join before mutation.
- `Session` separates command admission from dispatch and ends typing on
  commands, prefix/error notices and resize while preserving normal release
  and printable-repeat behavior.
- `CHANGELOG.md` describes grouped typing and whole compound undo steps.

## Files of Interest

- `src/buffer/edit/grouping.rs`: eligibility, continuity, merging and admission.
- `src/buffer.rs`, `src/buffer/edit.rs`, `src/buffer/edit/history.rs`: metadata
  boundaries, private candidates, whole-group publication and budget accounting.
- `src/session.rs`, `src/session/edit.rs`: ordinary insertion and command breaks.
- `tests/editing.rs`: forty-four transaction/editing/grouping integration tests.
- `scripts/check-pty.py`: complete Unicode typing groups and command boundaries.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; ninety-nine tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test editing`: passed; forty-four tests, zero skips. Added cases
  cover Unicode typing, command/prefix/resize boundaries, key phases, save
  checkpoints, late join rejection, cancellation, changed live views,
  multi-range replacements, redo branching, bulk transactions, 512 complete
  retained groups and a real newest operation above the 64 MiB soft target.
- `python3 scripts/check-pty.py --suite terminal`: passed; thirteen cases, zero
  skips. The added journey checks whole Unicode typing undo, command and Enter
  boundaries, new typing after undo, clean-state restoration and ordinary exit.
  The encoding journey uses explicit command boundaries to keep its separate
  undo-key checks meaningful. Owned fixtures stop after their checks finish.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
