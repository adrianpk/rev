# Slice 38: edit native Org footnotes and inline content

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-38-document-conveniences`
PR: pending

## Purpose

Deliver document conveniences under the
[approved plan](../../../plan/rev-org-mode.md#slice-38-edit-native-org-footnotes-and-inline-content)
and [tracker](../../../tracker/rev-org-mode.md#slice-38-edit-native-org-footnotes-and-inline-content).

## Delivered Behavior

T38.1 delivers footnote creation/following and numeric renumbering, literal named
entity insertion, region emphasis or paired markers, and selected affiliated
NAME/CAPTION/ATTR_BACKEND edits. Familiar footnote/emphasis bindings reach the
same native commands as M-x.

## Implementation Notes

The existing structural and inline scanners observe actual canonical rope
snapshots on the single cancellable Org worker. Publication rechecks the source,
view and prompt owner, then qualifies byte edits through the real buffer. Distant
footnote changes form one transaction and one undo group. Read-only publication
rejects through ordinary buffer authority. No file or process adapter is used.

## Contracts Added or Changed

- ORG27: native footnote, entity, markup and affiliated-data source actions.
- ORG02: source observations and checked edit/history publication.
- ORG30: canonical source and coordinates remain authoritative.

## Files of Interest

- `src/org/document/`: native source observations and edit plans.
- `src/session/document.rs`: typed prompts and checked source transactions.
- `src/session/org/document.rs`: bounded worker ownership.
- `docs/reference/org-document.md`: commands, formats and bounds.

## Validation

- `cargo test --test org_editing document:: -- --nocapture`: four native workflows pass for create/follow/undo, numeric and inline renumbering, marker pairs, all six Unicode region emphasis variants, entities and affiliated edits.
- `cargo clippy --lib --test org_editing --test commands -- -D warnings`: passed.
- `python3 scripts/check-config-reference.py`: 155 schema/catalog leaves, 54 RGB roles, 243 bindings and 19245 packaged bytes agree.
- Changed-source quality passes at unchanged limits.

T38.1/report introduction: `34374ac5f4b97bc735e4f554bc1d311a672e0035`.

T38.2 adds footnote/radio context dispatch, protected literal/unknown/macro
ranges and malformed/ambiguous note graph rejection. Existing clock-table
context actions keep their original owner. Sorted protected intervals bound
lookup work; cancellation and native record/depth/byte limits stay active.

- `cargo test --test org_editing document:: -- --nocapture`: seven native workflows pass, including literal/unknown/dynamic contexts, ordinary editing after rejection, contextual following, Unicode radio names, malformed references and protected lookalikes.
- `cargo clippy --lib --test org_editing --test commands -- -D warnings`: passed.
- Changed-source quality passes at unchanged limits.

## Risks and Follow-ups

Edge validation remains assigned to T38.3. Final ordered checks remain pending.
