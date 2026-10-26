# Slice 2: Introduce canonical rope buffers

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-02-buffers`
PR: `#2`

## Purpose

Establish canonical buffer ownership, qualified range admission, and bounded
borrowed text reads.

## Delivered Behavior

- `Buffers::new` creates one empty rope-backed buffer; additional buffers retain
  exact UTF-8 text and original line separators in their own canonical ropes.
- Process-local `BufferId` values remain stable as the registry grows and are
  not reused across registry lifetimes. Multiple caller handles resolve to
  borrowed references to the same buffer owner.
- Scalar ranges admit empty insertion positions at both ends. Byte ranges
  require exact UTF-8 scalar boundaries instead of silently rounding offsets.
- Reads reject foreign identities, mismatched revisions, reversed ranges, and
  out-of-bounds positions without changing text or revision.
- Slices, clipped chunk iterators, and bounded line fragments borrow Ropey
  storage and preserve UTF-8 and CRLF without flattening the document.

## Implementation Notes

- T2.1: `f3fc075194321f8603140bdb717597dac12a1688` introduces the registry,
  canonical rope owner, typed identities, qualified spans, and initial storage
  cases.
- T2.2: `9487765a9240a3bb49de35dc3a117d8523432ba8` extends the core suite to
  six integration tests covering admission failures, shared handles, and real
  multi-chunk reads.
- A checked atomic sequence allocates buffer identities across registries;
  exhaustion returns a typed error instead of wrapping and aliasing a buffer.
- `Buffer` owns a private Rope and is not cloneable. Registry lookup returns
  `&Buffer`; callers receive immutable borrowed slices or chunks. Paths and
  caller handles do not become additional text owners.
- The initial publication revision is 1. `Span` carries a `Stamp` and scalar
  range; every qualified read rechecks both metadata and bounds. Byte admission
  verifies round-trip offsets because Ropey's byte-to-character conversion
  otherwise rounds an interior byte down to its containing scalar.
- New buffer-owner functions have measured cyclomatic complexity at most 4;
  error formatting is 7. Their logical nesting is at most 1. All first-party
  source remains within the configured safety, parameter, nesting, and
  complexity limits.

## Contracts Added or Changed

- `BufferId` identifies only buffer lifetime; `Stamp` qualifies source revision,
  independently of future saved-state tracking.
- `Buffers` owns live stores and supplies an initial identity, creation from
  Rust UTF-8 text, and checked borrowed lookup.
- `Buffer::span` and `byte_span` admit ranges; `slice` and `chunks` revalidate
  prepared ranges. `line` borrows a caller-bounded scalar fragment within a
  current line, including its original separator where requested.
- Expected failures return typed `Error` values with identity, revision, range,
  byte, or line context.

## Files of Interest

- `src/buffer.rs`: registry, canonical rope ownership, identities, qualified
  ranges, bounded reads, and errors.
- `src/lib.rs`: exposes the native buffer module.
- `tests/core.rs`: six real core integration tests, including a 4096-line
  Unicode/CRLF fixture spanning multiple rope chunks.

## Validation

Successful commands, in canonical order. Looper records the final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; eight tests, including six core integration tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test core`: passed; six tests, zero skips. Covers empty and
  multiline stores, scalar/byte insertion bounds, stale/foreign spans,
  unchanged state after rejection, bounded multi-chunk/line reads, shared
  borrowed handles, registry growth, and non-reused identities.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
