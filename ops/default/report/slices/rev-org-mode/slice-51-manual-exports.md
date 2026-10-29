# Slice 51: export Org to Texinfo and man formats

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-51-manual-exports`
PR: `#152`

## Purpose

Deliver native manual formats under the
[approved plan](../../../plan/rev-org-mode.md#slice-51-export-org-to-texinfo-and-man-formats)
and [tracker](../../../tracker/rev-org-mode.md#slice-51-export-org-to-texinfo-and-man-formats).

## Delivered Behavior

T51.1 renders immutable snapshots as UTF-8 Texinfo and man documents. Native
hierarchy, list markers, emphasis and literal source/table data retain checked
structure. Texinfo node names are generated independently of title text. Man
request introducers, quotes and escapes in source data cannot become requests.

T51.2 adds `org-export-to-texinfo` and `org-export-to-man` through the existing
scope/options/destination workflow. Named targets and unique heading links resolve
within the selected snapshot; named and inline notes retain stable first-use
numbers. Missing, ambiguous, duplicate or cyclic relationships reject output.
Unknown resource kinds, rich forms, math and dynamic reports stay literal with
diagnostics. Source and export blocks never become formatter directives.

T51.3 fixes complete handwritten golden documents for both formats and checks
actual rendered bytes with an independent structural reader. Eight workflows
cover publication of dirty BOM/CRLF text, cancellation at every prompt, declined
replacement, changed destinations/includes/source, source hard-link aliases,
read-only subtree/region selection, unsafe scalars, exact final-byte limits and
ordinary undo. References must name a target actually emitted, including targets
inside notes. The owned RGB/NO_COLOR journey exercises both new commands.

## Implementation Notes

Rendering shares snapshot cancellation, deadline and output bounds. It has no
file publication, asset-reading, process or network authority. Manual output
never probes or invokes makeinfo, groff or a shell. Unsupported forms remain
escaped source with diagnostics; unsafe control scalars reject output.

Task commits: T51.1 `82ca74dfcd2676049306ecca49ce6d7fa6c23bc4`, T51.2 `531f206eb57aa704cb3dfcc69901bae86244d0a8`, T51.3 `a8db4be915afe1a851315640a115e9dcccbca44c`.

Canonical PR: [#152](https://forge.adrianpk.com/adrianpk/rev/pulls/152).

## Contracts Added or Changed

- ORG25: native Texinfo and man rendering from checked current source.
- ORG27: literal source remains data during document conversion.

## Files of Interest

- `src/org/export/manual.rs`: native format ownership and bounded traversal.
- `src/org/export/manual/headings.rs`: manual envelopes and relative hierarchy.
- `tests/org_io/manual_exports.rs`: public snapshot renderer expectations.

## Validation

- `cargo test --test org_io manual_`: one renderer workflow passed, zero failures or ignored tests.
- Changed-source quality passed with unchanged limits after backend dispatch was factored into a small helper.

- `cargo test --test org_io manual_`: three renderer/reference workflows passed, zero failures or ignored tests.
- `cargo test --test commands visibility_keys`: compiled catalog round-trip passed.
- `cargo clippy --lib --test org_io --test commands -- -D warnings`: passed.
- Changed-source quality passed after reference indexing and inline dispatch were split at the unchanged complexity limit. A colon-containing heading fixture exposed an overly broad resource-scheme guard; heading references now retain that punctuation.

- `cargo test --test org_io manual_`: eight renderer, selection and publication workflows passed, zero failures or ignored tests; the expanded complete golden was also rechecked with `cargo test --test org_io manual_golden`.
- `python3 scripts/check-manual-exports.py`: GNU makeinfo and groff accepted both exact golden documents; visible hierarchy, references, notes, list starts and inert directive text passed, zero skipped checks. This is an explicit validation adapter, never a runtime export action.
- Region selection exposed inherited title/date in the manual envelope; region output now excludes that metadata. Formatter validation exposed first-item paragraph separation and terminal SGR output: native list paragraphs now follow their markers, and the adapter strips font encodings while retaining same-line marker/body assertions.

## Risks and Follow-ups

None recorded.
