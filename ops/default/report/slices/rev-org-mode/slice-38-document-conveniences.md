# Slice 38: edit native Org footnotes and inline content

Status: reviewing
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

T38.2: `508aee2c666ba74aae01bc8e39b5875dc61fa4aa`.

T38.3 verifies source format and owner boundaries, including inline-definition
navigation, literal emphasis removal, combining characters and extended
graphemes, insertion before a later section, direct affiliated-line editing,
read-only publication, shared point/mark mapping and stale/cancelled drafts.
Unchanged view publication deliberately retires stale fold revisions; fresh
independent visibility commands use current source observations.

- `cargo test --lib --test org_editing document -- --nocapture`: two native resource-bound tests and 16 editor workflows pass. Real snapshots reject cancellation, expired deadlines, excessive bytes and excessive footnote/radio records. Numeric-note syntax checks use sorted protected intervals and a set of recognized starts, avoiding repeated whole-observation searches.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-document`: two owned RGB/NO_COLOR sessions pass creation/following, Unicode definition entry, source undo/redo, region emphasis, entity/affiliated prompts, radio following/return, literal rejection and cancellation. Final source bytes retain BOM, CRLF and EOF state; the full owned fixture inventory agrees. Native children exit and are reaped; inactive work produces no polling output.
- `cargo clippy --lib --test org_editing --test commands -- -D warnings`: passed.
- Changed-source quality passes at unchanged limits.

The representative journey adds two short owned terminals to the existing driver.
Variant coverage stays in native tests; gate limits and assertions are unchanged.
The terminal fixture owns an empty valid configuration from startup so normal
first-run configuration creation cannot masquerade as an unexpected operation.
Canvas comparisons account for terminal trailing-space display; persisted bytes
remain independently exact.

Canonical final checks will bind the clean pushed head after PR references are
committed. Required commands retain their approved order; focused evidence covers
shared source/job, command, file, window and existing link behavior.

## Risks and Follow-ups

The full delivery-set gate remains controller-owned.
