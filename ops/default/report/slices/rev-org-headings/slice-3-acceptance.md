# Slice 3: Joint acceptance and guidance

Status: reviewing
Delivery set: rev-org-headings
Plan: [Heading Presentation Delivery](../../../plan/rev-org-headings.md)
Tracker: [Heading Presentation Tracker](../../../tracker/rev-org-headings.md)
Branch: `feat/rev-org-headings-03-acceptance`
PR: pending

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

After canonical PR references are committed/pushed, Looper records these exact
ordered checks on the clean PR head. The canonical PR and retained receipt own
that final result; the controller later records integrated gate evidence.

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-org-headings.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `git diff --check`

Gate prerequisites are present locally: the installed Rust toolchain and cached
locked dependencies, Linux procfs, Bash, rg, util-linux script, stty, kill,
GNU timeout and Spore. The driver recreates and owns all fixtures per run;
no persistent service, server, provider, font install or credential is needed.
The guide build derives `target/userguide` from canonical Markdown and validates
Spore locally. No publication follows this slice. Development target footprint
was 935 MiB with 6335 MiB filesystem space and 6765 MiB available RAM before the
final checks; all build output remains worktree-local.

## Risks and Follow-ups

The integrated full gate and delivery closure remain controller-owned and
pending integration. Emitted Unicode protocol evidence does not establish the
hosting terminal font's rasterization or glyph coverage.
