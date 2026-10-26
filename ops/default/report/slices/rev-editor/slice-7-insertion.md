# Slice 7: Edit printable text and graphemes

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-07-insertion`
PR: pending

## Purpose

Connect direct text input and grapheme deletion to checked buffer transactions,
visible cursor updates, and protection of unsaved text.

## Delivered Behavior

- Inserts printable Unicode directly into the displayed canonical rope.
  Enter inserts the prevailing LF/CRLF separator, with LF for ties and empty
  text; Tab inserts a literal tab. Existing bytes remain unchanged outside the
  accepted replacement.
- Backspace removes the preceding complete grapheme. `C-d` and Delete remove
  the following complete grapheme. Combining text, emoji modifiers/ZWJ sequences,
  flags, and CRLF remain coherent; buffer boundaries create no empty transaction.
- Publishes text, reversible history, mapped point/mark, and newline metadata
  through the existing edit owner. The initiating cursor is explicit and remains
  a valid grapheme boundary after joins.
- Rejects mutating input on read-only buffers with a visible echo message and
  unchanged text, history, positions, and newline metadata. Unknown modified
  keys and raw controls cannot become document bytes.
- Allows appropriate editing repeats while ignoring releases. Binding and
  named-command dispatch use the same implemented handlers for newline, tab,
  backward deletion, forward deletion, cancellation, and guarded exit.
- Refuses `C-x C-c` while any buffer has unsaved changes. The session preserves
  text and remains editable, without offering an unavailable save action.

## Implementation Notes

- T7.1: `c87c86662528731c53845685640d2f5165256c12` adds direct input handlers,
  coherent newline accounting, guarded exit, and the initial Unreleased changelog.
- T7.2: `1dfd5d17b148f0b5ddc7f5440231dff3ebeea8ed` completes twenty-one editing
  tests and a real executable Unicode input/deletion/quit-refusal journey.
- Newline choice reads constant-time metadata. Initial CRLF counting scans
  validated input once. Preparation then recounts only changed scalar ranges
  and their immediate neighbors in the source and candidate revisions.
- Neighbor windows merge before counting, preventing duplicate CRLF accounting
  for adjacent multi-range changes. Candidate metadata publishes with the rope;
  late preparation failure, cancellation, or stale/read-only rejection cannot
  change confirmed counts. Unaffected document text is never flattened per key.
- The session owns its canonical buffers and qualified view. Its native event,
  command, and frame boundaries support direct integration tests using the same
  production path as the terminal loop.
- Quit admission examines every live buffer's dirty metadata. Distinct edits
  that recreate equal bytes remain conservatively modified under the retained
  initial-state policy.
- The added PTY journey verifies actual visible input, one-step emoji deletion,
  tab/newline cursor movement, an EOF deletion boundary, repeated quit refusal,
  and continued editing. Its owner terminates the dirty fixture after those
  checks. Other terminal cases retain ordinary exit/restoration coverage.
- Source checks pass safe Rust, five parameters, nesting at most three, and
  cyclomatic paths at most ten.

## Contracts Added or Changed

- `Session` routes normalized input and exact named commands through common
  handlers, and exposes immutable active-buffer/view state plus fresh frames.
- `Command` adds `newline`, `insert-tab`, `backward-delete-char`, and
  `delete-char` only after their handlers work. `C-d`, Backspace, Delete, Enter,
  and Tab bind to those variants; printable input uses checked scalar insertion.
- `Input::cancel` clears focused transient state for both binding and named
  dispatch. Editing Enter/Tab repeats do not weaken prompt confirmation rules.
- `Buffer::newline` returns the prevailing current separator. CRLF metadata is
  part of private preparation and atomic publication; logical LF line indexing
  and source bytes retain their existing policy.
- `Buffers::is_modified` guards session exit across all live canonical buffers.
  Rejected exit returns a redraw/message rather than an exit action.
- `CHANGELOG.md` begins the Unreleased record for usable terminal editing,
  visible status/echo, and unsaved-text protection.

## Files of Interest

- `src/session.rs`, `src/session/edit.rs`: event/command handlers and checked edits.
- `src/command.rs`, `src/input.rs`, `src/input/key.rs`: bindings and key admission.
- `src/buffer.rs`, `src/buffer/edit.rs`, `src/buffer/endings.rs`: newline metadata
  and coherent transaction publication.
- `tests/editing.rs`: twenty-one editing and command-input integration tests.
- `scripts/check-pty.py`: actual executable editing and dirty-quit journey.
- `CHANGELOG.md`: user-facing Unreleased outcomes.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; fifty-six tests, including twenty-one editing tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test editing`: passed; twenty-one tests, zero skips. Added cases
  cover direct visible input, bound grapheme/CRLF deletion, read-only rejection,
  key phases, isolated controls/prefixes, named-handler equivalence, dirty quit
  guards, prevailing newlines, scalar-range seam accounting against a whole-text
  oracle, compound neighbor windows, and newline metadata rollback.
- `python3 scripts/check-pty.py --suite terminal`: passed; nine cases, zero
  skips. The new journey verifies Unicode text and cursor/status updates,
  deletion, tabs/newlines, an EOF boundary, preserved unsaved text, and editing
  after rejected quit. Owned fixtures stop only after their checks finish.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
