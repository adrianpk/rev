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

Pi RPC traffic now retains the exact invocation identity and deadline through
request correlation, discovery admission, generation and independent cancellation.
Assistant accumulation, tool correlation/budgets, settlement and the unchanged
passive lifecycle catalog have separate handlers.

The Pi supervisor retains launch policy, pinned root, request and profile through
child cleanup. Root/version admission, command setup, history binding and
switching, discovery grants, generation, result publication and idle retirement
have explicit phases. Mailbox admission/retirement remains atomic; guards are
released before notifications and protocol waits.

The AI session owner separates capture admission, fresh observation encoding,
retained-worker compatibility, operation polling/completion and result publication.
The native source map owns its aggregate admission. Copied conflict context
separates native capture, role/saved-source projection and bounded encoding.
Literal command persistence separates name admission from publication.

The complete Slice 6 implementation remains in progress. The full quality check
has 151 blocking Clippy diagnostics, down from 196, and one source finding,
down from 34. All nine AI session files pass canonical scoped quality with no
blocking diagnostics. The remaining source finding is the existing public
retained-session signature; selected LSP, shell and terminal diagnostics remain.

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
- RPC correlation preserves every command/event, prompt disposition, model/root
  and session check, unsupported-UI/tool rejection, exact output JSON and byte
  handling. Eight turns, 32 calls, 4 MiB tool output and 2 MiB records are unchanged.
- Discovery grants retain their exact nonce/root identity, fingerprint, private
  exclusions and capped remaining timeout. Cancellation still sends clear-queue
  before abort under its independent five-second bound.
- Reset publication still precedes restored-session switching; history sync
  still precedes generated-result publication. Pending request selection and
  idle retirement remain under one mailbox lock, released before the next read.
- Version admission retains its five-second bound and one-second exit wait.
  Stream setup/draining, owned child stop, private resource lifetime and cleanup
  failure publication stay on the supervisor; no worker IO moved to input/render.
- AI submission preserves the admission/error order, exact envelope fields,
  model precedence, invocation identity and captured source authority. Retained
  root/model/grant rejection keeps the canceled child owned until cleanup.
- Native maps retain private/credential exclusions, 16 buffers, 1 MiB per source,
  the configured aggregate byte cap, link admission and NUL rejection. Conflict
  context retains the shared Git memory lease, complete encoding cap, exact
  role/object/saved/dirty observations and absence of edit/stage/continue rights.
- Polling retains acceptance, query, reset and epoch rules before presentation.
  Result acceptance retains focus, readonly/revision and association checks,
  checked window publication, single-undo behavior and authority consumption
  before cleanup. Saved commands retain collision/capacity checks and publish
  through the existing root owner.
- Remaining work includes selected LSP, shell, terminal and adapter ownership/
  conversion diagnostics and the public compatibility decision below.

## Contracts Added or Changed

No package, configuration, state or wire contract changed. SDK/plugin protocols,
source bounds, existing assertions, test names, timeouts and resource ceilings
are preserved.

## Files of Interest

- `src/process/capture.rs` and `src/process/cancellation.rs`: launch phases,
  positive process identity and bounded cleanup ownership.
- `src/ai/target.rs`: qualified linear-region capture.
- `src/ai/pi/resource.rs`: private extraction and resource identity.
- `src/ai/pi/history.rs` and `worker.rs`: binding requests, index publication,
  retained-session supervision and generation-bound idle ownership.
- `src/ai/pi/protocol.rs` and `protocol/generation.rs`: invocation-scoped RPC,
  bounded generation events, tool observations and settled output.
- `src/ai/pi.rs` and `io.rs`: explicit shared ownership, mailbox guard release,
  typed child identity and retained stream/cleanup boundaries.
- `src/session/ai.rs`, `ai/capture.rs` and `ai/envelope.rs`: captured invocation
  admission, serialization, retained reuse and polling/publication boundaries.
- `src/session/ai/context.rs` and `conflicts.rs`: admitted fresh native maps and
  complete conflict snapshots.
- `src/session/ai/results.rs` and `save.rs`: captured result acceptance/cleanup
  and literal command publication; borrowed fixture configuration is explicit.
- `tests/ai/discovery.rs`: explicit prefix-insertion position in the native map.

## Validation

Working-tree implementation evidence follows. Exact clean-HEAD Looper validation
and canonical PR preparation remain pending. The previous full native run
predates the Pi and AI session decompositions. Current focused checks cover the
changed owners; no previous result is final clean-HEAD evidence.

- Failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/ai.rs --scope src/ai --scope src/lsp.rs --scope src/lsp --scope src/process.rs --scope src/process --scope src/shell.rs --scope src/shell --scope src/terminal.rs --scope src/terminal --scope src/session/ai.rs --scope src/session/ai --scope src/session/lsp.rs --scope src/session/lsp --scope src/session/shell.rs --scope src/session/shell --scope src/session/terminals.rs --scope src/session/terminals`. All-target compilation succeeds; 151 selected Clippy diagnostics and one source finding remain across 117 selected files.
- Previously passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test ai --test lsp --test lsp_installation --test shell_commands --test integrated_terminal --test pi -- --test-threads=1`. The first implementation cohort passed all 213 ordinary native cases across the six suites, with three existing explicit upstream-acquisition cases ignored by default. A fresh final-head run remains required.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`. 106 reachable pages, 743 local links/anchors, 684 commands and 299 configuration entries.
- Previously passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/ai/pi`. All-target compilation, scoped Clippy and source policy pass for eight private Pi files, with zero blocking diagnostics and 847 diagnostics deferred outside this smaller selection.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test ai --test pi -- --test-threads=1`. All 26 AI and 15 Pi cases pass after the AI session decomposition, including native source maps, captured result ownership, directory admission, deadlines, cancellation, binding capacity and retained-history reuse.
- Previously passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib process:: -- --test-threads=1`. Ten native process cases passed for the unchanged first-cohort process implementation.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/session/ai.rs --scope src/session/ai`. All-target compilation, scoped Clippy and source policy pass for all nine AI session files, with zero blocking diagnostics and 844 selected-profile diagnostics deferred outside this selection.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib session::ai:: -- --test-threads=1`. Six native result-publication cases preserve literal bytes, readonly/revision guards, presentation controls and captured apply/discard ownership.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration conflicts::explanation:: -- --test-threads=1`. All four entrypoints pass, including the existing child-only RPC fixture; the native cases cover copied text/binary/dirty source facts, optional-AI admission and historical results without edit/stage/continue rights.
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
