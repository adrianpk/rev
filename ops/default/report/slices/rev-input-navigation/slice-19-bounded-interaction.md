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

## Risks and Follow-ups

Linux process accounting has clock-tick resolution. Controlled native time proves
cadence and work bounds; the real PTY separately measures event-loop latency.
