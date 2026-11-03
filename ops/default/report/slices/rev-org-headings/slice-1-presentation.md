# Slice 1: Configured native markers

Status: delivered
Delivery set: rev-org-headings
Plan: [Heading Presentation Delivery](../../../plan/rev-org-headings.md)
Tracker: [Heading Presentation Tracker](../../../tracker/rev-org-headings.md)
Branch: `feat/rev-org-headings-01-presentation`
PR: `#238`

## Purpose

Display configured markers on recognized native Org headings while retaining
canonical stars, prefix width, editing coordinates and bounded observation.

## Delivered Behavior

- Complete startup defaults enable pretty headings, the six reference markers,
  leading-star hiding and cyclic overflow. Includes replace the entire marker
  array and retain each leaf's assigning source.
- Lists contain 1..32 markers, each at most 32 UTF-8 bytes, one printable
  non-whitespace grapheme and one terminal cell. The total payload cannot exceed
  1024 bytes. Invalid candidates, including invalid overridden includes, fall
  back coherently to complete built-ins.
- Levels 1..8 render with the reference cycle. Custom markers, final-marker
  overflow, visible leading stars and plain source presentation are available
  through startup preferences. Disabled Org mode remains disabled.
- Current complete native structure authorizes decoration. Absent, incomplete,
  canceled or stale observations retain source rendering. Syntax coloring can
  be disabled or byte-limited without disabling heading observation.
- Each original star retains its own cell, including while point is in the
  prefix. Cursor, selection, copy, clipping, gutters, Unicode title boundaries,
  wrap positions, save bytes, dirty state and history retain their meanings.

## Implementation Notes

The existing Org worker supplies the exact-stamp outline. A binary search for
an exact heading start uses that owner's structural level; the frame neither
parses nor scans the document. Projection replaces only the visible ASCII
prefix cells after canonical clipping or word wrapping. No extra document
index, idle scanner or source/display coordinate system is retained.

The immutable view policy holds a bounded shared marker list, retained on
rebind and copied by reference on split. The existing syntax roles continue to
color the projected cells. No font, parser, dependency or global setting is
installed. Native width/segmentation admission also accepts supported combining
and private-use graphemes without claiming terminal glyph coverage.

The literal fixture uses the native source-block editor's documented comma
escaping for star-leading content. Raw column-zero stars are native heading
tokens; the fixture correction preserves the actual literal-block ownership
contract rather than changing structural recognition for presentation.

## Contracts Added or Changed

- OGP01, OGP03 and OGP05: level-based markers, native structural admission and
  the complete four-field startup catalog, sample, template and schema checks.
- OGP02 and OGP04: source/history fidelity and unchanged prefix/cell/wrap
  geometry, verified against source presentation through real native frames.
- OGP07 and the native portion of OGP08: terminal font ownership, bounded
  viewport projection, current evidence and observation with coloring disabled.

## Files of Interest

- `src/config/headings.rs`, `src/config/{merge,project,validate}.rs`: typed
  preferences, bounded glyph admission, composition and provenance.
- `src/org/visibility.rs`, `src/buffer/org.rs`: current complete heading lookup.
- `src/display/headings.rs`, `src/display.rs`: viewport-only prefix projection.
- `src/view.rs`, `src/session/org.rs`: startup policy and existing worker demand.
- `tests/org_headings.rs`, `tests/org_headings/`: native session, observation and
  source/display geometry coverage.
- `tests/configuration/org_headings.rs`: include/provenance and glyph bounds.
- `docs/reference/{config.toml,defaults.md}`, `CHANGELOG.md`: usable preferences,
  font ownership and the user-visible Unreleased outcome.

## Validation

Focused development checks passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration org_headings:: -- --test-threads=1` (2 tests).
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test display org_colors:: -- --test-threads=1` (3 tests).
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings -- --test-threads=1` (7 tests, 7.96 seconds).
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings geometry::unchanged_geometry -- --test-threads=1` (expanded clipping assertions, 1 test, 5.86 seconds).
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_editing projection:: -- --test-threads=1` (3 tests).
- `git diff --check`.

Both profiles and coloring states cover levels 1..8 and exact BOM/CRLF save
bytes. The geometry cases compare every visible pointer target, cursor,
selection and continuation row at widths 40 and 80, wrapping on/off and five
horizontal offsets. Byte/node-limited and canceled observations remain literal;
a current edit and undo recompute the native level without presentation history.

The first controller batch `3HWITYJ7MVJSW5LBIOEJKZ56RW` at
`3c9b56432d4fd5b2c2f1105634bd0bcd4aa32e2c` passed format, strict Clippy
and all 149 configuration tests. It stopped on `tiny_geometry` and
`org_timestamps::timestamp_calendar_geometry`: passive fold retirement marked
the view as having an explicit visibility choice even without active folds.
`reveal_fold` now retires only stale derived state; explicit `clear_folds`
retains its initialization-suppression semantics. Existing assertions remain
unchanged. Focused correction checks passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test display tiny_geometry -- --test-threads=1` (1 test).
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test display org_timestamps::timestamp_calendar_geometry -- --test-threads=1` (1 test).
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::ai::results -- --test-threads=1` (4 native source/result and initial-drawer cases).

Canonical PR #238 merged through controller-owned Rebase + Fast-forward into
dev at `5e662a9691b7ab08c7c89dd37a1f7d3d1dce217a` on
2026-11-03T15:52:56Z. Receipt `DT6X246MDOZQJMVKQPBPAAAEQF` passed the
exact five configured commands at that clean head: format, strict all-target
Clippy, 195 tests (149 configuration, 39 display, 7 headings), documentation
and whitespace. Documentation verified 429 command identities, 235 settings,
70 pages and 434 local links/anchors. The batch took 113.94 seconds.

Final target footprint was 732 MiB with 3981 MiB filesystem space available.
Builds retained debug-disabled, non-incremental, one-job policy and the
worktree-local target directory. This closes Slice 1 of 3.

The later integrated delivery-set gate passed at
`18f625997de4c0ac2e1ae50d6fea667fe14f2c2d`, receipt
`T2QVMEBIU5HJTNBIOEXKDCUFSU`. All three slices are delivered and the set is
Completed. The [Slice 3 closure](slice-3-acceptance.md#delivery-set-closure)
records the combined 420-test, native/resource/PTY and local guide evidence.

## Risks and Follow-ups

All planned successor slices are delivered and the integrated gate passed.
Terminal glyph coverage remains a terminal/font property.
