# Slice 2: Index Org structure with bounded work

Status: reviewing
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-02-incremental-index`
PR: [#103](https://forge.adrianpk.com/adrianpk/rev/pulls/103)

## Purpose

Build bounded source indexes and owned observations that remain consistent with
canonical edits while ordinary editing continues.

## Delivered Behavior

The native index retains exact logical-line offsets and outline-prefix ancestry.
Coverage explicitly distinguishes pending, complete, cancelled and byte/node/depth
limits. Partial observations expose only complete source lines. UTF-8, CRLF,
empty EOF rows and original source remain intact.

Canonical publication invalidates an installed index from the affected outline
scope through EOF, retaining the unaffected prefix. Resumed analysis moves that
prefix and reconstructs its ancestry. Grouped typing, disjoint edits and undo/redo
use each actual publication's coordinates rather than grouped history ranges.

One observation worker slot owns explicit analysis or IO work. Cancellation keeps
that slot occupied until its worker is reaped. Polling is nonblocking; results
are checked against the current document and revision before exposure. Dropping
the owner requests cancellation and joins its worker.

## Implementation Notes

The index stores offsets rather than document text. Analysis retains shared
immutable rope nodes; no full-document string or row-vector copy occurs on the
input path. Each compute step processes at most 4096 bytes or 10ms, checking
cancellation between scalar observations. An already running native operation
completes before the deadline is checked again. A blocked OS read cannot be
forcibly interrupted by this contract.

Factory limits are 8MiB of source, 200000 nodes and depth 256; admitted maxima
are 32MiB, 1000000 nodes and depth 1024. Reaching a limit retains editable source
and reports incomplete coverage. Optional buffer caches start absent, and no
permanent worker or idle loop is installed.

An indexed outline prefix is a source candidate. Complete coverage means that
the source was fully indexed; semantic recognition, including literal/block
contexts, remains assigned to Slices 3 and 4. The 1MiB/10000-heading fixture and
cancellation assertions are native regression evidence, not terminal latency or
RSS measurements.

## Contracts Added or Changed

- ORG02: source indexing preserves exact bytes and qualifies each exposed range;
  stale or foreign indexes cannot be installed or resumed.
- ORG18: enforce finite observation bounds, cooperative cancellation, one owned
  worker slot and cleanup while preserving canonical edit/history admission.

## Files of Interest

- `src/org/index.rs` and `src/org/index/`: offsets, ancestry, limits, bounded
  scanning, coverage and incremental resumption.
- `src/org/job.rs`: shared observation ownership, cancellation and result admission.
- `src/buffer/org.rs` and `src/buffer/edit.rs`: optional cache admission and
  invalidation at canonical publication.
- `tests/org_source/`: source limits, prefix reuse, real typing history, large
  edits, stale results, worker cancellation and cleanup.

## Validation

Implementation checks passed:

- `cargo check --lib`
- `cargo test --test org_source`: 22 passed, zero failed or ignored.
- `cargo clippy --test org_source -- -D warnings`
- `git diff --check`

The first controller batch stopped at Clippy's type-complexity warning for a
new test table. Removing its redundant tuple annotation and using inferred
slices fixed the declaration while preserving all six cases and assertions.

The final clean PR head requires the following exact controller-recorded batch.
Its receipt and the canonical PR retain the final candidate and check outcomes:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test editing`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

None recorded.
