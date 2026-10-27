# Slice 29: Bound startup and completion work

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `perf/rev-config-ui-29-configuration-ui-performance`
PR: [#78](https://forge.adrianpk.com/adrianpk/rev/pulls/78)

## Purpose

Measure admitted startup and completion work, then detect repeated filesystem,
parser, redraw or idle work with finite owned fixtures and enforced budgets.

## Delivered Behavior

- `python3 scripts/check-configuration-ui.py --suite performance` builds release
  editor/probe binaries, requires seven native workloads plus one real PTY,
  validates exact sample counts and reports zero skips. `--workload` selects
  graph, entries, names, redraw, syntax or idle for focused diagnosis.
- One include fixture reaches 64 source occurrences, 16 root-to-leaf edges,
  256 KiB per-source and 2 MiB total simultaneously. Padding reaches byte limits;
  supported root preferences and 48 explicit leaves exercise actual loading and
  resolution. Three repetitions must retain the resolved tab policy.
- Directory fixtures contain 4,099 and 2,102 immediate entries. Observation
  reaches the 4,096-entry cap or the 512-KiB name-byte cap, marks incompleteness
  and retains bounded snapshots. Each snapshot serves 256 prefix filters.
- A separate three-entry directory with nested descendants and a directory
  symlink provides a traversal control even if a capped snapshot omits children.
- Three shared editing panes, expanded tree and file-completion area render
  64 identical 100×30 frames from 512 tabbed Unicode lines with visible syntax.
  Draw preserves canonical document identity and the query cursor.
- A 65,536-byte Rust file admits parsing at equality. A one-byte edit crosses
  the cutoff, producing zero parser callbacks over 32 refreshes. Undo permits
  real parser work again, while an unchanged refresh executes zero callbacks.
- Linux inotify watches nine owned directories before native process startup.
  A flushed barrier separates real setup reads from cached work. Filtering,
  redraw and parser phases must emit zero fixture open/access events; no phase
  may traverse nested children. Overflow or lost watches fails the check.
- The configured release editor opens the maximal graph and reaches a complete
  terminal frame within two seconds. Editing and file-prompt idle intervals
  retain the original no-repaint/CPU budget and must produce zero fixture reads.
  Normal exit verifies termios/color restoration, child reaping and closed PTYs.
- Bounded resident-memory checks cover configuration/candidate/projection work.
  All native and PTY processes stay below 128 MiB peak RSS; 256 cached filters
  may increase the preceding observation high-water mark by at most 4 MiB.
- Complete fixture entry/byte fingerprints remain identical after all checks.
  The historical two-fixture/eight-workload performance runner is unchanged.

## Implementation Notes

- T29.1: `651051c5f19fc716234a380255d7b9ebf64e5e49` adds deterministic fixtures, release native measurements,
  sample/timing/memory admission and the performance suite entry point.
- T29.2: `f6534d4974f24dcb244a14c29bc17b5ccae0e412` adds read/traversal positive controls, cached-work watches,
  configured PTY idle and bounded child/descriptor ownership.
- T29.3: `7c2a072fb2d69ab4f5386ce9141f3cfed25a7056` records the environment, measurements and budgets below and emits total
  measurement elapsed time separately from compilation. No runtime tuning or
  runtime repair is required by the measured workloads.
- Reuse includes the existing hardware report, PTY owner, idle CPU/output oracle,
  environment isolation, completed-frame detection and terminal cleanup checks.
  Native probes call existing configuration, buffer, parser, completion and
  composition owners without adding runtime instrumentation or dependencies.
- Each native setup/work phase has a 15-second deadline and a 64-KiB output
  bound. Build deadline is 180 seconds; the existing PTY deadline is eight
  seconds. Fixture enumeration admits at most 6,400 entries and 256 KiB per file;
  filesystem evidence admits at most 8,192 queued events and nine watches.
- The resource sketch is a 2-MiB bounded input graph, at most 4,096 retained
  candidates/512 KiB names plus paths/labels, one visible 100×30 canvas and a
  64-KiB syntax input. RSS guards include allocator, parser and process overhead;
  they are measured regression guards, not exact heap-allocation counts or new
  product memory limits. Cached growth catches retained per-filter copies.

## Contracts Added or Changed

- CFG08: actual maximum graph loading/resolution, bounded time and resident work.
- CFG17-CFG19: configured retention/presentation/syntax owners retain bounded
  redraw state; exact syntax cutoff and unchanged-revision work remain visible.
- UI04: entry/name limits, cached filter reuse and shallow directory observation
  have actual kernel open/access evidence and bounded memory checks.
- Baseline R32-R35: historical load/search/edit/scroll/split/highlighting/parser
  fixtures, parser-work ratio and terminal idle budgets remain unchanged and are
  required by the slice's original performance command.

## Files of Interest

- `scripts/check-configuration-ui.py`: performance selection and release build.
- `scripts/configuration_ui/performance_fixtures.py`: exact graph/directory/input sizes.
- `scripts/configuration_ui/performance.py`: records, budgets, identity and process bounds.
- `scripts/configuration_ui/observation.py`: owned Linux open/access/traversal evidence.
- `scripts/configuration_ui/performance_terminal.py`: configured startup and idle PTY.
- `examples/configuration-performance.rs` and its three concrete helper modules:
  native graph, observation/filter, panel and syntax workload boundaries.

## Validation

Focused release measurements on Linux 7.2.2-arch1-1, x86_64, AMD Ryzen 5 7430U,
12 logical CPUs, 14.54 GiB RAM, Rust 1.98.0, release profile. PTY geometry is
80×24; the native nested composition is 100×30. These are local reference
measurements, not universal hardware latency guarantees.

| Workload | Samples | Median µs | Maximum µs | Enforced maximum µs | Peak RSS KiB |
| --- | --- | --- | --- | --- | --- |
| Graph load/resolve | 3 | 9163.918 | 9315.648 | 1000000 | 4868 |
| Entry-cap observation | 3 | 2279.388 | 2649.951 | 250000 | 4844 |
| Entry-cap cached filter | 256 | 16.801 | 39.162 | 10000 | 4944 |
| Name-cap observation | 3 | 6143.776 | 6455.900 | 250000 | 5612 |
| Name-cap cached filter | 256 | 8.245 | 35.094 | 10000 | 5616 |
| Nested panel redraw | 64 | 1550.185 | 1664.736 | 100000 | 8792 |
| Oversize syntax refresh | 32 | 0.030 | 565.301 | 10000 | 9640 |

- Observations retained 4,096 entries/16,384 name bytes and 2,072 entries/524,216
  name bytes respectively. Cached filter high-water growth was 100 and 4 KiB.
- Graph loading generated 264 watched open/access events as a read control.
  Other native setup controls generated 10, 10, 94 and 90 events. Every cached
  phase generated zero events and every traversal guard passed. Kernel events
  may coalesce; these values are event evidence, not syscall counts.
- Syntax positive control executed 809 callbacks; oversized and unchanged
  refreshes executed zero. The source returned to its clean saved state.
- Configured PTY startup was 36.929 ms against a 2,000-ms budget, peak RSS
  9,128 KiB. Editing/prompt intervals were 350.399/350.142 ms with zero output,
  zero CPU ticks and zero fixture reads. The original idle CPU budget remains
  max(two ticks, 10% of measured interval), rounded up by the existing oracle.
- Every focused fixture contained 6,280 entries with identical digest
  `21c823e580cedb321477b02cd6f01248baa86342f98d550b9316c17b1b4a2a8c`.

Successful development commands:

- `python3 scripts/check-configuration-ui.py --suite performance --workload graph`
- `python3 scripts/check-configuration-ui.py --suite performance --workload entries`
- `python3 scripts/check-configuration-ui.py --suite performance --workload names`
- `python3 scripts/check-configuration-ui.py --suite performance --workload redraw`
- `python3 scripts/check-configuration-ui.py --suite performance --workload syntax`
- `python3 scripts/check-configuration-ui.py --suite performance --workload idle`
- Focused source checks passed for all four new Rust files: maximum five
  parameters, nesting one and cyclomatic complexity nine. Python AST syntax and
  parameter bounds passed for the runner and performance helper modules.

Final readiness requires controller evidence for the clean PR head after the
PR-reference follow-up commit, in this exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `python3 scripts/check-configuration-ui.py --suite performance`
- `python3 scripts/check-performance.py`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

The performance harness requires Linux /proc and inotify, like the existing
Linux PTY/resource harness. Required facilities fail visibly instead of skipping
work. Open/access watches prove owned fixture activity rather than all system
metadata syscalls; RSS measures resident process high-water marks rather than
individual allocations. Full delivery acceptance retains its approved next
slice and controller-owned gate.
