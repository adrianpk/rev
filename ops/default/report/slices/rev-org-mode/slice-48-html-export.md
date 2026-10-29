# Slice 48: export native Org documents to HTML

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-48-html-export`
PR: pending

## Purpose

Deliver native HTML export under the
[approved plan](../../../plan/rev-org-mode.md#slice-48-export-native-org-documents-to-html)
and [tracker](../../../tracker/rev-org-mode.md#slice-48-export-native-org-documents-to-html).

## Delivered Behavior

T48.1 renders checked source snapshots as complete UTF-8 HTML documents with
escaped metadata, headings, lists, table cells, links, local image references,
footnotes with backlinks and literal source blocks. Raw export blocks remain
escaped data. Executable or unsupported links retain literal source with a
diagnostic. Missing or duplicate note definitions reject the entire output.

T48.2 adds `org-export-to-html` to the compiled command catalog and existing
scope/options/destination flow. Affiliated names, captions and admitted HTML
attributes retain typed bounds. Event/style attributes and invalid geometry
reject before publication. Local assets cannot traverse parent directories;
internal references must resolve within the selected output. Heading links and
contents use stable local identifiers; link labels cannot create nested anchors.

## Implementation Notes

HTML traverses existing native structure and inline observations. Rendering has
no file, process or network authority. It shares the checked snapshot policy,
output limit, cancellation and operation deadline. Recorded report bodies reuse
that evidence and retain their existing data without recalculation.

## Contracts Added or Changed

- ORG25: native HTML documents from checked current source.
- ORG30: inert local image references retain readable source and ownership.

## Files of Interest

- `src/org/export/html.rs`: bounded native HTML traversal and document envelope.
- `src/org/export/html/notes.rs`: checked note definitions and backlinks.
- `tests/org_io/html_exports.rs`: independent document expectations.

## Validation

- `cargo test --test org_io html_export`: three workflows passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_io -- -D warnings`: passed.
- Changed-source quality: passed with unchanged native limits.
- `cargo test --test org_io html_export`: five HTML renderer and publication workflows passed after command/attribute integration, zero failures or ignored tests.
- `cargo clippy --lib --test org_io --test commands -- -D warnings`: passed.

## Risks and Follow-ups

None recorded.
