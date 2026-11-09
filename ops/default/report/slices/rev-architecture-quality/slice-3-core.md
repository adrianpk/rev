# Slice 3: Core editing and projection

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-03-core`
PR: pending
Implementation: bab0731eebfa4b92059590c56f112213b45b0163

## Purpose

Resolve the approved core editing and projection ownership, representation and
source-bound findings while preserving native editing and view contracts.

## Delivered Behavior

Edit admission separates capacity checks and mapped anchors from candidate
publication. Reload prefix matching and macro line-start retention have explicit
steps. Existing source revisions, local history and complete shared-point mapping
remain in the canonical buffer and window owners.

Projection separates command annotations, startup artwork/control parsing,
semantic colors, gutter markers and tree prefixes. Wrapped-line resumption uses
the native Cell row/column type. Viewport movement and unwrapped reveal keep
source positions separate from disposable terminal geometry.

Shared cache, grammar and terminal handles use explicit reference-counted clones.
Coordinate and wrap-cache guards release before returning observations. Private
conversion inputs borrow; the final automatic-history group moves to its last
buffer rather than being cloned again.

## Implementation Notes

The complete frozen source selection covers 158 native files. All initial 23
source-bound diagnostics now pass without changing the five/three/ten limits.
The complete scoped Clippy driver also passes with zero blocking diagnostics;
1,285 selected-profile diagnostics in later owners remain explicitly deferred.

Native integer narrowing uses checked conversions after source, pane, popup,
selection, emulator or policy bounds. Float-to-cell rounding remains deliberate:
local reasoned expectations retain the existing tree fraction and pane allocation
flooring/saturation. Public delete, kill, rename and save publication consume
captured authority even on rejection; their exact signatures are preserved with
local reasoned needless-pass-by-value expectations.

The internal Git renderer consumes an already prepared frame. Its five session
callers only adapt frame construction. The grammar registry borrows its optional
root; its two configuration callers adapt that internal signature.

## Contracts Added or Changed

Public editing APIs, source units, revision checks, history and resource limits
remain unchanged. Borrowed presentation descriptions have explicit copy semantics.
No dependency, wire, state or configuration-format change is introduced.

## Files of Interest

- `src/buffer/`: checked editing and source ownership.
- `src/coordinates/`, `src/view/`, `src/window/`: source and view projection.
- `src/display/`, `src/syntax/`, `src/gutter/`, `src/tree/`, `src/term/`: native
  rendering and derived state.

## Validation

Ad hoc checks used clean implementation HEAD bab0731eebfa4b92059590c56f112213b45b0163.
The frozen ordered acceptance sequence has not passed; no final Looper receipt
or canonical PR is recorded.

The required commands have these outcomes:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/lib.rs --scope src/main.rs --scope src/search.rs --scope src/search --scope src/rectangle.rs --scope src/rectangle --scope src/pointer.rs --scope src/pointer --scope src/panel.rs --scope src/panel --scope src/kill.rs --scope src/jump.rs --scope src/buffer.rs --scope src/buffer --scope src/coordinates.rs --scope src/coordinates --scope src/view.rs --scope src/view --scope src/window.rs --scope src/window --scope src/display.rs --scope src/display --scope src/syntax.rs --scope src/syntax --scope src/gutter.rs --scope src/gutter --scope src/tree.rs --scope src/tree --scope src/term.rs --scope src/term` — passed. Formatting, global first-party compiler/all-group admission, all 158 selected sources and the five/three/ten source bounds pass; coverage is 1,799 Rust files, two packages and 73 Cargo roots, with two roots selected.
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing --test coordinates --test display --test windows --test syntax --test pointer --test gutter --test tree --test core -- --test-threads=1` — failed. Coordinates passed 24 tests and core passed six; display passed 44 and failed three before Cargo stopped the remaining targets.
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check` — passed. Documentation covers 106 reachable pages, 743 local links/anchors, 684 commands and 299 configuration entries.
4. `git diff --check` — passed.

Additional native checks retain the existing inputs, assertions and limits:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- --test-threads=1` — 185 passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test wrapping --test integrated_terminal -- --test-threads=1` — 49 integrated-terminal and 26 wrapping tests passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- presentation --test-threads=1` — 66 presentation tests passed; 176 other Git cases were filtered out.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing --test windows --test syntax --test pointer --test gutter --test tree -- --test-threads=1` — editing passed 114 tests; gutter passed 36 and failed one, then Cargo stopped that focused command.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --no-fail-fast --test pointer --test syntax --test tree --test windows -- --test-threads=1` — pointer passed 68 tests, syntax 30, tree 84 and windows 41. This focused diagnostic run continued between targets; it does not replace the frozen command.

An isolated clean worktree at pre-slice HEAD 8bae0f8596db70668725f375affcccd2fac1012d
reproduced all four failures without Slice 3 changes:

- `display::org_timestamps::timestamp_calendar_geometry`: the selected View gains derived visibility/fold state and fails the unchanged equality assertion.
- `display::org_timestamps::timestamp_calendar_pointer`: the expected `[30]` date selection is absent.
- `display::palette::panel_colors`: the observed first entry uses a directory foreground where the assertion expects a configuration foreground.
- `gutter::surfaces::wrapped_overlays`: a highlighted row falls outside the asserted content area.

Baseline commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test display -- --test-threads=1` — the same 44 passed and three failed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test gutter -- surfaces::wrapped_overlays --exact --test-threads=1` — the same overlay assertion failed; 36 other gutter tests were filtered out.

## Approved Prerequisite Corrections

The maintainer approved bounded correction of the four reproduced baseline
failures while preserving meaningful coverage, geometry cases, resource limits
and every frozen command. All corrections are in the existing three test files;
no editor runtime change is required for these prerequisites.

- Calendar fixtures finish initial Org view preparation before a transient
  captures authority. Exact view preservation, invalid-cell rejection, mouse
  selection, source text, history and undo assertions remain.
- The palette test identifies `config.toml` by its source path. The parent
  navigation row no longer masquerades as the configuration entry.
- The wrapped overlay test checks active-file highlighting against the actual
  tree content area, the matching entry and active source. It retains every
  source overlay bound, exact searched/selected text and geometry case, including
  narrow frames where the tree is suppressed.

Focused calendar and wrapped-overlay reruns pass. The display target passes all
47 tests after the fixture updates. Full ordered clean-HEAD acceptance is still
pending; the earlier baseline failures remain above as historical diagnostics.

## Risks and Follow-ups

Canonical PR preparation and final clean-HEAD validation remain pending. The
approved fixture corrections preserve the original regression contracts.

Other selected-profile owners, staged enforcement and the full integrated gate
remain later delivery work. Scoped quality evidence does not establish full
workspace adoption. The existing vendored dependency warning remains outside
first-party ownership.
