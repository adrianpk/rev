# Keyboard Macros Tracker

Status: Approved
Date: 2026-11-08
Delivery set: rev-keyboard-macros
Approved: owner requested the delivery workflow and delegated reference-compatible decisions on 2026-11-08
Execution authorized: one eight-slice Looper run with automatic integration into dev
Plan: [Keyboard Macros Delivery](../plan/rev-keyboard-macros.md)
Specification: [Keyboard Macros](../spec/editor-keyboard-macros.md)
Model: [Macro Library and State Ownership](../spec/editor-keyboard-macros-model.md)
Concern: [Pending Editor Work](../README.md)
Active slice: Slice 1 of 8, pending execution startup
Progress owner: selected Looper checkpoint and canonical validation evidence

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Recording and replay | feat/rev-keyboard-macros-01-recording | feat(macros): record and replay native editing input | ops/default/report/slices/rev-keyboard-macros/slice-1-recording.md |
| Slice 2 | Named commands and prompt capture | feat/rev-keyboard-macros-02-capture | feat(macros): preserve named commands and accepted prompt input | ops/default/report/slices/rev-keyboard-macros/slice-2-capture.md |
| Slice 3 | Ring, append and counters | feat/rev-keyboard-macros-03-ring | feat(macros): add ring workflows append and counters | ops/default/report/slices/rev-keyboard-macros/slice-3-ring.md |
| Slice 4 | Queries, regions and nested execution | feat/rev-keyboard-macros-04-queries | feat(macros): add query pauses and checked region execution | ops/default/report/slices/rev-keyboard-macros/slice-4-queries.md |
| Slice 5 | Named snapshots and key bindings | feat/rev-keyboard-macros-05-names | feat(macros): expose named snapshots and session bindings | ops/default/report/slices/rev-keyboard-macros/slice-5-names.md |
| Slice 6 | Explicit persistent library | feat/rev-keyboard-macros-06-library | feat(macros): persist explicitly saved definitions as TOML | ops/default/report/slices/rev-keyboard-macros/slice-6-library.md |
| Slice 7 | Definition editing and macro menu | feat/rev-keyboard-macros-07-editing | feat(macros): edit definitions and manage the macro ring | ops/default/report/slices/rev-keyboard-macros/slice-7-editing.md |
| Slice 8 | Acceptance and guidance | test/rev-keyboard-macros-08-acceptance | test(macros): verify complete workflows and document usage | ops/default/report/slices/rev-keyboard-macros/slice-8-acceptance.md |

## Slice 1: Recording and replay

State: pending
Dependency: published approved preparation and controller preflight.
Branch: feat/rev-keyboard-macros-01-recording
PR: pending
Integrated: pending
Report: ops/default/report/slices/rev-keyboard-macros/slice-1-recording.md
Validation: exact ordered slice selection in the plan and frozen configuration

- [ ] T1.1: Add bounded recording/runner ownership, normalized key/text capture, F3/F4 and C-x controls, indicators, cancellation and positive prefix replay through normal input.
- [ ] T1.2: Deliver native per-iteration undo boundaries, typed stops, modal entry checks and cooperative scheduling without an idle worker; unsupported input transfers stop explicitly.
- [ ] T1.3: Introduce the native macros integration target and recording/runner fixtures; align command/default catalogs and record the first usable Unreleased outcome.

## Slice 2: Named commands and prompt capture

State: pending
Dependency: Slice 1 integrated.
Branch: feat/rev-keyboard-macros-02-capture
PR: pending
Integrated: pending
Report: ops/default/report/slices/rev-keyboard-macros/slice-2-capture.md
Validation: exact ordered slice selection in the plan and frozen configuration

- [ ] T2.1: Normalize accepted M-x/colon invocations to one canonical name with typed prefix and role-qualified prompt values; preserve ordinary-key current-map semantics.
- [ ] T2.2: Integrate search, paths/buffer arguments, literal completion contracts, fresh-consent pauses and typed stop/wait observation without replaying selector rows or echo parsing.
- [ ] T2.3: Verify both profiles, changed ranking/bindings, provider-independent completion, missing names, unsupported jobs and exactly-once effects in native capture fixtures.

## Slice 3: Ring, append and counters

State: pending
Dependency: Slice 2 integrated.
Branch: feat/rev-keyboard-macros-03-ring
PR: pending
Integrated: pending
Report: ops/default/report/slices/rev-keyboard-macros/slice-3-ring.md
Validation: exact ordered slice selection in the plan and frozen configuration

- [ ] T3.1: Deliver the global bounded ring, rotate/swap/delete/second-call actions, temporary repeat maps and append variants with independent named-snapshot semantics.
- [ ] T3.2: Implement current/previous/iteration-start counters, checked operations, captured formats and the two fully documented macro configuration leaves.
- [ ] T3.3: Verify counter formats/overflow, repeated controls, append failures, history grouping, bounds and catalog/default consistency through ring/counter fixtures.

## Slice 4: Queries, regions and nested execution

State: pending
Dependency: Slice 3 integrated.
Branch: feat/rev-keyboard-macros-04-queries
PR: pending
Integrated: pending
Report: ops/default/report/slices/rev-keyboard-macros/slice-4-queries.md
Validation: exact ordered slice selection in the plan and frozen configuration

- [ ] T4.1: Implement query continue/skip/stop choices and bounded native manual-edit pause/resume/abort, excluding manual input from recording.
- [ ] T4.2: Apply definitions to checked original logical region-line anchors and enforce shared run budgets across region iterations and admitted manual pauses.
- [ ] T4.3: Validate Unicode/wrapping, line insert/delete remapping, cancellation/partial history, source invalidation, pause ownership and idle release in native fixtures.

## Slice 5: Named snapshots and key bindings

State: pending
Dependency: Slice 4 integrated.
Branch: feat/rev-keyboard-macros-05-names
PR: pending
Integrated: pending
Report: ops/default/report/slices/rev-keyboard-macros/slice-5-names.md
Validation: exact ordered slice selection in the plan and frozen configuration

- [ ] T5.1: Admit bounded independent named snapshots as native dynamic commands with canonical identities, optional descriptions, M-x/colon ranking and effective key hints.
- [ ] T5.2: Implement naming/replacement and session key capture, reserved slots, consent and collision precedence while preserving compiled, prompt, AI and user-configured owners.
- [ ] T5.3: Verify both profiles, pinned nested named calls, cycle/depth rejection, missing definitions, namespace/binding collisions, session-only naming and command-history behavior through names fixtures.

## Slice 6: Explicit persistent library

State: pending
Dependency: Slice 5 integrated.
Branch: feat/rev-keyboard-macros-06-library
PR: pending
Integrated: pending
Report: ops/default/report/slices/rev-keyboard-macros/slice-6-library.md
Validation: exact ordered slice selection in the plan and frozen configuration

- [ ] T6.1: Implement schema-1 library encoding/admission and private concern-owned state publication with exact limits, nonblocking locks, revisions and fresh-record concurrency.
- [ ] T6.2: Deliver explicit save/reload/delete and inert TOML export, startup admission, optional prefixed binding inclusion and preservation of unsaved/conflicting state.
- [ ] T6.3: Verify restart, absent/invalid/oversized/symlink state, unrelated concurrent writes, stale same-record conflicts, uncertain publication and unchanged sibling stores.

## Slice 7: Definition editing and macro menu

State: pending
Dependency: Slice 6 integrated.
Branch: feat/rev-keyboard-macros-07-editing
PR: pending
Integrated: pending
Report: ops/default/report/slices/rev-keyboard-macros/slice-7-editing.md
Validation: exact ordered slice selection in the plan and frozen configuration

- [ ] T7.1: Deliver checked TOML definition buffers, key/region helpers, 300-input lossage projection and target revision checks without implicit saving.
- [ ] T7.2: Implement bounded step editing and the read-only macro menu with stable rows, property edits, marks/flags/copy/delete confirmation and named contextual actions.
- [ ] T7.3: Verify cancelled/stale/invalid editor drafts, step changes and retained source effects, menu targeting, all local actions, both-profile focus and complete descriptions.

## Slice 8: Acceptance and guidance

State: pending
Dependency: Slice 7 integrated.
Branch: test/rev-keyboard-macros-08-acceptance
PR: pending
Integrated: pending
Report: ops/default/report/slices/rev-keyboard-macros/slice-8-acceptance.md
Validation: exact ordered slice selection in the plan and frozen configuration

- [ ] T8.1: Complete integrated KMA01-KMA09 acceptance, representative real PTY function-key/C-x journeys, restart and concurrent storage boundaries plus large-buffer cancellation/idle probes.
- [ ] T8.2: Add concise User Guide and how-to paths, complete command/default/state references and explanation; keep Markdown-derived Spore and the complete sample configuration aligned.
- [ ] T8.3: Finalize all reports and task evidence, reconcile catalogs and Unreleased outcomes, and prepare the unchanged selected integrated controller gate and documentation-only closure.

## Final gate and closure

- [ ] Verify eight canonical integrations and delivered report identities.
- [ ] Run the unchanged ten-command selected gate on exact clean dev.
- [ ] Record tested candidate, controller receipt and actual acceptance coverage.
- [ ] Complete specification/model/plan/tracker and reconcile the index.
- [ ] Confirm only the thirteen configured closure documents changed afterward.

No implementation or validation result is claimed by this preparation. Retained
controller state owns live progress. Task evidence must use actual commits, PRs,
checks and reports rather than inferred activity.
