# Slice 8: Preserve gutter geometry across editor surfaces

Status: delivered
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `fix/rev-input-08-gutter-geometry`
PR: [#88](https://forge.adrianpk.com/adrianpk/rev/pulls/88)
Integrated: `36f51884dcb612d158f30e983e4bfc744a6b1425`

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
- File-completion candidates remain unnumbered and preserve every editing view
  through shrink/zero/grow and cancellation.
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

Bounded-work inspection confirms that `Projection`, `Policy` and `Geometry`
retain only fixed-size metadata; number rows are produced only inside the visible
height loop. Hidden/suppressed lanes return before row materialization. Full line
counts use existing rope metadata and point lookup uses the existing line index;
there is no gutter timer, scanner or background owner. The native idle deadline
returns `Wait`. Two owned PTYs separately observe no output or process CPU ticks
during a 120 ms idle sample. These are bounded functional/idle observations,
not the later combined performance workload gate.

The gutter ticket is solved after verified integration of its final primary
Slice 8, completing its assigned primary-slice closure condition.

## Contracts Added or Changed

GUT05 retains the final text coordinate boundary; GUT06 protects complete text
minima and requested policy through degradation. GUT13 keeps visible bounded
projection; GUT14 verifies native and terminal editor boundaries.

## Files of Interest

- `tests/gutter/workspace.rs`: native nested workspace and decoded field isolation.
- `tests/gutter/surfaces.rs`: explicit physical search/region/syntax/cursor bounds and rectangle capture.
- `src/gutter/geometry.rs`, `src/panel/composition.rs`: existing checked owners.

- `tests/gutter/completion.rs`: real path candidates, complete view retention and idle admission.
- `scripts/configuration_ui/gutter_geometry.py`: owned nested terminal/history/idle journeys.

## Validation

- PASS: `cargo test --test gutter workspace::` covers 48 actual frame projections
  across factory/maximal formatting, tree/completion regions, narrow/zero sizes
  and restoration, without source edits or history entries.

- PASS: `cargo test --test gutter surfaces::` covers five fixed physical-origin
  cases: left, right, hidden and both narrow sides in a tree/nested workspace.
  Actual underlined cells, region/syntax spans, cursor positions, peer points,
  status/border exclusion and copied `fn` rectangle rows remain coherent.
- PASS: `python3 scripts/check-source.py` retains all unchanged source limits.

- PASS: `cargo test --test gutter completion::` verifies path candidates and query
  ownership across zero/tiny and restored geometry, complete view retention,
  canonical source/history and a native idle deadline.
- PASS: `python3 scripts/check-pty.py --suite gutter-geometry` (two owned sessions,
  zero skipped). RGB and monochrome journeys verify nested independent relative
  origins and sides, tree/path coexistence, zero resize and restored points,
  edit/save/undo/redo with exact BOM/Unicode/CRLF bytes, view close and idle output/
  CPU-tick samples. Every owned child and terminal descriptor is restored/reaped;
  eight-second and one-MiB limits remain unchanged.

PASS at the integrated candidate: canonical evidence `4A5ONRVDX3SWXO2BURK7XEQHND` records all five checks and 190 native tests:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test gutter --test display --test panels --test windows --test search --test path_completion`
- `python3 scripts/check-source.py`
- `git diff --check`

PASS on the same candidate: focused evidence `ZT6RD6WPWNZZWLVLV7CTY4VZ3G` records 70 native tests, reference checks and two owned PTYs:

- `cargo test --test rectangles --test syntax`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-pty.py --suite gutter-geometry`
- `git diff --check`

## Risks and Follow-ups

None recorded.
