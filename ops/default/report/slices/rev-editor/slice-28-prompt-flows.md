# Slice 28: Complete focused argument and cancel flows

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-28-prompt-flows`
PR: pending

## Purpose

Complete focused command, argument and decision lifetimes so cancellation and
errors retain the selected editing context without deferred file or close effects.

## Delivered Behavior

- M-x entry participates in the session's pending interaction state. C-g cancels
  it without resetting the intended vertical column or recenter cycle. Accepted,
  empty and unknown names retire command entry before the next editing command;
  accepted handlers can acquire their own argument or decision stage.
- Tab in a path argument retains its exact draft and displays `Enter a path;
  C-g cancels`. Buffer-name Tab continues through live-name completion. Successful
  prompt edits clear outdated input or completion feedback while retaining focus.
- Find/write/save and buffer-close stages retain their focused owners through
  editing, fresh acceptance and resize. Query/path text never enters the document.
  Invalid, pasted or held decision input cannot grant consent or skip a stage.
- C-g at command, buffer-name, close-question, destination or overwrite stages
  drops pending authority. Source text, qualified points, selected view and history
  remain intact. A later independent save retains the buffer and shared panes.
- Failed ingress, destination publication or required close-save returns input to
  editing with the source open. Emacs prompt movement/deletion operates on the
  path draft, and a subsequently accepted Unicode path reaches normal file ingress.

## Implementation Notes

- T28.1: `6f6cb8a889335c8e14596e81717910f557523c85` adds explicit pending
  command-entry lifetime, retirement before dispatch, focused completion routing,
  feedback recovery, three reproduced regressions and the prompt reference.
- T28.2: `716a3022b6edb898e41fe0c6bf6cef5f940d2d42` adds cancellation
  across all current stages, exact shared-view/history snapshots, fresh key-phase
  admission, file failure/recovery, edited Unicode paths and two real PTY journeys.
- The initial regressions reproduced navigation loss after canceled M-x, an
  internal buffer-completion diagnostic in path focus and stale argument feedback.
  All three pass after the focused lifetime changes.
- The session retires command entry when its input context returns to editing,
  preserving C-g's existing pending-interaction cancellation path. File, buffer,
  save and close owners continue to perform their existing checked operations.
- Native fixtures use owned temporary files and full state comparisons. The PTY
  journeys use existing bounded frame waits, real resize reports and exact disk
  byte checks. Only owned fixtures stop after their checks finish.

## Contracts Added or Changed

- The internal pending state now includes command focus, alongside existing
  argument, buffer, inventory, save and close stages.
- Accepted command outcomes retire transient entry before shared dispatch;
  rejected/empty command completion retires it before reporting the message.
- Completion routing distinguishes live buffer-name arguments from path entry.
  Successful command and argument edits clear feedback for the previous input.
- `docs/reference/prompts.md` defines stage sequences, Emacs prompt editing,
  fresh decisions, failure recovery and cancellation. The command reference and
  reference index link it; `CHANGELOG.md` records the observable corrections.

## Files of Interest

- `src/session.rs`: pending command stage and focused outcome routing.
- `src/session/prompt.rs`: stage retirement and argument completion ownership.
- `src/session/extended.rs`: M-x acquisition and prompt feedback recovery.
- `tests/commands/flows.rs`: eleven native focused-flow cases with owned files.
- `scripts/check-pty.py`: real path editing/cancel and nested close/save journeys.
- `docs/reference/prompts.md`: durable focused interaction contract.
- `docs/reference/commands.md`, `docs/reference/index.md`: reference navigation.
- `CHANGELOG.md`: cancellation navigation and argument feedback fixes.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 291 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test commands`: passed; sixty-two tests, zero skips. Eleven
  focused-flow cases cover command retirement, all cancellation stages, shared
  views, fresh acceptance, failure recovery, Unicode path editing and geometry.
- `python3 scripts/check-pty.py --suite terminal`: passed; fifty-eight cases,
  zero skips. New journeys verify retained vertical/recenter navigation, actual
  Emacs path editing, feedback clearing, canceled find/write, each nested close
  stage, resize during consent, exact saved bytes and later save without closing
  shared text. Terminal restoration is checked after accepted exit.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
