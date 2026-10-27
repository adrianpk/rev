# Slice 2: Add Emacs-style line navigation

Status: reviewing
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-02-goto-line`
PR: [#82](https://forge.adrianpk.com/adrianpk/rev/pulls/82)

## Purpose

Navigate to a one-based logical line through the existing native command,
keymap and argument owners while preserving canonical text and view ownership.

## Delivered Behavior

- `M-g g`, `M-g M-g` and `M-x goto-line` open the same line-number prompt.
  `M-g` alone remains a prefix and C-g retains cancellation.
- The existing configured prompt limit bounds input. Signed 64-bit decimal
  integers are admitted; surrounding whitespace is ignored. Invalid or
  overflowing input retains the draft for correction without moving point.
- An existing positive line goes to its start. Nonpositive values clamp to
  buffer start; requests beyond the last logical line clamp to exact EOF.
- Cancellation retains the complete selected view, including intended column
  and recenter state. Successful navigation reveals only that view and uses
  the ordinary shared mark, including in read-only buffers.
- Acceptance rejects an intervening source revision without moving the newer
  mapped view or altering its document state.

## Implementation Notes

The compiled registry and inherited editor map own the command and both aliases.
The pending prompt captures ViewId and source stamp. Acceptance verifies both
before deriving a checked line-byte-zero coordinate from the canonical rope.
A candidate view resets navigation goals and reveals the destination before
publication. Prompt entry itself preserves the retained view and navigation
sequence. No text transaction is published.

## Contracts Added or Changed

NAV01 adds the compiled command, bindings and bounded numeric argument.
NAV02 retains ordinary per-view movement, C-g and shared-buffer history owners.

## Files of Interest

- `src/command.rs`: compiled goto-line identity.
- `src/input/keymap/defaults.rs`: inherited M-g sequences.
- `src/session/goto.rs`: argument validation, ownership and checked destination.
- `tests/commands/goto.rs` and `tests/commands/goto/bounds.rs`: native dispatch,
  clamping, invalid-input recovery and complete state comparisons.
- `scripts/configuration_ui/navigation.py`: real controlling PTY journeys through
  `scripts/check-pty.py --suite goto-line`.
- `docs/reference/commands.md`, `bindings.md`, `prompts.md` and `config.toml`:
  delivered command behavior and both inherited aliases.

## Validation

- PASS: `cargo test --test commands goto::` (eight native dispatch, boundary,
  cancellation, shared/read-only, stale-source, invalid-number and prompt-limit
  regressions).
- Logical-bound cases cover empty/single-line, LF/CRLF, Unicode, final empty
  lines and both signed extremes, comparing exact positions and document state.
- PASS: `python3 scripts/check-pty.py --suite goto-line` (two isolated controlling
  PTYs: 80 Unicode/CRLF logical lines without final newline, and an empty file).
  Both exercise native aliases, M-x, 64-bit overflow and correction, clamping,
  actual cursor reveal, cancellation, terminal restoration and unchanged bytes.
- Invalid-number cases include empty/sign-only, fractional/internal-space,
  nondecimal/Unicode digits and both overflowing signs. A configured three-scalar
  prompt rejects oversize paste wholly and accepts a bounded retry.
- The reference sample includes both new aliases; the acceptance evidence guard
  retains its exact schema, palette and optional-example assertions and expects
  111 defaults instead of 109, accounting for these two inherited bindings.

The final clean PR HEAD is validated by the controller with this exact canonical
five-command list; successful evidence is stored outside the repository:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test commands --test editing --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

The controller also records this focused list separately for the same final HEAD:

- `cargo test --test configuration`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-pty.py --suite goto-line`
- `git diff --check`

The canonical PR list preserves the configured prefix, including its final
whitespace check.

## Risks and Follow-ups

None recorded.
