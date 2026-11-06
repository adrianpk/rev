# Slice 1: Decorated presentation

Status: reviewing
Delivery set: rev-better-file-tree
Plan: [Better File Tree Delivery](../../../plan/rev-better-file-tree.md)
Tracker: [Better File Tree Tracker](../../../tracker/rev-better-file-tree.md)
Branch: `feat/rev-better-file-tree-01-decoration`
PR: pending

## Purpose

Deliver BFT01, BFT03 and the ordinary-symbol portion of BFT02/BFT04 through
startup configuration and the existing cached tree renderer.

## Delivered Behavior

- Trees use ordinary kind symbols, separate directory disclosure and semantic
  foregrounds by default. The root, directories, source files, documents,
  configuration files, other files and symlinks have distinct presentation.
- `tree.decorated = false` retains the previous minimal labels under either
  gutter font preference. Omission selects decoration without showing the tree.
- Every new leaf uses existing include precedence, source origins and whole
  candidate admission. Existing background and focused selection colors retain
  their roles; minimal labels ignore entry foregrounds.
- Classification consumes observed type and basename only. Root, symlink and
  directory identity win over the compiled ASCII-insensitive filename catalog.
  Unknown files remain actionable; names, sorting and canonical paths are intact.
- Directory slashes, incomplete diagnostics, lazy observation, refresh, focus,
  navigation and full-row pointer policy retain their existing owners.

## Implementation Notes

T1.1 is implemented in `0adbdac90ae076d3fb0d3ad3dff6b43c1fff2270`. T1.2 and its renderer/classification regressions
are implemented in `a95c18e140eb6b0d78ac2e269af1c39accd3c28a`. T1.3 completes the reference, defaults catalog,
Unreleased outcome and this report in its introduction commit.

`Entry::kind` derives one fixed presentation kind from cached observation data.
`tree::Kind` owns the compiled basename/suffix catalog, ordinary symbol and
palette-role mapping. No content read, language analysis, Git process, extra
entry state or dependency enters classification or painting.

The label builder retains the old minimal branch and adds an ordinary four-cell
disclosure/icon prefix with the existing two-cell nesting step. The renderer
supplies an optional kind through its tree tone; existing selection precedence,
clipping and safe filename drawing remain in the display layer.

## Contracts Added or Changed

- `tree.decorated`: startup boolean, compiled default true.
- `palette.tree_entries`: seven RGB foreground leaves with the approved defaults.
  `palette.tree` stays scalar; selection keys keep their meaning.
- `Entry::kind` and `tree::Kind`: derived visual identity, independent of parser
  support, filesystem access and entry activation.
- The complete sample and defaults/reference document the delivered ordinary
  presentation. One reader-facing Unreleased entry describes the usable change.

## Files of Interest

- `src/config.rs`, `src/config/palette.rs`, `src/config/merge.rs`,
  `src/config/project.rs`, `src/config/validate.rs`: admitted startup leaves.
- `src/tree/entry.rs`, `src/tree/presentation.rs`, `src/tree.rs`: derived kinds
  and minimal/decorated labels over the retained viewport.
- `src/display/tree.rs`, `src/display/style.rs`: foregrounds and selection.
- `tests/configuration/tree.rs`, `tests/tree/decoration.rs`,
  `tests/display/tree.rs`: composition, catalog, labels, cells and NO_COLOR.
- `docs/reference/config.toml`, `docs/reference/defaults.md`,
  `docs/reference/file-tree.md`, `CHANGELOG.md`: sample and user behavior.

## Validation

The five required commands passed locally in order:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test tree --test configuration --test display -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The suites passed 271 tests: configuration 160, display 41 and tree 70. The two
isolated NO_COLOR child checks also passed. The eight new tests cover omitted,
false/true and composed preferences, every palette role and invalid leaf,
all approved filename categories, directory/symlink precedence, exact minimal
labels, directory states, emitted cell colors and selection. Removing an already
observed fixture before movement/painting verifies cached behavior.

An initial new rejection fixture incorrectly repeated an unknown palette key in
the root and include, so the loader correctly diagnosed the root first. Making
the root valid isolated the intended include error; all four focused tree
configuration tests then passed. Both classification/label tests and both tree
renderer tests also passed in focused runs. No assertion was disabled or weakened.

Observed successful command times were 2.77s formatting, 31.50s Clippy, 112.00s
tests, 1.09s documentation and 0.01s whitespace, with build reuse. The initial
worktree compile took 93s before the first test attempt. Documentation validation
covered 85 reachable pages, 570 local links/anchors, 596 commands and 289
configuration entries against the live schema and sample.

The final PR-reference HEAD receives its own ordered clean-HEAD evidence through
the controller's validate command; the receipt owns that candidate and timing.
These slice checks do not replace the controller-owned final gate. Test fixtures
are owned and cleaned by their suites; no persistent service was started.

## Risks and Follow-ups

Slice 2 owns the Nerd Fonts catalog and shared checked prefix geometry. Slice 3
owns combined acceptance and complete User Guide/Spore reconciliation. This
slice makes no physical-font, full-gate, release or deployment claim.
