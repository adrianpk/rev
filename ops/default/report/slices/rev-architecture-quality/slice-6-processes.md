# Slice 6: External process workflows

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-06-processes`
PR: pending

## Purpose

Resolve selected ownership, conversion, guard-lifetime and source-bound findings
in native AI/Pi, LSP, shell, terminal and process owners and their session adapters.

## Delivered Behavior

The first implementation cohort separates process argument validation, fixed
editor setup, repository environment admission, child capture and finite group
cleanup. Positive child identities use checked conversion and shared cancellation
ownership is explicit.

AI source selection has one linear-region admission step. Pi resource extraction
separates ancestor/namespace admission from resource creation and file inspection.
Pi history separates index encoding and publication, exclusive binding selection,
retired-file cleanup, initial session-file admission and identity observation.

The AI source-map fixture now explicitly moves to the beginning of a restored
buffer before its second prefix insertion. The unchanged slice base reproduced
its previous failure: switching buffers retains the previous point. All original
Unicode text, revision, completeness, dirty-state and disk assertions remain.

The complete Slice 6 implementation remains in progress. The full quality check
has 186 blocking Clippy diagnostics, down from 196, and 21 source findings, down
from 34. The completed 11-file cohort passes its scoped quality check.

## Implementation Notes

- Process launch retains the original admission/error order, exact argv, fixed
  editor drivers, environment clearing and private-index checks. Cleanup retains
  the unreaped leader, TERM/KILL sequence, 500 ms grace, two-second deadline,
  stream draining and retained failure ownership.
- Pi extraction preserves private namespace permissions, ordinary-file and
  inode/owner checks, complete resource bytes and guarded cleanup.
- Pi history retains lock order, exact serialized fields, compare-before-publish
  rechecks, sync order, replacement-before-retirement, first-creation race
  handling, 16 bindings, 64 KiB index and 32 MiB session limits.
- A private borrowed `Selection` carries the exact binding key, candidate path and
  reset choice from the worker into the store. Package APIs remain unchanged.
- Remaining work includes Pi protocol/worker and AI session source decomposition,
  plus selected LSP, shell, terminal and mailbox ownership/conversion diagnostics.

## Contracts Added or Changed

No package, configuration, state or wire contract changed. SDK/plugin protocols,
source bounds, existing assertions, test names, timeouts and resource ceilings
are preserved.

## Files of Interest

- `src/process/capture.rs` and `src/process/cancellation.rs`: launch phases,
  positive process identity and bounded cleanup ownership.
- `src/ai/target.rs`: qualified linear-region capture.
- `src/ai/pi/resource.rs`: private extraction and resource identity.
- `src/ai/pi/history.rs` and `worker.rs`: binding requests, index publication and
  session identity; broader worker decomposition remains pending.
- `tests/ai/discovery.rs`: explicit prefix-insertion position in the native map.

## Validation

Working-tree implementation evidence follows. Exact clean-HEAD Looper validation
and canonical PR preparation remain pending.

- Failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/ai.rs --scope src/ai --scope src/lsp.rs --scope src/lsp --scope src/process.rs --scope src/process --scope src/shell.rs --scope src/shell --scope src/terminal.rs --scope src/terminal --scope src/session/ai.rs --scope src/session/ai --scope src/session/lsp.rs --scope src/session/lsp --scope src/session/shell.rs --scope src/session/shell --scope src/session/terminals.rs --scope src/session/terminals`. All-target compilation succeeds; 186 selected Clippy diagnostics and 21 source findings remain across 115 selected files.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test ai --test lsp --test lsp_installation --test shell_commands --test integrated_terminal --test pi -- --test-threads=1`. All 213 ordinary native cases pass across the six suites; three existing explicit upstream-acquisition cases remain ignored by default.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`. 106 reachable pages, 743 local links/anchors, 684 commands and 299 configuration entries.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/process.rs --scope src/process --scope src/ai/target.rs --scope src/ai/pi/history.rs --scope src/ai/pi/resource.rs`. All-target compilation, scoped Clippy and source policy pass for 11 files, with zero blocking diagnostics and 879 diagnostics deferred outside this smaller cohort.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib process:: -- --test-threads=1`. Ten native process cases pass.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test pi -- --test-threads=1`. All 15 Pi cases pass, including private history, capacity, concurrency, cancellation and directory admission.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test ai -- discovery::native_source_map --exact --test-threads=1`. The explicit native motion corrects the independently reproduced baseline fixture failure.
- Passed: `git diff --check`.

The unchanged slice base `ebfe3f4798c6` reproduces the source-map failure before
that fixture correction. Baseline comparison restored all six implementation
files byte-for-byte before the Pi run; no primary or other worktree source was
changed. No assertion or runtime interaction behavior was changed to meet it.

## Risks and Follow-ups

The existing public `Operation::retained(policy, root, message, epoch, state,
reset)` has six parameters. The frozen five-parameter guard and approved package
API compatibility requirement conflict at that signature. A bounded proposal is
`Operation::retained(policy, root, message, epoch, History { state, reset })`,
with a public history request and adaptations only in `src/ai/pi.rs`,
`src/session/ai/capture.rs` and `tests/pi/sessions.rs`. These are its two existing
callers. This proposal is not applied; an explicit compatibility decision is
required before changing that public signature. Other private slice corrections
remain authorized and pending.

The controller owns merge, full delivery-set validation and successor startup.
