# Slice 51: export Org to Texinfo and man formats

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-51-manual-exports`
PR: pending

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

## Implementation Notes

Rendering shares snapshot cancellation, deadline and output bounds. It has no
file publication, asset-reading, process or network authority. Manual output
never probes or invokes makeinfo, groff or a shell. Unsupported forms remain
escaped source with diagnostics; unsafe control scalars reject output.

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

## Risks and Follow-ups

None recorded.
