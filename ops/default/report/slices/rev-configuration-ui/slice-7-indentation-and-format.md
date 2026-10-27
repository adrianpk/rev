# Slice 7: Apply indentation and new-file defaults

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-07-indentation-and-format`
PR: [#56](https://forge.adrianpk.com/adrianpk/rev/pulls/56)
Integrated: `e6d781ad529cdb57ca20ede62ac76984c6a77505`

## Purpose

Apply the effective buffer indentation policy and initialize new-file format
defaults while retaining literal paste, canonical history and existing-file
encoding guarantees.

## Delivered Behavior

- Routes bound Tab and named `insert-tab` through the same policy-aware handler.
  Tab style inserts one U+0009; space style inserts enough spaces to reach the
  next indentation stop using display columns rather than scalar counts.
- Keeps indentation width independent of literal-tab display width, including
  existing tabs, nonaligned columns and wide/combining/ZWJ text. The compiled
  default now inserts spaces at four-column stops.
- Preserves bracketed paste literally as one undo operation, including tabs,
  mixed line separators and command-looking bytes.
- Applies BOM and initial newline defaults only during new-buffer creation,
  including scratch, explicit creation, absent visits and final-buffer
  replacement. Neither visiting nor serializing creates missing disk entries.
- Uses the configured creation separator when a new buffer has no separators.
  Existing text retains prevailing separator selection; LF wins mixed ties.
- Preserves loaded BOM, LF/CRLF, mixed separators and absent final newline under
  opposing defaults, editing-policy replacement, word edits, saves and undo/redo.
- Retains read-only refusal and canonical edit/history/saved-state boundaries.

## Implementation Notes

- T7.1: `8800c20c6a6df0a1f0796dc26607d80faf30b111` adds policy-aware Tab
  insertion, focused indentation/paste tests and the visible default update.
- T7.2: `7c34b501f11ed17174d108dbdfc38419b817e3ee` initializes format
  metadata once in new-buffer construction while loaded ingress bypasses it.
- T7.3: `8be9fb6807d7b69038667689cf29f64370729540` adds real bound/M-x
  admission, Return/save/undo, unrelated word edits and
  final-buffer format coverage and this report. All eight new tests pass.
- Indentation uses the existing checked `replace` operation. It publishes one
  replacement and retains shared-view mapping, command boundaries and history.
  Admitted widths bound space allocation to at most 32 bytes.
- New-file BOM remains format metadata rather than a rope character. Initial
  newline preference is fallback metadata; separator counts continue to update
  only through atomic text publication. Creation does not normalize supplied text.
- `from_loaded` retains its recognized BOM and ordinary LF fallback when no
  separator exists. Applying a later editing policy changes neither format field.
- The default-input and key-repeat expectations now assert spaces and their
  resulting scalar positions. Existing help, bindings/default reference and
  Unreleased notes describe that observable change without advertising startup
  configuration before its integration.

## Contracts Added or Changed

- `insert-tab` applies the selected buffer's `IndentStyle` and `indent_width`;
  its display column comes from the same tab-aware coordinate owner as rendering.
- New-buffer construction receives global file defaults. Validated file ingress
  retains separate ownership of imported text and recognized format metadata.
- `Buffer::newline()` uses the creation fallback only when no separator exists;
  prevailing LF/CRLF selection and LF mixed-tie behavior remain intact.
- `Buffer::apply_settings` remains limited to editing/display policy. It cannot
  convert BOM, separators or final-newline state.

## Files of Interest

- `src/session.rs`, `session/edit.rs`: common indentation dispatch and checked edit.
- `src/buffer.rs`, `buffer/visits.rs`, `buffer/lifecycle.rs`: creation-only format
  metadata across scratch, visits and replacement buffers.
- `tests/commands/indent.rs`: stop geometry, literal paste, bound/M-x equivalence
  and read-only refusal.
- `tests/files/format.rs`: creation/import separation, real save/history journeys,
  existing byte fidelity and final-buffer metadata.
- `tests/editing.rs`: default direct-input and repeat regression expectations.
- `docs/how-to/edit-files.md`, `docs/reference/bindings.md`, `defaults.md`,
  `CHANGELOG.md`: the visible default Tab behavior.

## Validation

Development checks:

- `cargo test --test commands indent::`: passed; three tests.
- `cargo test --test files format::`: passed; five tests.
- `cargo clippy --lib --test commands --test files -- -D warnings`: passed.

The controller recorded all seven ordered checks passing on clean head
`e6d781ad529cdb57ca20ede62ac76984c6a77505`, validation
`S7M2KNEOYW3XTZSQU4M3ESKR2Y`: 92 editing, 67 file and 66 command tests.
Canonical PR #56 integrated that exact head into dev by Rebase + Fast-forward.
Required commands, in canonical order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test editing`
- `cargo test --test files`
- `cargo test --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

Rectangle transfer semantics and automatic path-change policy transitions remain
assigned to Slices 8-9. Startup user-configuration activation remains assigned
to Slice 20; supplied snapshots exercise these consumers before that activation.
Configuration-reference publication and baseline reconciliation remain assigned
to their recorded slices.
