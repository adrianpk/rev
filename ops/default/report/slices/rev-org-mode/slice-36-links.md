# Slice 36: store and follow checked Org links

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-36-links`
PR: [#137](https://forge.adrianpk.com/adrianpk/rev/pulls/137)

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

T36.2 adds an explicit yes/no confirmation for configured external viewer argv.
The source point/revision and actual local resource are checked again before
launch; no recognition or startup action executes a viewer. Missing tools,
nonzero status, output overflow, timeout and cancellation remain observable.

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

The shared Unix child-lifetime primitive creates one owned process group, uses
nonblocking stdout/stderr with a combined byte cap, and kills the group/reaps the
direct child before releasing its worker. A completed leader cannot leave owned
descendants retaining output streams. This is lifetime ownership, not a sandbox
or a hard deadline for blocked kernel IO. Viewer data is literal argv with one
resource token; stdin is closed and the source directory is the working directory.

## Task Commits

- T36.1/report introduction: `227706a4d09c195344cf0386a92927f2669f84ad`.
- T36.2: `79ae6f95b1cb51644b9eeb53608d1029b1433151`.
- T36.3: `0538842cc831cb1d8a46b06aceff41e48168667b`.

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
- `python3 scripts/check-config-reference.py`: 155 schema/catalog leaves,
  54 RGB roles, 240 bindings and 19152 exact packaged bytes agree.
- `cargo test --lib process_ -- --nocapture`: three real-program tests pass for
  exact/overflow output, missing programs, status, timeout/cancel, direct-child
  reaping and descendants retaining output streams.
- `cargo test --test org_io --test configuration links -- --nocapture`: seven
  tests pass for native visits/insertion, literal argv/working directory,
  confirmation/no/C-g, media/native overrides and typed viewer configuration.
- `cargo clippy --lib --test org_io --test configuration -- -D warnings`: passed.
- Changed-source quality passes at unchanged parameter, nesting and complexity limits.

T36.3 checks:

- `cargo test --lib --test org_io links -- --nocapture`: 16 tests pass for
  ambiguous headings/targets/CUSTOM_IDs/IDs, incomplete or unreadable selected
  sources, invalid encoded paths, rejected executable forms, literal blocks,
  canonical dirty/read-only buffers, attachment traversal/symlink escape,
  shared-view insertion/undo, stale prompt/return points, checked destination
  replacement and native record limits. Viewer C-g and source/view invalidation
  reap the actual child and release polling; changed local resource kinds reject.
- `cargo clippy --lib --test org_io -- -D warnings`: passed.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-links`:
  four owned RGB/NO_COLOR sessions pass insertion/undo, native following and
  return, read-only storage/insertion rejection, explicit viewer confirmation,
  cancellation and literal resource delivery. Exact source/BOM/CRLF bytes and the
  entire fixture file set are checked; terminal processes exit and are reaped.
- Changed-source quality passes at unchanged limits.

The terminal fixture uses the existing read-only host to set the actual canonical
buffer admission flag. Disk permissions remain separate from that flag. An
accepted unchanged link edit has its own undo entry, matching ordinary history.
Existing terminal cases, deadlines and byte bounds remain unchanged.

Canonical PR #137 targets dev. Final controller receipts bind the clean pushed
head after this PR-reference commit, with the five required commands in approved
order. Focused evidence also covers shared configuration, file, window/display
and process-lifetime contracts plus owned link and capture terminal journeys.

## Risks and Follow-ups

Native target resolution requires a complete bounded source observation. External
viewer cleanup owns its Unix process group and direct child; it does not sandbox
programs or guarantee interruption of blocked kernel IO. The full delivery-set
gate remains controller-owned.
