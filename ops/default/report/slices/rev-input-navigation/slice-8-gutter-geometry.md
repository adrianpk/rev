# Slice 8: Preserve gutter geometry across editor surfaces

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `fix/rev-input-08-gutter-geometry`
PR: pending

## Purpose

Verify the accepted gutter degradation and coordinate contract across native
editing surfaces, shared panes and actual terminal interaction.

## Delivered Behavior

- Actual number-field paint stays within each admitted editing lane when nested
  panes coexist with a tree, command completion and independent view policies.
- Complete number fields yield before the protected three text columns, including
  mixed-width sibling panes and the largest admitted digit/padding format.
- Search, syntax, linear selection, rectangle selection and accepted cursors
  share final text origins on either side, hidden fields and horizontal clipping.
- Rectangle capture retains document-relative columns and exact source rows.
- Zero/tiny suspension and shrink/grow cycles retain view identities, points,
  requested gutter policies, canonical source and completion drafts.

## Implementation Notes

The existing central `View::gutter_geometry` partition already supplies complete
field suppression and restoration. Native tests compare all decoded number-field
background cells with exactly the admitted lane union, rather than inferring
presentation from policy metadata. Four factory/maximal-format and command/plain
combinations cross twelve dimensions, including tree thresholds and exact field
admission widths. No layout fallback or acceptance condition is weakened.

The coordinate-consumer audit found no additional coordinate recipe to replace:
`Frame::panel` partitions once before document/syntax/region/cursor projection;
search obtains the same `View::gutter_geometry` text area. `View::reveal` uses
that area's width while tab stops remain buffer-owned. Rectangle capture maps
canonical points through `Coordinates`, never physical gutter cells. Status uses
the original pane's distinct reserved row. Tree/completion retain separate panels.
`Panel::position` admits only content-local cells; final text panels reject their
right/bottom edges. The checked partition supplies the later hit-mapping bounds.

## Contracts Added or Changed

GUT05 retains the final text coordinate boundary; GUT06 protects complete text
minima and requested policy through degradation. GUT13 keeps visible bounded
projection; GUT14 verifies native and terminal editor boundaries.

## Files of Interest

- `tests/gutter/workspace.rs`: native nested workspace and decoded field isolation.
- `tests/gutter/surfaces.rs`: explicit physical search/region/syntax/cursor bounds and rectangle capture.
- `src/gutter/geometry.rs`, `src/panel/composition.rs`: existing checked owners.

## Validation

- PASS: `cargo test --test gutter workspace::` covers 48 actual frame projections
  across factory/maximal formatting, tree/completion regions, narrow/zero sizes
  and restoration, without source edits or history entries.

- PASS: `cargo test --test gutter surfaces::` covers five fixed physical-origin
  cases: left, right, hidden and both narrow sides in a tree/nested workspace.
  Actual underlined cells, region/syntax spans, cursor positions, peer points,
  status/border exclusion and copied `fn` rectangle rows remain coherent.
- PASS: `python3 scripts/check-source.py` retains all unchanged source limits.

## Risks and Follow-ups

None recorded.
