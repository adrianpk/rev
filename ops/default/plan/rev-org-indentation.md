# Org Indentation Delivery

Status: Approved
Date: 2026-11-05
Delivery set: rev-org-indentation
Slice strategy: layered
Approved: product owner, 2026-11-05; specification approved, four-slice preparation and one autonomous Looper delivery through dev requested
Planning baseline: `0d4f859de075606743d9a8c7bae207094e4664cd`
Spec: [Org Visual Indentation](../spec/editor-org-indentation.md)
Model: [Org Visual Indentation Model](../spec/editor-org-indentation-model.md)
Tracker: [Org Indentation Tracker](../tracker/rev-org-indentation.md)
Concern: [Pending Work Index](../README.md)
Active slice: Slice 2 - Configured editing presentation

## Outcome and strategy

Deliver OIN01-OIN05 in one retained Looper lifecycle through four slices, within
the owner's limit of six. The user surface is one boolean and three commands:
`org.indent`, `org-indent-mode`, `org-indent-enable`, `org-indent-disable`.
Enabled body text follows its heading; disabled returns current presentation.
The startup default is enabled, with two visual cells per level and a
buffer-local runtime override. Source text is never padded or reformatted.

Layered slicing is justified because a margin affects rendering, wrap, cursor,
hit-testing and overlays together. Slice 1 supplies a passing internal geometry
layer; Slice 2 makes the complete configured path usable; Slice 3 adds runtime
controls; Slice 4 establishes joint acceptance and guidance. Do not expose a
setting or command backed by incomplete mapping, or commit failing/skipped
placeholder tests. No fifth hardening slice is reserved automatically.

The owner's approval and startup request authorize preparation commit/push on
dev, the four dedicated slice worktrees and canonical PRs, controller-owned
Rebase + Fast-forward integration into dev, the selected final gate and
documentation closure. Use installed Looper help and one retained controller;
the owner does not need to merge individual slice PRs. Main promotion, releases,
public publication and installation are separate operations.

The delivered prerequisites are native Org recognition, shared source/cell
coordinates, wrapping, folding, gutter/bookmark display, pointer selection,
completion anchors, startup TOML and both interaction profiles. No new external
prerequisite or dependency is needed. Follow global instructions and applicable
Rust overlays; preserve the existing owners rather than add a second layout.

Bounded native recovery may diagnose at most six incidents, at most two per
family, retry two restored environmental prerequisites and integrate at most
two final-gate repair PRs within OIN01-OIN05. Repairs do not add slices, alter
the frozen gate, weaken assertions, expand product scope or change dependencies.
The same run, conversation and consumed budgets remain authoritative. Unknown
semantic decisions still require owner direction.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Projection and checked mappings | `feat/rev-org-indentation-01-projection` | `feat(org): model virtual indentation in checked layout` | `ops/default/report/slices/rev-org-indentation/slice-1-projection.md` |
| Slice 2 | Configured editing presentation | `feat/rev-org-indentation-02-presentation` | `feat(org): align heading bodies with configured visual indentation` | `ops/default/report/slices/rev-org-indentation/slice-2-presentation.md` |
| Slice 3 | Buffer-local runtime commands | `feat/rev-org-indentation-03-controls` | `feat(org): control visual indentation per buffer` | `ops/default/report/slices/rev-org-indentation/slice-3-controls.md` |
| Slice 4 | Joint acceptance and guidance | `test/rev-org-indentation-04-acceptance` | `test(org): verify visual indentation and document controls` | `ops/default/report/slices/rev-org-indentation/slice-4-acceptance.md` |

## Slice 1: Projection and checked mappings

Dependency: approved preparation on clean dev. Requirements: OIN01, OIN03, OIN05.

- T1.1: Extend existing Org/view owners with current enclosing-heading evidence,
  virtual-margin and fragment mappings. Inspect `src/org`, `src/buffer`,
  `src/view/{headings,wrap,visual,hit}.rs` and `src/coordinates`; retain one
  source authority and preserve disabled-mode geometry.
- T1.2: Make the internal path account for preamble, parent/deeper/sibling
  transitions, literal blocks, source tabs/Unicode, continuation alignment,
  horizontal clipping and narrow-pane caps without document-wide frame scans.
  Include effective policy and structural coverage in layout invalidation.
- T1.3: Verify the internal projection and round-trip mappings at their native
  owner boundaries, with table-driven cases, source immutability and bounded
  deep/long input. No public configuration or command surface is required in
  this slice; introduce `tests/org_indentation.rs` when Slice 2 exposes the
  integrated path rather than publish a test-only production API.

## Slice 2: Configured editing presentation

Dependency: Slice 1 integrated. Requirements: OIN01-OIN03, OIN05.

- T2.1: Admit `org.indent` with complete built-ins, typed composition, provenance,
  coherent invalid-candidate handling, startup snapshot, exhaustive sample and
  current configuration reference. Enable the complete path at the chosen
  default, not a renderer-only padding pass.
- T2.2: Integrate rendering and cursor with wrap/Visual Line movement, source
  navigation, pointer/drag, selection/rectangles, search and completion anchors,
  gutters/bookmarks and folding. Preserve source-only copy/save/edit history,
  table alignment, code blocks and independent pretty-heading/source choices.
- T2.3: Extend real frame/session tests for enabled and disabled operation,
  continuations, tabs/wide graphemes, panes, logical line identity and stale
  observations. Pin legacy unchanged-geometry assertions to `org.indent=false`
  where they describe the old policy; add real enabled geometry expectations.
  Do not weaken assertions or disable coverage to accommodate the new default.
  Retain the same explicit policy in older heading-acceptance fixtures whose
  purpose is to verify unchanged source/pretty geometry.

## Slice 3: Buffer-local runtime commands

Dependency: Slice 2 integrated. Requirements: OIN02-OIN05.

- T3.1: Wire the three named controls through the shared native M-x/colon
  catalog and existing configurable keymaps, including read-only Org results,
  explicit state feedback, outside-Org rejection and idempotent enable/disable.
  Add no default shortcut and do not write TOML.
- T3.2: Preserve one buffer-owned choice across multiple views, split/rebind,
  source edits and undo/redo, heading moves, folding, resize, external reload,
  Org suspension and reactivation. Retire the override on buffer close.
- T3.3: Verify those transitions in native sessions under both interaction
  profiles, including independence from per-view pretty-heading overrides and
  buffer-local wrapping. Update command/configuration catalogs and shipped
  Unreleased behavior with the functioning surface.

## Slice 4: Joint acceptance and guidance

Dependency: Slice 3 integrated. Requirements: OIN01-OIN05 together.

- T4.1: Add a bounded Rust acceptance driver and Bash entrypoint at
  `scripts/check-org-indentation.sh`, reusing existing owned PTY/frame helpers.
  Exercise one representative toggle/edit/wrap/pointer journey in each profile,
  terminal restoration, large/deep outline work and repeated reflow retention.
  Account actual cache bounds and no new idle deadline. No Python prerequisite.
- T4.2: Add concise enabled/disabled text examples and controls to the Markdown
  Org learning path; update exact reference/configuration/command entries and
  derive the Spore guide from Markdown. Reconcile the heading and wrapping
  specification/model clauses named by the new spec without relabeling history,
  including the soft-wrap specification's continuation contract.
  Keep Org guidance in its own learning path and update Unreleased only.
- T4.3: Record all four reports, requirements/tasks/commits, native and PTY
  evidence, bounds and limits. Hand off the integrated exact dev candidate to
  the controller for the selected gate; close artifacts only after it passes.

Each slice uses one canonical PR to dev and a durable report at its mapped path.
Use Conventional Commits without task suffixes. Keep PR bodies in the established
Summary, Changes, Validation format, with successful checks as Validation bullets.
Reports are written during implementation, not fabricated during planning.

## Slice validation

All slice checks use the existing low-debug, one-job development profile:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
The local Looper configuration pins the following ordered commands per slice;
the prefix above applies to each Cargo command except formatting.

| Slice | Required checks in order |
| --- | --- |
| Slice 1 | `cargo fmt --all -- --check`; `cargo clippy --locked --lib --bin rev -- -D warnings`; `cargo test --locked --lib view:: -- --test-threads=1`; `cargo test --locked --test org_headings --test wrapping -- --test-threads=1`; `git diff --check`. Put private projection acceptance under its view owner so the native filter includes it. |
| Slice 2 | `cargo fmt --all -- --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo test --locked --test org_indentation --test org_headings --test wrapping --test display --test pointer --test gutter --test configuration -- --test-threads=1`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |
| Slice 3 | `cargo fmt --all -- --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo test --locked --test org_indentation --test commands --test configuration --test external_changes -- --test-threads=1`; `cargo test --locked --test modal contexts:: -- --test-threads=1`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |
| Slice 4 | `cargo fmt --all -- --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo test --locked --test org_indentation --test org_headings -- --test-threads=1`; `bash -n scripts/check-org-indentation.sh`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |

The real PTY/resource journey and strict Spore build run in the controller's
combined gate after integration, not once in every slice. Slice 4 may use
focused driver checks during development without claiming the final gate ran.
Compilation dominates cold validation; existing selected native suites should
remain a minutes-scale gate, not a whole-repository multi-journey run. Record
actual timing and diagnose unexpected cost without raising deadlines silently.

## Final validation selection

During slices, run format/compile and relevant native filters for changed
behavior. The final gate protects shared projection and affected configuration,
commands and source lifetimes, without chaining historical delivery scripts:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_indentation --test org_headings --test wrapping --test display --test pointer --test gutter --test configuration --test commands -- --test-threads=1
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_editing visibility -- --test-threads=1
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test external_changes -- --test-threads=1
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-org-indentation.sh
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh
git diff --check
```

These are required delivery checks, not claims of passing planning validation.
The new test target and acceptance entrypoint are created in their assigned
slices. Existing native/PTY helpers avoid a second test framework; Spore runs
once after the Markdown changes. Geometry changes justify selected shared-suite
regressions; unrelated Git, terminal, LSP-provider and Babel journeys are not
added. Record actual gate duration and any failure before considering a retry;
never expand an active controller's limits or substitute a prior candidate.
