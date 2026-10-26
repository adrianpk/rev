# Slice 4: Publish coherent text edits

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-04-edits`
PR: `#4`
Integrated: Rebase + Fast-forward into dev at `709ec2682b739a896c012bc5451766ab5d7dff4d`.

## Purpose

Publish checked text replacements, reversible records, and dependent positions
as one coherent buffer operation.

## Delivered Behavior

- Prepares ordered, disjoint scalar replacements against one buffer revision.
  Applies the batch in descending base-coordinate order so earlier offsets
  remain valid despite unequal replacement lengths.
- Rejects stale or foreign ranges, invalid bounds, overlap, ambiguous equal
  starts, empty batches, invalid points, and read-only writes without changing
  canonical text or history.
- Completes replacements, reversible fragments, point mapping, mark mapping,
  and the explicit initiating cursor on a private candidate before publication.
  Dropping a prepared candidate cancels it without changing confirmed state.
- Rechecks the live revision, read-only state, points, and mark at publication.
  Changed source state rejects the candidate without overwriting newer work.
- Publishes one revision and one reversible transaction per accepted batch.
  Points use right affinity and the buffer mark uses left affinity. Newly
  joined graphemes are adjusted to valid boundaries, including CRLF and emoji.

## Implementation Notes

- T4.1: `2fed813e6ac8c1a5761e1a2733ccda7b9998d9fb` adds the checked owner,
  preparation/publication contracts, reversible fragments, and initial tests.
- T4.2: `ba6a69da8fad140524c0abaa4a7dc000624e31ff` completes ten editing
  integration tests, including failed final-cursor preparation and exact state
  preservation after rejected publication.
- The candidate is Ropey's copy-on-write clone. Only the published buffer rope
  is canonical; no whole-document String copy enters the edit path.
- History retains removed/inserted rope fragments, before/after scalar spans,
  qualified point snapshots, mark snapshots, and publication stamps. The stamp
  pair identifies the revision transition for derived-cache invalidation.
- Publication reserves history capacity before changing confirmed state. After
  that boundary, it performs no recoverable work or callbacks. Allocator abort
  and process termination are outside the recoverable preparation contract.
- Each transaction currently forms one complete history record. Retention uses
  the approved 512-record and soft 64 MiB logical-payload bounds, pruning whole
  oldest records while always retaining the newest reversible operation.
- New production functions measure at most 8 cyclomatic paths, nesting at most
  2, and at most five parameters including receivers.

## Contracts Added or Changed

- `Replacement` carries a qualified base-revision scalar span and Rust UTF-8
  replacement text. Distinct ordered starts remove insertion-order ambiguity.
- `Buffer::prepare` returns an opaque `Prepared` candidate without publication.
  `Buffer::publish` is the only text mutation path and consumes that candidate.
- `Point` qualifies view-owned positions with a buffer stamp. The caller supplies
  every live point for the target buffer; an optional `Cursor` sets the initiating
  point explicitly in final coordinates. The buffer owns its shared mark.
- `Transaction` exposes immutable reversible fragments and before/after state.
  Its final stamp agrees with the published rope and every mapped point.
- Mutable buffer access exposes checked operations and read-only/mark state,
  without exposing the canonical rope mutably. Coordinates can borrow private
  candidate text internally for grapheme adjustment.

## Files of Interest

- `src/buffer.rs`: canonical buffer state and checked mutable registry access.
- `src/buffer/edit.rs`: preparation, admission, atomic publication, and records.
- `src/buffer/edit/mapping.rs`: exact per-replacement mapping and affinity.
- `src/coordinates.rs`: internal borrowed candidate-coordinate constructor.
- `tests/editing.rs`: ten public-boundary editing integration tests.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; twenty-six tests, including ten editing tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test editing`: passed; ten tests, zero skips. Cases cover
  coherent publication, reversible UTF-8/CRLF fragments, batch admission,
  recoverable preparation failure, read-only rechecks, changed live state,
  exact affinity, adjacent ranges, a real chunk-seam join, and whole-record
  retention. Point, mark, text, revision, and history remain unchanged on failure.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
