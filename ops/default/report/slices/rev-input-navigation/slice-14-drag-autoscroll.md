# Slice 14: Extend pointer selection with bounded autoscroll

Status: reviewing
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-14-drag-autoscroll`
PR: pending

## Purpose

Extend the canonical linear region beyond the source pane's visible text through
bounded active-drag work in the existing native input owner.

## Delivered Behavior

A captured outside drag scrolls only its revision-qualified source view and maps
its point to a checked source boundary at the near edge. Reentry pauses the
work; release retains the ordinary region and removes capture. Source bytes,
revision and history remain unchanged.

## Implementation Notes

The capture holds one current pointer position and one deadline. Checked native
coordinates clamp the edge row to the document and the column to the source text
partition; other panes and decorations never supply selectable text. Every step
revalidates the source view and revision. A blocked document edge retires the
deadline. Invalid source stamps, missing ownership and failed endpoint validation
retire capture before any later report can reuse it. The existing host deadline
entrypoint performs at most one step.
The native input owner waits for the earlier Escape or drag deadline, blocks
indefinitely when neither exists, and rechecks ready input after a timeout.
Cancellation/release is dispatched before due pointer work. Motion updates one
position without moving its deadline; a late wake performs one step and starts
the next 100 ms interval from that wake. Each step is bounded by the lesser of
the native factory scroll step (three) and the current content height. Existing
32-event/four-millisecond input batch limits remain unchanged.

## Contracts Added or Changed

PTR04 extends the native mark/point region across the source viewport edge.
PTR05 retains finite source ownership and removes work when capture ends.

## Files of Interest

- `src/session/pointer/autoscroll.rs`: captured edge, checked source projection and deadline service.
- `src/session/pointer/selection.rs`: edge tracking and capture retirement.
- `src/session/events.rs`: controlled-clock native host entrypoints.
- `tests/pointer/autoscroll*.rs`: controlled time, source/Unicode bounds and cancellation.
- `src/session/events/tests.rs`: actual native polling boundary and idle behavior.
- `scripts/configuration_ui/pointer_autoscroll.py`: real SGR events, byte fidelity and owned PTYs.

## Validation

Development `cargo test --test pointer` passed 42 groups, including controlled
100 ms source-edge extension, pause/release and upward scrolling from a lower
pane without changing its peer. `python3 scripts/check-source.py` passed the
unchanged safe Rust, parameter, nesting and complexity limits.

Development `cargo test --lib --test pointer` passed 13 library and 46 pointer
groups. It covers one-row height bounds, 1,000 reports without deadline debt,
late wakes, document limits/empty text, due-time input cancellation and injected
native poll-boundary release priority. Focused Clippy and the unchanged source
quality guard passed.

Development `cargo test --lib --test pointer autoscroll` passed ten focused
groups: nine integration groups plus stale view/document/invalid-point fault
injection. It adds read-only tab widths 1/4/8/32 on both gutter sides with
horizontal offsets, whole Unicode/CRLF endpoints, direction reset, shared and
different-buffer peers, host mode-owner cancellation and no stale due work.
`cargo clippy --lib --test pointer -- -D warnings` passed.

`python3 scripts/check-pty.py --suite pointer-autoscroll` passed two owned native
sessions. Real SGR drags trigger the native wait loop, pause inside, stop on
release, retain the native region, and preserve exact BOM/CRLF bytes through
copy/kill/save/undo/redo. An upward drag beyond a lower pane retains source
focus and its independent peer; reported focus loss and resize retire pending
work before delayed input. Idle samples after pause, release, focus loss and
resize emit no terminal output and consume no process CPU ticks.

The PTYs retain the existing eight-second deadline, 1-MiB output ceiling,
owned HOME/XDG roots, 32-entry/64-KiB snapshot bounds, raw-input assertion,
terminal restoration, child reaping and descriptor closure. Snapshots return to
their original bytes, no preference directory is created, and no observed
stage writes OSC 52. The warm fixture takes under two seconds excluding build.

Final canonical and focused evidence is recorded through the controller on the
clean candidate after task and PR-reference commits. Required ordered commands:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test commands --test display --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused final commands:

- `cargo test --lib --test coordinates --test editing --test tree`
- `python3 scripts/check-pty.py --suite pointer-focus`
- `python3 scripts/check-pty.py --suite pointer-scroll`
- `python3 scripts/check-pty.py --suite pointer-selection`
- `python3 scripts/check-pty.py --suite pointer-autoscroll`
- `git diff --check`


## Risks and Follow-ups

Ordinary reporting and typed mouse controls remain assigned to Slice 15.
