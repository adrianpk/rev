# Slice 1: Restore responsive vertical navigation

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `fix/rev-input-01-navigation-responsiveness`
PR: pending

## Purpose

Remove the measured release tail during repeated vertical navigation while
preserving ordered input and canonical editing state.

## Delivered Behavior

Investigation completed; correction and regression validation are in progress.

## Implementation Notes

Baseline: `57074c6e7744eb22ab4f518c91997f019a6cb318`, unoptimized debug
binary built with `cargo build --bin rev`. Controlling Linux PTYs used isolated
HOME/XDG roots and a plain-text fixture containing 400 or 100,000 copies of
`navigation row with ordinary text` plus LF. Each sample delivered 120 Down
reports at the stated rate, drained output continuously, and observed completed
frames and their logical status positions. Every event reached its expected
line. These are controlled PTY measurements, not physical-terminal observations.

| Surface | Input rate | 400-line release tail | 100,000-line release tail |
| --- | --- | --- | --- |
| 80x24 | 30/s | 3.78 ms | 3.28 ms |
| 80x24 | 120/s | 4.22 ms | 3.45 ms |
| 80x24 | 1,000/s burst | 393.05 ms | 318.52 ms |
| 240x70 | 30/s | 25.43 ms | 22.21 ms |
| 240x70 | 120/s | 1,923.12 ms | 2,071.49 ms |

The native loop handles one event, composes every visible row and writes the
entire surface before reading the next event. Wide-surface samples emitted
2,403,592 bytes in 120 redraws, taking 2.92-3.06 seconds to consume one second
of input. The demonstrated bottleneck is repeated complete rendering, rather
than document length or a timer that generates additional motion. Ordinary
80x24 navigation at 30/s did not reproduce a perceptible tail.

The correction will consume only immediately available events, retaining each
handler transition, and render after at most 32 events or 4 ms of event work.
No wait fills a batch. A completed-frame responsiveness ceiling of 150 ms for
first visible progress and release is selected from the measured 21-25 ms
wide-surface single-frame cost, with scheduling margin. Bounds, reversal,
ordered editing, cancellation and Escape remain observable assertions.

## Contracts Added or Changed

R07 and R16 retain per-view reveal, logical movement and valid canonical point.
No input event may be discarded to reduce the measured backlog.

## Files of Interest

- `src/session.rs`: terminal event/render loop.
- `src/session/events.rs`: native event consumption and Escape expiry.
- `src/display.rs`: complete frame composition and terminal output.

## Validation

- PASS: `cargo build --bin rev` for the baseline measurement binary.
- Exact clean-HEAD slice validation remains pending.

## Risks and Follow-ups

The measurements do not establish responsiveness for arbitrary terminal or
transport delays. Complete combined terminal and resource acceptance belongs
to the approved later slices and delivery gate.
