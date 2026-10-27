# Slice 11: Focus panes and place point by pointer

Status: delivered
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-11-pointer-hit-focus`
PR: [#91](https://forge.adrianpk.com/adrianpk/rev/pulls/91)
Integrated: `be336d69ef607bec7fb4a5ebdfed1f6aa5bdcb4a`

## Purpose

Resolve native primary presses against the current composed workspace and place
point through the existing checked movement owner.

## Delivered Behavior

- Primary presses select the owning editing pane and resolve its visible text
  through the same gutter partition used by rendering.
- Number fields and status rows can focus their pane without changing point.
  Shared separators, prompts and outside cells do not identify document text.
- Complete tab, wide, combining, control and ZWJ glyph cells map to the cluster
  start. Current tab policy, top/left offsets and either gutter side are honored.
- Exact EOL and final-empty-row boundaries remain selectable. Trailing fill,
  beyond-EOF rows and clipped glyph fragments focus without moving point.
- Tree and editing focus use existing focus owners; active prompt owners retain
  their input. Shared and different-file panes retain independent points.
- Resize, pane deletion and suspended geometry resolve fresh live bounds.
  Active marks survive point movement, including in read-only buffers.
- A press ends an adjacent typing group without adding an undo item.
  Undo/redo preserve the ordered edits around the new point.

## Implementation Notes

Hit destinations are disposable projections of current composition, view and
buffer revision. No hit map is retained. Checked movement validates the target
point, resets navigation and reveals it through the ordinary motion owner.
Workspace reveal is synchronized before hit resolution, matching resize drawing.

## Contracts Added or Changed

PTR02 identifies pane focus. PTR03 maps visible cells to checked source points
without treating decorations or padding as source positions.

## Files of Interest

- `src/pointer/hit.rs`: current composed pane ownership.
- `src/view/hit.rs`: checked gutter/text projection.
- `src/session/pointer.rs`: native press dispatch and existing focus transitions.
- `src/session/motion.rs`: shared checked movement owner.
- `src/coordinates/hit.rs`, `tests/pointer/cells.rs`: checked painted-cell boundaries.
- `tests/pointer/focus.rs`, `tests/pointer/workspace.rs`: pane/tree focus,
  prompt isolation, geometry lifecycle, active regions and typing history.
- `scripts/configuration_ui/pointer_focus.py`: actual SGR reports through the
  ordinary binary, current rendered cursor and owned terminal cleanup.
- `docs/reference/terminal-input.md`: native hit semantics and activation boundary.

## Validation

Development checks passed:

- `cargo test --test pointer`: twenty groups passed, including shared-pane point
  placement, frame cursor agreement, decorations, literal glyph/EOF/clipping
  mappings, tab widths 1/4/8/32, nonzero origins and rejected stale/foreign or
  overflowing candidates. Workspace cases cover tree prompts, different files,
  nested right/relative gutters, changed/closed/tiny panes, scrolled text,
  read-only active regions and typing undo/redo. Prompt/decision/inventory
  owners retain rendered drafts, cursor and point. Canonical bytes are preserved.
- `cargo test --test pointer workspace::tree_focus`: tree prompt cancellation
  retains tree focus before a later press returns to editing.
- `cargo clippy --test pointer -- -D warnings`: focused native lint passed.
- `python3 scripts/check-pty.py --suite pointer-focus`: two owned sessions passed
  for SGR tabbed Unicode point mapping, split/tree focus, number-field focus,
  preserved goto-line draft, resized ownership, tiny suspension, restoration
  and byte fidelity. The ordinary binary receives actual SGR reports directly;
  factory reporting remains inactive.
- `python3 scripts/check-source.py`: unchanged safe-Rust and complexity limits.

PTY fixtures retain the existing eight-second deadline, 1-MiB output bound,
owned HOME/XDG paths, raw-mode assertion, unchanged filesystem snapshot, full
terminal restoration, child reaping and descriptor closure. Idle observation
checks zero output and CPU ticks over 120 ms. No runtime queue or timer is added.

Development fixture corrections use the file-open status as readiness (opening
a file clears the startup echo), and recognize the renderer's two completed
SGR-reset rows when a tiny frame hides the cursor. The older resize helper's
bare-echo terminator was unsuitable for that frame. Tiny cell bounds, hidden
cursor, retained points, input barrier and resource limits remain asserted.

Canonical evidence `IBT4WLILH3CAXCVW2IBSGQN2V6` passed all five ordered
checks and 289 native cases on clean head
`be336d69ef607bec7fb4a5ebdfed1f6aa5bdcb4a`, integrated into dev through PR #91:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test gutter --test commands --test display --test panels --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused evidence `RCVKDXA4MARYXGAGBUIJICY374` passed 35 library/coordinate
cases and fifteen owned PTYs on that same head:

- `cargo test --lib --test coordinates`
- `python3 scripts/check-pty.py --suite pointer-lifetime`
- `python3 scripts/check-pty.py --suite pointer-focus`
- `git diff --check`


## Risks and Follow-ups

Factory reporting remains disabled until the recorded handler/settings slices
integrate. Wheel routing, drag capture and autoscroll belong to their recorded
later slices. This slice covers primary press focus and point placement.
