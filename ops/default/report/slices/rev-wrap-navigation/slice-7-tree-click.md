# Slice 7: Clicked tree entry and activation

Status: delivered
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-07-tree-click`
PR: [#180](https://forge.adrianpk.com/adrianpk/rev/pulls/180)

## Purpose

Deliver NAV05 and NAV06 through checked native tree row selection and activation.

## Delivered Behavior

A primary press selects its rendered tree entry and uses the existing native
open/toggle action. Files open in the remembered editing pane; directories and
the root expand/collapse without replacing the chosen root. Blank and status cells focus only; separators never activate entries. Tree presses do not create a document drag capture.

## Implementation Notes

Tree hit testing uses the same retained viewport row owner as drawing. The
opaque hit carries root, viewport origin, preorder slot, full path and observed
type; selecting it checks those together without IO. A stale slot cannot select
its replacement. Clipped labels and indentation never become resource identity.

Activation rechecks the observed entry before invoking the native tree-open
owner. A missing or changed-type file cannot become a new-file visit or a new
root; failed opening retains the clicked cached selection and dirty source.
Directory observation remains explicit, lazy and bounded by existing owners.
The native tree-open method is shared within the session, not duplicated.

## Contracts Added or Changed

Primary tree clicks select the checked row and activate it by default. Existing
text hits, wheel targeting, prompt isolation, source identity and canonical file
reuse remain authoritative. The startup `mouse.tree_activate_on_click` Boolean defaults true; false selects
and focuses without filesystem reads or activation. Its partial loader, ordered
include merge/origin tracking, strict type/scope validation, complete sample and
compiled default use the normal configuration owners. The startup snapshot stays
frozen; global mouse disablement wins. PTR02 and its consolidated baseline now
carry the approved NAV05 policy, preserving historical gate evidence.

## Files of Interest

- `src/tree/hit.rs`: opaque row identity and checked selection.
- `src/pointer/hit.rs`: composed content bounds and row identity.
- `src/session/pointer.rs`: native focus/select/activate routing.
- `src/tree/entry.rs`: current observed-kind check before activation.
- `tests/pointer/tree_click.rs`, `tree_click_policy.rs`: native file, directory,
  root, clipping, viewport/refresh, configuration, aliases and failed actions.
- `src/config.rs`, `config/merge.rs`, `config/project.rs`: typed startup leaf.
- `docs/reference/config.toml`: exhaustive reference and compiled sample asset.
- `scripts/check-directory-targets.py --tree-click`: focused existing-PTY mode.
- Pointer/reader/configuration references and consolidated PTR02/R28 baseline.

## Validation

The native clicked-file regression failed against the prior focus-only route.
After implementation, five focused click tests passed through real crossterm
presses and composed frames. They cover remembered split ownership, dirty
source/history preservation, repeated directory toggles, blank/status/divider
cells, drag/release isolation, clipped Unicode names, scroll/resize/refresh and
missing/changed/invalid file failures.

The focused retained-hit test also passed across sorting shifts, changed kinds,
collapse and root replacement; rejected hits preserve the current selection.
T7.2 development checks passed all 133 configuration cases and all 64 other
pointer cases; the wheel-focus fixture now uses a footer press instead of an
activating entry press, and its focused rerun passed. The original wheel/source
and focus assertions remain intact. New policy cases cover both Booleans under
global admission, frozen startup, selection-only without filesystem access,
canonical dirty aliases and denied-directory recovery.

The existing directory-target probe's focused SGR mode passed two sessions:

- `python3 scripts/check-directory-targets.py --tree-click`

The sessions exercise native directory/file presses under true and false,
keyboard activation after selection-only, exact fixture bytes and terminal
restoration. Its first selection-only wait expected the source pane's active
marker. The corrected assertion requires tree focus, the inactive source's exact
file/point status and absence of the active marker. The eight-second/1 MiB driver
limits and complete-frame checks remain unchanged. No new gate or runtime
dependency is introduced.

T7.1 commit `65023e2ee5bb5ea94ea2ac34d5c70adcf159ca41` introduces checked
row identity, native activation and this report. T7.2 commit
`a6a3c97795feb45b63fbde01374c316d9a4b2cb1` adds the startup preference,
configuration/reference coverage and focused SGR journeys.

Controller receipt `WJGF4Y5IGUFXSHIPQFHPWJC5TP` passed these exact ordered
commands at clean head `a6a3c97795feb45b63fbde01374c316d9a4b2cb1`:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test pointer --test tree --test panels --test configuration --test files`
3. `git diff --check`
4. `cargo clippy --locked --all-targets -- -D warnings`
5. `cargo run --locked --quiet --example documentation-check`
6. `python3 scripts/check-directory-targets.py --tree-click`

The native suites passed 359 tests: 133 configuration, 74 file, 19 panel,
65 pointer and 68 tree cases. Documentation passed 305 commands, 180 configuration
entries, 58 pages and 330 local links/anchors. Both SGR sessions passed, retaining
exact fixture bytes and restoring terminal attributes.

Canonical PR #180 merged into `dev` at
`754d7af1ed7915e4db1e508b7a3fbbbe73c62c0b`. Final configured receipt
`N7OR4FAOUUHBTWEANHW5AT3ALC` passed the required three commands and 359 tests
on that exact clean head. The controller recorded the merge in Acta as
`01M681T9WK1PTFDH9SY5G0SR8H`. Slice 7 is delivered.

## Delivery-set closure

The controller passed the pinned full gate on integrated candidate
`6b2afd4e07f269c979cd26faf60b6863314e69e9` in receipt
`F7OQU2CWIOPB3VW234VVECCQNG`. All eight slices and both authorized repairs
are integrated, and the delivery set is completed. This final result does not
relabel this report's earlier focused evidence. See the
[tracker](../../../tracker/rev-wrap-navigation.md#final-gate-and-closure) for the
exact command and aggregate results.

## Risks and Follow-ups

Activation uses existing filesystem admission and cannot eliminate external
changes between observations. No background scanner, double-click timer or
new pointer input mechanism is added. The pointer ticket is solved with canonical merge evidence.
