# Slice 8: Implement character and word motion

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-08-motion`
PR: `#8`
Integrated: Rebase + Fast-forward into dev at `def2f4445d1b5adf4fae08226a7accb95cbc11f8`.

## Purpose

Move the displayed point through checked Unicode source positions and reveal
its destination without changing canonical text or edit history.

## Delivered Behavior

- `C-f` and `C-b` move over complete graphemes, including combining text,
  emoji sequences, flags and CRLF. Right and Left use the same handlers.
- `C-a` and `C-e` select logical line boundaries, excluding the whole LF/CRLF
  separator and retaining lone CR as visible text. Home and End are aliases.
- `M-f` and `M-b` skip separators and consume one word run in the selected
  direction. Words contain graphemes with Unicode letters, numbers or `_`;
  punctuation, whitespace and emoji separate runs consistently.
- `M-<` and `M->` select buffer start and end. All motions clamp at document
  boundaries, work on empty and read-only buffers, and reveal point within the
  usable viewport's logical rows and terminal cells.
- Movement preserves source revision, text, mark, newline metadata, existing
  history and dirty state. Supported repeats move once; releases are ignored.

## Implementation Notes

- T8.1: `61360a264a44ebbdbc9139d9cb4ce63d352c9a50` adds checked coordinate
  destinations, shared motion handlers, bindings and initial command tests.
- T8.2: `0b9fb1b4931394fa419a1b2209d0a8720dd1adb8` completes boundary,
  rope-seam, registry, viewport and real executable input coverage.
- Word scanning retains the safe chunk-aware grapheme cursor in either
  direction. Classification borrows each rope fragment; it never flattens the
  document or logical line into a per-command string.
- The session validates the qualified source point, prepares a candidate view,
  computes its checked destination and reveals it before publishing the view.
  Canonical buffer publication is unnecessary for movement.
- Binding lookup also owns repeat admission, keeping reported Alt, Control and
  unmodified alias dispatch consistent with the public command registry.
- Source checks pass safe Rust, five parameters, nesting at most three and
  cyclomatic paths at most ten.

## Contracts Added or Changed

- `Coordinates` adds logical line start/end, buffer end and bidirectional word
  destinations. Word queries reject out-of-range and interior-cluster input.
- `Motion` identifies eight implemented navigation commands. `Command::named`
  and normalized key input reach the same session handlers.
- `Session` publishes point and scroll together after validation, preserving
  the buffer's revision-qualified document state.
- `CHANGELOG.md` records usable grapheme, word, line and buffer navigation.

## Files of Interest

- `src/coordinates/motion.rs`: checked line and chunk-aware word destinations.
- `src/command/motion.rs`, `src/command.rs`: implemented command identities.
- `src/input/bindings.rs`, `src/input/key.rs`: shared bindings and key phases.
- `src/session/motion.rs`: qualified point movement and viewport reveal.
- `tests/coordinates.rs`: nineteen coordinate and motion integration tests.
- `scripts/check-pty.py`: actual Emacs and arrow/Home/End key journey.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; sixty-seven tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test coordinates`: passed; nineteen tests, zero skips. Coverage
  includes mixed scripts, punctuation, combining/emoji/flag rope seams, CRLF,
  line destinations, empty/boundary clamps, named/bound handler equivalence,
  key phases, dirty read-only state preservation and viewport reveal.
- `python3 scripts/check-pty.py --suite terminal`: passed; ten cases, zero
  skips. The added journey checks actual character, word, line and buffer keys,
  Unicode text preservation, visible cursor/status positions, aliases and
  retained dirty quit protection. Owned fixtures stop after their checks finish.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
