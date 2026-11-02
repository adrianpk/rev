# Slice 7: Acceptance and user guidance

Status: drafting
Delivery set: rev-language-server-installation
Plan: `ops/default/plan/rev-language-server-installation.md`
Tracker: `ops/default/tracker/rev-language-server-installation.md`
Branch: `feat/rev-language-server-installation-07-acceptance`
PR: pending

## Purpose

Verify guided server management through the actual native and terminal owners,
then provide a complete learning path and matching configuration guidance.

## Delivered Behavior

One Rust/Bash management journey exercises both startup profiles. Native command
entry installs and verifies a managed Rust payload, updates it before retargeting,
removes the unreferenced prior version and preserves unsaved text, point, mode and
user settings. Actual Rust and Go servers complete fields present only in unsaved
text; each acceptance is one undo operation and leaves disk bytes unchanged.
A custom language with no grammar/catalog entry routes a controlled semantic
response, then switches to native-only without inventing grammar support.

The Linux PTYs exercise prompt cancellation, ordinary setup, verified acquisition
and read-only inventory while analysis remains disabled. They verify the saved
selection independently from transport readiness, preserve source bytes, observe
idle silence and restore terminal attributes, screen, cursor and paste modes.

## Implementation Notes

The fixture compresses the installed real Rust server once and returns controlled
release metadata and asset bytes through an owned curl peer. SHA-256 and the
native empty-root probe verify that payload before publication. This local fixture
is not upstream release provenance. Go uses the actual installed gopls with local
compiler selection and offline workspace analysis. The arbitrary-language protocol
peer proves routing, not a real language implementation's semantic quality.

The shared controlling-PTY reader/restoration boundary was extracted without
changing its eight-second deadline, 1 MiB cap, bounded channel or stop/reap owner.
Existing acceptance entry points use that same boundary. No production behavior,
runtime dependency or new editor command is introduced by this task.

## Contracts Added or Changed

The new script owns isolated HOME/XDG/configuration/workspace/cache roots and
finite child processes. Native cases have a 90-second outer guard; semantic
readiness is bounded to 45 seconds and management observations to 12 seconds.
Idle driver/PTY samples last 350 ms and allow at most one CPU tick. Initial
startup/idle must launch no child or acquisition; completed cases reap servers.
The source bounds remain safe Rust, five parameters, three nesting levels and
complexity at most ten. Final gate and closure remain controller-owned.

## Files of Interest

- `scripts/check-lsp-installation.sh`, `examples/lsp-installation-acceptance.rs`
  and its focused modules: isolated management, real semantics and resource evidence.
- `examples/completion_acceptance/terminal.rs`: shared existing PTY ownership,
  rendering observation and terminal restoration.

## Validation

Focused development checks passed:

- `cargo build --locked --jobs 2 --bin rev --example lsp-installation-acceptance --example source-check`
- `bash -n scripts/check-lsp-installation.sh examples/lsp_installation_acceptance/tools.sh`
- `target/debug/examples/source-check --root examples/lsp-installation-acceptance.rs --file examples/lsp_installation_acceptance/fixture.rs --file examples/lsp_installation_acceptance/native.rs --file examples/lsp_installation_acceptance/pty.rs --file examples/lsp_installation_acceptance/semantic.rs --file examples/lsp_installation_acceptance/support.rs --file examples/completion_acceptance/terminal.rs`

The built native/PTY modes passed for both profiles using owned temporary roots.
Non-modal/modal native runtime was 5,223/5,687 ms in the debug development runs;
compilation and compression are separate. The real installed servers were
rust-analyzer `0.3.3089-standalone` and gopls `v0.23.0`, with Go `1.27.1`.
The local Rust payload was 42,485,344 bytes, compressed to 15,162,611 bytes;
SHA-256 `75ddfdef2777449b4573fcda6c30ee022a33add55f08d5b1a90c83d646776d65`.
Debug driver peak RSS was at most 35,456 KiB; observed separate servers ranged
from 31,420 to 133,600 KiB. Idle samples used zero ticks. PTY output was
25,196/25,206 bytes. These are observed samples, not platform-wide limits.

Initial compilation exposed a shared-helper visibility issue. Source checks
required splitting new orchestration functions along operation boundaries.
The first semantic attempt correctly rejected a native fallback containing the
same field label; readiness now waits for both a semantic source and that field.
The first PTY attempt waited for wording clipped by an optional rust-src warning.
The driver now observes the visible ready result and separately asserts admitted
ready facts and the saved selected command. Source/selection assertions, eight-
second deadline, output cap and terminal restoration remain unchanged.

No new live upstream package acquisition is claimed here. Earlier slice reports
retain actual package-manager/probe evidence. Configured release validation and
the exact-candidate integrated gate have not run for this slice yet.

## Risks and Follow-ups

Guide and configuration narrative, final command sequence, PR references and
controller-owned integration/gate/closure remain pending in this active slice.
