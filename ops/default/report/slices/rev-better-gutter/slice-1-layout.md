# Slice 1: Outside marker layout

Status: delivered
Delivery set: rev-better-gutter
Plan: [Better Gutter Delivery](../../../plan/rev-better-gutter.md)
Tracker: [Better Gutter Tracker](../../../tracker/rev-better-gutter.md)
Branch: `feat/rev-better-gutter-01-layout`
PR: `#282`
Integrated: `8af7f4e02e77e31ca286d82b6651791b53e3eec8`

## Purpose

Deliver the outside bookmark-column geometry for BMK03, BTG02 and BTG04 while
preserving the existing number format and source-coordinate owners.

## Delivered Behavior

- Left-side markers precede the number field; right-side markers occupy its
  outer edge after the numbers. The separator stays beside source text.
- Number admission retains priority. A marker adds exactly one cell only when
  at least three source columns remain. Marker and number visibility remain
  independent, and wider panes restore the requested fields.
- Markers, numbers and source text occupy adjacent, disjoint areas with no
  folding column or folding-specific padding. Number padding is unchanged.
- Multiple bookmark names on a logical line share one marker. Wrapped
  continuations, filler and folded bodies retain their existing source identity.
- Pointer selection and copying use source text only. Completion follows the
  source cursor with either gutter side and marker visibility.

## Implementation Notes

T1.1-T1.3 are implemented in
`f4698fc7b506a610cbbef6aa12f6e5bd459be019`. These tasks share one cohesive geometry
change and its regression coverage.

`gutter::Geometry::with_marker` first admits the existing complete number field.
When the marker also fits, it partitions the outermost cell from the original
content and places the admitted number field inside it. All coordinate
construction continues through checked `Area` bounds.

Frame rendering, view layout, reveal, pointer hit testing, search, selection and
completion reuse their existing shared geometry owner. The prepared,
revision-qualified bookmark-row index and bounded visible-row painting remain
unchanged; no source scan, IO or persistence enters painting.

The gutter reference now illustrates both sides. One Unreleased outcome
describes the usable layout without claiming the later symbol/configuration work.

## Contracts Added or Changed

- Marker position moves outside the number field without changing total gutter
  width or source origin when the same fields are admitted.
- Exact area adjacency protects zero folding allocation across visibility,
  decimal widths, padding, narrow panes and maximal coordinate origins.
- Source bytes, revision, dirty state, copied text and history remain owned by
  the existing source and selection contracts.

## Files of Interest

- `src/gutter/geometry.rs`: admission and outside placement.
- `tests/gutter.rs`, `tests/gutter/geometry.rs`: exact areas and partition grid.
- `tests/bookmarks/markers.rs`: rendered order, names, folds, continuations,
  pointer selection, copy and source-state assertions.
- `tests/display.rs`, `tests/completion.rs`: initial frame and popup anchor.
- `docs/reference/gutter.md`, `CHANGELOG.md`: layout examples and user outcome.

## Validation

These required commands passed locally, in the approved order:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test gutter --test bookmarks --test display --test pointer --test wrapping -- --test-threads=1`
- `git diff --check`

The suites passed 223 tests: bookmarks 57, display 39, gutter 36, pointer 65 and
wrapping 26. The first combined run exposed the old blank-marker position in
`initial_frame`; its exact prefix was updated to the approved layout, verified
alone, and the complete required command then passed without skipped tests.

Additional focused checks passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test completion cursor_popup -- --exact --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`

The popup test covers both sides and both marker-visibility states, with resize
and source-history checks. Documentation validation covered 85 reachable pages
and 561 local links and anchors.

Observed local times were 3.36s for formatting, 39.39s for Clippy, 79.81s for the
successful suite command, 4.10s for focused completion and 1.51s for documentation.
The initial geometry-only run spent 2m15s compiling its new worktree target.

Looper receipt `LE6UT6QCD7YDRHULYWKTSCXVIZ` passed the same four required commands
against clean PR-reference HEAD `8af7f4e02e77e31ca286d82b6651791b53e3eec8`.
Canonical PR #282 was verified merged into dev at that same commit.
These slice results do not replace the controller-owned final gate.
Tests release their owned temporary fixtures; no server or PTY was started.

## Risks and Follow-ups

Fixed symbols, the font-support boolean and the compiled blue remain Slice 2.
Joint terminal acceptance, complete guidance and baseline reconciliation remain
Slice 3. Integration, the frozen final gate and successor startup belong to the
retained controller. No final-gate or physical terminal font result is claimed.
