# Slice 1: Restore responsive vertical navigation

Status: reviewing
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `fix/rev-input-01-navigation-responsiveness`
PR: pending

## Purpose

Remove the measured release tail during repeated vertical navigation while
preserving ordered input and canonical editing state.

## Delivered Behavior

Repeated Up/Down input is consumed in bounded batches before drawing. Every
event still reaches its native handler in order; no input is dropped. Ordinary
and large documents retain their bytes, accurate movement, bounds and undo.
The native scheduling and focused PTY regressions pass. The controller owns
the clean-HEAD handoff checks listed below.

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

At baseline, the native loop handled one event, composed every visible row and
wrote the entire surface before reading the next event. Wide-surface samples emitted
2,403,592 bytes in 120 redraws, taking 2.92-3.06 seconds to consume one second
of input. The demonstrated bottleneck is repeated complete rendering, rather
than document length or a timer that generates additional motion. Ordinary
80x24 navigation at 30/s did not reproduce a perceptible tail.

The correction consumes only immediately available events, retaining each
handler transition, and renders after at most 32 events or 4 ms of event work.
A handler already in progress runs to completion; the time budget is checked
before consuming another event. Exit stops consumption immediately, and a
subsequent ignored event cannot suppress a pending redraw.
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
- `src/session/events/tests.rs`: finite consumption, ordered history, cancellation,
  resize, Escape and immediate-exit regressions.
- `scripts/configuration_ui/navigation.py`: owned PTY repeat and release checks,
  invoked by `scripts/check-pty.py --suite navigation`.

## Validation

- PASS: `cargo build --bin rev` for the baseline measurement binary.
- Baseline regression: `python3 scripts/check-pty.py --suite navigation` failed
  with a 1,641.6 ms release tail against the 150 ms assertion.
- Implementation regression: the same command passed all four owned sessions,
  including eight direction/rate measurements, asymmetric reversal, ordered
  insertion/undo, BOF/EOF clamping, cancellation and unchanged file bytes. Wide
  120/s first updates were 19.82-24.23 ms and release tails 24.67-40.47 ms.
- PASS: `cargo test --lib session::events::tests` (five scheduling regressions).
- The extended PTY suite also passes standalone Escape expiry followed by
  ordinary insertion/undo and nested-prefix cancellation in every fixture.

Clean-HEAD handoff uses controller evidence for this exact ordered list:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test commands --test display --test windows --test recovery`
- `python3 scripts/check-source.py`
- `cargo test --lib session::events::tests`
- `python3 scripts/check-pty.py --suite navigation`
- `git diff --check`

## Risks and Follow-ups

The measurements do not establish responsiveness for arbitrary terminal or
transport delays. Complete combined terminal and resource acceptance belongs
to the approved later slices and delivery gate.
