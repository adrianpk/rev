# Slice 3: Core editing and projection

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-03-core`
PR: pending

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
Selected Clippy validation is in progress.

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

The complete source-only selection passes, and `cargo check --locked -p rev --lib`
passes with the required bounded build environment. The initial scoped Clippy
scan failed on existing ownership/representation diagnostics. The updated
all-target profile is running; the frozen native regression commands and final
clean-HEAD Looper validation remain pending.

## Risks and Follow-ups

Other approved owners and the final integrated gate remain controller-owned
delivery work. Scoped quality evidence does not establish full-workspace adoption.
