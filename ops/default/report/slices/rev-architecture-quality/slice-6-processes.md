# Slice 6: External process workflows

Status: delivered
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-06-processes`
PR: #20 - https://forge.adrianpk.com/rev/rev/pulls/20

## Purpose

Resolve selected ownership, conversion, guard-lifetime and source-bound findings
in native AI/Pi, LSP, shell, terminal and process owners and their session adapters.

## Delivered Behavior

The process owner separates argument validation, fixed editor setup, repository
environment admission, child capture and finite group cleanup. Positive child
identities use checked conversion and shared cancellation ownership is explicit.

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

LSP queue and completion owners release mutex guards before notification and
owned-event boxing. Shared references are explicit; frozen launch policy and
owned response fields transfer directly to their consumers. Installer helpers
borrow observations, retain sorted unique payload leases and use checked process
and persistence conversions. Public npm/removal request shapes are preserved.

Shell workers and result publication borrow their captured observations without
moving IO or waiting onto input/rendering. Terminal copy owns physical-row/history
conversion; legacy mouse and panel conversions use existing admitted bounds.
Terminal error context borrows its source without changing restoration behavior.

The approved retained-session compatibility exception introduces `History` to
carry the existing optional state namespace and reset choice together. The
five-parameter `Operation::retained` forwards those exact values to the existing
launch profile; its session caller and native Pi fixture retain their behavior.
All 117 slice files now satisfy the source guards, down from 34 findings.
All-target compilation and selected Clippy pass with zero blocking diagnostics,
down from 196; 693 diagnostics remain deferred outside this slice.

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
  reset choice from the worker into the store. The public retained entrypoint
  groups its two existing history options under the approved `History` request.
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
- LSP queue message priority, atomic permit accounting and generation/revision
  checks remain unchanged. Completion responses transfer owned fields while
  retaining immutable-field admission, ordering and fallback. Stream shutdown
  still precedes thread joining, child reaping and payload-lease release.
- Installer launch paths retain sorted unique inode leases. Typed child IDs
  retain the unreaped-leader/NOWAIT rule, TERM/KILL and finite cleanup deadlines.
  Timestamp encoding follows the existing signed-range validation. Narrow,
  reasoned Clippy expectations preserve the public owned npm inheritance and
  removal settings requests; their private helpers borrow.
- Shell input is still released before child IO, complete stdout is retained
  before source publication, and cancellation/result ownership is unchanged.
- Terminal conversions retain 20,000 history lines, 200 panel rows, legacy mouse
  coordinates at most 223, 1 MiB copy text, 8 MiB aggregate projections and 32
  states. Physical grid rows and native scalar coordinates remain distinct.
  Restoration still attempts every acquired obligation and retains failed ones.

## Contracts Added or Changed

The maintainer-approved Slice 6 plan exception changes the public Rust signature
from `Operation::retained(policy, root, message, epoch, state, reset)` to
`Operation::retained(policy, root, message, epoch, History { state, reset })`.
`History` exposes the existing `Option<PathBuf>` state namespace and boolean
reset choice. Only the two existing callers in `src/session/ai/capture.rs` and
`tests/pi/sessions.rs` are adapted; their supplied values are unchanged.

Other package APIs, configuration, state and wire contracts are preserved, as
are SDK/plugin protocols, source bounds, existing assertions, test names,
timeouts and resource ceilings.

## Files of Interest

- `src/process/capture.rs` and `src/process/cancellation.rs`: launch phases,
  positive process identity and bounded cleanup ownership.
- `src/ai/target.rs`: qualified linear-region capture.
- `src/ai/pi/resource.rs`: private extraction and resource identity.
- `src/ai/pi/history.rs` and `worker.rs`: binding requests, index publication,
  retained-session supervision and generation-bound idle ownership.
- `src/ai/pi/protocol.rs` and `protocol/generation.rs`: invocation-scoped RPC,
  bounded generation events, tool observations and settled output.
- `src/ai/pi.rs` and `io.rs`: the approved `History` request, explicit shared
  ownership, mailbox guard release, typed child identity and retained
  stream/cleanup boundaries.
- `src/session/ai.rs`, `ai/capture.rs` and `ai/envelope.rs`: captured invocation
  admission, serialization, retained reuse and polling/publication boundaries.
- `src/session/ai/context.rs` and `conflicts.rs`: admitted fresh native maps and
  complete conflict snapshots.
- `src/session/ai/results.rs` and `save.rs`: captured result acceptance/cleanup
  and literal command publication; borrowed fixture configuration is explicit.
- `src/lsp/budget.rs`, `inbox.rs`, `completion.rs` and `documents.rs`: bounded
  ledger/queue guards, shared activation and completion response ownership.
- `src/lsp/connection.rs`, `io.rs` and `install/`: frozen policy and streams,
  payload leases, typed helper identity, timestamps and public request shapes.
- `src/shell.rs`, `src/session/shell.rs` and `src/terminal/`: borrowed worker/result
  observations and unchanged error/restoration context.
- `src/session/terminals/`: admitted physical-grid/copy/mouse conversions.
- `tests/ai/discovery.rs`: explicit prefix-insertion position in the native map.
- `tests/pi/sessions.rs`: the existing retained-session caller supplies its
  unchanged state path and reset choice through `History`.

## Validation

All four frozen commands passed through `looper validate` on clean published
HEAD `2064a704049fe66c1e44e68a67b2b6e72024649a`, validation
`UMFBE7KRBFCBKKOG2ZRU2BATLG`. Canonical PR #20 integrated that exact head into
`dev`. Historical focused checks are labeled below.

- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/ai.rs --scope src/ai --scope src/lsp.rs --scope src/lsp --scope src/process.rs --scope src/process --scope src/shell.rs --scope src/shell --scope src/terminal.rs --scope src/terminal --scope src/session/ai.rs --scope src/session/ai --scope src/session/lsp.rs --scope src/session/lsp --scope src/session/shell.rs --scope src/session/shell --scope src/session/terminals.rs --scope src/session/terminals`. All-target compilation and selected Clippy pass across 117 selected files, with zero blocking diagnostics and 693 selected-profile diagnostics deferred outside this slice. All selected source guards pass, including the five-parameter retained-session signature.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test ai --test lsp --test lsp_installation --test shell_commands --test integrated_terminal --test pi -- --test-threads=1`. All 213 native cases pass across the six suites: 26 AI, 49 integrated-terminal, 46 LSP, 39 installation, 15 Pi and 38 shell. The same three explicit upstream-acquisition cases remain ignored by default.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`. 106 reachable pages, 743 local links/anchors, 684 commands and 299 configuration entries.
- Previously passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib lsp:: -- --test-threads=1`. Five native permit, control-slot and snapshot-retirement cases pass, including concurrent retirement and failed-worker lease release.
- Previously passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib process:: -- --test-threads=1`. Ten native process cases passed for the unchanged first-cohort process implementation.
- Previously passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib session::ai:: -- --test-threads=1`. Six native result-publication cases preserve literal bytes, readonly/revision guards, presentation controls and captured apply/discard ownership.
- Previously passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration conflicts::explanation:: -- --test-threads=1`. All four entrypoints pass, including the existing child-only RPC fixture; the native cases cover copied text/binary/dirty source facts, optional-AI admission and historical results without edit/stage/continue rights.
- Passed: `git diff --check`.

The unchanged slice base `ebfe3f4798c6` reproduces the source-map failure before
that fixture correction. Baseline comparison restored all six implementation
files byte-for-byte before the Pi run; no primary or other worktree source was
changed. No assertion or runtime interaction behavior was changed to meet it.

## Risks and Follow-ups

The approved `History` exception is a source-level public Rust API adjustment;
callers must group the existing state/reset arguments. No source limit,
protocol, assertion, timeout or resource budget was changed to satisfy the
checks. The three explicit live upstream-acquisition tests retain their existing
default ignores; the native installation fixtures remain covered.

The controller owns merge, full delivery-set validation and successor startup.
