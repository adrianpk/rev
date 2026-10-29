# Slice 36: store and follow checked Org links

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-36-links`
PR: pending

## Purpose

Deliver native checked Org links under the
[approved plan](../../../plan/rev-org-mode.md#slice-36-store-and-follow-checked-org-links)
and [tracker](../../../tracker/rev-org-mode.md#slice-36-store-and-follow-checked-org-links).
The [Org link reference](https://orgmode.org/manual/Handling-Links.html) supplies
storage, insertion, native target and return-navigation semantics.

## Delivered Behavior

T36.1 adds native store, insert/edit, follow, next/previous and mark-ring controls.
Internal targets, named elements, exact headings, CUSTOM_IDs, IDs, existing local
files and attachment links resolve from bounded actual source observations.
Percent-decoded UTF-8 paths remain literal data. Insertion uses ordinary checked
history, shared-view mapping and a post-insertion cursor; storing/navigation do
not change source text. Read-only buffers remain navigable.

## Implementation Notes

The existing byte-level inline recognizer now supplies pure objects separately
from buffer-qualified observations. Disk observations never receive synthetic
buffer identities. Structural context excludes literal blocks. Destination visits
reuse existing immutable file/buffer observations and qualify the point only
against the actual admitted registry owner. Missing/stale/ambiguous targets do
not replace the editing state.

ID lookup uses configured/retained agenda sources and open Org buffers. Attachment
following uses selected DIR/ID data and the native data-directory default, with
canonical containment checks and no folder creation. The existing single Org
worker owns observation and resolution; bounded prompt state owns insertion.
The session retains 32 stored links and 64 revision-qualified return points.

## Contracts Added or Changed

- ORG15: native link controls and checked data-only target resolution.
- ORG02: source qualifications, exact UTF-8 offsets and ordinary history.

## Files of Interest

- `src/org/links/`: pure link targets, bounded observations and checked visits.
- `src/org/inline/`: shared native recognition without fabricated stamps.
- `src/session/links/`: prompts, retained links and independent navigation.
- `src/session/org/links.rs`: ownership of existing cancellable worker slot.
- `docs/reference/org-links.md`: admitted forms, controls, sources and limits.

## Validation

- `cargo test --test org_io links -- --nocapture`: three native workflows pass,
  including dedicated/named/heading/custom-ID targets, file/ID/attachment visits,
  encoded paths, checked return points and stored insertion with undo.
- `cargo clippy --lib --test org_io -- -D warnings`: passed.
- `python3 scripts/check-config-reference.py`: 153 schema/catalog leaves,
  54 RGB roles, 240 bindings and 18982 exact packaged bytes agree.
- Changed-source quality passes at unchanged parameter, nesting and complexity limits.

## Risks and Follow-ups

T36.2 supplies explicit external-resource dispatch and the shared owned-child
lifetime. T36.3 extends ambiguity, read-only, stale-source and failure coverage.
The full delivery-set gate remains controller-owned.
