# Slice 7: Org workflow decomposition

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-07-org`
PR: pending

## Purpose

Resolve selected ownership and source-policy findings in native Org document
workflows and their existing session adapters.

## Delivered Behavior

Implementation pending. Parsing, preparation and publication must retain their
existing native owners, source text and user-visible behavior.

## Implementation Notes

Preserve navigation, folding, indentation, consent, timestamps, capture/agenda
locations, formulas, source-block behavior and document/resource limits. Helpers
remain under the selected Org and session concern roots.

## Contracts Added or Changed

None. Package, configuration, state, source-coordinate and wire APIs remain
compatible.

## Files of Interest

- `src/org.rs` and `src/org/`: native document workflow owners.
- Selected `src/session/` Org adapters: admission, preparation and publication.

## Validation

Pending; the four frozen ordered commands remain unchanged in the approved plan.

## Risks and Follow-ups

The controller owns merge, full delivery-set validation and successor startup.
