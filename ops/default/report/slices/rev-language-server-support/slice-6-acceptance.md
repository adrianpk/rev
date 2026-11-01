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
