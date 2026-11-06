# Command Ranking Tracker

Status: In progress
Date: 2026-11-06
Delivery set: rev-command-ranking
Approved: product owner; specification/model and the established preparation commits, three-slice plan, push and one autonomous Looper delivery through dev
Plan: [Command Ranking Delivery](../plan/rev-command-ranking.md)
Specification: [Command Recency and Frequency](../spec/editor-command-ranking.md)
Model: [Command Usage Model](../spec/editor-command-ranking-model.md)
Concern: [Pending Work Index](../README.md)
Active slice: Slice 2, Persistent usage
Progress owner: retained Looper checkpoint and canonical validation receipts

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Session ranking | `feat/rev-command-ranking-01-session` | `feat(commands): rank recent and frequent command candidates` | `ops/default/report/slices/rev-command-ranking/slice-1-session.md` |
| Slice 2 | Persistent usage | `feat/rev-command-ranking-02-persistence` | `feat(commands): persist bounded command usage across sessions` | `ops/default/report/slices/rev-command-ranking/slice-2-persistence.md` |
| Slice 3 | Acceptance and guidance | `test/rev-command-ranking-03-acceptance` | `test(commands): verify command ranking and document history` | `ops/default/report/slices/rev-command-ranking/slice-3-acceptance.md` |

## Slice 1: Session ranking

State: delivered
Dependency: approved committed preparation on clean dev
Requirements: UI02, CMR01-CMR03 and derived ranking model
Implementation: `3adc46aa4c2e815229c86995aa143ca10023e09e` implements T1.1/T1.2; `1be68be2cb5ca095139d1cd9364f699e4a241e09` implements T1.3 and introduces the report
PR: [#291](https://forge.adrianpk.com/adrianpk/rev/pulls/291), merged into `dev` at `17e16f07580cc32d743e6f8d34510dda14c85c2e`
Report: `ops/default/report/slices/rev-command-ranking/slice-1-session.md`
Validation: all five required commands passed locally, including 328 tests; canonical clean-HEAD evidence belongs to Looper receipts for the final PR-reference commit

- [x] T1.1: Deliver bounded usage and deterministic recent/frequent ranking
  within match ranks, with canonical identities and no duplicates.
- [x] T1.2: Observe shared named dispatch once and capture stable snapshots;
  preserve existing prompt and session-history controls.
- [x] T1.3: Verify native session behavior, document its delivered boundary
  and add one Unreleased product outcome.

## Slice 2: Persistent usage

State: reviewing
Dependency: Slice 1 integrated
Requirements: UI02, CMR01-CMR05 and complete usage model
Implementation: `380c07e992ba94d631f7e82d246cb1fc9f71391a` implements T2.1/T2.2; T2.3 completes real-boundary coverage, guidance and the report on this branch
PR: pending
Report: `ops/default/report/slices/rev-command-ranking/slice-2-persistence.md`
Validation: all five required commands passed locally, including 343 tests; canonical clean-HEAD evidence belongs to Looper receipts for the final PR-reference commit

- [x] T2.1: Deliver bounded versioned TOML/state storage with preserved invalid
  records and enforced event, name and byte limits.
- [x] T2.2: Wire explicit ownership, nonblocking publication, pending-event
  rebasing and safe failure/shutdown behavior.
- [x] T2.3: Deliver the command_usage target with real boundary/wiring checks;
  update state/defaults references and the same Unreleased outcome.

## Slice 3: Acceptance and guidance

State: pending
Dependency: Slice 2 integrated
Requirements: UI02 and CMR01-CMR05 together
Implementation: pending
PR: pending
Report: `ops/default/report/slices/rev-command-ranking/slice-3-acceptance.md`
Validation: pending; exact commands are in the plan and local configuration

- [ ] T3.1: Deliver bounded native, two-profile restart/PTY and paired-publisher
  acceptance using existing Rust/Bash helpers and owned cleanup.
- [ ] T3.2: Complete Diataxis and Markdown-derived Spore guidance; reconcile
  affected baselines without replacing historical evidence.
- [ ] T3.3: Record real report/commit/PR evidence, hand off the clean candidate
  for the controller gate and close only after it passes.

## Final gate and closure

The plan's Final gate section and local configuration define the ordered gate
and environment. No implementation, validation, timing or publication is claimed
at planning.

- [ ] Verify all three canonical PRs integrated and all reports delivered.
- [ ] Pass the unchanged final gate on the exact clean candidate.
- [ ] Record the receipt, commands, immutable report introductions, actual
  timings, state/PTY cleanup and evidence limits.
- [ ] Complete plan/tracker/specification/model and pending index.
- [ ] Preserve separate keybinding-hint, main, release and installation boundaries.

Technical recovery and final-gate repairs stay within the plan's finite
delegation and the same retained run, conversation and consumed budgets.
