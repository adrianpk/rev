# Slice 7: Acceptance and user guidance

Status: reviewing
Delivery set: rev-language-server-installation
Plan: `ops/default/plan/rev-language-server-installation.md`
Tracker: `ops/default/tracker/rev-language-server-installation.md`
Branch: `feat/rev-language-server-installation-07-acceptance`
PR: pending

## Purpose

Verify guided server management through the actual native and terminal owners,
and provide a complete learning path and matching configuration guidance.

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

The User Guide now leads through optional setup, managed acquisition, unsaved
completion, per-language choice and lifecycle. Its Markdown previous/next links
and Spore navigation include the new chapter. A complete standalone configuration
example includes installed Go, a custom language, native-only preferences and
literal environment overrides, while the default sample retains its defaults.

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

The documentation checker loads that example through the real configuration
owner and checks its admitted choices, custom suffix and local toolchain policy.
Live command/configuration inventories remain 417/219. The existing Markdown
projection carries the chapter and downloadable example into the local Spore
build; no public deployment or configuration migration is involved.

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
- User Guide chapter 12, how-to/reference, full configuration examples, shared
  command/configuration documentation checker and Spore navigation: one learning
  path with task details and exact reference contracts.

## Validation

Focused development checks passed:

- `cargo build --locked --jobs 2 --bin rev --example lsp-installation-acceptance --example source-check`
- `bash -n scripts/check-lsp-installation.sh examples/lsp_installation_acceptance/tools.sh`
- `cargo check --locked --examples`
- `cargo clippy --locked --example lsp-installation-acceptance -- -D warnings`
- `cargo run --locked --quiet --example documentation-check`
- `cargo test --locked --example documentation-check -- --test-threads=1`
- `target/debug/examples/source-check --source examples/documentation_check/catalogs.rs`
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

Documentation validation passed at 68 reachable pages, 413 links/anchors and
417/219 inventory entries; all seven documentation regression cases passed.
T7.1/report-introduction commit: `2dd2314c1bf4c3ee71b6a6c008e7c217465a5d89`.

No new live upstream package acquisition is claimed here. Earlier integrated
[Slice 4](slice-4-go-web.md) and [Slice 5](slice-5-catalog-recipes.md) reports retain
these separately executed upstream observations; they are not reclassified as
current-host versions or semantic evidence for every package:

| Historical actual acquisition | Recorded version/runtime | Recorded result |
| --- | --- | --- |
| gopls | `v0.24.0`, Go `1.27.0-X:nodwarf5` | Exact module/build provenance, probe and unsaved semantic field/undo/reap passed. |
| TypeScript server/compiler | Both `6.0.3`, Node `26.8.1`, npm `12.0.2` | Exact package lock and transport passed. |
| Web package | `4.10.0` | One package lock and three launch probes passed. |
| Marksman | `2026-02-08` | Official native digest/version/transport passed. |
| Ruby | ruby-lsp `0.26.11`, Bundler `4.0.23`, Ruby `3.4.10` | Isolated frozen lock and transport passed. |
| Python | Base server `1.15.0`, Python `3.14.7` | Fixed final-path venv/wheel report and transport passed. |
| OLS | Revision `9b9711592bed66266311c361dd7a307ecc87025b` | Failed/incomplete: the recorded installed Odin lacks `core:odin/tool-format`; no executable/probe/semantics claimed. |

T7.2 commit: `91608bcd5e60e5f16f6ed08f4aa74f72ae1e3195`.
T7.3 admission checks confirm tickets 1–6 are solved and this final implementation
ticket is reviewing. Admission filled Slice 2's missing resolution/closure fields
from its recorded closure commit `e70e5e68c840959a83c9d3c74c68f23e6fa2715c`
and verified merged PR #223; Slice 3's existing start timestamp now uses the
canonical `started_at` field. No timestamp or closure is inferred from file age.
Implementation/guidance are complete; the final ticket's solved transition
follows canonical merge. No ticket or set closure is inferred from local checks.
The first configured batch at `8cd7b6ef855bd41bc5291390d03fc37353ef193b`,
receipt `H5TKDVVZ7IRSDTO6PCE2NEBOZO`, passed format and failed Clippy on two
redundant borrows in the new PTY cancellation fixture. Removing those borrows
preserves behavior and assertions; focused example Clippy passed. Fresh
seven-command validation follows; the combined integrated gate remains pending
until controller-owned integration.

## Risks and Follow-ups

Configured validation and PR references remain pending. The controller owns
integration, this final ticket's post-merge closure, the unchanged integrated
gate and set closure. The arbitrary-language case uses a controlled peer; real
Rust/Go semantics are separate. Recorded OLS acquisition requires a compatible
external compiler. Unknown caches remain untouched. No gate result is claimed.
