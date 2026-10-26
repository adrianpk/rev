# Slice 1: Own terminal lifetime

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-01-terminal`
PR: `#1`
Integrated: Rebase + Fast-forward into dev at `a37fb53316149966ed3d7457fb6cd47a94561035`.

## Purpose

Establish executable terminal ownership and the required validation entry points
for the editor.

## Delivered Behavior

- Starts the `rev` terminal session with raw input, an alternate screen, a hidden
  hardware cursor, and deliberately flushed buffered output.
- Exits through `C-x C-c`, restoring cursor visibility, the original screen,
  and the previous terminal attributes. Release reports do not dispatch twice;
  `C-g` cancels the exit prefix and `C-q` does not quit.
- Restores acquired state after partial initialization, ordinary session or
  output errors, and recoverable unwinding. Startup errors return a nonzero
  status with operation context.

## Implementation Notes

- T1.1: `598eae082ff24fb65d9a0ae321394fa3d7c3803a` introduces the Rust 2024
  executable/library package, six exact dependency pins, Cargo.lock, and the
  terminal owner.
- T1.2: `e54df2cc72c8916b9fbe24dafa6ed8552b31f727` adds the source checker,
  PTY probes, and executable slice/full-gate scripts.
- `Terminal` owns raw mode separately from screen and cursor obligations.
  An attempted output transition retains its restoration obligation because a
  failed write can already have emitted part of an escape sequence. Restoration
  attempts all obligations independently and retains failed ones for drop to
  retry. Normal error returns preserve both session and cleanup errors.
- The source checker uses the pinned Rust grammar to distinguish comments and
  literals from code, check crate roots and parameters including receivers,
  and count control-flow branches, match alternatives/guards, short-circuit
  operators, and fallible `?` paths. It does not expand macros; Rust's
  `forbid(unsafe_code)` also applies to compiled macro expansions.
- Measured application cyclomatic complexity is at most 7, with logical nesting
  at most 2. The probe/checker maximum is 9; every checked function has at most
  five parameters. Identifiers use concise Rust/editor terms; comments explain
  restoration and test contracts.
- The full-gate entry point requires the complete acceptance PTY suite,
  performance runner, and acceptance-evidence checker. Missing suites or
  scripts fail rather than being skipped.

## Contracts Added or Changed

- Every first-party crate root declares `#![forbid(unsafe_code)]`.
- `Terminal::acquire` owns raw input before presentation entry; `enter` acquires
  presentation obligations, `writer` borrows buffered output, and `restore`
  attempts explicit cleanup. Drop handles early returns and unwinding.
- `scripts/check-slice.sh` runs the canonical check prefix, an optional focused
  command supplied as subprocess arguments, and `git diff --check` last.
- `scripts/check-all.sh` defines the required combined gate without conditional
  omissions. Its execution belongs to the controller after integration.

## Files of Interest

- `Cargo.toml`, `Cargo.lock`: package and exact dependency selection.
- `src/terminal.rs`: acquisition, buffered output, and restoration ownership.
- `src/lib.rs`, `src/main.rs`: session entry, real exit sequence, and errors.
- `scripts/check-source.py`, `examples/source-check.rs`: source rules and
  adversarial rule fixtures.
- `scripts/check-pty.py`, `examples/terminal-probe.rs`: real controlling PTYs
  and isolated failure injection outside the product executable.
- `scripts/check-slice.sh`, `scripts/check-all.sh`: validation entry points.

## Validation

Successful slice commands, in canonical order; final clean-HEAD evidence is
recorded through `looper validate` after the PR-reference follow-up commit.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; two table-driven tests cover five exit-input
  scenarios and thirteen source-rule fixtures.
- `python3 scripts/check-source.py`: passed.
- `python3 scripts/check-pty.py --suite terminal`: passed; six real-PTY cases,
  zero skips. Checks compare complete terminal attributes before/after exit and
  observe alternate-screen/cursor restoration bytes. The raw-only failure emits
  no screen or cursor transitions.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
