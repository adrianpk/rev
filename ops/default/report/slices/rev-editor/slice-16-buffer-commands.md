# Slice 16: Switch list and close buffers

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-16-buffer-commands`
PR: `#16`

## Purpose

Select and inspect live buffers, close them through real save/discard/cancel
choices, and retain valid editing state and canonical file ownership.

## Delivered Behavior

- `C-x b` and named `switch-to-buffer` select live names. Tab completes a unique
  name or shared Unicode prefix; Enter selects an exact name or the current
  buffer when empty. Unknown names and cancellation preserve the active owner.
- `C-x C-b` and named `list-buffers` present a read-only selectable inventory
  with unique labels, dirty/access indicators and visited paths. Arrows and
  Home/End move selection; Enter activates it and `C-g` cancels.
- `C-x k` and named `kill-buffer` choose a live buffer, defaulting to the current
  owner. Clean buffers close immediately. Modified buffers explicitly offer
  save, discard or cancel; Enter never supplies implicit consent.
- Required saving follows the chosen buffer through unnamed destination and
  overwrite/external-change prompts. Failed saves or canceled nested prompts
  retain it. Only a complete save of the unchanged close revision permits closure;
  pending publication warnings still require a decision at an old clean text state.
- Closing an inactive buffer preserves the active document and view. Closing the
  displayed buffer selects the first surviving identity deterministically. Closing
  the final buffer allocates a fresh clean scratch before removing the old owner.
- Buffer text and undo/redo survive selection changes and removed disk paths.
  Killing releases the canonical file index and unique label for a new visit;
  closed identities and their captured saves cannot alias later owners.

## Implementation Notes

- T16.1: `6788da0e8274452b56409596f5c3fdc09e2444e6` adds registry lifecycle,
  command bindings, completion, inventory and guarded close/save orchestration.
- T16.2: `1e528069250b38fec414c0305a485e161eef70de` adds owner/history,
  failure, cancellation, final replacement, index and real-terminal evidence.
- Inventory and completion borrow live canonical owners. Only visible inventory
  rows are projected; one selected identity and one bounded argument are retained,
  without a second text store or persistent inventory copy.
- Close requests capture buffer identity and text revision. Admission rejects
  newer edits before removing an owner, and final replacement allocation precedes
  registry mutation. Initial identity and canonical index remain valid afterward.
- Pending arguments, inventory selection, close choices and confirmed saves have
  distinct owners. Save continuations carry the chosen close request instead of
  assuming the active editing buffer is the save source.
- PTY waits accept a matching completed redraw, so text markers and cursor
  assertions observe the same frame rather than a partially received successor.
- List input consumes its keys and Enter repeats; only editing focus can route
  document keystrokes. Tiny/empty geometry retains input ownership safely.
- Existing real output faults exercise unresolved publication before close. These
  warnings cannot masquerade as a completed required save; a fresh guarded retry
  must complete before the save continuation removes its buffer.

## Contracts Added or Changed

- `Buffers::iter` borrows stable identity order and `named` resolves unique live
  labels. `prepare_kill` captures revision-qualified close authority; `kill`
  consumes it, repairs indexes/initial identity and returns a valid replacement.
- `Command::Buffer` delegates the switch, list and kill family through the common
  registry/handler, alongside existing file, motion and history commands.
- `Input::inventory`, selection outcomes and bounded argument completion keep
  transient input separate from text edits. Frame inventory projection renders
  labels and paths through the existing safe glyph/cell clipping owner.
- Session save continuations retain optional close authority across destination
  and overwrite decisions; only complete output can invoke the guarded close.
- `CHANGELOG.md` describes switching, listing and save/discard/cancel closure.

## Files of Interest

- `src/buffer/lifecycle.rs`: borrowed inventory, names and checked removal/replacement.
- `src/session/buffers.rs`, `src/session/buffers/`: selection, completion and close flow.
- `src/session/save.rs`, `src/session/file.rs`: target-specific required-save continuation.
- `src/display/inventory.rs`: bounded read-only inventory rows and selected cursor.
- `src/input/inventory.rs`, `src/input/bindings.rs`, `src/command/buffer.rs`: focused keys and commands.
- `tests/files/buffers.rs`, `tests/files/saving.rs`: lifecycle, state, ownership and save outcomes.
- `scripts/check-pty.py`: real buffer selection and close/save journeys.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 159 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test files`: passed; sixty tests, zero skips. Added cases cover
  Unicode completion, duplicate labels, active/inactive close, modified unnamed
  owners, retained history and removed paths, cancellation, denied/missing/read-only
  required saves, pending publication, nested consent, stale close requests,
  final replacement, released indexes, closed captures and bounded inventories.
- `python3 scripts/check-pty.py --suite terminal`: passed; twenty-eight cases,
  zero skips. New journeys exercise real switch/list/kill keys, duplicate-name
  selection, retained undo, inactive closure, canceled and failed required saves,
  unnamed save continuation, external-change consent, discard and final replacement.
  Owned fixtures stop after their checks finish; terminal restoration is checked.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
