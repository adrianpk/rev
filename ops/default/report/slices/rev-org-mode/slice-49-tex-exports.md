# Slice 49: export Org to LaTeX and Beamer

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-49-tex-exports`
PR: `#150`

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

T49.2 adds native LaTeX/Beamer commands and explicit PDF variants. Optional
`org.tools.pdf` is literal, validated argv with whole-token input/output paths.
Only an explicit PDF command and compiler confirmation launch the existing
bounded process owner. Private mode-0700 staging is cleaned after execution;
nonzero exit, absent tools, invalid or oversized output and stale evidence reject
before destination publication. PDF bytes use the existing checked atomic file
publisher, preserving observed replacement confirmation and source state.

T49.3 verifies exact escaped output, read-only subtree selection, finite renderer
budgets and cancellation. PDF adapter fixtures cover absent executable, failed
exit, missing/malformed/nonregular or oversized PDF, combined process output
limit, timeout, cancellation and stale canonical/disk/target evidence. Successful
fixtures verify binary bytes, mode preservation, private directory permissions,
direct-child reaping and workspace removal. The physical RGB/NO_COLOR journey
also checks native article/slides files and the default absent-compiler message.

## Implementation Notes

The renderer shares immutable source evidence, cancellation, operation deadline
and output limits. It has no process, network or publication authority. Math
recognition precedes entity conversion and protects native literal objects.
Only a fixed mathematical command subset is emitted; file/process directives,
TeX mouth escapes and unbalanced groups remain inert escaped text.

Task commits: T49.1 `6f41f860510f063903b357c64fff705f0fe0f814`, T49.2 `bac94f94cb0f9499f9c82ea914a3cb1395f30ab2`, T49.3 `6712a1ae4d8c19c1bc55ca5852ba5583bb077f50`.

Canonical PR: [#150](https://forge.adrianpk.com/adrianpk/rev/pulls/150).

## Contracts Added or Changed

- ORG25: native LaTeX and Beamer documents from checked current source.
- ORG30: bounded explicit external compilation remains separate from rendering.

## Files of Interest

- `src/org/export/tex.rs`: native document envelope and traversal.
- `src/org/export/tex/math.rs`: bounded math recognition and admission.
- `tests/org_io/tex_exports.rs`: independent renderer expectations.

## Validation

- `cargo test --test org_io tex_`: ten renderer/adapter workflows passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_io --test configuration --test commands -- -D warnings`: passed.
- The output-limit fixture initially buffered Python stdout while waiting. Explicit flushing now exposes the same bounded-output assertion without changing acceptance or time/output limits.

- `cargo test --test org_io tex_job`: two adapter/publication workflows passed, zero failures or ignored tests.
- `cargo test --test configuration pdf_configuration`: layered argv and rejection workflow passed.
- `cargo test --test commands visibility_keys`: compiled catalog round-trip passed.

- `cargo test --test org_io tex_export`: four workflows passed, zero failures or ignored tests.
- Changed-source quality passed with unchanged native limits.

Canonical PR #150 merged into dev at `833df9ffa01227d8f75296cb9faa58bbab1d984b`, verified against Forgejo and origin/dev. Required receipt `2GN2GD3L4BN6LCVTYYCREXHVO3` passed five ordered checks and 174 tests. Focused receipt `2NO2DSMTDZZHPCPALMXTS2GBPJ` passed 280 command/configuration tests, three process-owner tests, the configuration reference and two owned RGB/NO_COLOR terminal sessions. Zero failures or ignored tests; zero skipped sessions.

## Risks and Follow-ups

No TeX engine is installed in the validation environment. Fixture programs verify adapter and binary publication mechanics; they do not establish compiler semantics or PDF rendering.
