# Native Bookmarks Tracker

Status: Implementing
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
Active slice: Slice 4; canonical PR #188 awaiting controller checks and integration
Execution owner: Looper; checkpoint and canonical merge evidence own progress

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

- [x] T1.1: Implement the bounded native collection, stable IDs, set/no-overwrite/jump/delete and checked visit admission.
- [x] T1.2: Add C-x r bindings, vertical name completion/cancellation and complete current catalog/reference/native coverage.
- [x] Verify focused exact-HEAD checks, canonical PR integration and the Slice 1 report.

PR [#185](https://forge.adrianpk.com/adrianpk/rev/pulls/185) merged at
`8a77b4857940bc34310434657ed0fa259f5f7a4f`; exact-candidate receipt
`22OPS6UBYMWKZ5L2JUQOVXTVZI` passed 379 tests and all required checks.
See the canonical Slice 1 report for task commits and documentation evidence.

## Slice 2: Checked location lifecycle

Acceptance: BMK02, BMK06.

- [x] T2.1: Integrate anchor outcomes with prepared edits, retained moves, stale transitions and source undo/redo.
- [x] T2.2: Implement context capture/resolution, cancellation, live-buffer preference and rename/save-as/close/discard lifetimes.
- [x] Verify focused exact-HEAD checks, canonical PR integration and the Slice 2 report.

Implementation commits: T2.1 `dd5cb9fc0cc93b2934303ea78f7d3f2bdf04b1db`;
T2.2 `b63e7834d9b06ab0ceb62b183da8c05295b38e6b`.
[Slice 2 report](../report/slices/rev-bookmarks/slice-2-locations.md) is delivered.
Canonical PR [#186](https://forge.adrianpk.com/adrianpk/rev/pulls/186) merged at
`430441c4c3708bf70b416432767da4483dd1826c`; required receipt
`4X5FHJFBNWQNSJWC6TJFMCMRAT` passed 535 tests and all required checks.
Focused receipt `UX6F7DXZRLYYOJMSRMQENMH42J` passed owner tests, clippy and docs.

## Slice 3: Persistent collection

Acceptance: BMK05 and storage/limit parts of BMK07.

- [x] T3.1: Implement versioned bounded TOML, lossless paths, lazy admission, locked checked publication and real writer-conflict coverage.
- [x] T3.2: Add save/reload and quit policies, complete storage settings/defaults/references and recovery coverage.
- [x] Verify focused exact-HEAD checks, canonical PR integration and the Slice 3 report.

T3.1 storage commit: `88dc724e94768d26e5c7819bb9fedfe4ce47481b`.
T3.2 policy commit: `e642f451f6938722cc35b82c7cc6b5754f36997b`.
Canonical PR: [#187](https://forge.adrianpk.com/adrianpk/rev/pulls/187).
[Slice 3 report](../report/slices/rev-bookmarks/slice-3-persistence.md) records
native persistence, policy, failure and configuration coverage. PR #187 merged
at `1e260f4b0cf6b7928ca253c6edd4825dded1b5ed`; required receipt
`T6PVDVN5AX7IFM3ZIHASLJRYH5` passed 432 integration tests and all required
checks. Focused receipt `XC6MAZMV4MM4YCKZOULFJDTK77` passed 89 library tests,
all-target clippy and documentation.

## Slice 4: List and ordered navigation

Acceptance: complete BMK04.

- [x] T4.1: Implement native bookmark listing, stable row actions, deletion marks and local configurable keys.
- [x] T4.2: Implement current-file next/previous, wrap, same-position ordering and safe invalidation.
- [ ] Verify focused exact-HEAD checks, canonical PR integration and the Slice 4 report.

T4.1 native-list commit: `96b4dc6bf4cd285a6b5f9a40d9d8fe9dd81a5248`.
T4.2 ordered-navigation commit: `3ec7ffc43bbbac67654d17f557187b191ca6b661`.
Canonical PR: [#188](https://forge.adrianpk.com/adrianpk/rev/pulls/188).
[Slice 4 report](../report/slices/rev-bookmarks/slice-4-navigation.md) records
checked list and ordered-navigation behavior, native coverage and final checks.

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

Slices 1 through 3 are integrated. Slice 4 is active under the same controller run.
Preserve previous delivery evidence under its original owners.
