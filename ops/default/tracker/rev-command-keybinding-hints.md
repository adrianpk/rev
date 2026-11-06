# Command Keybinding Hints Tracker

Status: Approved
Date: 2026-11-06
Delivery set: rev-command-keybinding-hints
Approved: product owner; continue the established preparation commits, plan, slicing, push and one autonomous Looper delivery through dev
Plan: [Command Keybinding Hints Delivery](../plan/rev-command-keybinding-hints.md)
Specification: [Command Keybinding Hints](../spec/editor-command-keybinding-hints.md)
Concern: [Pending Work Index](../README.md)
Ticket: [TKT-20261031095151](../ticket/ready/20261031095151-show-keybindings-beside-m-x-commands.md)
Active slice: Slice 1 - Effective hints
Progress owner: retained Looper checkpoint and canonical validation receipts

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Effective hints | `feat/rev-command-keybinding-hints-01-effective` | `feat(commands): show effective keybinding hints in command selectors` | `ops/default/report/slices/rev-command-keybinding-hints/slice-1-effective.md` |
| Slice 2 | Acceptance and guidance | `test/rev-command-keybinding-hints-02-acceptance` | `test(commands): verify keybinding hints and document selector behavior` | `ops/default/report/slices/rev-command-keybinding-hints/slice-2-acceptance.md` |

## Slice 1: Effective hints

State: ready
Dependency: approved committed preparation on clean dev
Requirements: KBH01-KBH05; retain UI02 and CMR01-CMR05
Branch: `feat/rev-command-keybinding-hints-01-effective`
PR: pending
Report: `ops/default/report/slices/rev-command-keybinding-hints/slice-1-effective.md`
Validation: the five ordered Slice 1 commands recorded in the plan and local configuration

- [ ] T1.1: Inspect effective sequences through actual dispatch ownership and
  prepare a bounded originating-context snapshot before minibuffer activation.
- [ ] T1.2: Paint normalized primary hints with complete cell-safe labels,
  preserving canonical names, selection, ranking, invocation and usage.
- [ ] T1.3: Extend focused regression coverage, record the delivered outcome in
  Unreleased and produce the implementation report and ticket evidence.

## Slice 2: Acceptance and guidance

State: pending
Dependency: Slice 1 integrated
Requirements: KBH01-KBH05 together
Branch: `test/rev-command-keybinding-hints-02-acceptance`
PR: pending
Report: `ops/default/report/slices/rev-command-keybinding-hints/slice-2-acceptance.md`
Validation: the six ordered Slice 2 commands recorded in the plan and local configuration

- [ ] T2.1: Deliver native matrix coverage and one bounded input/render PTY
  journey per profile through the new Rust/Bash acceptance entrypoint.
- [ ] T2.2: Complete contextual Diataxis guidance, Markdown-derived Spore and
  affected baseline reconciliation before the final gate.
- [ ] T2.3: Record actual task/commit/PR/report evidence and hand off the clean
  candidate for the controller gate; close only after it passes.

## Final gate and closure

The plan defines the exact ordered gate, environment, affected contracts,
estimated cost, native/PTY limits and finite recovery authority. The local
configuration freezes these commands before execution. No implementation,
successful check, report or publication is claimed at preparation.

- [ ] Verify both canonical PRs integrated and reports delivered.
- [ ] Verify gate prerequisites and run the selected unchanged gate on the
  exact clean dev candidate under controller ownership.
- [ ] Record the successful receipt, exact commands, immutable introductions,
  actual stage timings, owned cleanup and evidence limits.
- [ ] Move the source ticket reviewing to solved, resolution fixed, with all
  source links and actual lifecycle metadata consistent.
- [ ] Complete plan, tracker, specification and pending index within the eight
  exact closure paths; preserve the tested runtime and acceptance assertions.

Technical recovery retains the same run and consumed budgets under the plan's
finite delegation. Main promotion, installation, release, tags, mirrors and
other open tickets remain outside this delivery.
