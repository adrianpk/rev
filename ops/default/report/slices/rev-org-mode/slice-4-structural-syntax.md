# Slice 4: Recognize Org headings metadata and blocks

Status: drafting
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-04-structural-syntax`
PR: pending

## Purpose

Recognize structural Org ownership over canonical source without transferring
editing or execution authority to observations.

## Delivered Behavior

Revision-qualified observations distinguish headings, sections, continuation and
nested list ownership, ordinary/property drawers, property values and additive
spelling, planning lines, keywords and affiliated metadata. Tables retain cells,
separator rows, captions, names and contiguous formula lines in their owned source.

## Implementation Notes

The recognizer retains a shared immutable rope snapshot and derived offsets.
Heading source includes descendants; section source stops before the next
heading. Heading titles, paragraph text and table cells expose explicit inline
contexts. Property drawers require their valid heading/planning or zeroth-section
position and valid property contents.

Reference boundary observations use clean Emacs 31.1 / Org 9.8.7 and the
[Org syntax reference](https://orgmode.org/worg/org-syntax.html). Native tests
use independent expected ranges and require no reference runtime.

## Contracts Added or Changed

- ORG02: exact revision-qualified source, preserved spelling and explicit malformed
  or incomplete observations.
- ORG04: complete heading, section and list ownership for later checked commands.
- ORG05: ordinary drawers and locally owned property drawers remain distinct.
- ORG12: table source retains affiliated metadata and formula lines.

## Files of Interest

- `src/org/structure.rs`: public structural observations and explicit owned scan.
- `src/org/structure/`: native classification, outline, element and list ownership.
- `tests/org_source/structure.rs`: source ranges and structural context fixtures.

## Validation

- `cargo check --lib`
- `cargo test --test org_source structure::`: four tests passed.

## Risks and Follow-ups

T4.2 block ownership and T4.3 recovery/observation validation remain in progress.
