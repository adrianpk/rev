# Slice 5: Gutter markers

Status: drafting
Delivery set: rev-bookmarks
Plan: [Native Bookmarks Delivery](../../../plan/rev-bookmarks.md)
Tracker: [Native Bookmarks Tracker](../../../tracker/rev-bookmarks.md)
Branch: `feat/rev-bookmarks-05-gutter`
PR: pending

## Purpose

Deliver BMK03 and the marker configuration parts of BMK07 through the native
gutter, view policy, source coordinates and frame projection.

## Delivered Behavior

An enabled editing view reserves one bookmark cell independently of number
visibility. Both gutter sides retain their existing number format and separator.
Numbers receive space first; the marker is omitted when it would reduce text
below three columns, and returns when space permits.

Only the first visible fragment of a resolved logical source line receives a
marker. Continuations, filler and folded bodies remain blank. Multiple names on
one line share the glyph while retaining their independent record identities.
Source edits, undo/redo and collection-only mutations invalidate the projection.

## Implementation Notes

A buffer-owned derived index holds logical rows, keyed by canonical source
stamp, file identity and collection generation. It is built once when invalid
from revision-current resolved anchors using the rope's existing line index.
Panels query that set for visible source rows; there is no per-row collection
scan, second source copy or serialized marker index.

## Contracts Added or Changed

The shared checked gutter partition includes an optional one-cell marker area.
Painting, wrapping, pointer hits, reveal and search use the resulting text area.
Marker projection performs no filesystem IO, relocation or persistence.

## Files of Interest

- `src/bookmark/markers.rs`: buffer/revision-qualified logical row index.
- `src/gutter/geometry.rs`: number-priority cell allocation and text protection.
- `src/display/gutter.rs`: first-fragment semantic marker projection.
- `tests/bookmarks/markers.rs`: native marker and geometry journeys.

## Validation

Development check `cargo test --locked --test bookmarks markers::` passed five
native marker tests. Initial test setup was corrected to use the native newline
command and distinguish logical rows from the view's mapped scroll origin.

Final exact-HEAD validation follows all task and canonical PR-reference commits:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test bookmarks --test gutter --test wrapping --test pointer --test search --test panels --test configuration`
3. `git diff --check`

Separate focused validation checks bookmark owner tests, all-target clippy and
the documentation catalog on the same candidate before the required batch.
Both batches must pass before handoff. The controller owns integration and the
combined gate after all recorded slices.

## Risks and Follow-ups

Unresolved, stale and unavailable records remain unpainted until a checked visit
establishes a current live anchor. Joint acceptance remains assigned to Slice 6.
