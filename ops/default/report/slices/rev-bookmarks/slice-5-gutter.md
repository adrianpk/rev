# Slice 5: Gutter markers

Status: delivered
Delivery set: rev-bookmarks
Plan: [Native Bookmarks Delivery](../../../plan/rev-bookmarks.md)
Tracker: [Native Bookmarks Tracker](../../../tracker/rev-bookmarks.md)
Branch: `feat/rev-bookmarks-05-gutter`
PR: [#189](https://forge.adrianpk.com/adrianpk/rev/pulls/189)

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

`bookmarks.markers` defaults to true for each initial editing view;
`bookmarks.marker` defaults to `●` and admits exactly one printable non-control
grapheme of terminal width one. `palette.gutter.bookmark` defaults to `#e0af68`.
`M-x bookmark-markers-mode` toggles only the selected view, with no factory
shortcut. Splits inherit visibility; file rebinds and close/reopen preserve the
view policy and independent bookmark records.

## Implementation Notes

A buffer-owned derived index holds logical rows, keyed by canonical source
stamp, file identity and collection generation. It is built once when invalid
from revision-current resolved anchors using the rope's existing line index.
Panels query that set for visible source rows; there is no per-row collection
scan, second source copy or serialized marker index.

First enabled marker demand for a file-backed view loads storage through session
startup/input/file transitions. The frame owner does not discover storage or
resolve loaded records. Existing fixed-width number, wrap, pointer and overlay
fixtures explicitly disable markers; factory and combined marker fixtures cover
the changed default geometry without relaxing their source assertions.

## Contracts Added or Changed

The shared checked gutter partition includes an optional one-cell marker area.
Painting, wrapping, pointer hits, reveal and search use the resulting text area.
Marker projection performs no filesystem IO, relocation or persistence.

## Files of Interest

- `src/bookmark/markers.rs`: buffer/revision-qualified logical row index.
- `src/gutter/geometry.rs`: number-priority cell allocation and text protection.
- `src/display/gutter.rs`: first-fragment semantic marker projection.
- `tests/bookmarks/markers.rs`: native marker and geometry journeys.
- `src/session/bookmarks/markers.rs`: per-view toggle and lazy admission boundary.
- `src/config/bookmarks.rs`: admitted marker settings and defaults.

## Validation

Development check `cargo test --locked --test bookmarks markers::` passed five
native marker tests. Initial test setup was corrected to use the native newline
command and distinguish logical rows from the view's mapped scroll origin.

Subsequent `cargo test --locked --test bookmarks --test configuration marker`
passed eight marker journeys and the Unicode/provenance configuration matrix.
`cargo test --locked --lib bookmark_index` passed cache reuse and record-only
invalidation. Native fold setup waits for the existing session-owned work before
asserting visibility; no source assertion or resource allowance was weakened.
The documentation checker passed 319 commands, 189 configuration entries,
58 pages and 331 links/anchors. Additional origin checks passed 39 display,
96 editing and 41 window tests after their fixed geometry fixtures were updated;
161 command and 74 file tests also passed during development.

The initial controller focused batch passed owner/origin tests but clippy
rejected a test module placed before its implementation. The module was moved
to the end of its file. The next focused batch passed; required configuration
checks then identified the packaged palette count still expecting 54 roles.
The fixture now expects all 55 roles and explicitly checks the new bookmark
color. Fresh validation must cover the resulting candidate.

Final exact-HEAD validation follows all task and canonical PR-reference commits:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test bookmarks --test gutter --test wrapping --test pointer --test search --test panels --test configuration`
3. `git diff --check`

Separate focused validation checks `cargo test --locked --lib bookmark`,
`cargo test --locked --test display --test windows --test commands --test editing --test files`,
`cargo clippy --locked --all-targets -- -D warnings` and
`cargo run --locked --quiet --example documentation-check` on the same candidate
before the required batch.
Both batches passed before handoff. The completed controller-owned combined
gate is recorded below.

## Risks and Follow-ups

Unresolved, stale and unavailable records remain unpainted until a checked visit
establishes a current live anchor. Joint acceptance was completed in Slice 6.

Canonical PR #189 merged through Rebase + Fast-forward into `dev` at
`52b4b8ee54939b00ba9b305b50e45857740f37e6`. Required receipt
`UOUFIBZVYUTY4EQBPP3AWIMW2D` passed the exact three-command list and 369
integration tests. Focused receipt `SP6SDUHTN66V4MVJPA2CCVBFHT` passed 12
bookmark owner tests, 411 origin/command/editing/file tests, all-target clippy
and documentation on the same clean candidate. The controller recorded the
canonical merge in Acta. Slice 5 of 6 completed.

## Delivery-set closure

Completed: 2026-10-31
Validated candidate: `41ba5f89465b2be02adc613cd796271169c245e0`
Gate receipt: `7WOA4YIWYISUYIDTDT3ZC5SI5A`

The controller passed the unchanged Full gate recorded in the
[tracker](../../../tracker/rev-bookmarks.md#final-gate-and-closure) after all
six slice PRs and three authorized repairs integrated. The delivery set is
completed; the slice receipts above retain their original candidate identities.
