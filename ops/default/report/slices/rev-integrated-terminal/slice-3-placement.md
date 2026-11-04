# Slice 3: Buffer Placement and Multiple Sessions

Status: reviewing
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-03-placement`
PR: `#250`

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

Splitting a terminal buffer mirrors the same process/emulator. Focus admits
geometry before publishing its resize authority; another view retains its own
bounded history offset. Moving one attachment leaves other mirrors intact.
Small screens suspend presentation/input while retaining the admitted grid and
recover on resize. Native pointer focus selects the terminal attachment without
borrowing a source point. The lower panel retains one selected session.

`terminal-open-here` uses the source resource's containing directory, or an
existing terminal's admitted creation context, without guessing its current cwd.
`terminal-open-other-pane` uses the next pane in native order, preparing a right
split when only one exists. It selects the live terminal without another child;
when none exists it creates a terminal directly in that prepared pane. Failed
split, launch or geometry admission retains the original layout and focus.

## Implementation Notes

Native views bind the existing terminal-backed buffer. Source return views are
retained separately; a source revision change requires a checked fresh rebind.
Cell rendering uses clipped column ranges so terminal content and status cannot
replace neighboring panes. A full-width terminal row avoids duplicate painting.
Process, emulator and resource admission remain under the existing registry.
Per-view terminal navigation is distinct from source point/top/left. Native
embedding viewport changes check retained history before publication. Live
focus is the sole resize authority; other attachments clip the admitted grid.
Source-return records are bounded by live view identities, and creation context
records are bounded by the session registry. Dependencies and resource limits
remain unchanged.

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

Task commits: T3.1 `e039a166383d`; T3.2 `5b3c81588ab0`; T3.3 `c861096dfe81`,
with inventory error-boundary correction `6a4f108e2343`.

## Validation

Development command
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel:: -- --test-threads=1`
passed nine scenarios, including direct-buffer entry, bidirectional moves,
alternate-screen/process identity, rejected geometry and original source guards.
The independent-session scenario covers hidden peer output, native filtered
selection, rename conflict rejection and original default process reuse.
Pane scenarios cover shared child identity, separate visible history offsets,
focus-owned geometry, tiny-screen rejection/recovery and directory/other-pane
entry. Final fixtures additionally require child cwd changes to survive moves
and other-pane geometry failures to retain the original layout.
Focused clean-head receipt `P2LCRJX3SCAVQOTP7KRQVZESLI` passed all nine panel
cases and fifteen selected source files on `6a4f108e2343333e8e087cd2d52d216b5ab738ba`.
Final exact-head evidence belongs to the controller's ordered Slice 3 receipt after
this PR-reference follow-up. Its identity stays outside the report to preserve
the final clean candidate.
Focused attempt `ZMPDHASBYTLDAGM3I6A5UU6YVO` found a compile error after
extracting inventory clipping. The helper now returns its coordinate error;
the existing caller performs the view-error conversion without behavior changes.

## Risks and Follow-ups

Complete live-input modes and retained
output copy navigation remain in their recorded later slices.
