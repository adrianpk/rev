# Slice 22: Kill lines and words directionally

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-22-kills`
PR: pending

## Purpose

Remove logical line tails and directional Unicode words through checked edits,
retain their exact text in direction order and undo each kill independently.

## Delivered Behavior

- `C-k` / `kill-line` removes text from point to logical line end. At line end,
  it removes the complete LF or CRLF separator. At buffer end it is a no-op.
- `M-d` / `kill-word` and `M-DEL` / `backward-kill-word` use the existing Unicode
  word-motion policy: skip separators and consume one word run in the requested
  direction. Combining graphemes and ZWJ emoji remain whole.
- Consecutive forward kills append retained fragments; backward kills prepend
  them. Changing direction within the same live chain preserves source order.
  Retained fragments preserve their exact Unicode and newline bytes.
- Intervening commands, typing, input notices or prefixes, native edits, file
  loads and view selection end continuity. Empty or refused kills retain the
  previously killed text and end the chain without adding history.
- Each successful kill is one independent undo/redo step, separate from adjacent
  typing. Shared view points and stored mark follow the existing checked edit
  map; successful publication deactivates the transient region.
- Read-only buffers reject all three edits without changing text, point, mark,
  activation, revision, dirty state, history or retained text. Native commands
  also respect transient prompt, inventory and decision focus.
- Actual terminal key reports reach the native handlers. File journeys verify
  independent undo, whole CRLF removal, exact surviving bytes, BOM preservation,
  successful saves and terminal restoration on exit.

## Implementation Notes

- T22.1: `142154d0ed100c595ce03e21096a3dca6bcc03bb` adds checked directional
  commands, key bindings, owned retained fragments and independent history.
- T22.2: `a956cbee3c9880cbf15e2664f9b1d0959552ff22` adds boundary, refusal,
  continuity, shared-view, focus and actual terminal evidence.
- Kill ranges reuse the coordinate owner for logical line ends, complete line
  separators and grapheme-aware directional word boundaries.
- The session prepares an owned Rope fragment and its composed retained-text
  candidate before publishing the checked deletion. Rejected edits discard the
  candidate. Successful publication installs it with the new revision, view
  identity and point as the next command's continuity token.
- The retained entry uses Rope fragments rather than a flattened document copy.
  Continuity requires the same live buffer revision, selected view and point.
  Ordinary edit publication still owns all text, history and shared-view mapping.
- Typing continuity stops before kills. Retained-text coalescing does not merge
  their history groups, and presentation resize events do not create edits.

## Contracts Added or Changed

- `Command::Kill` registers `kill-line`, `kill-word` and `backward-kill-word`
  with working native handlers and canonical control/meta bindings.
- `Session::killed_text` provides a borrowed view of the retained Rope entry.
  Preparation preserves the current entry until checked text publication succeeds.
- The session tracks kill continuity separately from typing history, input focus
  and view navigation. Non-kill interaction invalidates continuation authority.
- `CHANGELOG.md` describes the delivered line and word editing keys and their
  independent undo behavior.

## Files of Interest

- `src/command/kill.rs`, `src/input/bindings.rs`: ranges, names and key reports.
- `src/kill.rs`: owned retained fragments and qualified continuity token.
- `src/session/kill.rs`: focused command orchestration and checked publication.
- `src/session.rs`, `src/session/edit.rs`: command and edit continuity boundaries.
- `src/session/file.rs`, `src/session/windows.rs`: load and view boundaries.
- `tests/editing/kills.rs`, `tests/commands.rs`: native semantic evidence.
- `scripts/check-pty.py`: real line/word keys, undo, saves and terminal cleanup.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 215 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test editing`: passed; sixty-three tests, zero skips. Twelve
  kill cases cover LF/CRLF, Unicode punctuation and words, directional composition,
  separate undo/redo, empty boundaries, read-only admission, command/input/native
  continuity, shared point/mark mapping, view changes, transient focus and repeats.
- `python3 scripts/check-pty.py --suite terminal`: passed; forty-four cases,
  zero skips. New journeys exercise actual C-k, M-d and M-DEL reports, independent
  undo, whole CRLF removal and exact BOM/newline saves. Owned fixtures stop after
  their checks finish; terminal restoration is checked.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
