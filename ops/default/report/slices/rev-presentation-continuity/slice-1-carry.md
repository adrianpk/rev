# Slice 1: Contracts and checked carry

Status: reviewing
Delivery set: rev-presentation-continuity
Plan: `ops/default/plan/rev-presentation-continuity.md`
Tracker: `ops/default/tracker/rev-presentation-continuity.md`
Branch: `refactor/rev-presentation-continuity-01-carry`
PR: pending

## Purpose

Prepare the checked private publication layer before either format consumes carry.
See the [plan](../../../plan/rev-presentation-continuity.md) and
[tracker](../../../tracker/rev-presentation-continuity.md).

## Delivered Behavior

Reconciled OGP08, OIN05, MD06 and six related model clauses with the approved
continuity contract. Historical completed statuses, scenarios and receipts remain
intact; continuity delivery remains pending separately.

Prepared a private source-shape/mapping foundation for one same-line change.
Repeated edits merge into at most eight dirty logical rows; the ninth distinct
row retires the mapping. Checked cumulative byte deltas map current line starts
back to a complete base without treating scalar or cell positions as bytes.
No display consumer uses this foundation yet, so pending presentation retains
its existing literal behavior. This internal layer adds no changelog entry.

## Implementation Notes

Preparation uses the canonical transaction's actual before/after ropes rather
than replaying descending Tree-sitter edits or a composed typing-history record.
An immediate before stamp, matching source length and checked next revision
protect the mapping chain. Rejected shapes/gaps cannot resume without a fresh
complete base. Undo, redo, reload and permutations prepare no mapping.

The 32 KiB inspection budget charges removed/inserted fragments and reserves
both complete logical lines by byte length before any full-line materialization.
Oversized inputs reject mapping before scanning/copying the full line. Mapping
storage is a fixed eight-slot array with base/current stamps and byte lengths;
each occupied slot means the whole row is lexically dirty, including zero-net
edits. No outline/highlight clone, second document or whole-document index is
introduced. Org data stays in its existing owner. Markdown observation retention
and both format admission predicates remain with their integration slices.

The native reserved publication transfers the precomputed mapping together with
source state. Installing the mapping performs only the bounded assignment; it adds no
allocation, source scan, wait, timer, debounce or worker. Existing exact action
APIs and source/history/point publication stay authoritative.

## Contracts Added or Changed

[PC01-PC12](../../../spec/editor-presentation-continuity.md) owns the pending
continuity correction; baseline contracts distinguish it from historical gates.
The foundation prepares PC02, PC03, PC06, PC07 and PC11 without claiming format
admission, continuous colors/geometry or intermediate-terminal acceptance.

## Files of Interest

- `ops/default/spec/editor-presentation-continuity.md`: approved correction.
- The three baseline specifications and their model companions: reconciliation.
- `src/buffer/presentation.rs`: bounded mapping, complete-base provenance and checked deltas.
- `src/buffer/presentation/inspection.rs`: canonical one-change shape and charged byte budget.
- `src/buffer/edit.rs`, `src/buffer/edit/publication.rs`: preparation and reserved installation.
- `src/buffer/presentation/tests.rs`: parameterized mapping/publication boundaries.

## Validation

Local pre-submit slice checks passed in the required order:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`: workspace formatting, Clippy (zero blocking/deferred diagnostics) and native source policy; 1,885 first-party Rust files, two packages and all 75 Cargo roots.
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib buffer:: -- --test-threads=1`: 18 passed, including all 11 new parameterized carry-foundation tests; no failures or ignored tests. Cargo rebuild 52.13 seconds; test execution 1.77 seconds.
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test core --test coordinates --test editing -- --test-threads=1`: 6 core, 24 coordinate and 114 editing tests passed; no failures or ignored tests. Cargo compilation 3.63 seconds; coordinate/core/editing execution 19.03/0.76/75.90 seconds.
4. `git diff --check`: passed, including the staged source candidate.

The new cases exercise repeated replacements and zero-net dirty rows, two-row
composition in non-sorted order, wide/combining UTF-8 text, mapped Org heading
identity against fresh analysis, single-group typing, missing/incomplete/canceled
bases, CR/LF and multi-change refusal, rejected-gap continuity, eight/nine rows,
scan-budget boundaries, checked overflow, history/reload retirement and reserved
publication with stale-preparation rejection. They prove source mapping and
publication behavior, not the later format admission predicates.

During development the focused presentation selection first failed its CR
fixture because the range removed ordinary text; the corrected 11-case selection
passed. Initial source-policy review required factoring bounded inspection to
meet the complexity ceiling. The first full lint run rejected a read-only
mutation argument passed by value; borrowing it resolved the diagnostic. All
required checks above passed after these corrections. Vendored crossterm emits
its pre-existing unused-parentheses warning; first-party lint has no exceptions.

Exact final clean-HEAD evidence is recorded separately by the controller's
validator after the PR-reference commit and push. No PTY or delivery-set gate
was run in this slice. Native tests use their existing owned fixtures.

## Risks and Follow-ups

Org and Markdown admission/display integrations belong to Slices 2 and 3.
The source-shape mapping does not prove format structure and must not become a
display/action observation without those owner guards. Common cancellation,
mode/policy and geometry lifecycle acceptance remains in Slice 4; real terminal
continuity and user guidance remain in Slice 5. The delivery-set gate is pending
controller integration of all five slices.
