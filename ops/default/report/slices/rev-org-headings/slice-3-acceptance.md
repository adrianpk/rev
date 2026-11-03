# Slice 3: Joint acceptance and guidance

Status: delivered
Delivery set: rev-org-headings
Plan: [Heading Presentation Delivery](../../../plan/rev-org-headings.md)
Tracker: [Heading Presentation Tracker](../../../tracker/rev-org-headings.md)
Branch: `feat/rev-org-headings-03-acceptance`
PR: `#240`

## Purpose

Connect the delivered native heading policy and view controls to bounded Linux
terminal acceptance, representative resource observations and the existing
Markdown-derived User Guide.

## Delivered Behavior

- Native cases preserve BOM/CRLF bytes, source point, empty history and canonical
  copied stars through source/pretty/reset, cancellation and 40x12/80x24 resize
  under both editing profiles, with syntax coloring disabled.
- Four real PTYs start in both profiles at each size and resize to the other.
  They assert emitted markers for levels 1-8, both commands and feedback,
  unchanged cursor over a concealed star, selector cancellation, actual star
  copy/yank with one undo, clean source, idle silence and terminal restoration.
- A finite 778898-byte, 16896-line Org source with 512 headings checks bounded
  startup, 64 toggle/projection operations, a heading-level edit, immediate stale
  source fallback, refreshed markers and exact one-step undo. Idle has no pending
  deadline and consumes at most one CPU tick during a 350ms observation.
- The existing Org learning chapter teaches source/pretty/reset, canonical copy
  and per-pane startup reset. References, the exhaustive packaged sample,
  terminal journey instructions and Unreleased describe the actual controls.
  The terminal retains font, glyph and ligature ownership.

## Implementation Notes

The Rust driver reuses the existing shared util-linux terminal owner and source
quality checker. Its constructor now accepts the fixture filename; all existing
consumers still pass terminal.rs. Output remains capped at 1 MiB and each PTY
at eight seconds. The wrapper records only its owned exec'd editor PID; resize
changes that child's controlling TTY and signals that process. Existing cleanup,
alternate-screen, bracketed-paste and cursor-restoration assertions remain.

The new Bash journey builds normal release artifacts with one Cargo job and
inherits the four low-footprint Cargo environment settings. Native/resource
processes have 20-second outer guards; the four-PTY group has 35 seconds.
Each native observation, large startup, projection group and edit/undo group
retains a five-second bound. Temporary HOME/XDG/workspace roots isolate source
and state. Completed fixtures and children are cleaned by their existing owners.

Source-quality checks cover actual crate roots, the heading policy/projection/
view/control modules, new driver modules and shared terminal owner. They retain
safe Rust, at most five parameters, logical nesting three and cyclomatic ten.
Existing native heading/configuration/geometry/view/reload/readonly-result
coverage remains in the controller's unchanged integrated gate.

## Contracts Added or Changed

OGP01-OGP08 now have joint native, source-byte, real terminal and local guide
acceptance. This slice adds no command identity or preference; the existing
431-command and 235-setting catalogs remain authoritative. Source, recognition,
coordinate, view and history owners are unchanged. The single bounded startup
marker list and viewport projection retain no separate decoration index.

## Files of Interest

- `examples/org-heading-acceptance.rs`, `examples/org_heading_acceptance/`:
  native cases, finite resource observations and four real terminal journeys.
- `scripts/check-org-headings.sh`: one-job build, focused source quality,
  isolated stores, finite subprocess ownership and TTY checks.
- `examples/completion_acceptance/terminal.rs`: explicit fixture filename with
  the existing eight-second/1-MiB terminal boundary.
- `tests/external_changes/pty.rs`: unchanged kernel-wake assertions adapted to
  the shared constructor; other shared consumers retain their existing filename.
- `docs/tutorials/user-guide/06-org-notes.md`, its Org index, heading references,
  exhaustive sample, terminal journey how-to and `CHANGELOG.md`: current guidance.

## Validation

Focused development checks passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true cargo build --locked --release --jobs 1 --bin rev --example org-heading-acceptance --example source-check` (cold build: 5m13s).
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test external_changes pty::idle_native_wake -- --test-threads=1` (one test, both profiles, 1.49 seconds).
- `git diff --check`.

Direct owned-root runs of the release driver's native, resource and PTY modes
passed. The large fixture measured 319513us startup, 21355us for 64 selector
changes plus projections and 542843us for the level edit/undo; peak RSS was
19900 kB. Native idle used zero ticks. PTYs started in 50993-65525us, emitted
12711-32337 bytes each and measured 12912-14300 kB peak RSS, all with zero idle
ticks and restored TTYs. These are local development observations, not universal
performance guarantees. The counter assertion follows native history ownership:
one undo entry before undo, none afterward, with exact source restoration.

Initial exact-head batch `63EJSPL7JILXEIXQ4L2ARHVMIH` at
`ddd1380f243d43d02303120893cbbcc59cfbadff` passed format and stopped in
all-target Clippy: the unrelated wrap-navigation fixture owns its own
three-argument terminal constructor. Its call was mistakenly included in the
shared-constructor adaptation and is restored exactly. The shared owner still
accepts the explicit filename; no terminal assertion or check is removed.
The corrected candidate subsequently passed fresh exact-head validation below.

Canonical PR #240 merged through controller-owned Rebase + Fast-forward into
dev at `4bda46f32496016b514150f8fb4eb457786046f4` on
2026-11-03T16:45:11Z. Receipt `JIACPRIUXO7FT3BFVZOIYNBTD5` passed these
seven ordered checks on that exact clean head after PR-reference follow-ups,
in 35.35 seconds. This closes Slice 3 of 3; the subsequent successful
integrated gate is recorded below.

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-org-headings.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `git diff --check`

The final slice batch passed 16 heading tests (8.96 seconds), native cases in
both profiles, all four PTYs, source quality, 431-command/235-setting catalog
checks, 70 reachable documentation pages and 438 local links/anchors. It projected
and built 70 Markdown pages locally through Spore. Its large fixture measured
302752us startup, 20886us for 64 toggle/projections and 533719us for edit/undo,
with 19964 kB peak RSS and zero native idle ticks. PTYs measured 49753-65426us
startup, 12711-32337 output bytes and 12972-14268 kB peak RSS, all with zero idle
ticks and restored TTYs. Final worktree-local target footprint was 1012 MiB.

Gate preparation at 2026-11-03T16:47:53Z verified the installed Rust 1.98.0
toolchain, Bash, rg, util-linux script, stty, kill, GNU timeout, C compiler,
Spore and guide asset utilities. `cargo metadata --locked --offline --format-version 1`
resolved 91 cached packages to the primary worktree's own target; no dependency
acquisition or build was needed. Linux procfs is readable and an owned,
five-second PTY admission/restoration probe passed and exited. Shell drivers
passed Bash syntax checks. Filesystem admission was 6305 MiB free with
7025 MiB available host RAM, above the plan's 1 GiB build admission floor.

The driver recreates and owns every finite fixture per run; no persistent
service, server, provider, font install or credential is needed. Its eight-second/
1-MiB PTYs, native bounds and one-job release build remain unchanged. The guide
build derives `target/userguide` from canonical Markdown and validates locally.
All three canonical PR merges are ancestors of dev and recorded in Acta.
The controller subsequently passed the unchanged full gate on the clean
preparation commit recorded below. The prerequisite observations above remain
separate from that integrated gate evidence.

## Delivery-set closure

The controller passed the unchanged configured Full gate at immutable candidate
`18f625997de4c0ac2e1ae50d6fea667fe14f2c2d`, receipt
`T2QVMEBIU5HJTNBIOEXKDCUFSU`. It ran on primary dev from
2026-11-03T16:49:10Z to 2026-11-03T16:58:37Z, taking 566.81 seconds.
The exact combined command remains in the plan/tracker Full gate metadata.
Evidence is retained at
`/home/adrian/.local/state/looper/configs/rev-org-headings.json.state/checks/18f625997de4c0ac2e1ae50d6fea667fe14f2c2d.json`,
with its receipt-specific `.T2QVMEBIU5HJTNBIOEXKDCUFSU.log` alongside it.

All 420 selected native tests passed: 161 commands, 149 configuration, 39 display,
16 headings, 11 Org visibility, 18 modal contexts, 21 external-change and 5
native AI-result cases. Format, strict all-target Clippy, source quality,
both-profile native/resource acceptance and all four size/resize PTYs passed.
Documentation verified 431 commands, 235 settings, 70 reachable pages and 438
local links/anchors. All 70 Markdown pages projected and built locally through
Spore under `target/userguide`.

The 778898-byte/16896-line/512-heading fixture measured 310883us startup,
22032us for 64 toggle/projections, 551671us for level edit/undo and 20236 kB
peak RSS. Native idle consumed zero ticks over 350ms with no pending deadline.
PTY startup was 51125-70834us, output 12711-32337 bytes and peak RSS
13036-14492 kB. All four measured zero idle ticks and restored terminal state.
The eight-second/1-MiB PTY limits and finite native bounds were unchanged.

The completed gate's primary target was 29157 MiB including retained historical
build artifacts; the Slice 3 worktree target was 1012 MiB. Filesystem free space
was 6033 MiB at closure inspection. Builds used the pinned debug-disabled,
non-incremental, one-job policy and worktree-local output. Fixture guards and
owned terminal children completed; the journey's EXIT trap removed its private
roots. No persistent fixture was started for this delivery.

Canonical PRs #238, #239 and #240 are integrated and recorded in Acta. All three
reports are delivered; plan/tracker and the paired specification/model are
Completed, and the pending index records this delivered capability. Only the
configured closure documentation differs from the tested candidate. The
controller retains final checkpoint verification.

## Risks and Follow-ups

No implementation follow-up is recorded for this delivery. Emitted Unicode
protocol evidence does not establish the hosting terminal font's rasterization
or glyph coverage.
