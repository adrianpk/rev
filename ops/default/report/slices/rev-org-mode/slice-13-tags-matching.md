# Slice 13: edit and match Org task tags

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-13-tags-matching`
PR: pending

## Purpose

Deliver local tag editing and provenance with bounded tag/property matching.

## Delivered Behavior

C-c C-q, named tag entry and exact heading C-c C-c use the same checked suffix
edit. Removal is case-sensitive and local. Inspection distinguishes local,
ancestor and FILETAGS owners. Completion uses observed tags and data-only TAGS
candidates; cancellation leaves source unchanged. Native `org.tags.inherit`
defaults to true and never inserts inherited text into child headings.

`org-match-tags` evaluates the current heading through a reusable compiled
Boolean grammar with parentheses, implicit AND/NOT, TODO suffixes and typed
numeric/string comparisons. Requested properties project from actual source,
including admitted inheritance and derived heading metadata. Source remains
unchanged.

## Implementation Notes

The existing temporary observation worker derives suffixes and membership;
the existing outline retains the hierarchy. Tag records, candidates and metadata
contexts are bounded. Edits qualify the original source/view and use normal
prepare/publish and atomic history. No input/render parser or permanent worker
is introduced.

## Contracts Added or Changed

- ORG05: case-sensitive local and inherited tag membership with source owners.
- ORG23: bounded tag/property match semantics.

## Files of Interest

- `src/org/visibility/tags/`: native tag observations and checked suffix plans.
- `src/session/outline/tags.rs`: qualified prompts, completion and publication.
- `src/org/query/`: bounded compilation and iterative typed evaluation.
- `tests/org_source/matching.rs`: grammar/escaping/type/limit contracts.
- `tests/org_editing/tags.rs`: source preservation and local/inherited behavior.

## Validation

- `cargo test --test org_editing tags::`: four tests passed.
- `python3 scripts/check-config-reference.py`: 97 schema/catalog entries and
  136 default bindings agree; packaged template bytes and native copy pass.
- `cargo test --test org_source --test org_editing matching`: one native
  integration test and three public parser/evaluator tests passed.
- Focused source-quality checks passed for changed owners and tests.
- An owned clean Emacs observation confirmed factory ancestor/file inheritance
  and local-only membership when inheritance is disabled.

## Risks and Follow-ups

T13.3 boundary verification remains active.
