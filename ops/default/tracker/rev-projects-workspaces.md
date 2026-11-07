# Files, Directories, Projects and Workspaces Tracker

Status: Approved
Approved: product owner, 2026-11-07; approved core specification/model and the established preparation commits, ordered slicing, push and single autonomous Looper delivery through dev
Date: 2026-11-07
Delivery set: rev-projects-workspaces
Specification: [Files, Directories, Projects and Workspaces](../spec/editor-projects-workspaces.md)
Model: [Project and Workspace Model](../spec/editor-projects-workspaces-model.md)
Concern: [Pending Work Index](../README.md)
Plan: [Projects and Workspaces Delivery](../plan/rev-projects-workspaces.md)
Active slice: Slice 3 - Project registration and sessions
Progress owner: retained Looper checkpoint and canonical validation receipts
Full gate: the exact ordered command in the plan and frozen local configuration

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Context state and catalog | `feat/rev-projects-workspaces-01-state` | `feat(contexts): add typed contexts and atomic state storage` | `ops/default/report/slices/rev-projects-workspaces/slice-1-state.md` |
| Slice 2 | File and directory contexts | `feat/rev-projects-workspaces-02-directories` | `feat(contexts): preserve ordinary file and directory workflows` | `ops/default/report/slices/rev-projects-workspaces/slice-2-directories.md` |
| Slice 3 | Project registration and sessions | `feat/rev-projects-workspaces-03-projects` | `feat(projects): register projects and restore basic file sessions` | `ops/default/report/slices/rev-projects-workspaces/slice-3-projects.md` |
| Slice 4 | Workspace roots and trees | `feat/rev-projects-workspaces-04-workspaces` | `feat(workspaces): manage independent roots and workspace sessions` | `ops/default/report/slices/rev-projects-workspaces/slice-4-workspaces.md` |
| Slice 5 | Artifact lifecycle and conversion | `feat/rev-projects-workspaces-05-management` | `feat(contexts): manage project and workspace lifecycles` | `ops/default/report/slices/rev-projects-workspaces/slice-5-management.md` |
| Slice 6 | Startup screen and shared consumers | `feat/rev-projects-workspaces-06-startup` | `feat(startup): add the native startup screen and context selection` | `ops/default/report/slices/rev-projects-workspaces/slice-6-startup.md` |
| Slice 7 | Combined native and terminal acceptance | `test/rev-projects-workspaces-07-acceptance` | `test(contexts): verify native startup and persistent context workflows` | `ops/default/report/slices/rev-projects-workspaces/slice-7-acceptance.md` |
| Slice 8 | Guidance and gate preparation | `docs/rev-projects-workspaces-08-guidance` | `docs(contexts): document files projects workspaces and startup` | `ops/default/report/slices/rev-projects-workspaces/slice-8-guidance.md` |

## Slice 1: Context state and catalog

State: delivered
Dependency: approved committed preparation on clean dev
Requirements: PW01, PW09, PW15, PW17-PW19, PW22
Branch: `feat/rev-projects-workspaces-01-state`
PR: [#298](https://forge.adrianpk.com/adrianpk/rev/pulls/298)
Report: `ops/default/report/slices/rev-projects-workspaces/slice-1-state.md`
Validation: the exact ordered Slice 1 commands in the plan/local configuration
Implementation: T1.1/T1.2 in `834ee5bdf30d16d327b2da21a45e37c1281d7243`; T1.3 and the report introduction in `fd3d8f51132f3a74b9ce8824084f128df5d1b664`. All five local implementation checks passed; the clean-HEAD controller receipt is required before PR handoff.
Integration: PR #298 merged into dev at `975564c53cf7c6f420c7d7b3fbb8d5f46950e710`; all five clean-HEAD commands passed in receipt `3P5AYMCUAW33ZSH3VMPNL3LEAS`.

- [x] T1.1: Define typed context, artifact, root and session identities and the approved path/name/source-coordinate model; separate live, prepared and durable state.
- [x] T1.2: Implement the versioned aggregate contexts.toml codec and store with existing dependencies, checked revisions, bounded nonblocking publication, one atomic conversion commit point and preserved invalid input.
- [x] T1.3: Add tests/contexts.rs with meaningful native storage/domain cases, real isolated files, aliases/non-UTF-8 paths, stale writers, capacity rejection and publication-fault preservation; deliver the slice report.

## Slice 2: File and directory contexts

State: delivered
Dependency: Slice 1 integrated
Requirements: PW01, PW04, PW06, PW18-PW20, PW22
Branch: `feat/rev-projects-workspaces-02-directories`
PR: [#299](https://forge.adrianpk.com/adrianpk/rev/pulls/299)
Report: `ops/default/report/slices/rev-projects-workspaces/slice-2-directories.md`
Validation: the exact ordered Slice 2 commands in the plan/local configuration
Implementation: T2.1/T2.2 and T2.3 native cases/guidance in `5032eb9f5161c718a7dd820eb374376263e54bb8`; report introduction in `ab0f958003ad81ac8dc59d8f79c45d7e8e3416f8`. All five implementation checks passed, including 28 context, 75 file and 71 tree tests; final clean-HEAD evidence remains controller-owned.

Integration: PR #299 merged into dev at `4a1b2ddfef6e15a97f0c50d86c46df582612b884`; all five clean-HEAD commands passed in receipt `EKTPDNJVFR73HFHROBLVI2EBKW`.

- [x] T2.1: Admit standalone and directory contexts around existing file/tree owners. Preserve real source editing/saving, direct file/directory targets and explicit root intent; opening a directory never registers a Project or overwrites its saved session.
- [x] T2.2: Add generate-new-buffer, open-directory and context-close through native commands and checked replacement. Preserve in-context find-file visits and loose buffers; reuse source save/discard/cancel and owned-job retirement without exiting the application.
- [x] T2.3: Verify both profiles, standalone manual tree requests, directory cancellation, canonical buffer reuse, source stamps and stale asynchronous completions. Document each public operation actually delivered and update Unreleased through changelog-normalize.

## Slice 3: Project registration and sessions

State: reviewing
Dependency: Slice 2 integrated
Requirements: PW05-PW06, PW15-PW16, PW18-PW20, PW22
Branch: `feat/rev-projects-workspaces-03-projects`
PR: pending
Report: `ops/default/report/slices/rev-projects-workspaces/slice-3-projects.md`
Validation: the exact ordered Slice 3 commands in the plan/local configuration
Implementation: T3.1/T3.2/T3.3 in `d099a88b3562df9ce4ef3ad611ecf23f7a869782`, including nine new Project cases, persistent terminal departure and delivered guidance. All five implementation commands passed, including 37 context, 49 integrated-terminal and 25 recovery tests; final clean-HEAD evidence remains controller-owned.

- [x] T3.1: Deliver explicit project-create/project-promote and named project selection/listing, with unique roots/names, typed picker identities and no implicit path-based promotion.
- [x] T3.2: Capture ordered file-backed buffers, active file, source cursor/view anchors and loose files; restore using native loaders and checked source coordinates. Protect dirty/unnamed sources and preserve missing-file/root diagnostics without creating replacement files.
- [x] T3.3: Wire checkpoint ownership into accepted context departure and clean quit; selecting the same artifact is a no-op. Test source decisions, live promotion, restoration, clocks/terminal shutdown ownership and storage failures; document delivered commands and session behavior.

## Slice 4: Workspace roots and trees

State: planned
Dependency: Slice 3 integrated
Requirements: PW09-PW11, PW15-PW16, PW18-PW22
Branch: `feat/rev-projects-workspaces-04-workspaces`
PR: pending
Report: `ops/default/report/slices/rev-projects-workspaces/slice-4-workspaces.md`
Validation: the exact ordered Slice 4 commands in the plan/local configuration

- [ ] T4.1: Deliver empty Workspace creation, named selection/listing and independent basic sessions. Add directories or Project-derived paths without importing Project sessions or consuming registrations.
- [ ] T4.2: Project ordered root identities into the existing decorated/minimal tree, retaining lazy discovery, Nerd Fonts policy, hit geometry and canonical buffers across aliases/overlap. Preserve unavailable roots and their saved entries.
- [ ] T4.3: Deliver checked root removal, remapping surviving overlapping file references and resolving only affected buffers before committing membership. Test empty/partial/unavailable roots, both profiles and native pointer/keyboard behavior; document delivered workflows.

## Slice 5: Artifact lifecycle and conversion

State: planned
Dependency: Slice 4 integrated
Requirements: PW07-PW08, PW12-PW14, PW17-PW19, PW22
Branch: `feat/rev-projects-workspaces-05-management`
PR: pending
Report: `ops/default/report/slices/rev-projects-workspaces/slice-5-management.md`
Validation: the exact ordered Slice 5 commands in the plan/local configuration

- [ ] T5.1: Deliver rename/forget and explicit root relocation with stable IDs, root-relative remapping and unchanged source directories. Forgetting an active Project retains its live directory work; forgetting an active Workspace resolves sources and leaves no arbitrary replacement root.
- [ ] T5.2: Deliver atomic Project-to-Workspace conversion and independent duplication, including active/inactive sources, fresh Workspace/root IDs, retained live dirty text where permitted and cancellation before duplicate publication.
- [ ] T5.3: Test both sides of durable commit, restart recovery, stale writers, invalid/corrupt state, moved roots, root/name conflicts and source/session preservation. Keep revisions and all logical references consistent; document the lifecycle operations.

## Slice 6: Startup screen and shared consumers

State: planned
Dependency: Slice 5 integrated
Requirements: PW01-PW04, PW20-PW22; accepted P01/P03/P18
Branch: `feat/rev-projects-workspaces-06-startup`
PR: pending
Report: `ops/default/report/slices/rev-projects-workspaces/slice-6-startup.md`
Validation: the exact ordered Slice 6 commands in the plan/local configuration

- [ ] T6.1: Show the native startup screen for no-target executable startup; explicit paths still open directly. Wire all five actions to the implemented domain operations, retaining cancel/return behavior, native selectors, ranking, hints and both editing profiles.
- [ ] T6.2: Render the supplied src/session/startup/rev-logo.ans as passive native styled cells on a matching black content area. Preserve its hash, RGB/Unicode artwork and buffer palettes; use a plain rev label for NO_COLOR or insufficient space and retain action identity during resize.
- [ ] T6.3: Verify per-file/root authority for Git, Pi, LSP, shell jobs, terminals, Org and global bookmarks/usage. A Workspace is not automatic cross-root AI context or service activation. Extend focused startup/command/modal/display cases and document the delivered entrypoints.

## Slice 7: Combined native and terminal acceptance

State: planned
Dependency: Slice 6 integrated
Requirements: PW01-PW22 together
Branch: `test/rev-projects-workspaces-07-acceptance`
PR: pending
Report: `ops/default/report/slices/rev-projects-workspaces/slice-7-acceptance.md`
Validation: the exact ordered Slice 7 commands in the plan/local configuration

- [ ] T7.1: Create examples/projects-workspaces-acceptance.rs and scripts/check-projects-workspaces.sh with the existing Rust/Bash surface and PTY helpers; keep domain/storage variants in native cases and real editor journeys representative.
- [ ] T7.2: Cover one bounded real journey per profile across startup, explicit targets, promotion, Project/Workspace selection, sessions, membership, conversion, duplicate/cancel and shutdown. Exercise banner fitting/NO_COLOR, tree variants and Unicode source anchors without multiplying the entire matrix through PTYs.
- [ ] T7.3: Probe ordinary and capacity-bound catalog/session work, responsiveness, retained allocations and idle scheduling; measure the approved profile rather than assuming it. Preserve source/configuration bytes, restore TTYs and reap only owned processes; deliver evidence and the report.

## Slice 8: Guidance and gate preparation

State: planned
Dependency: Slice 7 integrated
Requirements: PW23 and joint PW01-PW22 reconciliation
Branch: `docs/rev-projects-workspaces-08-guidance`
PR: pending
Report: `ops/default/report/slices/rev-projects-workspaces/slice-8-guidance.md`
Validation: the exact ordered Slice 8 commands in the plan/local configuration

- [ ] T8.1: Complete the Diataxis guidance with short CLI examples, a guided file/directory/Project/Workspace path, focused lifecycle how-tos, exact command/state/limit reference and a concise context explanation. Use small textual UI examples where helpful.
- [ ] T8.2: Keep Markdown as the source for Spore and reconcile CLI/navigation/interaction baselines with approved context ownership. Retain historical evidence and unrelated pending work; document no inactive startup.restore_session option as current functionality.
- [ ] T8.3: Deliver all reports and exact task/commit/PR evidence, finalize one coherent Unreleased outcome and prepare the clean integrated candidate for the frozen controller gate. After it passes, close only the configured documentation paths and record actual evidence.


## Final gate and closure

These tasks remain pending until actual evidence exists. Looper owns progress;
this document records canonical task/commit/PR/report links, not another execution
controller.

- [ ] Verify all eight canonical PRs integrated into dev and all reports delivered.
- [ ] Verify the new target, acceptance script, guide tools and isolated fixtures
  are admitted before the unchanged controller gate.
- [ ] Pass the frozen combined gate on the exact clean integrated candidate.
- [ ] Record candidate/receipt, commands, actual phase timings, resource and
  PTY observations, owned cleanup and evidence limits.
- [ ] Complete plan, tracker, specification/model and index within the thirteen
  configured closure paths; preserve the tested runtime and assertions.

Only bounded technical recovery and two scoped final-gate repair PRs are
delegated under the plan. P19 full startup-state restoration, other pending
features, main promotion, installation, releases and mirrors stay separate.
