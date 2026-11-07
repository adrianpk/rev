# Org Initial Visibility Delivery

Status: Approved
Date: 2026-11-07
Delivery set: rev-org-initial-visibility
Slice strategy: behavior-first
Approved: product owner requested specification, plan and Looper delivery
Planning baseline: `70c4e95bbd31a3550f5928ccaa1fbc1b8945ab23`
Specification: [Org Initial Visibility and Drawers](../spec/editor-org-initial-visibility.md)
Model: [Org Visibility Policy](../spec/editor-org-initial-visibility-model.md)
Tracker: [Org Initial Visibility Tracker](../tracker/rev-org-initial-visibility.md)
Ticket: [TKT-20261103111614](../ticket/ready/20261103111614-configure-org-drawer-visibility.md)
Concern: [Pending Editor Work](../README.md)
Active slice: Slice 1 - Initial visibility and drawers

## Outcome and slicing

Deliver the startup policy, document precedence and whole-view drawer controls
as one coherent vertical increment. Existing Org observation, visibility and
view owners already provide the boundaries; separate type, configuration or
documentation slices would leave parts of the same small interaction unfinished.

Keep showeverything as the compatibility default. Support overview, content,
showall and showeverything, the recognized per-document override, three
drawer-only controls and explicit startup reset. Preserve org-show-all and
existing cycling, singular drawer, source-navigation and AI result behavior.

## Prerequisites and authority

Native Org, current view folding, configuration, both editing profiles and
complete command descriptions are delivered. The primary checkout must be clean
dev synchronized with origin/dev before startup. No other delivery may own it.

The requested workflow authorizes preparation commit/push on dev, the recorded
slice worktree/branch and canonical PR, Looper-owned Rebase + Fast-forward merge
into dev, the unchanged selected integrated gate and documentation-only closure.
The slice PR does not wait for another owner approval. Main integration,
releases, public guide deployment and binary installation remain separate.
Use installed controller defaults and preserve consumed budgets on recovery.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Initial visibility and drawers | `feat/rev-org-initial-visibility-01-policy` | `feat(org): configure initial visibility and control all drawers` | `ops/default/report/slices/rev-org-initial-visibility/slice-1-policy.md` |

## Slice 1: Initial visibility and drawers

Dependency: approved preparation committed and pushed; sufficient owned build capacity.
Requirements: OIV01-OIV05 and the model's configuration/view lifetime contract.

- T1.1: Move the real source ticket from ready to in_progress. Implement native
  enum defaults/admission/merging/provenance and recognized structural metadata.
  Integrate one-time initialization, explicit-choice suppression, split and
  reload admission with current observer/view owners. Preserve current limits,
  source navigation and deliberate AI REQUEST initialization. Do not scan source
  during drawing or add another background worker.
- T1.2: Add the three drawer controls and startup reset with their exact names
  and descriptions. Keep one canonical action per invocation and preserve
  existing cycling/full reveal. Extend existing native visibility/reveal/fold
  cases and the shared command/profile tests, plus a configuration module named
  org_visibility. Cover all policy values, precedence, literal/malformed data,
  delayed results, runtime choices, peer views, reload, empty/read-only targets,
  Unicode/EOF and unchanged source state. Use meaningful parameterized cases;
  every selected test filter must execute tests.
- T1.3: Update complete configuration, command and binding references, the live
  command-description inventory and exact catalog counts without rewriting
  historical delivery claims. Update the separate Org User Guide and project
  Markdown into Spore. Use changelog-normalize for the completed Unreleased
  behavior. Introduce the report with task/commit/PR evidence and move the ticket
  to reviewing before gate preparation. Keep all lifecycle links aligned.

Use Conventional Commits without task suffixes. Record correspondence in the
report/tracker. Update the current description inventory for the four new
commands while retaining the previous 618 exact summaries and their evidence.

## Ordered slice checks

Prepend every Cargo command and the guide build with:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
Freeze these twelve commands in this order:

1. `cargo fmt --all -- --check`
2. `cargo clippy --locked --all-targets -- -D warnings`
3. `cargo test --locked --test org_editing visibility -- --test-threads=1`
4. `cargo test --locked --test org_editing fold -- --test-threads=1`
5. `cargo test --locked --test commands visibility -- --test-threads=1`
6. `cargo test --locked --test commands catalog_reference -- --test-threads=1`
7. `cargo test --locked --test commands description -- --test-threads=1`
8. `cargo test --locked --test display hint -- --test-threads=1`
9. `cargo test --locked --test configuration org_visibility -- --test-threads=1`
10. `cargo run --locked --quiet --example documentation-check`
11. `bash scripts/build-user-guide.sh`
12. `git diff --check`

The existing command/profile fixture supplies native M-x/colon invocation.
Extend native visibility_* cases so the selected filter covers startup and
whole-view commands together with current reveal behavior. Check the generated
Org REQUEST presentation through existing native fixtures; no live provider,
LSP, shell, export toolchain or new PTY framework is needed for this change.

## Integrated gate and cost

The controller runs the same ordered commands on exact integrated clean dev.
All-target Clippy protects changed public configuration and command enums.
Runtime selection protects the affected fold, source/reveal, profile, registry
and configuration boundaries rather than repeating unrelated historical gates.

Expect several minutes with warm artifacts plus the slice worktree's first
compile cost. Use one build job, disabled incremental output and no debug
information. Retain separate mutable target directories; build no release
binary. Verify free space before startup and do not alter other active builds
or remove source/uncertain files to create capacity.

## Gate preparation and closure

Keep the ticket reviewing and reports delivered for the integrated gate.
After it passes, solve the ticket with resolution fixed and real lifecycle
evidence. Complete the plan, tracker, specification and model and reconcile
the index. Only the exact configured closure documents may change after the
tested candidate. Include ready/in_progress/reviewing/solved source-ticket
paths before freezing the configuration. Do not change runtime, tests,
required assertions, guide inputs or the current command definitions in closure.
