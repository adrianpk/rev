# Presentation Continuity Tracker

Status: Approved
Date: 2026-11-14
Delivery set: rev-presentation-continuity
Slice strategy: layered
Approved: product owner, 2026-11-14; approved continuity specification and explicit clarification that approval continues the established plan, slicing and autonomous Looper workflow; this five-slice map is derived under that authority
Plan: [Presentation Continuity Delivery](../plan/rev-presentation-continuity.md)
Spec: [Org and Markdown Presentation Continuity](../spec/editor-presentation-continuity.md)
Active slice: Slice 4 - Shared lifecycle and resource acceptance
Progress owner: native Looper checkpoint, canonical PR state and exact-candidate validation receipts

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Contracts and checked carry | `refactor/rev-presentation-continuity-01-carry` | `refactor(presentation): prepare bounded display carry` | `ops/default/report/slices/rev-presentation-continuity/slice-1-carry.md` |
| Slice 2 | Org presentation continuity | `fix/rev-presentation-continuity-02-org` | `fix(org): preserve headings during ordinary typing` | `ops/default/report/slices/rev-presentation-continuity/slice-2-org.md` |
| Slice 3 | Markdown presentation continuity | `fix/rev-presentation-continuity-03-markdown` | `fix(markdown): retain verified heading presentation` | `ops/default/report/slices/rev-presentation-continuity/slice-3-markdown.md` |
| Slice 4 | Shared lifecycle and resource acceptance | `test/rev-presentation-continuity-04-lifecycle` | `test(presentation): cover carry lifecycle and bounds` | `ops/default/report/slices/rev-presentation-continuity/slice-4-lifecycle.md` |
| Slice 5 | Terminal acceptance and guidance | `test/rev-presentation-continuity-05-acceptance` | `test(presentation): verify continuous Org and Markdown frames` | `ops/default/report/slices/rev-presentation-continuity/slice-5-acceptance.md` |

## Slice 1: Contracts and checked carry

State: delivered - PR #41 integration verified at `354c3ebb12ba3459860c5e28c1ff34de3ea140f9`
Requirements: PC02, PC03, PC06, PC07, PC11; OGP08, OIN05, MD06 contract reconciliation
Branch: `refactor/rev-presentation-continuity-01-carry`
PR: [#41](https://forge.adrianpk.com/rev/rev/pulls/41)
Report: `ops/default/report/slices/rev-presentation-continuity/slice-1-carry.md`
Validation: ordered Slice 1 checks passed locally (lint, 18 buffer tests, 144 core/coordinate/editing tests and whitespace). Final published clean-HEAD evidence belongs to the controller validator.
Task commits: T1.1 `5fa1b0b4ba100070957a28623f2d20fdf30d6123`; T1.2 and T1.3 `a18ca3a8827dcacc1315f4337ad581bdd57ac47f` (cohesive mapping/publication layer and regressions).

- [x] T1.1: Reconcile the approved OGP08, OIN05 and MD06 deltas and the six named model clauses before runtime work. Preserve every inherited scenario, completed status and historical receipt; identify the new continuity scope and its separate pending delivery.
- [x] T1.2: Prepare a private bounded carry transition from the canonical single-change before/after ropes, with exact/base/current identity, at most eight rows, a charged 32 KiB scan budget, checked row/byte deltas and infallible reserved installation. Retain existing owner data rather than cloning whole observations or introducing a second index.
- [x] T1.3: Add passing parameterized mapping/publication regressions under the buffer unit-test namespace for repeated same-row and two-row edits, multibyte source, rejected shapes, overflow and budget exhaustion. Keep existing literal behavior until each format's display integration is delivered; report this internal layer without a changelog entry.

## Slice 2: Org presentation continuity

State: delivered - PR #42 integration verified at `2845d1c9e69db465962e863bd10639127e80e5b0`
Requirements: PC01-PC04, PC06-PC11; reconciled OGP08 and OIN05
Branch: `fix/rev-presentation-continuity-02-org`
PR: [#42](https://forge.adrianpk.com/rev/rev/pulls/42)
Report: `ops/default/report/slices/rev-presentation-continuity/slice-2-org.md`
Validation: ordered Slice 2 checks passed locally (lint, seven geometry tests, 25 heading tests, 18 indentation tests and whitespace); focused buffer checks passed all 21 cases. Final published clean-HEAD evidence belongs to the controller validator.
Task commits: T2.1, T2.2 and T2.3 `d6d38d3fffc5701d1514f75f9a1c7ff5a96780f0` (cohesive Org admission/display integration and native regressions).

- [x] T2.1: Reuse the Org owner's actual bounded classifier for before/after body admission and recognized-prefix title admission. Preserve ASCII-space recognition and literal ownership; reject boundary/definition changes and unknown cases, with full-analysis comparison and mandatory positive carries.
- [x] T2.2: Integrate the current facade with Org indentation, concealment, marker rendering and palette-derived structural heading faces. Keep exact structural APIs unchanged, current source/hit mappings coherent and geometry identity stable through a proven carry and equal exact replacement.
- [x] T2.3: Extend Org heading/indentation fixtures with all four policy combinations and intermediate-frame continuity assertions, title/body changes, disabled colors, cancellation, stale-hit retirement, Unicode and source fidelity. Preserve existing lifecycle-test intent and add one concise release-worthy changelog outcome when the Org correction is usable.

## Slice 3: Markdown presentation continuity

State: delivered - PR #43 integration verified at `b87c7af693f8540715619be6553ab062b19d3675`
Requirements: PC01-PC03, PC05-PC11; reconciled MD06
Branch: `fix/rev-presentation-continuity-03-markdown`
PR: [#43](https://forge.adrianpk.com/rev/rev/pulls/43)
Report: `ops/default/report/slices/rev-presentation-continuity/slice-3-markdown.md`
Validation: ordered Slice 3 checks passed locally (lint, 55 Markdown tests and whitespace); focused buffer checks passed 24 cases. Final published clean-HEAD evidence belongs to the controller validator.
Task commits: T3.1, T3.2 and T3.3 `e4a505671a434f9bdbe7a3d52a8d2d1013e519e0` (cohesive Markdown admission/display integration and native regressions).

- [x] T3.1: Retain admitted complete Markdown base data behind the private adapter. Implement conservative before/after same-line body and ATX-title guards with stable indentation and non-blankness; reject fences, definitions, structural and uncertain contexts. Validate accepted cases against full pulldown-cmark output without introducing another parser or dependency.
- [x] T3.2: Use the current facade for heading/list symbols, structural heading colors and checked capture mappings. Preserve width/indentation, active palette and view policy; dirty changed rows and uncertain lexical context conservatively. Keep list/table actions exact and cancellation literal.
- [x] T3.3: Extend the Markdown fixture suite with mandatory positive body/title carries, unchanged neighboring list markers, definitions and literal-code refusals, monochrome/disabled-color behavior and intermediate-frame/source-fidelity checks. Extend the existing concise changelog outcome to the delivered Markdown correction.

## Slice 4: Shared lifecycle and resource acceptance

State: reviewing - canonical PR #44 open; controller validation and integration pending
Requirements: PC02, PC03, PC06-PC12; cross-format integration
Branch: `test/rev-presentation-continuity-04-lifecycle`
PR: [#44](https://forge.adrianpk.com/rev/rev/pulls/44)
Report: `ops/default/report/slices/rev-presentation-continuity/slice-4-lifecycle.md`
Validation: ordered Slice 4 checks passed locally (lint, 29 buffer tests, 23 selected continuity integration tests and whitespace); four native lifecycle/cache cases and one existing Rust source-revision case also passed. Final published clean-HEAD evidence belongs to the controller validator.
Task commits: T4.1, T4.2 and T4.3 `7f1f834fe8de4dde5c650039e3eafeece338430f` (cohesive shared lifecycle, geometry, authority and resource regressions).

- [x] T4.1: Complete and verify common retirement rules for cancellation, incomplete/limited results, rejected publication, ninth row, reload, undo/redo, buffer close and relevant mode/policy changes. Preserve exact worker/generation checks and current fold-clearing behavior.
- [x] T4.2: Add focused continuity cases in existing geometry/view owners for independent panes, policy transitions, wrapped and horizontally clipped projections, pointer/selection correspondence and equal/different exact replacements. Keep source stamps and unchanged-prefix cache reuse; record actual cache/mapping bounds and no additional timers/workers.
- [x] T4.3: Exercise repeated and two-row pending edits, lexical-context uncertainty beyond viewport edges, resource ceilings and native action-authority refusal. Report accepted/rejected admission counts with positive examples. Any runtime refinements are confined to the approved continuity contracts.

## Slice 5: Terminal acceptance and guidance

State: pending
Requirements: PC01-PC12; joint terminal acceptance and delivery closure
Branch: `test/rev-presentation-continuity-05-acceptance`
PR: pending
Report: `ops/default/report/slices/rev-presentation-continuity/slice-5-acceptance.md`
Validation: the exact ordered Slice 5 checks in the plan and frozen local configuration
Task commits: pending

- [ ] T5.1: Add scripts/check-presentation-continuity.sh as a narrow entrypoint over the existing native/PTY fixture, terminal ownership and cell-decoder infrastructure. Extend the existing Org/Markdown acceptance owners rather than building a new framework. Verify both formats and both input profiles with all intermediate states, real worker completion, colors/source controls, cleanup and exact source preservation.
- [ ] T5.2: Run minimal, approximately 50 KB and 500 KB body/title cases, bounded heading-density cases and at least five keys within 180 ms on one/two rows. Record whether work actually remained pending, every observed marker/title-column/foreground state, accepted/rejected counts and measurement limits. Keep graphical claims separate from emitted-terminal evidence.
- [ ] T5.3: Update the existing Org/Markdown reference and User Guide text, derive the Spore guide, preserve a concise changelog outcome and complete the slice report. Leave final set completion pending the controller's exact integrated full gate; afterward close only authorized Markdown artifacts and remove this set's integrated inactive clean worktrees.

## Delivery-set gate

State: pending - all five slices must integrate first.
Run the exact full gate recorded in the plan through the controller on clean
published primary dev. Record its candidate, receipt and actual results before
marking this set Completed. A slice count alone is not final acceptance.

## Worktree disposition

Slice 1 worktree removed non-forcibly after verified integration, clean/unlocked
state, no live process references and classification of its ignored Cargo target
and private hook cache as generated output. Slice 2 was removed under the same
checks after PR #42 integration. Slice 3 was removed under the same checks after PR #43 integration. All three branches are preserved. Pre-existing
and unrelated worktrees remain untouched.

## Final closure

Pending the integrated gate, delivered reports, all fifteen tasks closed,
consistent Markdown artifacts and clean synchronized dev. Slice 1 supplies the
internal mapping foundation; Slice 2 delivers Org admission/display. Slice 3 delivers Markdown admission/display. Shared lifecycle and terminal
acceptance remain pending in Slices 4-5; no final
delivery-set acceptance, release or installation is claimed.
