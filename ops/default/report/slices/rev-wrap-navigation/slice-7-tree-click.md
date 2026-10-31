# Slice 7: Clicked tree entry and activation

Status: drafting
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-07-tree-click`
PR: pending

## Purpose

Deliver NAV05 and NAV06 through checked native tree row selection and activation.

## Delivered Behavior

A primary press selects its rendered tree entry and uses the existing native
open/toggle action. Files open in the remembered editing pane; directories and
the root expand/collapse without replacing the chosen root. Blank, status and
separator cells focus only. Tree presses do not create a document drag capture.

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
reuse remain authoritative. The configured selection-only preference and full
reader/reference reconciliation belong to T7.2.

## Files of Interest

- `src/tree/hit.rs`: opaque row identity and checked selection.
- `src/pointer/hit.rs`: composed content bounds and row identity.
- `src/session/pointer.rs`: native focus/select/activate routing.
- `src/tree/entry.rs`: current observed-kind check before activation.
- `tests/pointer/tree_click.rs`: native file, directory, root, clipping,
  viewport/refresh, blank-cell and unsuccessful activation cases.

## Validation

The native clicked-file regression failed against the prior focus-only route.
After implementation, five focused click tests passed through real crossterm
presses and composed frames. They cover remembered split ownership, dirty
source/history preservation, repeated directory toggles, blank/status/divider
cells, drag/release isolation, clipped Unicode names, scroll/resize/refresh and
missing/changed/invalid file failures.

The focused retained-hit test also passed across sorting shifts, changed kinds,
collapse and root replacement; rejected hits preserve the current selection.
Clean-head controller checks and T7.2 configuration coverage are pending.

## Risks and Follow-ups

Activation uses existing filesystem admission and cannot eliminate external
changes between observations. No background scanner, double-click timer or
new pointer input mechanism is added. Complete the startup preference,
configuration samples/references and ticket transition before handoff.
