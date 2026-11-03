# Org Heading Presentation Delivery

Status: Approved
Date: 2026-11-03
Delivery set: rev-org-headings
Slice strategy: behavior-first
Approved: product owner, 2026-11-03; approved heading-presentation scope and explicit autonomous Looper startup; the agent derives this three-slice map under that authority
Planning baseline: `2324b522009266c331e528293aa80b1414b8765d`
Spec: [Org Heading Presentation](../spec/editor-org-heading-presentation.md)
Model: [Heading Presentation Model](../spec/editor-org-heading-presentation-model.md)
Concern: [Pending Work Index](../README.md)
Full gate: `cargo fmt --all -- --check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings --test configuration --test commands --test display -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_editing visibility -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test external_changes -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::ai::results -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-org-headings.sh && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh && git diff --check`
Active slice: Slice 3 - Joint acceptance and guidance
Tracker: [Heading Presentation Tracker](../tracker/rev-org-headings.md)

## Outcome and authority

Deliver the approved OGP01-OGP08 profile through one retained Looper lifecycle:
factory heading markers and complete startup preferences, independent native
view controls, checked source/display behavior, terminal acceptance and guidance.
The three-slice decomposition is preparation under the owner's approved scope
and explicit startup instruction, not a claim of separate review of each task.

This authorizes preparation commits/push to dev, dedicated slice worktrees,
one canonical PR per slice, controller-owned Rebase + Fast-forward integration
into dev, the exact final gate and documentation closure. Looper owns progress;
do not create a parallel tracker or request individual human slice merges.

The existing general-drawer-visibility and resolved AI-presentation captures
are versioned during preparation after the previous Pi set closed. Neither
becomes work in this set. All other tickets, native API, terminal and extension
proposals retain their own boundaries. Main promotion, releases, mirrors,
public website deployment and global editor installation are separate actions.

## Strategy and delivered prerequisites

Three behavior-first slices keep each integrated state useful. Slice 1 provides
configured rendering of recognized headings with correct unchanged-width source
geometry. Slice 2 adds the shared command surface and completes view/editing
lifecycle acceptance. Slice 3 connects the complete behavior to native/resource/
PTY evidence and the Markdown-derived Spore guide.

Delivered owners include the native Org index/outline and bounded analysis job,
buffer revision/history/readonly admission, scalar/cell/grapheme coordinates,
word-aware wrapping, per-view folds, native panels/pointer selection, modal
editing, immutable startup TOML and its exhaustive sample, M-x/colon selectors
and read-only Org result buffers. No new dependency, Unicode-width engine,
parser, font package, language server or AI provider is needed.

Inspect actual private boundaries before changing them. Reuse src/org,
src/buffer, src/config, src/view, src/display, src/session, src/window and the
shared command/input owners. Source stars remain canonical. Blank leading
cells and the final marker retain the original prefix width; do not add body
indentation, automatic prefix reveal or another source-to-display mapping.

Follow the applicable global instructions and Rust overlays. Use Conventional
Commits without task IDs in subjects. Keep the exact PR title, one report per
slice and Summary/Changes/Validation body structure with an immutable report
link. Update current configuration/command catalogs when each surface becomes
usable, and update Unreleased through changelog-normalize for shipped outcomes.
Do not advertise future handlers or change existing test assertions merely to
hide a regression.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Configured native markers | `feat/rev-org-headings-01-presentation` | `feat(org): display configurable native heading markers` | `ops/default/report/slices/rev-org-headings/slice-1-presentation.md` |
| Slice 2 | Independent view controls | `feat/rev-org-headings-02-view-controls` | `feat(org): control heading presentation per view` | `ops/default/report/slices/rev-org-headings/slice-2-view-controls.md` |
| Slice 3 | Joint acceptance and guidance | `feat/rev-org-headings-03-acceptance` | `test(org): verify heading presentation and document controls` | `ops/default/report/slices/rev-org-headings/slice-3-acceptance.md` |

## Slice 1: Configured native markers

Prerequisites: this approved preparation committed/pushed on dev; no active Rev
controller. Requirements: OGP01-OGP05, OGP07 and the native-work portion of OGP08.

- T1.1: Implement all four org.headings preferences, complete built-ins, typed
  composition/provenance, replacement of whole marker arrays, bounded single-cell
  admission and coherent whole-candidate failure. Update the exhaustive sample,
  packaged template, defaults reference and schema checks together.
- T1.2: Project recognized heading prefixes with the six default cyclic markers,
  configurable final-marker retention and leading-star hiding. Use current native
  structural evidence; retain literal/unverified source and the original cells,
  separator/title boundary, wrap points, source point and selection mappings.
  Trigger bounded native observation even when syntax coloring is disabled;
  never parse or scan the whole document from the frame renderer.
- T1.3: Establish tests/org_headings.rs for real native configuration/session/
  frame behavior: levels 1-8, custom/invalid lists, hide/source combinations,
  literal blocks, stale/incomplete evidence and exact source/dirty/history
  fidelity. Keep existing relevant rendering assertions honest.

A disabled Org mode remains disabled. Reuse existing heading palette roles and
native segmentation/width policy; no automatic font detection or new styling
system. Slice 1 need not expose Slice 2's commands.

## Slice 2: Independent view controls

Dependency: Slice 1 integrated. Requirements: OGP02-OGP04, OGP06, OGP08.

- T2.1: Implement org-pretty-headings-mode and
  org-reset-heading-presentation through the shared M-x/modal-colon catalog,
  effective existing keymaps and native read-only admission. Add no factory
  shortcut. Only the pretty flag has a runtime override; do not write TOML.
- T2.2: Preserve independent per-view choices through split, buffer switch,
  reset, local edits, undo and external reload. Keep source-backed cursor/click/
  selection geometry over every prefix cell, no automatic reveal, and unchanged
  word-wrap, gutter and ordinary structural-editing behavior.
- T2.3: Extend native cases in the same focused suite for both profiles,
  readonly Org/AI results, region/Visual copy and editing, literal contexts,
  reload invalidation, cancellation and split/reset independence. Reuse existing
  visibility and modal-context regressions rather than duplicate every motion.
  Cover original generated-text acceptance through the existing native AI owner,
  without starting a model or Pi process.

## Slice 3: Joint acceptance and guidance

Dependency: Slice 2 integrated. Requirements: OGP01-OGP08 together.

- T3.1: Add a bounded Rust org-heading-acceptance example and Bash
  scripts/check-org-headings.sh. Exercise native fixtures and real Linux PTYs in
  both profiles at 40x12/80x24 and resize, with explicit source/pretty/reset,
  source-star copy, emitted reference markers, source point and terminal
  restoration. Reuse existing PTY/source-quality helpers. Use a finite large
  Org fixture and measure projection/startup/idle behavior without an idle
  polling loop or per-line decoration store. Require no font installation,
  Python harness, language server, package manager or provider credentials.
- T3.2: Update the existing Org learning path, references, exhaustive sample,
  native catalogs and Unreleased. Show short source/pretty feedback and the two
  actual command names. Keep fonts/ligatures owned by the hosting terminal.
  Derive Spore from canonical Markdown and validate it locally; do not publish.
- T3.3: Complete the existing source/display, view lifecycle and source-byte
  evidence, create the final report and return ready_for_gate. The controller,
  rather than the worker, executes the unchanged full gate on clean integrated
  dev. Close approved artifacts only after its receipt passes.

## Required validation prefixes

The following lists are pinned exactly in the local Looper configuration.
Use them in order through looper validate on a clean exact HEAD. Run additional
focused experiments separately before the final required validation; do not
insert them into the frozen prefix or report unexecuted checks.

### Slice 1

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings --test configuration --test display -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

### Slice 2

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings --test commands --test display --test external_changes -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

### Slice 3

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-org-headings.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `git diff --check`

## Integrated gate and execution cost

The Full gate metadata is the exact combined command. It covers the focused
heading/configuration/command/frame suites, existing Org visibility and modal
contexts, real external reload, native AI result-byte guards, the new real PTY
journey, documentation and the Markdown-to-Spore build. It deliberately does
not chain historical Org/LSP/AI deliveries or require their external services.

All Cargo-building commands use CARGO_PROFILE_DEV_DEBUG=0,
CARGO_PROFILE_TEST_DEBUG=0, CARGO_INCREMENTAL=0 and CARGO_BUILD_JOBS=1.
The Bash journey must inherit these settings and use at most one Cargo job for
its release build. These reduce retained build/debug artifacts and concurrent
compiler memory; they do not alter product behavior or skip assertions.
Apply the same environment to exploratory Cargo commands, not just validation.
Keep a separate default target directory per worktree. Do not share mutable
targets across branches or place Cargo output on memory-backed /tmp.

Startup inspection found approximately 4.4 GiB free on the build filesystem and
2.6 GiB available host RAM. Budget one compiler job and a serial test runner.
Before each new build, check disk admission; below 1 GiB free, request bounded
native environmental recovery rather than launch another build, delete unknown
worktrees or interfere with another project's controller. Do not alter the
user's global Cargo configuration or erase existing primary build artifacts.
Record actual slice build footprint in the report so later slices can use the
same admission decision.

Expected warm native checks are minutes, not hours; a cold slice/release build
can add several minutes. Measure the final gate duration and selected test
counts. Keep repeated end-to-end execution to the one acceptance journey and
configured final gate, using focused native cases during implementation.
Protocol/PTY evidence proves emitted cells and lifecycle, not font rasterization.

## Bounded recovery and closure

Use the installed bounded-v1 recovery profile, with at most two semantic gate
repair PRs under fix/rev-org-headings-validation-N, linked to the existing
Slice 3 report. Repair only this approved heading/config/view/doc surface and
its focused acceptance; preserve the full gate, task/slice count, conversation,
exact-candidate evidence, budgets and prior history. The local pinned policy
defines exact file/directory boundaries, focused commands and excluded tickets.
Do not update Looper, dependencies, scope or other projects to make a check pass.

Preparation/read-only diagnostics and bounded environmental retries retain the
native controller's existing policies. Ambiguous decisions, excluded changes,
uncertain merges and unavailable capacity remain explicit blockers; never
manufacture an approval or an extra slice.

After the final integrated gate passes, mark the plan/tracker Completed, all
three reports delivered, the specification/model Completed and the pending
index delivered. Change only the generated, reviewed closure_paths. Runtime,
test, shipped reference and guide edits belong before the gate. No extra
provider probe, release, installation, successor or main merge follows closure.
