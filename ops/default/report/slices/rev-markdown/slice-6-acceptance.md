# Slice 6: Acceptance and guidance

Status: reviewing
Delivery set: rev-markdown
Plan: [Markdown Delivery](../../../plan/rev-markdown.md)
Tracker: [Markdown Tracker](../../../tracker/rev-markdown.md)
Branch: `feat/rev-markdown-06-acceptance`
PR: pending

## Purpose

Deliver T6.1-T6.3: joined native/PTY acceptance and complete reader guidance for
the native Markdown behavior from Slices 1-5. Prepare its canonical PR for the
controller-owned integrated delivery gate.

## Delivered Behavior

- Exercise the approved source fixture through both interaction profiles,
  colored and monochrome output, LF and BOM/CRLF. Decode actual terminal cells,
  retain source and pretty views, copy/save exact source and check Unicode
  geometry, independent panes and syntax-disabled presentation.
- Check list and table transactions through undo/redo and real contextual Return,
  resize owned Linux PTYs from 80x24 to 32x12, measure idle silence and restore
  terminal attributes, screen, cursor and bracketed-paste modes.
- Teach an offline note/list/table workflow in the canonical User Guide and
  provide a complete Markdown reference, with learning-path links and Spore
  navigation. Preserve the existing command/configuration/binding catalogs,
  packaged sample and guide/reference assertions.

## Implementation Notes

Implementation and report introduction: `0426fa99039d1b095940c38e3b8f6ed6db02817a`.

The acceptance crate uses one checked-in Markdown fixture, the existing bounded
PTY owner and the existing frame-cell decoder. Each PTY keeps the existing
eight-second deadline, one-MiB output cap and owned child reaping. Native
observation settling keeps the five-second bound. The shell wrapper owns its
temporary root and isolates every HOME/XDG store. Release builds remain offline,
single-job and private to the task worktree.

Canonical Markdown remains the documentation source. Publication navigation adds
the independent Markdown path while retaining every existing learning sequence.
The projection admits exactly the existing syntax reference's repository-root
parser notices link and renders the canonical notices as a reference page.
Unrelated repository files and deeper traversal remain rejected. Strict links,
heading aliases and Spore rendering cover every page; no service is changed.

## Contracts Added or Changed

- scripts/check-markdown.sh is the reproducible joined native/PTY check.
- No runtime command, setting, binding, dependency, finite bound, SDK/wire or
  persistence contract changes. Existing exact catalogs remain authoritative.
- The Markdown lesson and reference join canonical documentation and the local
  generated User Guide navigation. The existing Unreleased entry links them.

## Files of Interest

- examples/markdown-acceptance.rs and examples/markdown_acceptance/: one finite
  fixture matrix and native/PTY workflow assertions.
- scripts/check-markdown.sh: owned wrapper, isolated stores and bounded drivers.
- docs/tutorials/user-guide/23-markdown.md and docs/reference/markdown.md: guided
  practice and exact source/presentation/list/configuration contracts.
- publication/userguide/index.sptree and documentation indexes: reachable guide
  and reference projection without duplicating canonical content.
- examples/documentation-check.rs and examples/userguide-publication.rs: retain
  old navigation assertions, add Markdown and project the exact parser notices.

## Validation

Focused implementation checks pass eight native cases and four Linux PTY
journeys, with exact LF/BOM/CRLF saves, both profiles and both color policies.
Each PTY retains its eight-second/one-MiB limits, changes 80x24 to 32x12, passes
the 350-ms idle silence check with zero CPU ticks and restores its terminal.

Documentation verifies 707 commands, 322 settings, 110 reachable canonical pages
and 785 local links/anchors. All seven documentation-check and seven publication
tests pass, retaining all thirteen old cases and adding a bounded notices check.
The local guide build strictly checks links and renders 111 pages, including
the original parser notices. Its final build takes 3.08 s.

Initial fixture checks found an Option/Result conversion, the source checker's
current --source spelling and functions above the cyclomatic maximum. These
were corrected without policy changes. PTY predicates now wait for published
padding and refreshed structural symbols, rather than racing an unfinished
alignment; narrow modal save checks account for status filename truncation.
The first guide build exposed the existing repository-root notices link; the
explicit projection resolves it while retaining traversal rejection.

All six required development commands passed in canonical order on clean
published HEAD `0426fa99039d1b095940c38e3b8f6ed6db02817a`, from
2026-11-13T22:27:33Z to 2026-11-13T22:46:37Z. All 423 tests across three native
targets passed without failures, ignored cases or filtered cases.

| Target | Passed | Seconds |
| --- | --- | --- |
| commands | 205 | 672.54 |
| configuration | 169 | 318.13 |
| markdown | 49 | 120.62 |

Native targets took 1,111.29 s; their command took 1,118.64 s including preparation.
Full lint took 16.04 s, the joined acceptance check 6.08 s, documentation 0.55 s,
the guide build 2.21 s and whitespace 0.02 s. Lint and the immutable-index commit
hook cover all 1,882 Rust files, both packages and 75 Cargo roots with zero
blocking or deferred diagnostics. Exact command, configuration, binding and
packaged-default assertions retain 707 commands and 322 settings.

The four required PTYs start in 96,894-122,892 microseconds, emit 46,431-79,509
bytes and peak at 18,128-22,156 KiB RSS. Each passes the unchanged idle assertion
with zero CPU ticks. These are observed costs, not performance guarantees.
No command, timeout, bound or assertion was weakened for acceptance.

The generated lesson, Markdown index/reference and notices pages are present.
An HTML text check confirms literal hashes, Unicode, escaped-pipe source and the
original notice copyright/disclaimer survive rendering.

Required ordered commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test commands -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-markdown.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `git diff --check`

Focused command:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --example documentation-check --example userguide-publication -- --test-threads=1`

## Risks and Follow-ups

The controller owns Slice 6 integration, the frozen full delivery gate and final
set closure. This worker does not claim integrated acceptance or close the
specification, model, plan or tracker before that gate succeeds.
