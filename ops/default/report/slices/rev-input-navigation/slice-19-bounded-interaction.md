# Slice 19: Verify bounded input gutter and pointer work

Status: delivered
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `perf/rev-input-19-bounded-interaction`
PR: [#99](https://forge.adrianpk.com/adrianpk/rev/pulls/99)
Integrated: `fb59cf9f8fc643a5257714ef1b54acac1bca2538`

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

Hidden numbering projects the same 64 moving frames with zero rendered number
rows. Mouse-disabled admission ignores 256 press/drag/wheel/release reports
without moving point, creating a deadline or repainting. The PTY independently
feeds the same 256 reports at 120 reports per second, checks no output during
feeding and then measures quiet idle and a retained-point keyboard barrier. Two additional owned PTYs
observe first and second startup: exactly one final-name publication and four
creation/write/cleanup events occur on first start, with no mutation on second
start, unchanged complete bytes/inode/metadata and no idle filesystem activity.

## Implementation Notes

The existing configuration-performance example, collector, filesystem observer
and controlling PTY lifetime are reused. Number-row counts inspect rendered
separators; the public frame has no internal allocation counter. Existing
native tests and the hidden-field early return cover internal projection policy.
The large-buffer pointer fixture targets the actual glyph at column eight:
padding after a one-character line does not establish canonical text capture.
Outside release intentionally produces no redundant frame, so its completion
is checked by retained point and a quiet interval rather than expecting repaint.

Task commits:

- T19.1: `2947eb51e339a2ff68a6c21c57ff7de59e083f53`
- T19.2: `57d681ced9614b4f00f8d3223ff48a117a8d0666`
- T19.3: `79b903edd9acef3e30196cd2a00374dac002cafa`

## Contracts Added or Changed

Measurements exercise GUT13, PTR05, R07 and CFG23 without changing native behavior.

## Files of Interest

- `examples/configuration_performance/interaction.rs`: bounded native session owner.
- `examples/configuration_performance/interaction/gutter.rs`: moving gutter frames.
- `examples/configuration_performance/interaction/pointer.rs`: controlled tick and release work.
- `scripts/configuration_ui/interaction_terminal.py`: real input release and drag lifetime.
- `scripts/configuration_ui/performance.py`: existing collector integration.
- `scripts/configuration_ui/performance_bootstrap.py`: observed first/second startup.
- `scripts/configuration_ui/interaction_evidence.py`: exact candidate and bounded-work admission.
- `scripts/test_acceptance.py`: missing, skipped, stale and excessive-work rejection.

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

The focused source checker passed all five changed Rust example files within
safe Rust, five parameters, three nesting levels and cyclomatic ten. Development
`python3 -m unittest discover -s scripts -p test_acceptance.py` passed sixteen
guard tests, retaining all prior checks and adding mutation cases for stale
HEAD/tree, changed limits, CPU/wall growth, missing/duplicate workloads, hidden
rows, drag cadence/step/idle, disabled repaint, repeat order/count/release,
cleanup and absent/repeated/stale bootstrap evidence. The acceptance map retains
all thirty original entries and historical identity metadata, adds GUT13/R07
and connects pointer work to PTR05. Its complete performance set is now thirteen
native workloads and four owned PTYs; focused runs cannot establish completion.
The full-gate entrypoint and ordered command list remain unchanged.

Final required ordered commands:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test gutter --test commands --test display --test configuration`
- `python3 scripts/check-configuration-ui.py --suite performance`
- `python3 scripts/check-source.py`
- `git diff --check`

Final focused commands:

- `python3 -m unittest discover -s scripts -p test_acceptance.py`
- `python3 scripts/check-configuration-ui.py --suite prerequisites`
- `git diff --check`

Controller validation on the first PR-reference candidate passed formatting,
lint and the five native targets, then retained a redraw setup failure: the
old long Rust prefix clipped every Unicode glyph from the split-pane surface
once native gutters occupied their columns. The fixture now places the same
CJK, combining, emoji and ZWJ glyphs earlier in valid Rust comments. It retains
512 tabbed lines, three panes, the 100x30 surface, visible syntax, unchanged
canonical state and all timing/memory/output bounds. The original visible
Unicode assertion remains required. A focused redraw passed 64 frames with a
2.644-ms maximum and zero workload filesystem events. Setup failures now include
the workload identity and bounded child diagnostics instead of discarding the
retained error before the ready barrier.

Canonical evidence `NANWRDRXNO2WCGH53GAPKAEEQA` passed all six required
commands, 356 native tests, thirteen performance workloads and four owned PTYs
at the integrated clean candidate `fb59cf9f8fc643a5257714ef1b54acac1bca2538`.
Focused evidence `TBJU5PXHMIQAPDF7ULDEQQTXBQ` passed three commands, sixteen
guard tests and 47 requirement mappings on the same HEAD. Actual controller
performance output satisfies complete candidate/workload admission. Collection
took 6.741 seconds excluding build; arrow first/release updates were below
0.6 ms, the first drag tick was 101.723 ms, and measured idle intervals emitted
zero bytes and used zero process CPU ticks. The controller integrated PR #99
without changing its tested commit.

## Risks and Follow-ups

Linux process accounting has clock-tick resolution. Controlled native time proves
cadence and work bounds; the real PTY separately measures event-loop latency.

The disabled PTY feed is bounded at 2,500 ms wall and 200 ms CPU, followed by
the unchanged 350-ms idle check. A development multi-kilobyte instantaneous
report burst exposed a stall in the dependency's terminal decoder before the
keyboard barrier. The measured driver sends the complete same report count in
four-report groups at the declared rate, within the unchanged eight-second
session deadline. Controlled native admission still checks all 256 reports.
This slice makes no dependency or runtime change and does not establish support
for an instantaneous multi-kilobyte pointer burst.
