# Slice 47: export Org to text Markdown and Org

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-47-text-exports`
PR: pending

## Purpose

Deliver native text, Markdown and Org export under the
[approved plan](../../../plan/rev-org-mode.md#slice-47-export-org-to-text-markdown-and-org)
and [tracker](../../../tracker/rev-org-mode.md#slice-47-export-org-to-text-markdown-and-org).

## Delivered Behavior

T47.1 renders checked snapshots as ASCII, UTF-8 text, Markdown or Org. Native
headings, metadata/options, lists, tables, links, footnotes and literal blocks
retain their meaning. ASCII rejects unrepresentable Unicode before publication.
Markdown chooses safe code fences and escapes literal paragraph markup. Org
retains the prepared source syntax. No renderer edits or saves canonical source.

## Implementation Notes

Rendering uses the native structure and inline observations over immutable
projection text. Task keywords come from the existing native workflow owner.

## Contracts Added or Changed

- ORG25: native text, Markdown and Org rendering from checked snapshots.
- ORG27: table, link and footnote data retain explicit representation.

## Files of Interest

- `src/org/export/render.rs`: bounded native output traversal.
- `src/org/export/render/inline.rs`: markup, code, link and note conversion.
- `tests/org_io/text_exports.rs`: handwritten document expectations.

## Validation

- `cargo test --test org_io text_export`: four renderer workflows passed, zero failures or ignored tests.
- Changed-source quality: passed with unchanged native limits.

## Risks and Follow-ups

None recorded.
