# Slice 1: Selection and following

Status: reviewing
Delivery set: rev-tree-selection
Plan: [File Tree Selection Delivery](../../../plan/rev-tree-selection.md)
Tracker: [File Tree Selection Tracker](../../../tracker/rev-tree-selection.md)
Branch: `feat/rev-tree-selection-01-follow`
PR: pending

## Purpose

Deliver TSV01-TSV04: retain visible tree selection after activation and follow
eligible active editing files by default without changing browsing intent,
source state or root ownership.

## Delivered Behavior

- Startup configuration admits `tree.follow_active_buffer`, defaulting to true
  without rewriting existing files. False keeps the manual selection visible
  while editing buffers change. Includes, overrides and provenance use the
  existing configuration owners.
- Tree browsing and active-file indication remain independent. Tree and prompt
  origins choose one highlight; editing follows the active pane's loaded file
  route. Unnamed, outside, filtered, collapsed and unavailable targets have no
  fabricated active-file mark.
- Real file, focus and listing transitions can reveal an offscreen loaded row.
  Repainting and same-file point movement preserve manual scrolling. Explicit
  roots, folded directories, hidden trees and Workspace membership retain their
  existing owners.
- Minimal/decorated trees and both symbol catalogs use the existing selection
  palette or NO_COLOR reverse video. A source region on the same screen row
  keeps its separate selection. The cursor and focused footer remain owned by
  actual tree focus.

## Implementation Notes

T1.1: `0b5e987a20a02bbef1d7c53a1ae91749ab42b813` adds the startup policy,
retained matching state, event boundaries and shared presentation.
T1.2: `88087201423c6ea4e331d0328fd295ae94942607` adds native acceptance and
extends the existing pointer, display and Workspace route guards.
T1.3: this report introduction updates the tree reference, Markdown User Guide
and Unreleased notes. Canonical commit and PR references follow at handoff.

Tree state retains the editing view/path and one optional loaded match. Listing
publication invalidates that match; input/context transitions recompute it only
when its identity or eligibility changes. Paint consumes retained state without
path matching or filesystem I/O. Successful ingress can establish an alias
association; refresh retires that evidence until another visit. Matching prefers
the retained browsing route, then declared Workspace root order.

A one-shot reveal request belongs to actual follow transitions; explicit wheel
input supersedes an unpainted reveal. Browsing keeps
its own cursor, including when its row lies before a viewport revealed for the
active file. Prompt projection uses checked optional cursor coordinates.
No command, binding, palette field, dependency, background worker or persisted
selection state is added.

## Contracts Added or Changed

- Native startup boolean across defaults, partial admission, merging, source
  provenance, complete sample and direct configuration consumers.
- Loaded-entry matching and reveal independent of manual selection and source
  point/mark/history, with one deterministic overlapping Workspace route.
- Native acceptance covers both input profiles, default/manual policies,
  prompts, buffer/pane changes, missing targets, collapsed/filtered entries,
  admitted aliases, Save As/rename, scrolling and resize.
- Existing pointer and terminal-cell tests cover repeated activation,
  selection-only clicks, changed projections before release, full-row palette
  colors, monochrome feedback and simultaneous source selection.

## Files of Interest

- `src/tree.rs`, `src/tree/follow.rs`, `src/tree/entry.rs`: retained source,
  browsing and loaded-route ownership.
- `src/session/tree.rs`, native input, buffer and file boundaries: event-owned
  following and admitted alias evidence.
- `src/display/tree.rs`, `src/display/style.rs`, `src/display/selection.rs`:
  persistent full-row feedback with independent source highlighting.
- `tests/tree/follow.rs`, existing pointer/display tree tests and
  `tests/contexts/workspaces.rs`: native acceptance and route regressions.
- `docs/reference/config.toml`, `docs/reference/defaults.md`,
  `docs/reference/file-tree.md`, `docs/tutorials/user-guide/05-file-tree.md`,
  `CHANGELOG.md`: setting, user behavior and Markdown source for Spore.

## Validation

The ten approved commands passed locally during implementation:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test tree -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pointer tree -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test display tree -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts peer_tree_routes -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands catalog_reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `git diff --check`

These are focused slice checks: 77 tree tests, 16 pointer tests, three display
checks (including NO_COLOR and simultaneous source selection), the overlapping
Workspace route case and the 618-command catalog guard. Documentation passed
290 configuration entries, 91 reachable pages and 623 local links/anchors.
The guide build projected and rendered all 91 Markdown pages to ignored output.

Additional configuration admission and provenance coverage passed four tests:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration tree -- --test-threads=1`

Authoring checks exposed an offscreen browsing-cursor subtraction and an
unpainted follow reveal overriding a wheel event. Checked optional cursor
coordinates and explicit scroll intent correct those cases. Existing wheel
assertions now account for the default-follow viewport while retaining their
exact displacement and source-state checks.

Final clean PR-reference HEAD evidence belongs to the controller validate
receipt, recorded after the reference commit. The integrated delivery-set gate
and later closure remain controller-owned. Fixtures use existing native owners;
no preview server or external publication is part of this validation.

## Risks and Follow-ups

The optional legacy `python3 scripts/check-config-reference.py` checker fails
its fixed 54-role palette assertion. The same expression returns 57 on the
unchanged planning baseline and this candidate; neither palettes nor that
checker change in this slice. Native configuration and documentation checks
remain the selected setting/catalog evidence.
