# Slice 8: Acceptance and guidance

Status: delivered
Delivery set: rev-keyboard-macros
Plan: [Keyboard Macros Delivery](../../../plan/rev-keyboard-macros.md)
Tracker: [Keyboard Macros Tracker](../../../tracker/rev-keyboard-macros.md)
Branch: `test/rev-keyboard-macros-08-acceptance`
PR: [#326](https://forge.adrianpk.com/adrianpk/rev/pulls/326)
Integrated: c13f145e89efa5320961d6ee49caf408032b8392 on dev through canonical PR #326
Implementation: [050e6a6a616d](https://forge.adrianpk.com/adrianpk/rev/commit/050e6a6a616daed392ea87afec130d7f379bf875)

## Purpose

Deliver T8.1-T8.3: complete the native and real-terminal acceptance coverage for
KMA01-KMA09, teach the delivered workflows and prepare the unchanged controller
gate and documentation-only closure.

## Delivered Behavior

- A native beginner lesson records one line marker, replays it on the next two
  logical lines and undoes one iteration. Task guides cover counts, region lines,
  counters, queries, independent names/bindings, explicit saving and recovery.
- A complete macro reference connects every command/binding, preference, local
  map, schema field, owner and finite limit. An explanation distinguishes native
  key replay, accepted names, prompt roles, history and saved snapshot lifetimes.
- PTY journeys run the Cargo-built executable with real function-key and C-x
  input, both profiles, counted replay, undo, repeat keys, explicit save, restart,
  query cancellation, idle observation and terminal restoration.
- A native snapshot journey crosses ring eviction/deletion, a named binding,
  checked definition editing, reload and independent saved restart. A 2.2 MiB
  source probe checks bounded host work, typed cancellation, partial undo/redo
  and release of all automatic macro deadlines in both profiles.
- Query and step questions now display their existing native input labels.
  Terminal acceptance reproduced an empty question row despite an active native
  decision owner. The existing focus-message renderer now retains those labels,
  including return from manual editing; no answer or consent is auto-captured.

## Implementation Notes

The terminal fixture uses the already-linked rustix PTY, pidfd, poll and termios
primitives, the established native frame projection and Linux CPU/RSS helpers.
A child invocation of the same Rust test harness establishes its own controlling
terminal and execs CARGO_BIN_EXE_rev. Isolated XDG roots avoid user state. There is
no Python, added crate, external macro runtime or provider prerequisite.

Each child has an eight-second observation deadline and a cumulative 1 MiB
output ceiling. Checks verify exact saved source/library bytes, zero or bounded
idle CPU, silent terminal output, alternate-screen/bracketed-paste/cursor release,
restored termios fields and owned-child reaping. Failure cleanup addresses only
the fixture's retained pidfd. No existing editor or service is stopped.

The large-source probe observes one existing host turn before C-g. Cancellation
preserves partial source edits; each outer iteration remains a native undo unit.
The probe checks the original finite dispatch bound and no remaining native wake,
without increasing production limits or waiting for their exhaustion.

Markdown remains canonical. The chapter array, previous/next links, User Guide
index and Spore tree share one updated learning order. The required local build
checks and renders that projection under target; it does not publish a site.

## Contracts Added or Changed

- Native terminal and integrated lifecycle acceptance in the existing macros
  integration target, with shared isolated fixture and source/history owners.
- Visible existing query/step question labels; command semantics, native answer
  roles, identity, persistence, configuration defaults and limits are unchanged.
- One User Guide chapter, two task guides, a unified reference, an ownership
  explanation, indexes, publication navigation and the existing Unreleased entry.
- No approved requirement, gate selection, delivery identity or closure boundary
  is changed. Primary-dev integration and the final delivery gate remain owned
  by the controller.

## Files of Interest

- `tests/macros/pty.rs`: real executable input, restart, idle and restoration.
- `tests/macros/acceptance.rs`: large source, lifecycle and beginner lesson.
- `tests/macros/step_edit.rs`: rendered next-step assertion through native frames.
- `src/session/macros/queries.rs`, `step_edit.rs`, `src/session/save.rs`: existing
  native question labels retained by the focused message owner.
- `docs/tutorials/user-guide/21-keyboard-macros.md`: first successful recording.
- `docs/how-to/apply-keyboard-macros.md`, `save-keyboard-macros.md`: concrete tasks.
- `docs/reference/keyboard-macros.md`, `docs/explanation/keyboard-macro-ownership.md`:
  complete contract entry point and ownership rationale.
- `examples/documentation-check.rs`, `publication/userguide/index.sptree`:
  matching Markdown and Spore navigation.

## Validation

All ten frozen slice commands passed at clean final PR head
`c13f145e89efa5320961d6ee49caf408032b8392`, recorded once through the controller
validator after the PR-reference commit with receipt
`R27ZEACBE2XPSIL7VS7PWG5FVO`. Forgejo and Acta confirm canonical integration
through PR #326. This remains pre-integration slice evidence. The controller's
later exact integrated-dev gate is recorded in the delivery-set closure below.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check` — pass.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings` — pass.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- --test-threads=1` — pass, all 110 macro integration cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1` — pass, 52 selected command cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test editing -- --test-threads=1` — pass, 114 editing cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal -- prefix_arguments repeat count operator navigation --test-threads=1` — pass, 45 selected modal cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test completion -- manual_edit cancel stale --test-threads=1` — pass, 4 selected completion cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` — pass, 680 commands, 295 configuration entries, 99 pages and 681 links/anchors.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh` — pass, 99 projected pages, strict link checks and local HTML rendering.
- `git diff --check` — pass.

Additional focused development checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib -- session::macros::tests::step_edit queries_clock queries_budget --test-threads=1` — pass, 6 native clock, shared-budget, source/layout and stale-target cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --example documentation-check -- --test-threads=1` — pass, 7 catalog, link, reachability and navigation cases.

The successful focused terminal/source run observed 4 ms host turns on 2,228,224
source bytes and C-g handling in 69–113 microseconds. Three executable sessions
used 12,380–64,572 terminal bytes, restored terminal attributes/modes and reaped
successfully. Four 350 ms idle epochs emitted no output and consumed zero child
CPU ticks; observed peak RSS was 33,620–34,448 KiB. These are local observations,
not hard realtime or total-process memory guarantees.

All eight slice reports are delivered, their integration commits are ancestors
of dev, and each report matches its actual ordered receipt. The integrated-dev
gate passed as recorded below. Closure retains the plan's thirteen-file Markdown
boundary and cannot change runtime, tests, guide inputs or requirements.

| Requirement | Native acceptance retained or added |
| --- | --- |
| KMA01 | recording/runner fixtures, terminal function-key/C-x replay and tutorial. |
| KMA02 | prefix cases, region anchor mutation/wrapping fixtures and counted terminal replay. |
| KMA03 | actual saved restart, independent lifecycle, concurrent records and same-record/busy-lock refusal. |
| KMA04 | profile entry, source invalidation, iteration history, fresh consent and native terminal restoration. |
| KMA05 | accepted names/prefixes/prompt roles, changed ranking/bindings and provider-independent completion. |
| KMA06 | ring/append/counter fixtures, temporary repeat keys and independent snapshots. |
| KMA07 | query choices, manual nesting/resume/abort, visible questions and silent native user waits. |
| KMA08 | invalid/stale/cancelled drafts, key/region helpers, all step/menu controls and snapshot preservation. |
| KMA09 | finite admission fixtures, typed stops, 2.2 MiB cancellation/history probe and terminal/native idle release. |

## Risks and Follow-ups

The controller passed the integrated-dev gate; this documentation-only closure
records that evidence. No main promotion, tag, release, mirror or site publication,
binary install or global configuration change is performed. PTY evidence is for
the local Linux terminal boundary and does not prove every terminal emulator or provider. Native
atomic primitives retain their documented cooperative rather than hard realtime
boundary. Saved definitions remain valuable user content; explicit backup and
conflict-aware saving apply after this delivery.

## Delivery-set closure

Completed: 2026-11-08. All eight canonical PRs (#319-#326) are merged into dev
and their reports are delivered. The controller passed the unchanged selected
integrated gate on clean immutable candidate
`0b67a3659b4ec63e5188b062e255c2f9b19900c1`, receipt
`CAGQYNBGGZFQHTATRQRSHDE4K2`. Its supplied evidence and log bind the following
exact successful command to that candidate:

```sh
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test editing -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal -- prefix_arguments repeat count operator navigation --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test completion -- manual_edit cancel stale --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh && git diff --check
```

All five selected native targets passed with no failed or ignored tests:

| Target | Passed | Logged test time |
| --- | --- | --- |
| macros | 110 | 62.32s |
| commands | 52 | 195.09s |
| editing | 114 | 45.50s |
| modal | 45 | 52.84s |
| completion | 4 | 3.26s |
| Total | 325 | Individual suite timings above |

Formatting, strict lib/bin Clippy and whitespace checks passed. These are logged
suite timings; no aggregate gate duration is inferred. The full macro target
retains native source/history, input/command/prompt capture, query/region,
ring/counter, named/bound snapshot, persistent-library, concurrency and definition
editing acceptance. It also runs the real executable PTY, restart, large-source
cancellation, idle and terminal-restoration journeys mapped to KMA01-KMA09 above.
The earlier focused timing and RSS observations remain development evidence,
rather than measurements from this controller run.

Catalog validation passed 680 command names and 295 configuration entries.
Documentation passed 99 reachable pages and 681 local links/anchors. The strict
Spore guide build projected and rendered 99 Markdown pages into the existing
ignored local output. This establishes local guide validation; it does not
claim deployed publication or every physical terminal's rendering.

The tests retain their isolated fixture owners. Closure inspection found no
selected gate runner or owned macro PTY child still active. No process was
stopped and no fixture or persistent service was started during closure.
Existing interactive editor processes were preserved.

Immutable report introductions:

- [Slice 1](https://forge.adrianpk.com/adrianpk/rev/src/commit/ee097b5e41c321f83c3a0af1b0013ddeb1d7f2d2/ops/default/report/slices/rev-keyboard-macros/slice-1-recording.md), canonical PR #319.
- [Slice 2](https://forge.adrianpk.com/adrianpk/rev/src/commit/838968c735dd6903f9632e96095aa0acb0667af9/ops/default/report/slices/rev-keyboard-macros/slice-2-capture.md), canonical PR #320.
- [Slice 3](https://forge.adrianpk.com/adrianpk/rev/src/commit/68ec75071860e531f96046b4b99b63cbec8541f9/ops/default/report/slices/rev-keyboard-macros/slice-3-ring.md), canonical PR #321.
- [Slice 4](https://forge.adrianpk.com/adrianpk/rev/src/commit/f421903f80a606c529190e45cd51ccdecbbabdc6/ops/default/report/slices/rev-keyboard-macros/slice-4-queries.md), canonical PR #322.
- [Slice 5](https://forge.adrianpk.com/adrianpk/rev/src/commit/c504fcb0653d76cc6553796cc0c16b1716ed5a7b/ops/default/report/slices/rev-keyboard-macros/slice-5-names.md), canonical PR #323.
- [Slice 6](https://forge.adrianpk.com/adrianpk/rev/src/commit/5871093638f7093b496cfd81d3cce2fc1d84f2c5/ops/default/report/slices/rev-keyboard-macros/slice-6-library.md), canonical PR #324.
- [Slice 7](https://forge.adrianpk.com/adrianpk/rev/src/commit/17dcf0772016131104782240f155684e76cf25f7/ops/default/report/slices/rev-keyboard-macros/slice-7-editing.md), canonical PR #325.
- [Slice 8](https://forge.adrianpk.com/adrianpk/rev/src/commit/050e6a6a616daed392ea87afec130d7f379bf875/ops/default/report/slices/rev-keyboard-macros/slice-8-acceptance.md), canonical PR #326.

KMA01-KMA09, the model, all 24 tracker tasks and the eight recorded slices are
delivered. Specification/model, plan and tracker are Completed, and the index
entry is reconciled. Only the thirteen configured Markdown closure paths differ
from the tested candidate. Requirements, scenarios, model data contracts, runtime,
tests, guide inputs, limits and every required gate assertion retain their tested
content. Main promotion, tags, releases, binary installation, guide deployment,
mirror publication and successor startup were not performed.
