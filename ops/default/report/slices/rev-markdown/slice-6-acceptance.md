# Slice 6: Acceptance and guidance

Status: delivered
Delivery set: rev-markdown
Plan: [Markdown Delivery](../../../plan/rev-markdown.md)
Tracker: [Markdown Tracker](../../../tracker/rev-markdown.md)
Branch: `feat/rev-markdown-06-acceptance`
PR: [#39](https://forge.adrianpk.com/rev/rev/pulls/39)

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
Canonical PR: https://forge.adrianpk.com/rev/rev/pulls/39
Development evidence commit: `850b4a3754ae458027b224f49974e9cbe3b6c3a0`.

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

Controller receipt `BXKF6CWP6GZZW46WS7CGUMQWWG` passed all six required checks
in canonical order on clean published HEAD
`9a9f96f439544292ccab7b0d080eec471c29870f`, from 2026-11-13T22:48:40Z to
2026-11-13T23:09:22Z. Its 423 native tests passed without failures, ignored or
filtered cases: commands 205/717.67 s, configuration 169/300.50 s and Markdown
49/182.60 s, totaling 1,200.77 s. The complete receipt took 1,241.44 s.
The joined check passed eight native cases and four Linux PTYs; each restored
its terminal and passed the unchanged idle assertion. Full lint retained
1,882 files, both packages and 75 roots; catalogs retained 707 commands and
322 settings, and the guide rendered 111 pages.

Canonical PR #39 merged through Rebase + Fast-forward into dev at that exact
validated HEAD. Forgejo, fetched dev ancestry and Acta event
`01M7BJTVFF7E7NAEZDSG4QGZQ8` confirm integration. Slice 6 of 6 is delivered.
The task worktree was clean, unlocked, inactive and integrated before non-forced
removal; only its classified generated target output was ignored. All slice
branches and pre-existing worktrees remain.

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

All six slices are integrated and the controller passed the frozen full delivery
gate. The specification, model, plan and tracker are Completed. Physical font
coverage remains terminal-owned. The existing foreign crossterm unused_parens
warning remains nonblocking and outside first-party source policy.

## Delivery-set closure

Delivery-set status: Completed
Completed: 2026-11-14
Tested candidate: `4a01a05ee0ee02b0ec02aed1beac0cdc19c59664`.
Controller gate receipt: `X5PEQCHM7HGP4DHHSW4ZJ7I53J`.
All canonical PRs are merged: [#34](https://forge.adrianpk.com/rev/rev/pulls/34),
[#35](https://forge.adrianpk.com/rev/rev/pulls/35),
[#36](https://forge.adrianpk.com/rev/rev/pulls/36),
[#37](https://forge.adrianpk.com/rev/rev/pulls/37),
[#38](https://forge.adrianpk.com/rev/rev/pulls/38) and
[#39](https://forge.adrianpk.com/rev/rev/pulls/39).

The controller passed the exact unchanged combined command on the immutable
clean published candidate:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test syntax --test configuration --test commands --test display --test pointer --test org_headings --test org_indentation --test org_tables --test external_changes -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-markdown.sh && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh && git diff --check`

The successful receipt ran from 2026-11-13T23:17:51Z to
2026-11-14T00:00:02Z, taking 2,531.17 s including compilation and all commands.
Full workspace formatting, strict Clippy and native source policy passed across
1,882 first-party Rust files, both packages and all 75 Cargo roots, with zero
blocking or deferred diagnostics. All 721 selected native tests passed with
zero failures and zero ignored cases:

| Target | Passed | Test execution seconds |
| --- | --- | --- |
| commands | 205 | 925.50 |
| configuration | 169 | 336.06 |
| display | 47 | 45.07 |
| external_changes | 21 | 22.35 |
| markdown | 49 | 153.32 |
| org_headings | 19 | 33.10 |
| org_indentation | 18 | 26.63 |
| org_tables | 73 | 75.50 |
| pointer | 69 | 88.65 |
| syntax | 30 | 48.65 |
| modal contexts:: | 21 | 37.02 |

The ten complete targets passed 700 tests with zero filtered cases. The modal
command passed all 21 selected contexts:: cases and filtered 103 other modal
cases, exactly as approved. Native target execution totaled 1,791.85 s,
excluding compilation and the other gate work. No command, case, timeout,
resource bound or assertion was weakened for acceptance.

The joined acceptance driver passed all eight native cases and four Linux PTY
journeys across both profiles, both color policies and LF/BOM+CRLF sources.
It checked actual terminal cells, source/pretty views, six heading levels,
Setext, nested lists, literal code, Unicode hit maps, independent panes,
copy/save fidelity, contextual Return, list/table transactions and undo/redo.
Each PTY changed 80x24 to 32x12, restored terminal attributes, screen, cursor and
bracketed paste, and passed its 350-ms idle-silence check with zero CPU ticks.
Startup measured 109,572-177,123 microseconds, output 46,431-79,509 bytes and
peak RSS 18,568-21,984 KiB. Existing eight-second/one-MiB PTY limits and bounded
owned-child reaping remain intact. No owned test fixture or native build remains
active. These observations are measured costs, not performance guarantees;
physical font appearance retains its terminal-owned limit.

Exact catalogs passed 707 command names and 322 configuration entries against
compiled behavior and packaged defaults. Documentation checked 110 reachable
canonical pages and 785 local links/anchors. The canonical Markdown-to-Spore
User Guide passed strict links and rendered 111 pages, including parser notices.
The existing foreign crossterm unused_parens warning remained nonblocking.

All reports remain delivered. The specification, model, plan and tracker are
Completed, all 18 tasks are closed and the native Markdown operational-index
row is reconciled. All six integrated task worktrees were removed only after
ancestry, cleanliness and inactivity checks; slice branches, unrelated worktrees
and their private caches remain. Only the eleven configured closure Markdown
paths differ from the tested candidate. Runtime, tests, manifests, guide inputs,
executable drivers and every required assertion retain tested bytes. The original
receipt remains tied to the immutable candidate; final documentation closure
is committed, recorded in Acta and pushed separately on clean synchronized dev.
