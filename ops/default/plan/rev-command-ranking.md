# Command Ranking Delivery

Status: Completed
Completed: 2026-11-06
Validated candidate: `fea4fffdfa3e66d8a7d7f1cc031a6637199424af`
Gate validation: `342LB2K3MYK2Z2JS5ESVI4WSNP`
Date: 2026-11-06
Delivery set: rev-command-ranking
Slice strategy: behavior-first
Approved: product owner; specification/model and the established preparation commits, three-slice plan, push and one autonomous Looper delivery through dev
Planning baseline: `fd71ebfe0249`
Specification: [Command Recency and Frequency](../spec/editor-command-ranking.md)
Model: [Command Usage Model](../spec/editor-command-ranking-model.md)
Tracker: [Command Ranking Tracker](../tracker/rev-command-ranking.md)
Concern: [Pending Work Index](../README.md)
Active slice: None; all three slices delivered and the delivery set is Completed

## Outcome and strategy

Deliver the approved selector ordering in one retained Looper run. Empty M-x
and modal colon queries begin with four distinct recent commands, then up to
four frequent commands without duplicates, followed by the alphabetical
remainder. Typed matching remains authoritative. A 256-invocation window gives
frequency a finite lifetime and persists as user-wide XDG state.

The behavior-first strategy delivers useful session ranking in Slice 1,
cross-session persistence in Slice 2, and joint acceptance and documentation in
Slice 3. Each slice stays compilable and passing. No fourth slice is delegated.

Retain existing Tab, Return, navigation, cancellation, focus and M-p/M-n history
contracts and configurable input, visible-row and distinct session-history
limits. Count explicit named invocations only; no arguments, paths or document
contents enter usage state. Keybinding hints retain their separate open ticket.

## Prerequisites and authority

Native vertical completion, both profiles, registered AI names, XDG roots,
bounded storage patterns and Markdown-derived Spore guidance are delivered
prerequisites. Rust/Cargo, Bash, existing PTY helpers and Spore are sufficient.
No dependency, package or runtime addition is planned.

The owner's instruction authorizes preparation commits on dev, the three
named slice worktrees and canonical PRs, controller-owned Rebase + Fast-forward
integration into dev, the selected final gate and documentation closure. Use
one retained controller as progress owner; do not wait for owner merges of
slice PRs. Main promotion, installation, release, tags and mirrors remain separate.

Native recovery may diagnose six technical incidents, at most two per family,
retry two verified environmental prerequisites and integrate at most two
final-gate repair PRs within this feature. Preserve the same run, frozen checks,
conversation, consumed budgets and source authority. Dependency changes, new
product decisions, weaker assertions and additional slices are not delegated.
Retain the installed defaults of 200 worker turns and five stalled turns.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Session ranking | `feat/rev-command-ranking-01-session` | `feat(commands): rank recent and frequent command candidates` | `ops/default/report/slices/rev-command-ranking/slice-1-session.md` |
| Slice 2 | Persistent usage | `feat/rev-command-ranking-02-persistence` | `feat(commands): persist bounded command usage across sessions` | `ops/default/report/slices/rev-command-ranking/slice-2-persistence.md` |
| Slice 3 | Acceptance and guidance | `test/rev-command-ranking-03-acceptance` | `test(commands): verify command ranking and document history` | `ops/default/report/slices/rev-command-ranking/slice-3-acceptance.md` |

## Slice 1: Session ranking

Dependency: approved committed preparation on clean dev.
Requirements: UI02, CMR01-CMR03 and the model's derived ranking values.

- T1.1: Deliver a bounded 256-event usage owner and deterministic four/four
  ranking, with canonical identities, aliases, admitted AI names, exact-name
  priority, configured match ranks and stable alphabetical remainder.
- T1.2: Observe accepted M-x/colon dispatch once through their shared path.
  Exclude bindings and internal calls. Capture a usage snapshot when a selector
  opens; preserve chosen identity through resize and deferred work, and keep
  existing session-history and Tab/Return/cancel behavior unchanged.
- T1.3: Extend commands/modal/display coverage with parameterized recency,
  frequency, ties, overlap, filtering and invocation cases. Document delivered
  session behavior and use the changelog skill for one Unreleased outcome.
  Do not advertise persistence before Slice 2.

## Slice 2: Persistent usage

Dependency: Slice 1 integrated.
Requirements: CMR04-CMR05 with UI02/CMR01-CMR03 and the complete model.

- T2.1: Implement bounded versioned TOML and the state store with existing
  dependencies. Enforce 64 KiB input/output, 128-byte names and 256 events;
  preserve invalid, oversized and unsupported-version records.
- T2.2: Wire executable startup and explicit isolated session ownership,
  keeping pure Input/Session construction free of user-file discovery.
  Coalesce bounded pending events, serialize publishers without waiting,
  rebase onto latest state and atomically replace it. Avoid duplicate retries
  after ambiguous outcomes. State errors must not block editing or shutdown.
- T2.3: Add `tests/command_usage.rs` and focused helpers under
  `tests/command_usage/` as needed. Verify real temporary-file boundaries,
  capacity, missing/invalid state, contention, concurrency, publication failures
  and startup/dispatch/shutdown wiring. Do not expose unrelated internals for
  tests. Update reference/defaults and the same Unreleased product outcome.

## Slice 3: Acceptance and guidance

Dependency: Slice 2 integrated. Requirements: UI02 and CMR01-CMR05 together.

- T3.1: Add `examples/command-ranking-acceptance.rs`, helpers under
  `examples/command_ranking_acceptance/` and `scripts/check-command-ranking.sh`.
  Reuse native frame/surface and PTY helpers. Cover the matrix natively, one
  input/render/restart journey per profile and one paired real-publisher check.
- T3.2: Complete command/prompt/state references, defaults wording and the
  search-and-commands User Guide chapter with a short candidate-list preview.
  Project Markdown into Spore. Reconcile UI02 and affected configuration/model
  descriptions before the gate without replacing historical gate evidence.
- T3.3: Record actual reports, immutable introductions and task/commit/PR
  mappings; prepare the clean candidate for the controller's frozen gate.
  Only the controller runs it. Close after it passes; no planning placeholders
  or claims of integrated acceptance from a focused check.

## Slice validation

For every Cargo command except formatting, use:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
The local configuration records that prefix literally.

| Slice | Required commands in order, with the Cargo prefix above |
| --- | --- |
| Slice 1 | `cargo fmt --all -- --check`; `cargo clippy --locked --lib --bin rev -- -D warnings`; `cargo test --locked --test commands --test modal --test display -- --test-threads=1`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |
| Slice 2 | `cargo fmt --all -- --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo test --locked --test command_usage --test commands --test configuration -- --test-threads=1`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |
| Slice 3 | `cargo fmt --all -- --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo test --locked --test command_usage --test commands --test modal -- --test-threads=1`; `bash -n scripts/check-command-ranking.sh`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |

The new integration target and script are implementation obligations. Retain
substantive old selector tests; change alphabetical expectations only where the
approved ranking changes them. Use empty usage for alphabetical fallback cases.
Never disable tests, weaken assertions or use actual user state to pass checks.

## Final gate

Preparation: canonical PRs #291, #292 and #293 are merged into dev and all three
slice reports are delivered. Rust/Cargo 1.98, formatter, Clippy, Bash, the
PTY utilities and Spore are available. Locked offline metadata admits every gate
target; both gate entrypoints pass Bash syntax checks. The guide inputs are
present and the ignored output directory is writable. No application build was
active in the primary checkout during this preparation.

The controller passed the unchanged full gate on immutable candidate
`fea4fffdfa3e66d8a7d7f1cc031a6637199424af`, receipt `342LB2K3MYK2Z2JS5ESVI4WSNP`.
The [Slice 3 closure](../report/slices/rev-command-ranking/slice-3-acceptance.md#delivery-set-closure) records the exact command,
503 passing tests, acceptance measurements, local guide build and evidence limits.
Only the eight approved closure Markdown paths may differ from that candidate.

Run once on the exact clean integrated candidate, in order. The export also
applies to nested Cargo invocations in acceptance and guide scripts.

```sh
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test commands --test command_usage --test modal --test configuration --test display -- --test-threads=1
bash scripts/check-command-ranking.sh
cargo run --locked --quiet --example documentation-check
bash scripts/build-user-guide.sh
git diff --check
```

These suites protect dispatch, modal routing, state storage, settings
compatibility and shared candidate presentation. Do not append historical Git,
terminal, LSP, Pi, Org, bookmark or tree acceptance scripts. Named AI usage
checks require no provider or network request.

Build the acceptance editor/driver once. Isolate HOME/XDG roots; allow at most
two live fixture children at once, 1 MiB output per child and 30 seconds per
journey, excluding builds. Use one restart journey per profile, not a PTY per
ranking variant. Verify cleanup and terminal restoration. Runtime window/batch
and byte bounds remain distinct; quiescent usage adds no periodic polling.

The previous Better File Tree gate passed 395 tests, with compilation and
configuration checks contributing most of its minutes-scale cost. This is
historical context, not a measurement for this set. Expect minutes with cached
builds and added cold cost per worktree. Record actual durations and diagnose
unexpected cost without silently expanding deadlines or exhaustive coverage.

## Closure

Use Conventional Commits without task-ID suffixes and exactly Summary, Changes,
Validation PR sections. The final Changes bullet links the report through its
immutable introduction. Validation lists successful checks in configured order.

After the final gate, only the plan, tracker, pending index, three delivered
reports and this specification/model may change: eight exact Markdown paths
pinned in local configuration. Runtime, tests, guides, samples, changelog and
baseline reconciliation must precede the tested candidate.

Completion requires three integrated canonical PRs, delivered reports,
UI02/CMR01-CMR05 and the model implemented, the exact candidate green under the
unchanged gate and completed artifacts. Publication and installation are separate.

## Completed delivery evidence

Canonical PRs [#291](https://forge.adrianpk.com/adrianpk/rev/pulls/291),
[#292](https://forge.adrianpk.com/adrianpk/rev/pulls/292) and
[#293](https://forge.adrianpk.com/adrianpk/rev/pulls/293) are merged into dev;
all three reports are delivered. UI02, CMR01-CMR05 and the paired model are
implemented, and the controller-owned frozen gate passed. The plan, tracker,
specification/model and pending-work entry are Completed. The separate
keybinding-hint ticket and main/release/publication boundaries retain their scope.
