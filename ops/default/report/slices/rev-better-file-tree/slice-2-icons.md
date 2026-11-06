# Slice 2: Font icons and geometry

Status: reviewing
Delivery set: rev-better-file-tree
Plan: [Better File Tree Delivery](../../../plan/rev-better-file-tree.md)
Tracker: [Better File Tree Tracker](../../../tracker/rev-better-file-tree.md)
Branch: `feat/rev-better-file-tree-02-icons`
PR: `#288`

## Purpose

Complete the fixed font catalog and shared cell geometry for BFT01-BFT04 while
preserving canonical tree observations, input, palette and visibility policy.

## Delivered Behavior

- The frozen `gutter.nerd_fonts` startup boolean selects the approved Nerd Fonts
  icons for decorated roots, closed/open directories and every file kind. Its
  default stays false. Minimal labels ignore the font choice.
- Both catalogs share checked four-cell disclosure/icon fields and the existing
  two-cell nesting step. Names align; directory slashes and incomplete-list
  diagnostics retain their meaning. The existing glyph path sanitizes controls
  and clips whole graphemes, leaving spaces for partial wide glyphs.
- Full-row activation and selection use cached canonical entries, including
  disclosure, icon, name, clipped glyph and remaining row space. Scroll, resize,
  hidden panels and restored focus keep their existing behavior.
- Semantic entry colors, focused selection precedence and NO_COLOR remain
  useful across all four presentation/font combinations. The gutter bookmark
  follows the shared font preference even when the tree is minimal.

## Implementation Notes

T2.1 is implemented in `bc83d6a729972d9e079e200926ce720dfde5f41c`. T2.2 and the T2.3 native interaction/font
regressions are in `b1cdf00fa429dad260fb1550f7590944e7d4660e`. T2.3 also updates the reference, defaults sample,
shared gutter documentation, existing Unreleased outcome and this report in its
introduction commit `ac3dee145daf7bebb7dd1a21c6148997fca3b864`.

`tree::Kind` owns both fixed catalogs. The tree retains the startup font choice
from its admitted snapshot and forwards it through automatic file-parent and
explicit directory root replacement. Changing the configuration file during a
session changes neither the tree catalog nor the gutter marker.

`presentation::Prefix` checks indent, field columns and glyph widths before
building the label. Rendering retains the same row owner and `Composition`
content bounds used by pointer input; canonical hit identity stays separate from
clipped labels. No new filesystem read, font probing, provider, dependency or
entry record is introduced. Directory observation limits remain unchanged.

Slice 1's report and tracker were closed after verifying canonical PR #287 merged
at its validated final HEAD `219b8de4e1fd0957cccc0fe8fd9b5994b6063ed8`.
The plan/tracker now identify Slice 2 as active.

## Contracts Added or Changed

- Decorated tree labels now consume the existing frozen `gutter.nerd_fonts` leaf.
  No second font key or runtime font mutation is added.
- The compiled catalog emits U+F1B2, U+F07B/U+F07C, U+F121, U+F0219, U+E615,
  U+F15B and U+F481 for the approved kinds. All use a one-cell icon field under
  rev's terminal-cell policy.
- Checked prefix construction reports coordinate overflow through the existing
  rendering error path. Filename sanitizing and clipping keep their owner.
- References document the complete font choice; the existing Unreleased file-tree
  outcome is extended rather than duplicated.

## Files of Interest

- `src/tree/presentation.rs`, `src/tree/entry.rs`: fixed catalogs and checked prefix.
- `src/tree.rs`, `src/tree/root.rs`, `src/session.rs`: frozen startup preference
  and retained root-replacement behavior.
- `src/display/tree.rs`: checked label projection through existing glyph clipping.
- `tests/tree/decoration.rs`, `tests/tree/geometry.rs`: exact symbols, root changes,
  standalone snapshot API, Unicode/control names and narrow/hidden/zero geometry.
- `tests/display/tree.rs`, `tests/pointer/tree_decoration.rs`,
  `tests/gutter/shared.rs`: color, NO_COLOR, canonical clicks, viewport and shared flag.
- `docs/reference/file-tree.md`, `docs/reference/defaults.md`,
  `docs/reference/config.toml`, `docs/reference/gutter.md`, `CHANGELOG.md`: guidance.

## Validation

These five required commands passed locally in order:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test tree --test display --test pointer --test panels --test gutter -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The suites passed 235 tests: tree 71, display 41, pointer 67, panels 19 and gutter
37. Both isolated NO_COLOR child checks also passed; the tree child covers all
four preference combinations. The native matrix exercises both symbol catalogs,
closed/open directories, root changes, palette overrides and exact minimal labels.

Narrow geometry uses fourteen widths from one to twenty-four columns with wide
and combining names and control-text sanitization, plus hidden/zero surfaces and
restoration. Pointer tests press every visible cell at widths 5, 9 and 24 under
both policies and all four settings. Selection-only tests remove cached targets
before input; viewport tests remove the observed directory before scroll/resize.
The gutter test confirms the actual emitted bookmark and tree symbols from one
frozen preference after editing its source configuration on disk.

The first compile identified two root-replacement constructor calls requiring the
font argument. An initial geometry fixture also expected a composition for an
unusable zero-sized frame; it now asserts the actual absent-composition contract.
The focused `geometry::clipped_names` check then passed (one test, 3.34s), before
the complete five-command run passed. No test was disabled or weakened.

Observed successful command times were 2.72s formatting, 63.71s all-targets Clippy,
69.08s tests, 1.08s documentation and 0.01s whitespace. Clippy built fresh check
artifacts; the selected suites reused this worktree's test artifacts. The final
PR-reference HEAD receives separate ordered clean-HEAD evidence through the
controller's validate command. The receipt owns that candidate and timing.
Documentation validation checked 85 reachable pages, 571 local links/anchors,
596 commands and 289 configuration entries against the live schema and sample.
These slice checks do not replace the controller-owned final gate.

Fixtures use owned temporary paths and clean up after their checks. No server or
PTY journey was started for this slice.

## Risks and Follow-ups

Native checks prove selected/emitted code points and terminal-cell geometry,
not physical font drawing. Slice 3 owns combined native/PTY acceptance, complete
User Guide/Spore reconciliation and the integrated controller-gate handoff.
The selection/active-buffer synchronization ticket retains its separate scope.
