# Slice 49: export Org to LaTeX and Beamer

Status: implementing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-49-tex-exports`
PR: pending

## Purpose

Deliver native LaTeX and Beamer exports under the
[approved plan](../../../plan/rev-org-mode.md#slice-49-export-org-to-latex-and-beamer)
and [tracker](../../../tracker/rev-org-mode.md#slice-49-export-org-to-latex-and-beamer).

## Delivered Behavior

T49.1 renders checked source snapshots as complete article or Beamer documents.
Native headings, lists with numeric starts, tables, named and inline footnotes,
links and declared math fragments retain bounded structure. `H:1` through `H:5`
select the Beamer frame level; nested headings become blocks. Source and raw
export blocks remain escaped literal data. Unsupported math controls remain
escaped with a diagnostic; missing, duplicate or cyclic note definitions reject
output.

## Implementation Notes

The renderer shares immutable source evidence, cancellation, operation deadline
and output limits. It has no process, network or publication authority. Math
recognition precedes entity conversion and protects native literal objects.
Only a fixed mathematical command subset is emitted; file/process directives,
TeX mouth escapes and unbalanced groups remain inert escaped text.

## Contracts Added or Changed

- ORG25: native LaTeX and Beamer documents from checked current source.
- ORG30: bounded explicit external compilation remains separate from rendering.

## Files of Interest

- `src/org/export/tex.rs`: native document envelope and traversal.
- `src/org/export/tex/math.rs`: bounded math recognition and admission.
- `tests/org_io/tex_exports.rs`: independent renderer expectations.

## Validation

- `cargo test --test org_io tex_export`: four workflows passed, zero failures or ignored tests.
- Changed-source quality passed with unchanged native limits.

## Risks and Follow-ups

Optional compiler integration and full slice validation remain in progress.
