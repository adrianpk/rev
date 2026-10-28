# Slice 6: Color native Org source semantically

Status: reviewing
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-06-semantic-colors`
PR: [#107](https://forge.adrianpk.com/adrianpk/rev/pulls/107)

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

Seventeen independent `palette.org` RGB leaves use the ordinary typed validation,
leaf merge, provenance and immutable frame palette. Factory defaults, the
compiled template, complete sample and catalog cover all 46 active RGB roles.
Every activated Org role has a real capture and a renderer-visible override test.

Task commits: T6.1 `89959a0f2e958560cd85713784ae93708bd84081`, T6.2
`92a42b6638ae1d2ef0a35af3983e4405db8c6f22`, T6.3
`d84c5f5c20af5e7c356e70935b3fb4a7094fa4c8`.

## Implementation Notes

COW rope snapshots leave analysis outside atomic editing and rendering. Retained
line indexes resume at the existing invalidated source scope. Structural and
inline ownership is re-observed from the snapshot to recover delimiters crossing
that scope. The exposed reindexed-byte count measures only index suffix work.
Finite byte/node/depth bounds and cancellation retain explicit coverage states.
Capture finalization checks cancellation before sorting and while building its
bounded query table.

The existing byte-to-grapheme/cell projection clips complete glyphs and keeps
selection/search precedence. Org source never enters the Rust grammar owner.

Combined native fixtures cover pending-result edits, shared panes, history,
BOM/CRLF safe-save, exact UTF-8 cutoff recovery, C-g, hidden/tiny panes and idle
cleanup. Read-only literal source receives colors without edit authority. An
isolated NO_COLOR process verifies original delimiters, no RGB output, selection
reverse video and search underlining. Clipped tab/wide-glyph fragments retain the
existing unstyled edge-cell contract.

## Contracts Added or Changed

- ORG16: native typed source presentation and opaque literal-block context.
- ORG17: semantic roles enter the existing display/configuration owners.

## Files of Interest

- `src/org/color.rs`, `src/org/color/`: stamped captures and bounded observation.
- `src/buffer/org.rs`: exact-revision admission and cache ownership.
- `src/session/org.rs`: shared temporary worker and native event deadlines.
- `src/display/syntax.rs`: existing source/cell projection extended for Org.
- `src/config/palette.rs`, `project.rs`, `merge.rs`, `validate.rs`: typed leaves.
- `docs/reference/config.toml`, `defaults.md`, `syntax.md`: complete references.
- `tests/configuration/org_palette.rs`, `tests/syntax/org_palette.rs`: default,
  inheritance, provenance, admission and actual rendered-cell coverage.
- `tests/org_source/color.rs`: ownership, stale results, suffix recovery and limits.
- `tests/org_editing/colors.rs`, `tests/display/org_colors.rs`: combined native
  revision/worker lifecycles, byte-safe saves, clipping, selection and NO_COLOR.

## Validation

Implementation checks passed:

- `cargo check --lib`
- `cargo test --test org_source color::`: five passed.
- `cargo test --test org_editing`: eight baseline tests passed during T6.1.
- `cargo test --test org_editing colors::`: four combined tests passed.
- `cargo test --test display org_colors::`: three display tests passed, including
  isolated NO_COLOR selection/search rendering.
- `cargo test --test configuration org_palette::`: two passed.
- `cargo test --test syntax org_palette::`: one test covers all 17 roles.
- `python3 scripts/check-config-reference.py`: 95 schema/catalog entries, 46 RGB
  roles, 115 default bindings and compiled-template/link checks passed.

Focused source-quality checks passed for changed native owners and fixtures.
After the PR-reference follow-up, the controller records the final clean pushed
candidate and outcomes for this exact ordered batch in its validation ledger:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test syntax --test display`
- `python3 scripts/check-source.py`
- `git diff --check`

A separate focused exact-head controller batch covers configuration tests and
`python3 scripts/check-config-reference.py`. Final receipts are recorded in Acta;
the integration follow-up retains their exact candidate and outcome here.

## Risks and Follow-ups

Semantic ownership is re-observed within the bounded snapshot; index suffix
reuse does not establish suffix-only semantic work. Task-state, tag and timestamp
roles activate with their later semantic owners. Planning and heading source
currently retain their enclosing roles.
