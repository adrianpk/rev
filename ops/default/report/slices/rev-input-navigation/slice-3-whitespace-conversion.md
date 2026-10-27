# Slice 3: Add explicit tabify and untabify

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-03-whitespace-conversion`
PR: pending

## Purpose

Convert explicitly selected whitespace under the effective buffer tab policy
without changing other text, settings or visual text columns.

## Delivered Behavior

- `M-x tabify` and `M-x untabify` use the existing admitted region. Missing
  regions report the ordinary region error; an empty region is a no-op.
- Untabify expands selected tabs at their true display columns. Tabify replaces
  selected ASCII whitespace runs only with a shorter tab/space representation.
  One-cell runs cannot shorten and stay unchanged.
- Conversion publishes disjoint replacements as one canonical transaction,
  using existing shared-view position mapping and history ownership.

## Implementation Notes

The buffer owner scans canonical graphemes from each selected line's beginning
so Unicode and unselected prefixes contribute their real columns. Only wholly
selected ASCII space/tab glyphs join a run. Line separators and all other
characters stay outside replacement spans. The configured 1..=32 tab width bounds
expansion per selected tab; tabify output never exceeds its input run.
A following zero-width cluster can join the last expanded space. One additional
space anchors that cluster's existing visible cell when required, preserving
its and subsequent text columns without changing Unicode bytes. This uses one
following grapheme of lookahead and at most one anchor space per run.
Temporary output is proportional to selected whitespace, produced spaces and
that following grapheme,
with checked allocation and position arithmetic; no document-wide copy or
retained per-line cache is introduced.

The session reuses active-region admission and rejects nonempty read-only
conversions before preparing output. It captures all shared points, computes the
initiating region endpoint, prepares every replacement against one revision and
publishes once through the ordinary view/history owner. No-op planning returns
before publication.

## Contracts Added or Changed

WSP01 exposes both native names on the admitted marked region with preserved
visual columns. WSP02 uses one checked transaction and ordinary safety/history.

## Files of Interest

- `src/buffer/whitespace.rs`: checked region/run projection and conversion.
- `src/session/whitespace.rs`: admission and atomic publication.
- `src/command.rs`: both compiled command identities.
- `tests/commands/whitespace.rs`: actual M-x conversion and missing-region guard.

## Validation

- PASS: `cargo test --test commands whitespace::` (three table-driven groups:
  actual M-x/missing-region behavior, 13 explicit expected-byte cases and 36
  width/prefix/run/direction cases with idempotence and non-whitespace columns).
- Focused source analysis passes unchanged parameter, nesting and complexity
  limits for the new owners and command/session dispatch.
- A regression reproduced a lost presentation cell after a tab followed by a
  combining mark. The selected replacement now anchors that existing cell,
  retaining the WSP01 column assertion and original Unicode source bytes.
- The width grid uses the existing 1..=32 admission range; no policy limit changes.
- History/fidelity, provenance and final clean-HEAD checks are in progress.

## Risks and Follow-ups

None recorded.
