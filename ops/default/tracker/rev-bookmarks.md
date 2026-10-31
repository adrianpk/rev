# Native Bookmarks Tracker

Status: Ready
Date: 2026-10-31
Delivery set: rev-bookmarks
Slice strategy: behavior-first
Approved: product owner, 2026-10-31; complete scope, six-slice map, autonomous implementation, canonical PR merges into dev, final gate and documentation closure; explicit startup request
Planning baseline: `3083e6467756755b56ac5e753dab9625c1d336da`
Spec: [Native Bookmarks](../spec/editor-bookmarks.md)
Model: [Bookmarks Model](../spec/editor-bookmarks-model.md)
Spec: [Editor Gutter](../spec/editor-gutter.md)
Model: [Gutter Model](../spec/editor-gutter-model.md)
Plan: [Bookmarks Plan](../plan/rev-bookmarks.md)
Concern: [Pending Work Index](../README.md)
Full gate: `cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-bookmarks.sh && cargo run --locked --quiet --example documentation-check && git diff --check`
Active slice: Slice 1; named bookmark commands, ready for controller startup
Execution owner: Looper after startup; this is the initial tracker, not a second scheduler

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Named bookmark commands | `feat/rev-bookmarks-01-commands` | `feat(bookmarks): set and visit named file locations` | `ops/default/report/slices/rev-bookmarks/slice-1-commands.md` |
| Slice 2 | Checked location lifecycle | `feat/rev-bookmarks-02-locations` | `feat(bookmarks): retain checked locations through edits` | `ops/default/report/slices/rev-bookmarks/slice-2-locations.md` |
| Slice 3 | Persistent collection | `feat/rev-bookmarks-03-persistence` | `feat(bookmarks): persist the user bookmark collection` | `ops/default/report/slices/rev-bookmarks/slice-3-persistence.md` |
| Slice 4 | List and ordered navigation | `feat/rev-bookmarks-04-navigation` | `feat(bookmarks): list and navigate saved locations` | `ops/default/report/slices/rev-bookmarks/slice-4-navigation.md` |
| Slice 5 | Gutter markers | `feat/rev-bookmarks-05-gutter` | `feat(gutter): show native bookmark markers` | `ops/default/report/slices/rev-bookmarks/slice-5-gutter.md` |
| Slice 6 | Joint acceptance and guidance | `feat/rev-bookmarks-06-acceptance` | `test(bookmarks): validate persistent navigation and markers` | `ops/default/report/slices/rev-bookmarks/slice-6-acceptance.md` |

## Slice 1: Named bookmark commands

Acceptance: BMK01; initial command and naming parts of BMK04-BMK05.

- [ ] T1.1: Implement the bounded native collection, stable IDs, set/no-overwrite/jump/delete and checked visit admission.
- [ ] T1.2: Add C-x r bindings, vertical name completion/cancellation and complete current catalog/reference/native coverage.
- [ ] Verify focused exact-HEAD checks, canonical PR integration and the Slice 1 report.

## Slice 2: Checked location lifecycle

Acceptance: BMK02, BMK06.

- [ ] T2.1: Integrate anchor outcomes with prepared edits, retained moves, stale transitions and source undo/redo.
- [ ] T2.2: Implement context capture/resolution, cancellation, live-buffer preference and rename/save-as/close/discard lifetimes.
- [ ] Verify focused exact-HEAD checks, canonical PR integration and the Slice 2 report.

## Slice 3: Persistent collection

Acceptance: BMK05 and storage/limit parts of BMK07.

- [ ] T3.1: Implement versioned bounded TOML, lossless paths, lazy admission, locked checked publication and real writer-conflict coverage.
- [ ] T3.2: Add save/reload and quit policies, complete storage settings/defaults/references and recovery coverage.
- [ ] Verify focused exact-HEAD checks, canonical PR integration and the Slice 3 report.

## Slice 4: List and ordered navigation

Acceptance: complete BMK04.

- [ ] T4.1: Implement native bookmark listing, stable row actions, deletion marks and local configurable keys.
- [ ] T4.2: Implement current-file next/previous, wrap, same-position ordering and safe invalidation.
- [ ] Verify focused exact-HEAD checks, canonical PR integration and the Slice 4 report.

## Slice 5: Gutter markers

Acceptance: BMK03 and marker parts of BMK07.

- [ ] T5.1: Integrate marker geometry and current logical-row lookup with numbering, wrapping, folds, pointer and search owners.
- [ ] T5.2: Add marker configuration, semantic palette, per-view toggle and complete sample/catalog/reference coverage.
- [ ] Verify focused exact-HEAD checks, canonical PR integration and the Slice 5 report.

## Slice 6: Joint acceptance and guidance

Acceptance: BMK01-BMK07 jointly.

- [ ] T6.1: Add the bounded Bash/Rust native/PTY restart journey and resource measurements.
- [ ] T6.2: Complete user guidance, focused text examples, changelog, sample/reference consistency and related gutter reconciliation.
- [ ] Verify focused exact-HEAD checks, canonical PR integration and the Slice 6 report.

## Final gate and closure

- [ ] Controller verifies all six canonical slice integrations.
- [ ] Controller runs the pinned Full gate on the clean integrated dev candidate.
- [ ] Record exact candidate, gate commands, result/receipt and any bounded repair evidence.
- [ ] Close only the configured operational documentation after the gate passes.

No implementation, test result, PR, controller run or completion is recorded by
this preparation. Preserve previous delivery evidence under its original owners.
