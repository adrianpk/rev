# Slice 6: Color native Org source semantically

Status: drafting
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-06-semantic-colors`
PR: pending

## Purpose

Present recognized native Org source through the existing semantic display owner.

## Delivered Behavior

Native structural and inline observations produce revision-qualified byte
captures. Headings, metadata, drawers, properties, planning lines, blocks, code,
comments, lists, tables, formulas, emphasis, links, targets, footnotes, entities
and malformed delimiters retain distinct typed roles. Literal bodies exclude
surrounding Org and inline recognition. Source bytes and editing state survive.

Visible Org buffers use one temporary session-owned observation worker, shared
across views. Native input polls only while that worker is owned. Edits invalidate
old styles immediately; publication rejects foreign or obsolete revisions.
Cancellation, hidden buffers and mode/policy changes retire owned work. Plain
source remains readable while observation is pending or unavailable.

## Implementation Notes

COW rope snapshots leave analysis outside atomic editing and rendering. Retained
line indexes resume at the existing invalidated source scope. Structural and
inline ownership is re-observed from the snapshot to recover delimiters crossing
that scope. The exposed reindexed-byte count measures only index suffix work.
Finite byte/node/depth bounds and cancellation retain explicit coverage states.

The existing byte-to-grapheme/cell projection clips complete glyphs and keeps
selection/search precedence. Org source never enters the Rust grammar owner.

## Contracts Added or Changed

- ORG16: native typed source presentation and opaque literal-block context.
- ORG17: semantic roles enter the existing display/configuration owners.

## Files of Interest

- `src/org/color.rs`, `src/org/color/`: stamped captures and bounded observation.
- `src/buffer/org.rs`: exact-revision admission and cache ownership.
- `src/session/org.rs`: shared temporary worker and native event deadlines.
- `src/display/syntax.rs`: existing source/cell projection extended for Org.
- `tests/org_source/color.rs`: ownership, stale results, suffix recovery and limits.

## Validation

Implementation checks passed:

- `cargo check --lib`
- `cargo test --test org_source color::`: four passed.
- `cargo test --test org_editing`: eight passed.

Required ordered exact-head controller validation will be recorded after the
palette and combined session tasks and PR-reference follow-up are committed.

## Risks and Follow-ups

Palette configuration and combined rendering/session verification remain in the
recorded T6.2 and T6.3 tasks. Task-state and timestamp-specific roles activate
with their later semantic owners; planning lines currently use one role.
