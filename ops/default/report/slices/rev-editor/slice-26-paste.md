# Slice 26: Insert bracketed Unicode paste atomically

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-26-paste`
PR: `#26`
Integrated: Rebase + Fast-forward into dev at `2dbaa9b2cc86e7ee9d3c253f55aaa6396fcd6d5d`.

## Purpose

Insert supported bracketed Unicode paste as one literal edit with coherent undo,
focused input ownership and terminal restoration.

## Delivered Behavior

- Terminal entry enables bracketed paste. Every restoration attempts to disable
  it before restoring cursor, display attributes, screen and raw mode. Failed
  disable obligations remain owned for retry without skipping other cleanup.
- A complete paste event clears pending chord/Meta prefixes and bypasses key
  decoding. Embedded C-x, C-g, Escape and command-looking text remain literal
  document data, displayed with existing visible control notation.
- Editing paste publishes one canonical replacement and history group, separate
  from adjacent typing. Unicode, tabs, LF/CRLF and final-newline absence survive
  exactly. Shared points and mark map through the same transaction; undo from
  either view removes the entire payload with valid grapheme cursor positions.
- Empty, NUL-containing and read-only paste attempts publish no document change.
  Paste ends kill/yank and typing continuity, including refused attempts.
  Releases and held suffixes cannot repeat the payload; a new press resumes input.
- Argument prompts accept single-line literal text at their own cursor within
  the existing 4096-scalar bound. Oversized or control-containing payloads reject
  wholly. Decision and inventory focus refuse paste without accepting a response.
  C-g after delivery cancels input without undoing already inserted text.

## Implementation Notes

- T26.1: `c944a880f3ef90d640df88c51daa3354261d07d6` adds terminal mode
  ownership, focused paste routing, one checked insertion, reference and initial
  native/fault evidence.
- T26.2: `fd88fccd2936c41dd2c88c3fddef5dce8c9ee953` adds large rope payloads,
  mixed-newline preservation, shared-view undo, focused bounds, phase/cancellation
  boundaries, yank invalidation, grapheme seams and real terminal paste journeys.
- The input owner routes one owned string to its focused context. The session
  closes typing continuity, rejects binary input before replacement and delegates
  accepted text to the existing checked buffer/window publication path.
- No payload is converted to key events or normalized to the prevailing newline
  style. Prompt admission checks the complete payload before insertion.
- The terminal fault probe covers both enable failure and disable retry. The PTY
  harness uses bounded frame waits and actual saved-byte comparisons; only owned
  fixtures are stopped, after their checks finish.

## Contracts Added or Changed

- `Outcome::Paste(String)` carries complete editing payloads independently of
  command decoding. Prefixes and repeat suppression are reset at paste admission.
- `Terminal` owns bracketed paste acquisition and restoration with the other
  acquired terminal obligations.
- Editing paste uses the canonical transaction, mark affinity, shared-view
  mapping, read-only admission and history rules without replacing those owners.
- `docs/reference/terminal-input.md` records literal paste, prompt/decision policy,
  NUL rejection and the boundary between Crossterm decoding and event fidelity.
- `CHANGELOG.md` describes atomic Unicode paste and exact newline preservation.

## Files of Interest

- `src/terminal.rs`: bracketed paste lifetime and failed cleanup retry.
- `src/input.rs`, `src/input/paste.rs`: focused routing and prefix boundaries.
- `src/input/prompt.rs`: atomic bounded single-line prompt insertion.
- `src/session.rs`, `src/session/paste.rs`: canonical edit and history continuity.
- `tests/commands/paste.rs`: twelve native paste regressions.
- `tests/commands.rs`, `tests/editing.rs`: updated event admission expectations.
- `examples/terminal-probe.rs`: acquisition and restoration fault injection.
- `scripts/check-pty.py`: real framing, literal controls, saved bytes and shared undo.
- `docs/reference/terminal-input.md`: terminal paste reference.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 267 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test commands`: passed; thirty-eight tests, zero skips. Twelve
  paste cases cover literal control bytes, large chunked payloads, LF/CRLF,
  admission, typing boundaries, shared points, focus, phases and joined graphemes.
- `python3 scripts/check-pty.py --suite terminal`: passed; fifty-four cases,
  zero skips. New evidence covers enable/disable failures, a large actual framed
  Unicode paste, NUL rejection, exact BOM/mixed-newline saving, one-step undo,
  canceled prompt paste, literal command bytes and undo through a shared pane.
- `git diff --check`: passed.

## Risks and Follow-ups

Terminal framing and UTF-8 decoding belong to Crossterm. Literal-event fidelity
applies to the Unicode string it delivers; terminals without bracketed paste
reports cannot supply that event boundary. The terminal input reference documents
this boundary.
