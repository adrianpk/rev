# Slice 9: Retain buffer policies across path changes

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-09-policy-transitions`
PR: [#58](https://forge.adrianpk.com/adrianpk/rev/pulls/58)

## Purpose

Keep rectangle transfer and successful path changes coherent with the source
and destination buffer policies while retaining file fidelity and history.

## Delivered Behavior

- Yank places every nonempty retained row at the same destination display
  column. A short row or an interior glyph receives the minimum left gap;
  the complete destination glyph stays intact. Empty rows gain no padding.
- Copied spaces and killed literal tabs remain unchanged in the retained rows.
  Literal tabs follow destination stops; no row is retabulated to force the
  source right edge. All row edits remain one reversible publication.
- Published saves and successful file renames resolve editing preferences from
  the registry's immutable startup snapshot after updating the visiting path.
  Directory renames refresh every affected visited descendant, including
  intended missing files, while preserving visiting alias basenames.
- Rejected writes, failed renames and canceled write-file prompts preserve the
  prior path and policy. Published saves with unresolved completion use the new
  policy and retain the existing warning and dirty-state rules.
- Path changes preserve text, BOM, separators, final-newline absence, revision,
  marks and history. Shared views retain independent text positions and
  synchronize their geometry and cached columns against the new buffer policy.

## Implementation Notes

- T9.1: `3d7be72f1dc132c4d0adba0d77489deaa66c0f3e` extends the existing
  insertion padding calculation to interior destination glyphs. It retains the
  source-store bytes, checked compound-edit path and whole-grapheme lookup.
- T9.2: `ae94322442e1b4b8e77704c8777cd0271206c77d` applies the registry
  snapshot only after a save has published or rename has succeeded. Rename
  already enumerates affected canonical owners; each receives its new policy
  when its location is reconciled.
- T9.3: `8572b9653894e08f1d084a870af0c52e8bc9b022` verifies cross-width
  shared insertion, canceled/failed/successful
  write-file with independent views, renamed descendants and aliases, and
  truthful saved-state replay. Config files are changed after snapshot creation
  to detect accidental rereads during later publication.
- The existing view policy synchronization and fresh coordinate derivation
  handle reprojecting unchanged text positions; no second policy or geometry
  cache is added. New-file format preferences are never reapplied.

## Contracts Added or Changed

- Rectangle left padding now accounts for the gap between a whole-glyph source
  boundary and the requested destination column, including interior tabs and
  wide clusters. Retained row length remains independent of requested width.
- The save publication and rename relocation owners update buffer editing
  policy alongside a successfully published association. Canonical owner,
  symlink, consent, external-change and observation guarantees remain active.
- Undo/redo changes document state without rolling back the current visiting
  association or its editing policy. Successful save alone acknowledges the
  published saved state; policy resolution does not create a history unit.

## Files of Interest

- `src/rectangle/yank.rs`: destination-column left-gap calculation.
- `src/buffer/save.rs`, `src/buffer/rename.rs`: publication-time policy refresh.
- `tests/rectangles/transfer.rs`: cross-width copied/killed rows and shared replay.
- `tests/files/policy.rs`: format fidelity, publication faults and shared views.
- `tests/tree/rename_policy.rs`: file and descendant policy transitions.
- `docs/reference/rectangles.md`, `CHANGELOG.md`: observable insertion behavior.

## Validation

Development checks:

- `cargo test --test rectangles transfer::`: passed; cross-width transfer matrix.
- `cargo test --test rectangles target_edges`: passed; tab and wide boundaries.
- `cargo test --test files policy::`: passed; save and publication outcomes.
- `cargo test --test tree policy::`: passed; successful and failed file renames.
- `cargo test --test files shared_policy`: passed; real write-file shared views.
- `cargo test --test tree directory_policy`: passed; descendants and aliases.
- `cargo test --test rectangles shared_transfer`: passed; shared atomic replay.
- `cargo clippy --lib --test rectangles --test tree --test files -- -D warnings`: passed.
- `python3 scripts/check-source.py`: passed.

All seven ordered checks passed on the clean final PR head
`a4d9565cf4fcff7487808fe2281f78ec1d1ea24f`, controller validation
`OOM5IPKLFDQUBFEXTAA6PRC44Y`: 40 rectangle, 58 tree and 70 file tests.
PR #58 integrated that exact commit into dev by Rebase + Fast-forward.
Commands, in canonical order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test rectangles`
- `cargo test --test tree`
- `cargo test --test files`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

Startup configuration activation remains assigned to Slice 20. These checks
supply validated snapshots directly; no runtime reload is introduced.
