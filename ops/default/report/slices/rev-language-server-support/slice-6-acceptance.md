# Slice 6: Joint source acceptance and guidance

Status: delivered
Delivery set: rev-language-server-support
Plan: [Optional LSP Delivery](../../../plan/rev-language-server-support.md)
Tracker: [Optional LSP Tracker](../../../tracker/rev-language-server-support.md)
Branch: `feat/rev-language-server-support-06-acceptance`
PR: [#208](https://forge.adrianpk.com/adrianpk/rev/pulls/208)

## Purpose

Establish LSP01-LSP06 and CMP01-CMP09 joint source acceptance with real installed
Rust semantics, controlled failures, the release executable and bounded terminal
journeys. Complete the user guidance and configuration/lifetime reference.

## Delivered Behavior

`scripts/check-lsp.sh` builds the normal release editor and
`examples/lsp-acceptance.rs`, verifies installed rust-analyzer/toolchain versions,
and batches source quality over every production Rust file, actual crate roots
and acceptance modules. The example requires real semantic completion of an
unsaved Rust field, accepts through the shared completion owner, undoes in one
step to unsaved text, preserves disk bytes and verifies owned-server cleanup.
The real child has separate HOME, XDG and Cargo directories, offline Cargo,
a dependency-free explicit Rust target, a 35-second semantic deadline and a
60-second outer timeout. A matching native name cannot satisfy Semantic source
identity. Initial empty results may use native fallback while the workspace loads.

A controlled stdio peer drives one continuous source journey: disabled/native,
enable/semantic, selected resolve with atomic same-document import/undo,
exclusion/native, restore/semantic, shared buffers/views, valid-empty/native,
EOF/native without respawn, explicit restart, healthy last-buffer retention,
disable and exit/reap. Retained idle connections and the driver each consume at
most one CPU tick over 350 ms and have no input polling deadline. All canonical
source/history/file assertions remain active through source changes.

The joint runner reuses the predecessor's native language, Org, external-grammar,
resource and PTY cases after source switching succeeds. A second release-editor
PTY checks semantic popup and resolved import/undo, exclusion/restore,
valid-empty/native fallback and disable/reap. Both PTYs retain eight-second
session and 1 MiB output limits, exact disk-source preservation and terminal
attribute, alternate-screen, bracketed-paste and cursor restoration assertions.
Only the owned temporary sources, stores, PTYs and children are cleaned up.

## Implementation Notes

The native PTY helper exposes its existing fixture operations to the joint
example without changing the original journey's limits or assertions. Real
server memory is sampled after accept/undo in the isolated semantic child; native/editor memory
and executable bytes are reported separately. The wrapper retains the normal
release profile and two build jobs. It requires the installed executable and
cached locked dependencies, without server installation or network access.

The first compile exposed acceptance-module visibility at the crate boundary;
exports now match the example's parent entrypoint. Source checking required
splitting terminal setup/shutdown and retained-idle observation into their
existing responsibilities. These driver changes preserve all observable
assertions, deadlines and resource limits. The Rust fixture declares its
`main.rs` target explicitly so Cargo metadata describes the actual source tree.

## Contracts Added or Changed

The optional-server how-to covers installed Rust activation, unsaved suggestions,
one-undo imports, per-buffer exceptions, explicit restart and another configured
language without a bundled grammar. The reference states sharing/root selection,
capability/edit admission, failure behavior and finite limits. The existing
completion guide, User Guide, full TOML sample/catalog and Unreleased entry link
to that behavior. Factory activation and configuration leaves are unchanged.

Native completion specification/model metadata now points to the focused joint
candidate evidence while preserving native candidate
`0f7cc36cb1082aa56954fa602294f47aac80e889` and receipt
`DAIRXONWTTLVG6RDIQQZKBSGXA`. Their completed native closure is not overwritten;
the controller still owns the LSP integrated full gate and final joint closure.

## Files of Interest

- `scripts/check-lsp.sh`, `examples/lsp-acceptance.rs`: repeatable normal-release joint entrypoint and complete source inventory.
- `examples/lsp_acceptance/real.rs`: isolated real semantic source, undo, disk and server resource proof.
- `examples/lsp_acceptance/controlled.rs`, `support.rs`: source switching, resolved edits, sharing and finite lifetime ownership.
- `examples/lsp_acceptance/pty.rs`, `examples/completion_acceptance/pty.rs`: shared terminal boundary and semantic/native release-editor journey.
- `docs/how-to/use-language-server.md`, `docs/reference/language-servers.md`: setup/recovery and exact protocol/lifetime contracts.
- `docs/reference/defaults.md`, `config.toml`: complete current setting behavior and sample.
- `ops/default/spec/editor-autocompletion.md`, `editor-autocompletion-model.md`: joint candidate evidence with historical native gate preserved.

## Validation

Successful development commands:

- `cargo build --locked --release --jobs 2 --bin rev --example lsp-acceptance --example source-check`: normal release build passed (3m 29s cold worktree).
- `cargo clippy --locked --example lsp-acceptance -- -D warnings`: passed.
- `cargo run --locked --quiet --example documentation-check`: 331 commands,
  211 configuration entries and 65 reachable pages passed.
- `cargo run --locked --quiet --example source-check -- --root examples/lsp-acceptance.rs --file examples/lsp_acceptance/mod.rs --file examples/lsp_acceptance/support.rs --file examples/lsp_acceptance/controlled.rs --file examples/lsp_acceptance/real.rs --file examples/lsp_acceptance/pty.rs --file examples/completion_acceptance/pty.rs`: safe Rust, parameters at most five, nesting at most three and complexity at most ten passed.

The release example was invoked with an owned temporary fixture, built editor
and canonical PTY wrapper under isolated HOME/XDG. All real/control/native and
both terminal journeys passed. Actual focused normal-release measurements:

| Observation | Result |
| --- | --- |
| Installed server | rust-analyzer 0.3.3089-standalone |
| Rust toolchain | rustc 1.98.0 (88d9e12ae 2026-08-18), Arch Linux rust 1:1.98.0-1 |
| Cargo | cargo 1.98.0 (797e8a9bc 2026-08-05), Arch Linux rust 1:1.98.0-1 |
| Release editor with integrated LSP | 18,901,808 bytes |
| Native predecessor release | 18,068,672 bytes; preserved native report/receipt |
| Delta / ratio to native predecessor | +833,136 bytes / 1.046109 |
| Installed unsaved semantic readiness | 1,492,364 microseconds |
| Separate real server peak RSS / CPU / threads | 32,020 kB / 6 ticks / 30 threads |
| Isolated semantic driver peak RSS | 13,440 kB; excludes separate server |
| Native acceptance peak RSS | 32,792 kB |
| Native / LSP release-editor PTY peak RSS | 14,676 / 13,152 kB |
| Both editor PTY idle samples | 350 ms silence, 0 CPU ticks |
| Native 40-peer observation request | 2,163 microseconds; 31 retained peer names |
| Native 1024-declaration / 128-event burst | 215,000 microseconds; longest event/frame 823 microseconds |
| Native burst idle | 0 CPU ticks over 350 ms, no idle deadline |
| Native PTY output | 44,192 bytes; both PTYs enforce the 1 MiB cap |

These observations separate editor client cost from the independently installed
server. The older grammar-only baseline and native full-gate measurements remain
in the [predecessor report](../rev-autocompletion/slice-8-acceptance.md);
its experiment ratio is not used as an integrated executable-size ceiling.

Final exact-HEAD validation follows the PR-reference commit through the controller
using precisely the configured order:

- `cargo fmt --all -- --check`
- `cargo test --locked --test lsp --test completion --test language_assets --test coordinates --test editing --test commands --test configuration --test recovery`
- `bash -n scripts/check-lsp.sh`
- `bash scripts/check-lsp.sh`
- `cargo run --locked --quiet --example documentation-check`
- `git diff --check`

T6.1 is commit `7c2ffc712178d15f498a08ba1eba4dfcd81f27d5`.
T6.2 is commit `2a6ca297349212a82876c8d40c96f2cd0129d274`, which introduces this report and the completed guidance with the measured joint
candidate evidence. Canonical integration and the integrated full gate remain
controller-owned.

## Risks and Follow-ups

The installed-server result covers the recorded toolchain and admitted profile;
resource observations vary with machine and server workload. The unchanged full
gate and final joint acceptance closure follow canonical Slice 6 integration.

## Integration

Canonical PR [#208](https://forge.adrianpk.com/adrianpk/rev/pulls/208) merged
through Rebase + Fast-forward into dev at
`c632d2fd42b2f1a5fdd5505002fb535655cc6790`. Controller receipt
`GUMCW2YHQ2HCMGI2WB4QGVKLST` passed all six configured commands, 539 tests,
all 724 production source files and both bounded terminal journeys at that clean
head. The controller recorded the merge in Acta. The integrated full gate and
final joint acceptance remain pending controller execution.

## Gate repair 1: Observed process-group shutdown

Repair branch: `fix/rev-language-server-gate-1`.
Exact PR title: `fix(process): wait for observed descendant shutdown`.
Repair PR: [#209](https://forge.adrianpk.com/adrianpk/rev/pulls/209).
Repair implementation: `90bdc572b65d850130c7d13df180dc11743ab8d7`.

The controller retained failed full-gate candidate
`dc8621820ab418c38995ac12065916ce175e78f1` and receipt
`4KDHCR4VZITQICSI75OSOIAQYA`. Formatting and all-target Clippy passed before
the library suite failed with `cancel: descendant still running` in
`org::io::process::tests::process_lifetime` (112 passed, one failed). The
isolated baseline test passed; the retained parallel run demonstrates the
timing-dependent failure rather than a deterministic single-test failure.

The signal utility acknowledges signal delivery, and waiting for the leader
acknowledges only that child. Cleanup previously marked the whole group stopped
at that point. The process owner now observes Linux process-group membership
and state before releasing cleanup. Only the owned group is signalled. Active
members keep cleanup pending; zombies/dead members count as stopped, consistent
with the existing descendant assertion, while the direct child is still reaped.

The signal helper and observation share one absolute 250 ms deadline, with the
existing 2 ms cleanup polling interval. The separate 250 ms stream-drain budget,
job duration/output limits and existing immediate descendant/direct-reap and
fixture deadline assertions are unchanged. Closed streams alone do not establish
termination. Stat reads are bounded to 4 KiB; disappearing tasks are expected,
while unknown observation failures remain cleanup errors. Parsing ignores comm
name bytes, including parentheses and truncated UTF-8. No idle polling is added.

`process_closed_output` performs 32 bounded cancellation cycles with eight
redirected descendants each and immediately asserts they are stopped after the
operation returns. `process_observation` requires a still-live group with closed
output to reach its observation deadline without being signalled, then checks
ordinary cleanup/reap and preservation of another group. Real executable names
cover comm delimiters and UTF-8 truncation. All fixtures own their processes.

Successful focused development checks:

- `cargo test --locked --lib org::io::process::tests -- --nocapture`: all five cases passed.
- `cargo test --locked --lib org::io::process::tests::process_observation -- --nocapture`: extended real-name/group-ownership case passed.
- `cargo test --locked --lib org::io::process::tests::process_lifetime -- --nocapture`: ten further repetitions passed with the original immediate assertions unchanged.
- `cargo clippy --locked --lib -- -D warnings`: passed.
- `cargo run --locked --quiet --example source-check -- --file src/org/io/process.rs --file src/org/io/process/group.rs --file src/org/io/process/streams.rs --file src/org/io/process/tests.rs`: safe Rust, parameters at most five, nesting at most three and complexity at most ten passed.

Final focused exact-head validation passed after the repair PR-reference follow-up
at `144f6c83c50a6ef7a8435187b40c89709d04dfbb`, receipt
`TGWCVK3EGLU72SON23W3EXZM3B`, using the last repair identity's pinned order:

- `cargo fmt --all -- --check`
- `cargo test --locked --lib org::io::process::tests`
- `cargo test --locked --test org_io --test org_source --test lsp`
- `git diff --check`

The controller integrated canonical [PR #209](https://forge.adrianpk.com/adrianpk/rev/pulls/209)
at the same validated commit. Canonical merge state and ancestry are verified;
the merge is recorded in Acta. Gate prerequisites were refreshed after integration:
installed tools, locked offline dependency metadata, cached grammar source inputs,
the exact committed minimal Rust fixture's offline metadata and owned PTY
allocation are ready. Temporary preparation files and descriptors were cleaned up.

The existing Slice 6 introduction and implementation/merge evidence remain
unchanged. The controller's unchanged full-gate rerun and final joint closure
remain pending. No server or full gate was started during preparation.

## Gate repair 2: Deterministic closed-output cancellation

Repair status: reviewing.
Branch: `fix/rev-language-server-gate-2`.
Exact title: `fix(process): make closed-output cancellation deterministic`.
Repair PR: pending.

The retained full gate at `189ea68179c74ff2a28cbe8b76bc6dfe43c86d1a`, receipt
`WHDZLX5DVWMTD5YGUTR7HGKMYZ`, passed formatting and all-target Clippy, then
failed the cancellation-error assertion in `process_closed_output` (114 library
tests passed, one failed). That assertion discarded the actual error. Adding
its text to the diagnostic did not reproduce the failure: the isolated case,
six library runs and eight concurrent isolated copies passed before correction.
The original hidden error remains unknown.

The fixture had a scheduling race: its background worker started collection's
one-second deadline before the test observed readiness and requested cancellation.
A delayed cancelling thread could lose to another error or the job deadline.
The fixture now owns the real launched shell first, waits for both output EOFs
and all eight descendants' readiness, and verifies the leader is still live.
It then posts cancellation and enters the same private `Owned::run` used by
`Program::run_input`. That entry point retains the existing collect-then-stop
sequence and cleanup-error precedence. The test exercises actual cancelled
process cleanup without duplicating that sequence in the fixture.

All 32 attempts retain the cancellation-result assertion, eight-descendant count,
immediate stopped-member assertions and one-second overall limit. Every attempt
also asserts that the direct child was reaped. The original active-worker
cancellation/timeout test remains intact. Job/output bounds, the shared 250 ms
signal/observation budget, stream-drain budget and group isolation are unchanged;
there is no added retry or wait after cancellation returns.

Successful focused development checks:

- `cargo test --locked --lib org::io::process::tests -- --nocapture`: five process regressions passed.
- `cargo clippy --locked --lib -- -D warnings`: passed.
- `cargo run --locked --quiet --example source-check -- --file src/org/io/process.rs --file src/org/io/process/group.rs --file src/org/io/process/streams.rs --file src/org/io/process/tests.rs`: safe Rust, parameters at most five, nesting at most three and complexity at most ten passed.

Eight concurrent copies of the corrected closed-output case also passed: 256
attempts and 2,048 descendants, with every original timing and immediate
shutdown assertion active. The owned test processes and temporary logs were
cleaned up after completion.

The covering authorization after repair 1 requires the following ordered
exact-head checks, recorded by the controller after the PR-reference follow-up:

- `cargo fmt --all -- --check`
- `cargo test --locked --lib`
- `cargo test --locked --test org_io --test org_source --test lsp`
- `git diff --check`

The existing Slice 6 introduction, delivered slice state and earlier receipts
remain unchanged. The controller owns repair integration and the unchanged
full-gate rerun; joint acceptance remains pending.
