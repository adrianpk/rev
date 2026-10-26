# Slice 44: Drive user journeys through a PTY

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `test/rev-44-terminal-journeys`
PR: `#44`
Integrated: `0e341d4a277e3b7b6159d1fa9c7978defe3d16b8`

## Purpose

Exercise connected editor workflows through real controlling PTYs, using the
normal executable, commands and filesystem rather than internal session access.

## Delivered Behavior

- `python3 scripts/check-pty.py --suite acceptance` builds the real editor and
  runs four named journeys. The existing terminal suite remains selectable and
  retains all 90 boundary cases. Acceptance builds no fault-probe executable.
- The document journey creates Unicode/CRLF source, saves it as Rust, checks
  foreground styling, bound/named motion, nested shared views, forward/backward
  search, literal replacement and three undo reports. It saves exact bytes and
  reopens them in another editor process with fresh history.
- The blocks journey combines linear copy/kill/yank and yank-pop with separate
  rectangle rows, cross-buffer insertion/padding and whole undo/redo. It checks
  actual BOM/separator bytes and preserves the distinct linear ring content.
- The workspace journey combines shared Rust views, tree rename/create/open/
  delete, updated-path saving, buffer inventory/switching and guarded close.
  Deleted open text remains available and reappears only after explicit saving.
- The failure journey retains decision and path focus through zero, one-cell,
  tiny and restored geometry. Paste and Return cannot grant consent. Canceled
  quit, failed required saving, continued shared editing and independent retry
  precede a genuinely accepted final exit.
- Each accepted exit verifies termios, bracketed-paste disable, visible cursor,
  attribute reset and alternate-screen restoration. New fixtures use only owned
  temporary files, bounded output and finite condition waits.

## Implementation Notes

- T44.1: `c4b87864c2fe712ad819f0884df2744294c55664` adds the acceptance selector, completed-
  prefix waits, accepted-exit checks and the first three connected journeys.
  All seven ordered checks pass: 448 native tests, three acceptance journeys
  and 90 terminal cases.
- T44.2: `2de738248d6db0dac5a2a83e65ea43f32b8e79cd` adds the resize/failure journey, dynamic
  echo-row waits, raw-mode checks after resize and documentation of suite commands
  and keyboard reports. All seven ordered checks pass: 448 native tests, four
  acceptance journeys and 90 terminal cases, with zero skips.
- Acceptance subprocesses use only the normal optional file argument and actual
  key/M-x handlers. No product option, debug command, runtime test seam, Rust
  source change or dependency is introduced.
- Frame waits reject intermediate C-x/ESC feedback before checking command
  results. Window counts, file bytes and selected-buffer transitions therefore
  follow consumed input rather than matching an earlier prefix frame.
- The four journeys use five real editor processes, including a separate reopen.
  Each process keeps the existing eight-second fixture deadline and one-MiB
  output bound. Waits use observable conditions; cleanup reaps only owned
  fixture processes after their checks finish.
- Geometry checks use real TIOCSWINSZ reports and validate emitted coordinates.
  Raw input remains acquired during resize; normal attributes are compared
  exactly after accepted exit. Filesystem checks compare bytes after publication
  and distinguish retained open text from disk deletion.
- Pure state and explicit key-phase variants stay in native Rust tests. PTY
  assertions concern reported input, completed rendering, actual files and
  terminal lifetime. Unsupported physical-chord claims are not converted into
  skipped semantic checks.

## Contracts Added or Changed

- The PTY runner accepts `--suite acceptance` and fails on a missing marker,
  rejected journey, deadline, byte mismatch or restoration failure. Its success
  count is derived from the executed journey list.
- The acceptance suite proves connected workflows through the real executable;
  terminal boundary regressions remain separately selectable.
- New keyboard evidence sends CSI-u C-/, legacy shared C-_ undo, NUL C-SPC,
  C-x u and compatible Escape-prefixed Meta reports. Documentation distinguishes
  those reports from physical terminal/multiplexer delivery.
- The how-to guide and documentation indexes expose the suite commands and
  describe bounded fixtures, success output and evidence interpretation.

## Files of Interest

- `scripts/check-pty.py`: acceptance selector, four journeys, completed-input
  waits, accepted-exit checks and resize/raw-state assertions.
- `docs/how-to/terminal-journeys.md`: running the suites and interpreting reports.
- `docs/how-to/index.md`, `docs/index.md`: discoverable validation guidance.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 448 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `python3 scripts/check-pty.py --suite acceptance`: passed; four journeys,
  zero skips, with real file-byte and terminal-state assertions.
- `python3 scripts/check-pty.py --suite terminal`: passed; 90 cases, zero skips.
- `git diff --check`: passed.

## Risks and Follow-ups

PTY evidence uses Linux controlling terminals and the recorded byte reports.
Physical chords also depend on terminal and multiplexer delivery, as documented
in the terminal-input reference. No physical-keyboard compatibility is inferred
from a successful injected report.
