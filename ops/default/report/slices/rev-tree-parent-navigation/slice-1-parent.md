# Slice 1: Parent navigation

Status: reviewing
Delivery set: rev-tree-parent-navigation
Plan: [Parent Navigation Delivery](../../../plan/rev-tree-parent-navigation.md)
Tracker: [Parent Navigation Tracker](../../../tracker/rev-tree-parent-navigation.md)
Branch: `feat/rev-tree-parent-navigation-01-parent`
PR: `#312`
Integrated: pending

## Purpose

Deliver TUP01-TUP04: browse automatic file-tree ancestors while retaining the
directory just left as the selected, visible entry and preserving editing state.

## Delivered Behavior

- Tree-local `^` and `tree-up-directory` show an unanchored automatic root's
  parent in both profiles. Repeated navigation selects each directory just left;
  Enter expands the selected directory through the existing tree action.
- Explicit directory, Project, Workspace and configured cwd roots refuse before
  observing outside their boundary. Filesystem root reports that no parent exists.
- Parent listing admission uses existing filtering, ordering, symlink rules,
  4096-entry and 512 KiB name limits. Root, listing and selection publish only
  when the previous directory is represented. Incomplete admitted observations
  retain the existing diagnostic; failures preserve loaded state and focus.
- Editing context, process cwd, live buffers, dirty text, points, marks, split
  views and history remain intact. Automatic file-parent following remains
  active for later visits. No new configuration, persistence or dependency is added.
- Minimal/decorated trees retain ordinary and Nerd Fonts symbols, semantic
  colors, selected-row feedback, separators and bounded clipping. References,
  command inventory, Markdown guide and Unreleased notes describe the action.

## Implementation Notes

T1.1: `d5e64b2fc37f98b51f887dbef87030cd665f24f4` adds transactional native
root eligibility and parent observation in the existing Tree owner.

T1.2: `341e6755d878f749d92aed316381fff376d2f076` registers the shared command,
description, binding and acceptance. It aligns the compiled registry, command
and binding references, packaged configuration sample and live description
inventory at 624 names. Historical description delivery evidence remains unchanged.

T1.3: `22c7a47ad13f8623058ce3c78934aa1e3a157046` introduces this report and
updates tree/reference guidance, the tree map's
24-entry count, Markdown User Guide and reader-facing Unreleased behavior.
The Spore build projects the Markdown sources to ignored local output; it does
not maintain a second versioned guide or publish a site.

Canonical [PR #312](https://forge.adrianpk.com/adrianpk/rev/pulls/312) targets
`dev` with the approved title. Its report link uses the introduction commit
above. Integration remains pending with the controller.

The Tree owner prepares the complete observation before any live assignment.
Session routing applies the existing tree-focus admission and repeat suppression;
the normal frame owner reveals selection. Named commands, configured remapping
and effective hints share the same canonical action. Source caret behavior stays
owned by the selected editing profile.

## Contracts Added or Changed

- `Tree::up_directory()` admits only automatic unanchored roots and requires a
  real directory return entry. Missing, hidden, replaced and unrepresentable
  return paths fail without a fallback selection.
- `TreeCommand::UpDirectory` exposes the exact approved description:
  `Show the parent directory of an unanchored file tree.`
- Both profiles inherit `^` in the tree map. The editor/modal maps retain their
  source behavior; a source-focused named invocation refuses without clearing
  its active region.
- Native fixtures cover repeated upward movement, later automatic visits,
  offscreen return selection, all fixed-root kinds, filesystem root, observation
  failures and both outcomes at the existing listing bound. Dirty split views
  and source history are checked on successful and failed transitions.
- Terminal-cell acceptance covers minimal/decorated presentation with both font
  flags at widths 1, 6, 24 and 40, existing semantic palettes, full-row selection
  and split-pane geometry. Existing monochrome and pointer checks remain active.

## Files of Interest

- `src/tree/root.rs`: root policy and checked parent publication.
- `src/command/tree.rs`, `src/command.rs`, `src/input/keymap/defaults.rs`,
  `src/session/tree.rs`: canonical action, description, input and focus routing.
- `tests/tree/parent.rs`, `tests/commands/context_keymaps.rs`,
  `tests/display/tree.rs`: native root, command, source and presentation acceptance.
- `docs/reference/file-tree.md`, `docs/reference/bindings.md`,
  `docs/reference/commands.md`, `docs/reference/defaults.md`,
  `docs/reference/config.toml`, `docs/tutorials/user-guide/05-file-tree.md`:
  behavior, defaults, remapping and Markdown source for the Spore guide.

## Validation

All nine commands below passed during authoring. The final clean PR-reference
HEAD is validated through `looper validate` with the same unchanged commands
in canonical order. Its saved receipt is the
authoritative candidate evidence; integration and the delivery-set gate remain
controller-owned.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test tree -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test files directories -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- tree_up_directory catalog_reference description tree_actions tree_transitions tree_open_sequence --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pointer tree -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `git diff --check`

Authoring checks passed formatting, lib/bin Clippy, 82 tree tests including five
new parent cases, three directory tests, six command tests and 16 pointer tests.
Documentation checked 624 command names, 291 configuration entries, 91 reachable
pages and 628 local links/anchors. The local Spore build projected all 91 pages.

Additional focused presentation validation passed four tree tests, including the
new post-navigation terminal-cell case:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test display tree -- --test-threads=1`

Authoring corrected the compiled registry size, test API/diagnostic expectations
and documented tree-map count. No acceptance assertion, traversal limit or
required command was removed or weakened. Fixtures use the existing native
owners and remove their own temporary directories; no external service is needed.

## Risks and Follow-ups

None recorded. Canonical integration, the unchanged integrated gate and
documentation-only closure remain with the controller.
