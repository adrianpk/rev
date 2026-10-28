# Slice 13: edit and match Org task tags

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-13-tags-matching`
PR: [#114](https://forge.adrianpk.com/adrianpk/rev/pulls/114)

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
- `tests/org_editing/tags.rs`, `tag_edges.rs`: actual editing, provenance,
  read-only/stale/deeper-context rejection, shared views and source preservation.
- `tests/configuration/org_tags.rs`: typed include policy and provenance.
- `scripts/configuration_ui/org_tags.py`: tag/query steps within the existing
  owned RGB/NO_COLOR outline journey.

## Validation

Development checks passed:

- `cargo check`
- `cargo test --test org_editing tags::`: four initial tag tests passed.
- `cargo test --test org_source --test org_editing matching`: one actual-session
  test and three public parser/evaluator tests passed during grammar delivery.
- `cargo test --test org_editing --test org_source --test configuration tag`:
  11 native tag tests and one configuration policy test passed.
- `cargo test --test org_source matching::`: all four grammar/type/boundary tests
  passed, including exact byte/depth/node/projection caps and their successors.
- `python3 scripts/check-config-reference.py`: schema/catalog, sample bindings,
  template bytes, native copying and links agree.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-outline`:
  two owned RGB/NO_COLOR sessions passed, zero skipped. Original outline, list
  and property assertions remain intact. Tag entry/completion, local/inherited
  inspection, queries, malformed-input rejection and cancellation restore exact
  BOM/CRLF/EOF bytes and preserve cleanup assertions.

Focused source-quality checks passed for changed Rust owners and tests.
The first check exposed excess dispatch complexity and declaration nesting;
small parser, prompt and declaration helpers retain the same assertions.
Exact-cap coverage admitted 32 nested groups with the atom excluded from group
nesting. Completion computes a shared Unicode prefix in linear candidate text
work, and evaluation indexes tag membership instead of rescanning all tags per
atom. No backtracking evaluator, callback, dependency or permanent worker is added.

Punctuated TODO regression fixtures first reproduced tag-name validation rejecting
`NEXT-ACTION`. Task declarations and TODO suffixes now share bounded task-keyword
admission while heading tags retain their stricter spelling. Both public and
actual-session regressions pass; unrelated source and history remain unchanged.

The first required controller batch reported a Clippy `manual_inspect` finding
in tag candidate collection. The observation now uses `inspect` for its side
effects, preserving source observations and all behavior assertions. Final
receipts are regenerated after that correction.

An owned clean Emacs observation confirmed factory ancestor/file inheritance
and local-only membership when inheritance is disabled. Native semantics were
checked against [tags](https://orgmode.org/manual/Tags.html),
[setting tags](https://orgmode.org/manual/Setting-Tags.html),
[tag inheritance](https://orgmode.org/manual/Tag-Inheritance.html) and
[matching](https://orgmode.org/manual/Matching-tags-and-properties.html).
Parentheses follow the approved native grammar.

Canonical PR [#114](https://forge.adrianpk.com/adrianpk/rev/pulls/114)
merged into `dev` at `2f81bd7547f3a0ae3eb4d4cb9f9c34476a66d7c3`.
Required receipt `Z3FB7EH4EYOYLNJTXEGM5EBJKV` passed all five commands at that
exact clean HEAD, including 312 Rust tests:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test org_source --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused receipt `6EIW7RMFBU4IUTWXW26OJUDRZY` at the same HEAD passed:

- `cargo test --test configuration`: 99 tests.
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-outline`:
  two owned RGB/NO_COLOR sessions, zero skipped, original assertions retained,
  exact source bytes and terminal/fd/child cleanup checked.
- `git diff --check`

## Risks and Follow-ups

Tag observations admit at most 4096 headings, 8192 tag/candidate occurrences and
8192 metadata contexts; each heading admits 128 local names. Queries admit 4096
UTF-8 bytes, 32 nested forms and 512 nodes. Temporary metadata is capped at
1048576 bytes. Incomplete or malformed input rejects without changing source.
Inspection displays at most 32 source origins and completion shows at most 16
candidates. Numeric operands use finite f64 comparison; string operands compare
case-sensitively. No other follow-up is recorded.
