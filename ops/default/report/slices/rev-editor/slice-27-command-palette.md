# Slice 27: Run discoverable commands through M-x

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-27-command-palette`
PR: `#27`

## Purpose

Discover and invoke implemented commands through focused M-x entry, with fixed
name matching, completion and session history using the existing shared handlers.

## Delivered Behavior

- M-x opens labeled command entry with up to three inline candidates. Lowercase
  hyphenated names match by prefix first and then substring, alphabetically
  within each group. The catalog contains 42 working commands.
- Tab completes the first candidate and cycles the original filtered set.
  Editing restarts matching. Return invokes an exact name or the first matching
  candidate. Empty input invokes nothing; unknown names report an error and
  return to editing. No-match Tab retains command focus.
- Up/Down and C-p/C-n navigate session-local command history and restore the
  draft query after the newest entry. History retains up to 64 distinct accepted
  names in recency order. Canceled and unknown input do not enter history.
- Query edits and single-line paste stay in the bounded prompt. C-g cancels
  without invoking the pending name or inserting query text. Releases and held
  Return cannot duplicate invocation. Tiny/zero resize retains command entry.
- Every accepted command reaches the same handler used by bindings, including
  file/buffer argument continuation, window commands, edits, undo and redo.
  Read-only admission remains authoritative. M-x entry preserves the current
  yank capture for named yank-pop; cancellation ends that continuity.

## Implementation Notes

- T27.1: `57f42e8a1fdc3ad80d2d89c725d75e9f32f3d22b` adds fixed catalog
  matching, command focus, completion/history, shared dispatch, initial native
  evidence and command reference.
- T27.2: `3e4e30cbbd9c1bee8d8a5c9135ad13ad0b194c11` exercises all 42
  named handlers, fourteen real binding/name paths, unknown and empty input,
  cancellation, history, arguments, phases, read-only admission and geometry.
- The input owner retains bounded query, completion-cycle and history-navigation
  state. Candidates derive only from the static command enum. Accepted names
  become the same command outcome as a binding; there is no command interpreter.
- Command prompt closure precedes handler invocation so existing focused argument
  owners can acquire input. The M-x wrapper closes typing continuity while the
  eventual command owns kill/yank continuity.
- Candidate hints use the existing clipped prompt surface. No separate overlay,
  thread, timer or dependency is introduced. The two new PTY journeys use existing
  bounded frame waits and stop only owned fixtures after their checks.

## Contracts Added or Changed

- `Command::ExecuteExtended` and its `execute-extended-command` name open M-x;
  `Command::matching` defines deterministic fixed-name ranking.
- `Focus::Command`, `Input::commands` and `Input::palette` expose focused command
  entry. `Palette` owns prompt text, candidate cycling and draft restoration.
- Command history records accepted invocations, including handlers that later
  refuse the operation; it is neither document history nor persistent storage.
- `docs/reference/commands.md` records matching, invocation, completion, history,
  cancellation and the complete implemented catalog. README and reference index
  link it; `CHANGELOG.md` describes the delivered user capability.

## Files of Interest

- `src/command.rs`: static catalog, public names and deterministic matching.
- `src/input/bindings.rs`: M-x binding through normalized Meta input.
- `src/input.rs`, `src/input/palette.rs`: command focus and completion ownership.
- `src/input/palette/history.rs`: bounded history navigation and query draft.
- `src/input/paste.rs`: literal paste stays in command focus.
- `src/session.rs`, `src/session/extended.rs`: shared dispatch and candidate hints.
- `tests/commands/palette.rs`: thirteen native cases across registry and handlers.
- `scripts/check-pty.py`: real command, argument, save/reopen and exit journeys.
- `docs/reference/commands.md`: durable command-entry contract and catalog.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 280 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test commands`: passed; fifty-one tests, zero skips. Thirteen
  palette cases cover ranking, cycling, history, cancellation, focus, arguments,
  all advertised handlers, binding parity, yank continuity and admission.
- `python3 scripts/check-pty.py --suite terminal`: passed; fifty-six cases,
  zero skips. New journeys drive actual M-x/query/Tab/history input, tiny/zero
  resize, unknown names, cancel, named windows/edit/undo/redo, file arguments,
  canceled write, exact BOM/CRLF save, canonical reopen and accepted named exit.
- `git diff --check`: passed.

## Risks and Follow-ups

History lasts only for the current session. Physical M-x availability follows
the documented terminal report policy. These boundaries are recorded in the
command and terminal input references.
