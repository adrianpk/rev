# Slice 5: Decode prefixes and focused prompts

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-05-input`
PR: `#5`
Integrated: Rebase + Fast-forward into dev at `2332faed5849d40c33930c903f946b847a3da55a`.

## Purpose

Normalize keyboard events and route each key through one focused context,
with cancellable nested prefixes and editable argument/decision prompts.

## Delivered Behavior

- Recognizes pending `C-x` and `C-x r` sequences. Invalid suffixes consume the
  suffix, clear pending state, and produce a concise message. `C-g` cancels
  prefixes or prompts and restores editing focus.
- Dispatches the working exit sequence through a small static command enum.
  Its public registry contains `kill-terminal` and `keyboard-quit`, with exact
  name lookup. Runtime feedback shows pending prefixes, cancellation, and
  unbound/invalid sequences.
- Provides labeled argument and decision prompt APIs for command flows.
  Argument text supports grapheme-aware motion/deletion, Home/End, and Emacs
  `C-a`, `C-e`, `C-b`, `C-f`, `C-h`, and `C-d` editing. Enter returns the
  argument once; decisions require explicit `y` or `n` rather than an implicit
  affirmative answer.
- Bounds argument text to 4096 Unicode scalars. Oversized initial text,
  control characters, capacity exhaustion, and attempts to replace an active
  prompt return errors without changing the retained prompt.
- Ignores releases and non-key events. Prefix/command/confirmation repeats
  cannot cause duplicate dispatch. Ordinary text and cursor-edit repeats remain
  available; a held confirmation cannot leak into editing after completion.
- Routes prompt input separately from document input. Tests use the actual
  buffer edit owner to establish that prefix/prompt events preserve text,
  revision, and history, followed by one publication for resumed editing text.

## Implementation Notes

- T5.1: `a7a927c5b09b80ba7f343346d05f341e2f2eca56` adds input normalization,
  the static registry, focused prompts, runtime routing, and initial tests.
- T5.2: `aa5dc2231a84db61fafb0b6b027fb1d72aeb5472` completes eleven input
  integration tests and a real PTY nested-prefix recovery journey.
- `Input` owns one context and one optional prefix enum; prefix handling never
  blocks event polling or accumulates an unbounded chord string. Opening a
  prompt clears a pending prefix only after admission succeeds.
- A prompt owns a bounded UTF-8 string, independent of the canonical document
  rope. Its cursor is a byte offset at a grapheme boundary. Insertion or deletion
  repairs newly joined clusters before further editing.
- Argument and decision completion produce separate typed outcomes. Only an
  editing outcome reaches the document consumer in the isolation integration
  test. The runtime currently uses the implemented exit/cancel/prefix handlers;
  argument-taking command flows use the new prompt API as they are delivered.
- The terminal PTY suite is included after the focused command suite because
  this slice replaces the running session's input interpreter. It covers the
  actual executable's exit, prefix feedback/recovery, and terminal restoration.
- The source checker passes the required safe-Rust, five-parameter,
  three-level nesting, and ten-path cyclomatic limits.

## Contracts Added or Changed

- `Command` has one static list and exact lowercase, hyphenated names.
- `Input::handle` accepts Crossterm events and returns one typed outcome.
  Control-letter case is normalized; printable shifted characters retain their
  reported character. Unbound modified keys and raw control characters cannot
  become editing text.
- `Focus` distinguishes editing, argument, and decision contexts. Prompt
  acquisition is checked, cancellation discards pending input, and completion
  restores editing without forwarding the completing key.
- `Prompt` exposes immutable text, label, and cursor data for presentation.
  Input limits reject visibly rather than truncate. Non-key events preserve
  pending input state and never become editing commands.
- The terminal session delegates keyboard interpretation to `Input`, replacing
  its separate exit-only prefix decoder.

## Files of Interest

- `src/command.rs`: implemented command enum and public names.
- `src/input.rs`: focus, prefix state, event routing, and completion outcomes.
- `src/input/key.rs`: key normalization and repeat/release admission.
- `src/input/prompt.rs`: bounded argument text and grapheme-aware editing.
- `src/lib.rs`: runtime input integration and concise feedback.
- `tests/commands.rs`: eleven public-boundary integration tests.
- `scripts/check-pty.py`: real executable prefix recovery and terminal checks.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; thirty-six tests, including eleven input tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test commands`: passed; eleven tests, zero skips. Cases cover
  registry/binding identity, nested prefix cancellation and invalid suffixes,
  prompt focus, explicit decisions, key phases, normalized events, grapheme
  editing, scalar limits, checked prompt acquisition, and real buffer isolation.
- `python3 scripts/check-pty.py --suite terminal`: passed; seven cases, zero
  skips. The added journey observes nested-prefix feedback and cancellation,
  rejects a bare exit suffix, recovers after an invalid sequence, then exits
  normally with restored terminal state.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
