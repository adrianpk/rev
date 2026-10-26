# Slice 42: Complete guarded multi-buffer session exit

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-42-session-quit`
PR: `#42`

## Purpose

Complete guarded session exit across all live modified buffers, preserving the
session when a decision is canceled or a required save does not complete.

## Delivered Behavior

- `C-x C-c` and `M-x save-buffers-kill-terminal` share one real command. Clean
  sessions exit immediately. Modified buffers appear once in stable identity
  order, including hidden and unnamed buffers and deleted-path warnings.
- Each named buffer asks y to save, n to discard for this exit, or C-g to cancel.
  Unnamed text uses the existing destination prompt. Occupied destinations and
  externally changed or removed files require separate fresh save consent.
- Canceling any quit/path/conflict stage keeps all buffers and windows. Earlier
  completed saves remain saved; earlier discard decisions do not mark text clean
  or carry into another exit attempt. Declining a save conflict aborts exit.
- A required save advances only after complete publication of current text.
  Read-only rejection, missing parents and published-but-uncertain completion
  retain the live session and actual save warning. A later independent save
  cannot resume a stopped exit.
- Shared windows retain their points, marks and history; one buffer produces one
  question regardless of its view count. Resize keeps the pending interaction
  through zero geometry, and cancellation preserves the remaining workspace focus.
- A fully accepted save/discard sequence releases the terminal guard. Real PTY
  journeys verify restored termios, alternate screen, bracketed paste and cursor.

## Implementation Notes

- T42.1: `28058fec1a0122eaba717c25089908cc793d12ff` implements the bounded per-exit buffer queue,
  typed save continuation, canonical command, focused cancellation, recovery
  tests and reference/changelog updates. Retained file/window journeys adapt
  to the new decision stage in `577e8c2c0b21e93ac8edd514ac11741b2879930f`. All seven
  ordered checks pass there: 437 native tests, five recovery tests and 84 PTY cases.
- T42.2: `d0674f73fcb8b185ceb9ed7e7d5bfd572f2deeb8` adds shared-view cancellation, removed
  destinations and deletion warnings, failures after earlier saves, named
  read-only rejection, held-suffix refusal and two representative PTY journeys.
  All seven ordered checks pass: 441 native tests, nine recovery tests and
  86 PTY cases, with zero skips.
- The quit coordinator owns a queue of unique canonical buffer identities,
  bounded by the current registry. Save/path/conflict stages own their remaining
  continuation; dropping a canceled or failed stage drops all remaining exit
  authority. No buffer is removed or falsely marked clean to implement discard.
- Saving uses the existing same-directory publication owner. The completion
  handler checks both publication status and the current buffer's unsaved state
  before continuing. No dependency, unsafe code or runtime fault switch is added.
- Native uncertainty evidence performs real filesystem writes and replacement,
  injecting only the directory-sync error through the existing Output boundary.
  It proves publication retention and stopped exit, not physical crash durability.
- The focused recovery suite completes in under a second locally. Two added PTY
  journeys use owned temporary files and the existing finite eight-second fixture
  deadlines, with condition waits and cleanup after checks finish.

## Contracts Added or Changed

- The discoverable exit command is `save-buffers-kill-terminal`, matching its
  required binding and behavior. All required decisions and saves precede exit.
- Discard consent belongs only to the current exit attempt. Cancellation retains
  text, history and windows; completed prior writes remain effective.
- Only complete saving of current text satisfies required exit saving. Uncertain
  publication remains visible and modified, even when replacement occurred.
- The focused-prompt reference, command catalog and existing Unreleased entry
  describe the delivered session flow and its failure/cancellation behavior.

## Files of Interest

- `src/session/quit.rs`: per-exit decision queue and cancellation owner.
- `src/session/save.rs`, `src/session/file.rs`: typed save/path continuations.
- `src/session/prompt.rs`, `src/session.rs`: focused cancel and decision rendering.
- `src/command.rs`: required canonical exit command name.
- `tests/recovery.rs`, `tests/recovery/quit.rs`, `tests/recovery/failures.rs`:
  native session lifecycle cases through real filesystem boundaries.
- `src/session/quit/tests.rs`: real publication with a later directory-sync failure.
- `scripts/check-pty.py`: sequential multi-buffer exit and failed-save journeys.
- `docs/reference/prompts.md`, `docs/reference/commands.md`, `CHANGELOG.md`:
  delivered commands and behavior.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 441 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test recovery`: passed; nine tests, zero skips.
- `python3 scripts/check-pty.py --suite terminal`: passed; 86 cases, zero skips.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
