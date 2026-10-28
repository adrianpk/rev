# Slice 19: Verify bounded input gutter and pointer work

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `perf/rev-input-19-bounded-interaction`
PR: pending

## Purpose

Measure native input, visible gutter projection and pointer lifetime on the
existing bounded release collector and its owned fixtures.

## Delivered Behavior

A 200,000-byte document supplies 100,001 indexed logical lines within the
existing 256-KiB per-file fixture limit. Absolute and relative gutters each
project 64 moving frames with exactly 22 rendered number rows. Controlled drag
expiry executes 64 single three-row steps, rejects immediate catch-up and
retains neither a deadline nor movement after release. A release-binary PTY
measures both arrow directions at 120 reports per second, release drift and
actual drag-edge expiry followed by quiet idle.

## Implementation Notes

The existing configuration-performance example, collector, filesystem observer
and controlling PTY lifetime are reused. Number-row counts inspect rendered
separators; the public frame has no internal allocation counter. Existing
native tests and the hidden-field early return cover internal projection policy.
The large-buffer pointer fixture targets the actual glyph at column eight:
padding after a one-character line does not establish canonical text capture.
Outside release intentionally produces no redundant frame, so its completion
is checked by retained point and a quiet interval rather than expecting repaint.

## Contracts Added or Changed

Measurements exercise GUT13, PTR05, R07 and CFG23 without changing native behavior.

## Files of Interest

- `examples/configuration_performance/interaction.rs`: bounded native session owner.
- `examples/configuration_performance/interaction/gutter.rs`: moving gutter frames.
- `examples/configuration_performance/interaction/pointer.rs`: controlled tick and release work.
- `scripts/configuration_ui/interaction_terminal.py`: real input release and drag lifetime.
- `scripts/configuration_ui/performance.py`: existing collector integration.

## Validation

Development focused commands passed:

- `python3 scripts/check-configuration-ui.py --suite performance --workload gutter`
- `python3 scripts/check-configuration-ui.py --suite performance --workload drag`
- `python3 scripts/check-configuration-ui.py --suite performance --workload idle`

Visible gutter frame maxima were below 0.7 ms and controlled active drag below
0.6 ms. Real arrow first/release updates were below 0.8 ms; first drag expiry
was 101.435 ms. All idle intervals emitted zero bytes and measured zero CPU.
Native and PTY peak RSS remained below 9 MiB. These are development measurements;
final exact clean-HEAD controller evidence is recorded after PR references.

Each native record includes a measured wall interval and process user/system CPU
ticks in addition to latency samples and RSS. The interval starts at process
entry for setup observations and resets at the workload barrier and each record.
New native interaction batches allow at most 1,000 ms wall and 500 ms CPU;
visible frame/active tick maxima retain the existing 100-ms redraw ceiling and
released expiry retains the 10-ms cached-work ceiling. Arrow updates retain the
150-ms first/release budget, with 200-ms CPU per 30-report direction. The real
first drag tick allows 100-250 ms wall and 100 ms CPU. These finite new bounds
leave the original 128-MiB RSS, 64-KiB native output, 15-second native deadline,
eight-second PTY lifetime, 1-MiB PTY output and all prior workload limits intact.

Development measured the 64-frame batches at 17.7-18.2 ms wall and 20 ms CPU
(two accounting ticks); active drag took 17.5 ms wall and 20 ms CPU. Released
expiry took 0.077 ms with zero ticks. A clean HEAD/tree identity is captured
before collection and checked again afterward; dirty or changed candidates
cannot emit successful performance evidence.

## Risks and Follow-ups

Linux process accounting has clock-tick resolution. Controlled native time proves
cadence and work bounds; the real PTY separately measures event-loop latency.
