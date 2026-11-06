# Command Keybinding Hints Tracker

Status: In progress
Date: 2026-11-06
Delivery set: rev-command-keybinding-hints
Approved: product owner; continue the established preparation commits, plan, slicing, push and one autonomous Looper delivery through dev
Plan: [Command Keybinding Hints Delivery](../plan/rev-command-keybinding-hints.md)
Specification: [Command Keybinding Hints](../spec/editor-command-keybinding-hints.md)
Concern: [Pending Work Index](../README.md)
Ticket: [TKT-20261031095151](../ticket/reviewing/20261031095151-show-keybindings-beside-m-x-commands.md)
Active slice: None - all recorded slices integrated; controller gate pending
Progress owner: retained Looper checkpoint and canonical validation receipts

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Effective hints | `feat/rev-command-keybinding-hints-01-effective` | `feat(commands): show effective keybinding hints in command selectors` | `ops/default/report/slices/rev-command-keybinding-hints/slice-1-effective.md` |
| Slice 2 | Acceptance and guidance | `test/rev-command-keybinding-hints-02-acceptance` | `test(commands): verify keybinding hints and document selector behavior` | `ops/default/report/slices/rev-command-keybinding-hints/slice-2-acceptance.md` |

## Slice 1: Effective hints

State: completed
Dependency: approved committed preparation on clean dev
Requirements: KBH01-KBH05; retain UI02 and CMR01-CMR05
Branch: `feat/rev-command-keybinding-hints-01-effective`
PR: [#294](https://forge.adrianpk.com/adrianpk/rev/pulls/294)
Report: `ops/default/report/slices/rev-command-keybinding-hints/slice-1-effective.md`
Validation: the five ordered Slice 1 commands recorded in the plan and local configuration

- [x] T1.1: Inspect effective sequences through actual dispatch ownership and
  prepare a bounded originating-context snapshot before minibuffer activation.
- [x] T1.2: Paint normalized primary hints with complete cell-safe labels,
  preserving canonical names, selection, ranking, invocation and usage.
- [x] T1.3: Extend focused regression coverage, record the delivered outcome in
  Unreleased and produce the implementation report and ticket evidence.

## Slice 2: Acceptance and guidance

State: completed
Dependency: Slice 1 integrated
Requirements: KBH01-KBH05 together
Branch: `test/rev-command-keybinding-hints-02-acceptance`
PR: [#295](https://forge.adrianpk.com/adrianpk/rev/pulls/295)
Report: `ops/default/report/slices/rev-command-keybinding-hints/slice-2-acceptance.md`
Validation: the six ordered Slice 2 commands recorded in the plan and local configuration

- [x] T2.1: Deliver native matrix coverage and one bounded input/render PTY
  journey per profile through the new Rust/Bash acceptance entrypoint.
- [x] T2.2: Complete contextual Diataxis guidance, Markdown-derived Spore and
  affected baseline reconciliation before the final gate.
- [x] T2.3: Record actual task/commit/PR/report evidence and hand off the clean
  candidate for the controller gate; close only after it passes.

## Final gate and closure

The plan defines the exact ordered gate, environment, affected contracts,
estimated cost, native/PTY limits and finite recovery authority. The local
configuration freezes these commands before execution. The original preparation made no implementation or validation claim. Slice 1
review evidence is recorded below; publication remains outside this set.

- [x] Verify both canonical PRs integrated and reports delivered.
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

## Slice 1 review evidence

- T1.1 and T1.2 source: `c18cdc0b5bb3feb05074e4d2bcba07c0afcfba6b`.
- T1.3: `844ec6c5fb9d0c6f71d1de2ca6f98538423d8d12` extends existing
  commands/modal/display tests, records one Unreleased outcome and introduces
  the canonical Slice 1 report.
- Canonical [PR #294](https://forge.adrianpk.com/adrianpk/rev/pulls/294) targets `dev`;
  the report link pins its full immutable introduction commit.
- All five required slice commands passed locally: 336 tests (172 commands,
  44 display, 120 modal), 85 documentation pages and 576 local links/anchors.
- Eight focused hint tests passed, including configured/normalized sequences,
  local ownership, all modal states, clipping, identity and actual cell colors.
- Final clean-HEAD ordered evidence is recorded by Looper for the PR-reference
  commit. PR #294 is integrated at that exact commit; the combined gate remains pending.

Slice 1 integrated: `05365b43a51f5aaa7217c489117afc78289b0abe`. Its clean-HEAD
receipt `D6ITQPUQYZ5CJ5CUMWE6JOV236` passed all five ordered commands. Canonical
Forgejo confirms PR #294 merged into dev on 2026-11-06. Slice 2 subsequently
integrated through PR #295; no recorded slice remains active.


## Slice 2 review evidence

- T2.1: `2a66698097f1a66ec1efbf18dc61c9f33c4990f7` adds the bounded
  native/PTY entrypoint and preserves usage assertions for annotated rows.
- T2.2: `266f6384283bbdc9c791d1c2c9250d510bfd3023` updates canonical
  guidance and reconciles UI02 while preserving historical gate evidence.
- The native matrix and both-profile focused PTY journeys pass. Each journey
  observed 29,637 bytes, zero idle CPU ticks and restored terminal attributes;
  all owned temporary fixtures were cleaned.
- The six required commands pass locally: 15 command-usage tests, all-target
  Clippy, Bash syntax and 85 documentation pages with 578 local links/anchors.
  The local strict Spore build projects all 85 Markdown pages.
- T2.3 introduces the Slice 2 report in
  `543c5324ca45fbdf83c31fe272a3516b0c781f94`. Canonical [PR #295](https://forge.adrianpk.com/adrianpk/rev/pulls/295)
  targets dev and pins that immutable introduction. Its following reference
  commit records the review; exact clean-HEAD evidence is recorded once there.
  Canonical PR #295 merged into dev at that exact validated head on 2026-11-06;
  the combined controller gate remains pending.

Slice 2 integrated: `3952b5915ec7048ec7892e4e51c61ec3094e7f5d`. Its clean-HEAD
receipt `AFRA7O7QCHCSKEGJ3IXL7ZQO43` passed all six ordered commands, including
15 command-usage tests. The immutable report introduction remains
`543c5324ca45fbdf83c31fe272a3516b0c781f94`. Canonical Forgejo and Acta both
confirm the merge. These slice receipts do not validate the later delivered-
document commit or replace the controller's full gate.

## Full-gate prerequisites

Preparation on 2026-11-06 verified Rust/Cargo, rustfmt, Clippy, Bash, Git,
script, stty, kill, timeout, Spore and guide asset utilities. Locked offline
Cargo metadata resolves the dependency graph and admits the rev binary, all
five selected test targets and all three selected examples. Native surface and
PTY helpers, canonical guide inputs and Spore theme assets are readable.
Target and temporary roots permit owned output creation; probes were removed.
Linux PTY and process-observation access are available.

These prerequisite-only commands passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo metadata --locked --offline --format-version 1`
- `bash -n scripts/check-command-keybinding-hints.sh`
- `bash -n scripts/build-user-guide.sh`

The worker did not run the combined acceptance script or full gate. The
controller must validate the exact clean pushed dev candidate after this
metadata closure. The plan's gate, resource limits and assertions are unchanged.
