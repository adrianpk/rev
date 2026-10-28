# Slice 1: Define native Org source contracts

Status: reviewing
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-01-source-contract`
PR: pending

## Purpose

Establish native Org source observations that retain the canonical buffer's
identity, revision, text and edit authority.

## Delivered Behavior

`Source` borrows one immutable buffer revision. Byte ranges are admitted only
at valid UTF-8 scalar boundaries and retain the buffer's checked scalar span.
Text access rejects foreign or stale ranges, including after undo restores the
original bytes. Valid observations borrow rope slices without copying a document.

Forms retain a classification and source location. Recognized, unknown and
malformed forms preserve their original text, including executable-looking
directives, BOM, CRLF, spacing and final-newline state. Reading observations
does not publish edits or create history entries.

## Implementation Notes

The private source and form modules extend the native `org` owner. They reuse
`Buffer::byte_span`, `Buffer::slice`, `Stamp` and `Span`; no separate text store,
file loader, configuration owner or execution path is introduced.

Form classifications are supplied by the recognizer. This slice defines their
representation and admission, while indexing and syntax recognition belong to
Slices 2 through 4. Source ranges use scalar boundaries; editor view points
separately require grapheme boundaries. The new tests verify that distinction
and the existing read-only and transaction admission rules.

## Contracts Added or Changed

- ORG01: establish the in-house Org module and source observation interface.
- ORG02: preserve original bytes and reject invalid, foreign or stale locations
  before reading forms or preparing edits.
- ORG18: borrow canonical rope slices without whole-document copies or background
  work; reuse existing point, history and read-only boundaries.

## Files of Interest

- `src/org.rs`: native Org public source contracts.
- `src/org/source.rs`: checked byte admission, revision-qualified locations and
  borrowed source access.
- `src/org/form.rs`: data-only recognized, unknown and malformed observations.
- `tests/org_source.rs`: six native tests with named boundary and fidelity cases,
  including unrelated heading edits and stale locations after undo.

## Validation

Implementation checks passed:

- `cargo check --lib`
- `cargo test --test org_source`: six passed, zero failed or ignored.
- `cargo run --quiet --example source-check -- --source src/org.rs --source src/org/source.rs --source src/org/form.rs --root tests/org_source.rs`
- `git diff --check`

The initial combined test run passed all 24 coordinate tests and exposed an
incorrect assumption in the new edit test about scalar ranges versus view-point
boundaries. The corrected test passed before all six Org tests passed together.

The final clean PR head requires this exact controller-recorded batch. Its
receipt and the canonical PR retain the final candidate and check outcomes:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test coordinates`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

None recorded.
