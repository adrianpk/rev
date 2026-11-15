# Slice 1: Configuration and language policies

Status: drafting
Delivery set: rev-delimiter-editing
Plan: [Automatic Delimiter Editing Delivery](../../../plan/rev-delimiter-editing.md)
Tracker: [Automatic Delimiter Editing Tracker](../../../tracker/rev-delimiter-editing.md)
Branch: `feat/rev-delimiter-editing-01-config`
PR: pending

## Purpose

Deliver the approved five editing preferences and native buffer policy without
automatic source edits or future presentation settings.

## Delivered Behavior

Global editing leaves default true. Partial global and singular language tables
retain absent siblings and merge explicit values with independent origin paths.
Immutable snapshots resolve language policy independently of grammar availability.

## Implementation Notes

The existing typed loader, candidate validation, merge and snapshot owners retain
configuration admission. No discovery path or dependency is added.

## Contracts Added or Changed

`pairs` and `language.<id>.pairs` admit only `enabled`, `wrap_selection`,
`delete_empty`, `skip_closing` and `quotes`, each a Boolean.

## Files of Interest

- `src/config/pairs.rs`: partial presence, compiled defaults and leaf overlay.
- `src/config/resolve.rs`: immutable effective policy.

## Validation

Pending the slice's focused checks.

## Risks and Follow-ups

Automatic typing, deletion, wrapping and command dispatch belong to later slices.
