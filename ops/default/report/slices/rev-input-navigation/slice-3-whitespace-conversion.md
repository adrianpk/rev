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
characters stay outside replacement spans. The configured tab width bounds the
maximum expansion per selected tab; tabify output never exceeds its input run.
Temporary output is proportional to selected whitespace and produced spaces,
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

- PASS: `cargo test --test commands whitespace::` (initial real-command cases).
- Focused source analysis passes unchanged parameter, nesting and complexity
  limits for the new owners and command/session dispatch.
- Unicode/policy, history/fidelity and final clean-HEAD checks are in progress.

## Risks and Follow-ups

None recorded.
