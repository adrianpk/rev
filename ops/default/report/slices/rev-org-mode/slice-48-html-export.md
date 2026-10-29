# Slice 48: export native Org documents to HTML

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-48-html-export`
PR: `#149`

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

T48.3 independently parses a representative complete document with the standard
HTML parser, checking element balance, local targets, duplicate identifiers,
metadata, exact table cells, image attributes, notes and inert code. The fixture
includes a note in a heading, table headers and adjacent mixed lists starting at
three. Native tests retain read-only and unsaved source, selection and undo state;
failed declarations, missing targets, cancellation and small output budgets
preserve existing destinations. RGB and NO_COLOR terminal sessions parse actual
HTML files and retain source BOM/CRLF, ordinary undo and owned process cleanup.

## Implementation Notes

HTML traverses existing native structure and inline observations. Rendering has
no file, process or network authority. It shares the checked snapshot policy,
output limit, cancellation and operation deadline. Recorded report bodies reuse
that evidence and retain their existing data without recalculation.
The existing structural list owner now separates ordered and unordered groups;
the independent document fixture exposed their previous grouping. HTML emits LF
newlines while source bytes remain unchanged. The verifier tracks actual open
elements and retains descendant text for semantic checks.

Task commits: T48.1 `7c588728d1e8e63777d53f2d66ecbf9de778d645`, T48.2 `04ea12326ef3c09d00a526d1c1e992db4f43b049`, T48.3 `e79c5728052617240f0071397eed8539db4072a4`.

Canonical PR: [#149](https://forge.adrianpk.com/adrianpk/rev/pulls/149).

## Contracts Added or Changed

- ORG25: native HTML documents from checked current source.
- ORG30: inert local image references retain readable source and ownership.

## Files of Interest

- `src/org/export/html.rs`: bounded native HTML traversal and document envelope.
- `src/org/export/html/notes.rs`: checked note definitions and backlinks.
- `tests/org_io/html_exports.rs`: independent document expectations and publication failures.
- `scripts/configuration_ui/html_document.py`: independent generated-markup parser.
- `tests/fixtures/org-html/document.org`: representative document input.

## Validation

- `cargo test --test org_io html_export`: three workflows passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_io -- -D warnings`: passed.
- Changed-source quality: passed with unchanged native limits.
- `cargo test --test org_io html_export`: five HTML renderer and publication workflows passed after command/attribute integration, zero failures or ignored tests.
- `cargo clippy --lib --test org_io --test commands -- -D warnings`: passed.

- `cargo test --test org_io html_export`: eight HTML workflows passed, zero failures or ignored tests.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-exports`: two owned RGB/NO_COLOR sessions passed, zero skipped.
- The independent mixed-list and terminal checks initially exposed lost list starts and CRLF output differences. Native list boundaries and HTML newline output were corrected; the same assertions now pass.

Canonical PR #149 merged into dev at
`b874708ae51ca19b0763fbe40d643f573a6cbe12`, verified against Forgejo
and origin/dev. Required receipt `IAS6YQZAUP54CRXIR7LEVJQJY2` passed five
ordered checks and 164 tests. Focused receipt `DUICAZUTDZVZ4M7AKPJPRKXDIN`
passed 378 command/editing tests, the configuration reference and two owned
RGB/NO_COLOR terminal sessions. Zero failures, ignored tests or skipped sessions.

## Risks and Follow-ups

None recorded.
