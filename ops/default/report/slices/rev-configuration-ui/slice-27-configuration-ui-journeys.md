# Slice 27: Exercise combined terminal workflows

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `test/rev-config-ui-27-configuration-ui-journeys`
PR: [#76](https://forge.adrianpk.com/adrianpk/rev/pulls/76)

## Purpose

Exercise startup preferences, command/file completion and shared panel behavior
through connected real terminal workflows with bounded, explicitly owned fixtures.

## Delivered Behavior

- `python3 scripts/check-configuration-ui.py --suite terminal` builds the editor
  and existing terminal fault probe and runs eight controlling-PTY sessions with
  isolated HOME/XDG paths. Focused `--case` selection retains failure visibility.
- Configured RGB and NO_COLOR sessions assert complete visible rows and cursor
  columns for literal tabs, configured foreground/background/cursor and candidate
  colors, custom editor/minibuffer keys, M-x candidate completion/selection and
  separate command-history recall/draft restoration.
- The same sessions use M-x find-file, directory and Unicode-name completion,
  exact selected-file acceptance and a freely typed new filename despite matching
  suggestions. Failed ingress retains an editable draft, query cursor and source
  document. Observation, completion, visiting and editing create no file; an
  explicit save alone publishes the expected BOM/CRLF bytes.
- Missing and invalid configuration sessions prove silent missing sources or a
  visible diagnostic, default tab cells/cursor columns, default palette/cursor,
  an unbound custom chord and default new-file indentation/format bytes. Complete
  fixture snapshots retain the original source/config and passive XDG roots.
- Workspace RGB and NO_COLOR sessions assert a 12-column tree, configured 0.6
  nested split shares and three shared editing panes. Tree expansion uses its
  configured alias. Shared edit/undo reaches every pane without saving.
- Distinct Go and Rust buffer tab policies coexist across shared windows.
  Horizontal reveal, page overlap, EOF and beginning boundaries retain peer views.
  A selected path prompt survives zero, one-cell, narrow and restored geometry;
  cancellation restores the exact retained canvas and cursor. Window cycling
  visits the three editing panes without entering the tree.
- Existing terminal-owner output-error and unwind modes exit with expected codes
  after actual cursor/text color emission. Normal and error paths verify ordered
  paste disable, visible cursor, text reset, cursor reset where enabled, alternate
  screen release and restored termios.
- Every case closes and reaps only its owned child and verifies both PTY file
  descriptors are closed. Eight-second session deadlines, the existing 1-MiB
  output limit, at most 32 snapshot entries and 64 KiB per fixture file bound work.
  The runner reports measured terminal elapsed time separately from compilation.

## Implementation Notes

- T27.1: `1389238803bfea95120cd4632784c3e354209100` connects configured and
  fallback startup to command/history and Unicode path/new-file journeys, reusing
  the existing controlling-PTY owner and completed-frame decoder.
- T27.2: `a9232206e549dad33269bd0740d366c9f2a97385` combines tree/nested
  geometry, shared policies, boundaries and exact prompt-resize cancellation.
- T27.3: `9d431003f671edead10fa8529ad8f741dc6456af` adds output-error/unwind checks,
  ordered cleanup assertions, child/descriptor verification, retained ingress
  errors, bounded fixture capture, elapsed reporting and operating documentation.
- The new concrete helper modules separate fixture/frame operations, connected
  prompt journeys, workspace journeys and terminal lifetime. They use Python's
  standard library and existing test helpers, without runtime source changes.
- Frame assertions decode complete padded rows, account for wide/combining fixture
  text and inspect actual cursor positions. Selection color assertions supplement
  visible candidate/order checks. Split geometry and per-buffer tab stops are
  asserted through rendered content and status positions.
- Input steps wait for observable completed redraws, with selection/cursor/layout
  predicates where a prefix frame could otherwise match early. Resize follows a
  completed selection event; no fixed sleeps or enlarged session deadlines.
- Fixture snapshots distinguish unchanged observation from explicit owned saves;
  the two workspace cases publish nothing. Cleanup remains in finally blocks,
  including failed assertions, and never selects an unrelated process.
- Python AST checks passed for syntax, at most five parameters per function and
  control nesting at most two. No native quality limits or gate definitions change.

## Contracts Added or Changed

- CFG22: actual executable effects from startup palette/tab/key preferences,
  fallback and new-file defaults, independently of schema/sample consistency.
- UI01: vertical command candidates, configured selection/completion aliases and
  separate session-history navigation with visible query/cursor ownership.
- UI02: shared tree/editing/completion geometry, nested ratios and retained views
  through zero/tiny/growing terminal sizes.
- UI03-UI04: directory/Unicode completion, selected versus typed acceptance,
  retained failures and no filesystem creation by observation or visits.
- Baseline R03-R07: shared text/history and canonical file/terminal ownership stay
  intact across the combined interactions.

## Files of Interest

- `scripts/check-configuration-ui.py`: terminal suite and focused case selection.
- `scripts/configuration_ui/support.py`: isolated fixtures, completed frames,
  resource bounds and verified cleanup through the existing PTY owner.
- `scripts/configuration_ui/journeys.py`: connected startup/command/path cases.
- `scripts/configuration_ui/workspace.py`: shared pane/tree/resize cases.
- `scripts/configuration_ui/lifetime.py`: existing terminal-owner failure modes.
- `docs/how-to/terminal-journeys.md`: commands, cases, evidence and resource bounds.

## Validation

Successful focused development commands:

- `python3 scripts/check-configuration-ui.py --suite terminal --case configured`: RGB and NO_COLOR sessions passed; actual policies, command/history and path/new-file behavior preserved exact fixture bytes except the explicit save.
- `python3 scripts/check-configuration-ui.py --suite terminal --case missing`: silent missing config, default effects and passive XDG roots passed.
- `python3 scripts/check-configuration-ui.py --suite terminal --case rejected`: invalid-candidate fallback, default effects and retained source/config passed.
- `python3 scripts/check-configuration-ui.py --suite terminal --case workspace`: RGB and NO_COLOR shared-tree/panel/policy/boundary/resize sessions passed without disk changes.
- `python3 scripts/check-configuration-ui.py --suite terminal --case io`: expected output-error exit, ordered terminal restoration and owned resource cleanup passed.
- `python3 scripts/check-configuration-ui.py --suite terminal --case unwind`: expected unwind exit, ordered terminal restoration and owned resource cleanup passed.
- Python AST parsing and function-bound checks passed for all five runner/helper modules.

All seven commands passed on clean PR head `27629069653ee043ff0cb5be935fa16f5353f76d` through controller
validation `LKBX5GARCR45ND5OOC5K46TYG5`. Canonical PR #76 integrated that exact
head into dev by Rebase + Fast-forward. The terminal suite passed eight sessions
with zero skips in 3.063 seconds excluding compilation; 119 native tests passed.
The exact ordered commands were:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `python3 scripts/check-configuration-ui.py --suite terminal`
- `cargo test --test configuration`
- `cargo test --test path_completion`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

The PTYs prove supported reports and emitted terminal behavior; physical key
interception and true-color fidelity remain terminal/multiplexer capabilities.
The two fault cases exercise the existing terminal owner through its dedicated
probe. The other six sessions use the actual editor. Native state variants,
bounded-work measurements and final integration evidence retain their approved
subsequent checks.
