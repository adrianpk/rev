# Slice 46: Measure representative editing workloads

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `perf/rev-46-performance`
PR: `#46`

## Purpose

Measure the representative mixed-width UTF-8 workload and detect missing work,
repeated full parsing and sustained idle rendering with the existing native
owners and real executable.

## Delivered Behavior

- `python3 scripts/check-performance.py` builds the editor and measurement
  example in release mode, generates two deterministic fixtures and runs all
  eight native workloads plus real PTY startup/idle/restoration checks.
- Every fixture includes CJK, combining text, emoji/ZWJ sequences, tabs, mixed
  LF/CRLF, a BOM and no final newline. Function rows have varying lengths;
  every 251st row includes a 512-character tail.
- Native measurements cover loading, middle-file literal search, 32 paired
  Unicode insertions/deletions, 16 page scrolls, eight nested shared splits and
  16 unchanged highlighting redraws. Every redraw validates a complete styled
  frame and serializes it to an output sink. Final document and disk bytes
  must equal the generated fixture.
- A real Tree-sitter progress callback measures 32 accepted identifier and
  unfinished-parameter refreshes against a cold parse of the same input.
  Unchanged refreshes must invoke zero callbacks; a local refresh must use
  less than one quarter of the cold parse's callback count. Lost parse seeds,
  plain-text fallback and missing work fail the command.
- Real PTYs wait for a completed Rust frame, measure a 350-ms idle interval,
  require no output and reject sustained CPU work. Accepted exit checks raw
  attributes, paste disable, cursor visibility and screen restoration through
  the existing harness. All temporary files and processes are owned and reaped.

## Implementation Notes

- T46.1: `c5c2f3ea49e0cdb1a0dd1cf9ac476b3d3ee17b4e` adds deterministic fixtures, release measurements and real PTY startup.
- T46.2: `b67b5b3d5aabd03c396f407fcc44a4e3f9f91d01` adds parser-work and idle oracles, diagnostic bounds and cohesive
  fixture steps within the source-quality limits.
- Runtime inspection followed the measured paths: `Buffer::candidate` clones
  Ropey's shared tree, while history stores changed fragments; newline counts
  examine edit neighborhoods. Search scans borrowed rope slices. Parser input
  borrows chunks and query predicates borrow node spans. Rendering clips visible
  rows and columns. No whole-document String conversion was found in these paths.
- `Session::run` waits for input between redraw outcomes. `Syntax::refresh`
  rejects repeated work for the same stamp. Both behaviors are checked by the
  measured idle and unchanged-revision observations below.
- The parser guard compares actual work on the same fixture, independent of a
  latency target. Native insertion occurs just after the middle function's
  opening parenthesis, temporarily leaving an unfinished parameter; timing
  includes text publication, incremental parsing, projection and serialization.

## Contracts Added or Changed

- The performance command fails on absent/duplicate workload records, invalid
  timing/sample data, missing highlighting or shared views, changed fixture bytes,
  lost incremental parsing, substantial repeated parse work or idle repaint/CPU work.
- Subprocesses and PTYs retain finite deadlines and bounded diagnostics. No
  additional dependency or product command-line option is introduced.
- Each run reports fixture hashes, release/compiler details, architecture,
  CPU model/count, memory, terminal geometry and measurements without host names.

## Files of Interest

- `scripts/check-performance.py`: fixture generation, measurement completeness,
  hardware context and real PTY startup/idle/restoration.
- `examples/performance.rs`: real native session workloads and byte preservation.
- `examples/performance/work.rs`: cold/incremental/unchanged parser work comparison.

## Validation

All six ordered checks passed through Looper on task HEAD `b67b5b3d5aabd03c396f407fcc44a4e3f9f91d01` under
validation `HAEZFUM4SMGR7ZAVJYBW5EGTF7`. Final clean PR-head evidence is recorded
after the report and PR-reference commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 454 native tests, zero ignored.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `python3 scripts/check-performance.py`: passed; two fixtures, eight native
  workloads each and two real PTY startup/idle/restoration checks.
- `git diff --check`: passed.

Recorded hardware: AMD Ryzen 5 7430U with Radeon Graphics; 12 logical CPUs;
14.54 GiB RAM; x86_64; Linux
7.2.2-arch1-1; `rustc 1.98.0 (88d9e12ae 2026-08-18) (Arch Linux rust 1:1.98.0-1)`; release profile; 80 by 24 cells.

Fixture identity:

- `20000` lines, `1650955` bytes, SHA-256
  `4b335088cc22e90327f41802ce71649a16e0c303db1ad475f3ae0838e682095c`.
- `40000` lines, `3313038` bytes, SHA-256
  `8f19cc1db9063cb1ba704ab83793e1d80db30f6b6eada395ae4c197e5d3b1327`.

Measured medians in milliseconds; load, cold parse and PTY startup have one
sample per fixture. Other sample counts are given above. Generated files are
already in the filesystem cache; release compilation is excluded from timings.

| Workload | 20,000 lines | 40,000 lines |
| --- | ---: | ---: |
| Native load and first frame | 167.850 | 359.797 |
| Middle-file literal search | 16.812 | 33.925 |
| Insert and redraw | 21.488 | 43.941 |
| Delete and redraw | 28.272 | 61.610 |
| Page scroll and redraw | 0.818 | 0.741 |
| Nested shared split redraw | 1.865 | 1.346 |
| Unchanged highlighting/redraw | 0.757 | 0.716 |
| Incremental parser refresh | 24.583 | 58.679 |
| Cold parser reference | 137.502 | 280.993 |
| Real PTY startup | 167.229 | 305.459 |

Parser and idle observations:

- 20,000 lines: cold parse `8603` callbacks;
  maximum local refresh `603`; unchanged revision `0`.
  PTY idle `350.299` ms, output `0` bytes, CPU `0.000` ms.
- 40,000 lines: cold parse `17204` callbacks;
  maximum local refresh `1204`; unchanged revision `0`.
  PTY idle `350.400` ms, output `0` bytes, CPU `0.000` ms.

## Risks and Follow-ups

Measurements apply to these fixtures, this hardware and the recorded build.
Native redraw timings use an output sink. PTY startup includes real terminal
I/O but excludes graphical terminal-emulator rendering. Callback counts measure
Tree-sitter progress and do not constitute an allocation profile or a universal
latency/file-size guarantee.
