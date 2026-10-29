# Slice 55: tangle Org source with bounded noweb references

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-55-tangling`
PR: pending

## Purpose

Deliver native tangling under the
[approved plan](../../../plan/rev-org-mode.md#slice-55-tangle-org-source-with-bounded-noweb-references)
and [tracker](../../../tracker/rev-org-mode.md#slice-55-tangle-org-source-with-bounded-noweb-references).

## Delivered Behavior

Explicit whole-buffer or current-block tangling extracts declared source into
separate checked files. Named and grouped noweb references expand recursively,
retaining relative indentation and source-order concatenation. Each existing
output requires replacement confirmation; per-file outcomes remain inspectable
after cancellation, stale targets or partial failure. Canonical source, unsaved
edits, history, BOM and newline format remain unchanged.

## Implementation Notes

A bounded, cancellable source observation produces one immutable plan before
file effects. All expansions and destinations pass preflight together. Ordinary
checked atomic publication retains each file's real outcome and mode. Pinned
parent directories reject symlink replacement. The editor retains exclusive
source input during publication. Literal native headers never evaluate code.

See the [source block reference](../../../../../docs/reference/org-source-blocks.md#tangling)
and the Org manual's [tangling](https://orgmode.org/manual/Extracting-Source-Code.html)
and [noweb syntax](https://orgmode.org/manual/Noweb-Reference-Syntax.html).

## Contracts Added or Changed

- ORG28: bounded named expansion and explicit native source tangling.
- ORG18: checked file effects, cancellation and retained independent outcomes.

## Files of Interest

- `src/org/tangle.rs` and `src/org/tangle/`: pure expansion and checked file plans.
- `src/session/tangling.rs`: source ownership, replacement consent and outcomes.
- `tests/org_source/tangling.rs` and `tangle_bounds.rs`: reference, selection,
  header, path, revision and resource contracts.
- `tests/org_io/tangling.rs`: real editor and filesystem workflows.

## Validation

Development checks passed:

- `cargo check --lib`
- `cargo test --test org_source --test org_io tangling`
- `cargo test --test org_source --test org_io tangle`
- `cargo test --test org_io tangling`

The initial eight focused tests passed. Subsequent boundary tests admit exactly
64 reference levels, 512 source blocks, 256 files and 8 MiB of combined output,
then reject excess. Dense acyclic expansion reaches the independent 100000-step
ceiling. Whole-plan preflight rejects symlink parents/leaves, hard-link aliases
of source/outputs and live output buffers. Replaced parents cannot redirect a
later write; earlier successful outputs remain recorded. Nine editor/filesystem
tangling workflows now pass, including the real whole-buffer/current-block chords,
read-only source, default language paths, preserved replacement permissions,
independent consent, invalid answers and queued cancellation. Required exact-head
checks remain pending.

## Risks and Follow-ups

Source is limited to 8 MiB and 512 blocks. Tangling caps combined outputs at
8 MiB, 256 files, 64 reference levels and 100000 expansion steps. No executable
noweb forms, variable evaluation, comments, shebang or mode transformations are
interpreted. Unsupported requested transformations reject before writing.
