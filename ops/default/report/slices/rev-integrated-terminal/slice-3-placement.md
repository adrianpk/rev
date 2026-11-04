# Slice 3: Buffer Placement and Multiple Sessions

Status: drafting
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-03-placement`
PR: pending

## Purpose

Present the same live terminal in the lower panel or ordinary editing panes,
with multiple independent sessions and checked native focus/geometry ownership.

## Delivered Behavior

`terminal-new-buffer` creates a terminal directly in the selected editing pane.
`terminal-move-to-buffer` and `terminal-move-to-panel` relocate the same typed
buffer and process. They preserve child working state, retained output and the
alternate screen. Geometry rejection retains the original valid placement.
C-g returns to the source view, preserving unsaved text and the live session.

`terminal-new` creates an independent lower-panel session. Stable names start
with `terminal`, then `terminal-2` and subsequent identities under normal native
name conflict handling. `terminal-list` filters the ordinary inventory to
retained terminal buffers; listing never launches a child. `terminal-rename`
requires a unique 1-256 byte name without control characters and preserves
buffer/session identity. The live instance default remains unchanged when
another independent session is created. Hidden peers continue receiving output.

## Implementation Notes

Native views bind the existing terminal-backed buffer. Source return views are
retained separately; a source revision change requires a checked fresh rebind.
Cell rendering uses clipped column ranges so terminal content and status cannot
replace neighboring panes. A full-width terminal row avoids duplicate painting.
Process, emulator and resource admission remain under the existing registry.

## Contracts Added or Changed

Main-area terminal focus has the same source-admission guard as panel focus.
Pane detachment and visibility do not close the session. Native source reveal
does not overwrite terminal viewport state. Other pane views share the one
session/emulator and retain their own ordinary view state.

## Files of Interest

- `src/session/terminals/placement.rs`: checked placement and source return.
- `src/session/terminals/render.rs`: native pane and panel terminal surfaces.
- `src/display/terminal.rs`: clipped structured cell ranges.
- `tests/integrated_terminal/panel.rs`: real-child native placement fixtures.

## Validation

Development command
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel:: -- --test-threads=1`
passed seven scenarios, including direct-buffer entry, bidirectional moves,
alternate-screen/process identity, rejected geometry and original source guards.
The independent-session scenario covers hidden peer output, native filtered
selection, rename conflict rejection and original default process reuse.
Final exact-head evidence belongs to the controller's ordered Slice 3 receipt.

## Risks and Follow-ups

Directory entry and complete pane-focus acceptance remain active Slice 3 tasks.
Complete live-input modes and retained
output copy navigation remain in their recorded later slices.
