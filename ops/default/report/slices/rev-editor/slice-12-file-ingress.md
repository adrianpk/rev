# Slice 12: Load exact UTF-8 file content

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-12-file-ingress`
PR: `#12`
Integrated: Rebase + Fast-forward into dev at `9bf25810d05f17fb04d3b535899a1d1ebf955caa`.

## Purpose

Load existing UTF-8 files as clean canonical buffers while preserving exact text
and format bytes and refusing damaged or binary input before publication.

## Delivered Behavior

- `rev [file]` accepts one optional existing regular file. Successful ingress
  starts at point and scroll zero with a clean history root. Extra arguments
  fail before terminal acquisition; file failures report after restoration.
- Strict incremental UTF-8 decoding rejects malformed or incomplete scalars.
  Any NUL rejects the candidate as unsupported binary input. Read failures,
  missing paths, directories, devices and unreadable files retain readable
  path context and their I/O error kind.
- Exactly one leading UTF-8 BOM becomes format metadata, outside text and
  coordinate domains. Interior or additional BOM scalars remain text. The
  serializer emits the recognized prefix once and writes borrowed rope chunks.
- LF, CRLF, mixed separators and final-newline absence survive an unchanged
  round trip byte for byte. Existing separator counts select inserted newlines,
  with LF on ties and for empty content. Edits and undo retain BOM metadata.
- Failed session ingress preserves the active document, revision, logical
  state, mark, undo/redo, point, scroll and pending input prefix. Successful
  ingress cancels the old prefix while retaining previous buffers and their
  unsaved-change protection.

## Implementation Notes

- T12.1: `5f874847bc942eb55481037004383fb01d46699f` implements ingress,
  canonical installation, exact serialization and startup. The initialization
  owner in `69ad76b8616b1139b27aa9b6cdc8f057ac4be078` separates startup
  admission from terminal event dispatch within the source complexity limit.
- T12.2: `f012fa51de4fc8d29f1afcfc18f37e5d2fb9053f` adds real filesystem,
  fragmented-reader, active-session and executable PTY coverage. The permission
  fixture uses an octal Unix mode in `c744436be30be980975a269f3ff228da2d94c697`.
- A private `Loaded` candidate owns the decoded rope until strict validation
  finishes. Installation consumes it into one canonical buffer; no whole-file
  String or byte-vector copy enters the loading or serialization path.
- File metadata rejects non-regular input before opening and checks the opened
  handle again. Session admission completes recoverable work before publishing
  the registry entry and replacing the active view.
- Serialization writes format and text bytes to a caller-owned writer. The
  caller owns flush, sync and filesystem publication; writing bytes does not
  acknowledge a save or change the logical clean marker.
- Source checks pass safe Rust, five parameters, nesting at most three and
  cyclomatic paths at most ten.

## Contracts Added or Changed

- `file::Loaded::read` returns only fully decoded, NUL-free ingress, preserving
  the distinction between format metadata and editable text.
- `file::load` reads existing regular paths with contextual I/O errors.
  `file::write` serializes exact current bytes without changing buffer state.
- `Buffers::load` consumes validated ingress as a fresh clean owner.
  `Buffer::has_bom` exposes retained format metadata without exposing the rope.
- `Session::load` provides the common ingress/publication boundary and a
  readable error echo. `rev::run` accepts an optional startup path.
- `CHANGELOG.md` describes opening UTF-8 files and preserving their format.

## Files of Interest

- `src/file.rs`: strict ingress, regular-file admission and chunked serialization.
- `src/buffer.rs`: consumed rope installation, clean roots and BOM metadata.
- `src/session/file.rs`, `src/session.rs`: private admission and startup ownership.
- `src/main.rs`, `src/lib.rs`: native path arguments and terminal error cleanup.
- `tests/files.rs`: eleven format, I/O, history and session-preservation tests.
- `scripts/check-pty.py`: real startup editing and terminal restoration journeys.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 110 tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test files`: passed; eleven tests, zero skips. Cases cover empty,
  mixed and BOM files, byte-identical output/reload, one-byte read fragments,
  late invalid data, injected read failures, actual permissions, missing and
  non-regular paths, retained active state and reversible format-aware edits.
- `python3 scripts/check-pty.py --suite terminal`: passed; twenty cases, zero
  skips. Added journeys exercise Unicode startup, hidden BOM, CRLF editing,
  imported clean history, unchanged source bytes, five actual ingress failures
  with restored terminal state, and early excess-argument refusal. Owned
  fixtures stop after their checks finish.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
