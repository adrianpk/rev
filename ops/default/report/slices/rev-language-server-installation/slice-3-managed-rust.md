# Slice 3: Managed Rust installation

Status: delivered
Delivery set: rev-language-server-installation
Plan: [Installation Delivery](../../../plan/rev-language-server-installation.md)
Tracker: [Installation Tracker](../../../tracker/rev-language-server-installation.md)
Branch: `feat/rev-language-server-installation-03-managed-rust`
PR: `#224`

## Purpose

Acquire, verify and explicitly select a managed Rust server through the existing
native command surface, keeping installation facts, user choices and continuous
analysis separate.

## Delivered Behavior

- `lsp-install-server` offers official Rust acquisition on GNU/Linux x86-64 and
  reuse of a ready installation. `lsp-setup` offers `install:rust-analyzer` for
  Rust; setup and selection also accept compatible `ready:<id>` records.
- Explicit acquisition resolves one exact stable release, checks the immutable
  official asset URL/length and release SHA-256, unpacks one executable and
  admits it through the native initialize/shutdown/exit protocol in an owned
  empty root. No user document enters that probe.
- Unique final payload directories and schema-1 installation facts use the
  existing XDG data root. Ready facts precede the conflict-checked user-root
  transaction. A settings conflict leaves the server ready and unselected;
  failed acquisition/probes retain incomplete candidates and earlier ready data.
- Selection preserves known associations, inherited options/settings/environment,
  custom preset-named profiles, runtime mode, exclusions and current editing
  state. Installation while LSP is disabled does not enable continuous analysis.
- The read-only inventory shows managed origin, concrete version, readiness,
  selected profiles and job phase. Viewing it never resolves remote versions or
  launches helpers. Corrupt facts block management without blocking external
  registration or ordinary editing.

## Implementation Notes

The manager reuses the native `Job`/`Cancellation` owner. No idle worker or timer
is introduced. One data-directory OS lock excludes managers across processes and
state roots; the existing state mutation lock also covers settings publication
without recursive acquisition. The writer preserves selected-root symlinks,
comments/includes and complete captured-source conflict checks.

Optional curl/gzip/SHA-256 helpers use structured argv and owned working roots,
nonblocking sockets, finite deadlines and transfer/storage budgets. Process-group
supervision uses the approved, already locked rustix 1.1.5 safe process APIs.
`waitid` with `NOWAIT` retains the leader until every group signal is finished;
cleanup reuses the existing bounded Linux process-group observation. Parent
copies of child descriptors are dropped immediately after spawn so EOF remains
an actual termination observation. Cleanup and committed-publication durability
uncertainty remain explicit outcomes. No first-party unsafe code is added.

The native asset ceiling is 256 MiB, candidate payload 1 GiB, disposable cache
2 GiB, registry 32 records/2 MiB, captured helper output 1 MiB and retained stderr
64 KiB. The total operation deadline is 900 seconds, with an initial two-second
stop grace and bounded forced stop/reap. Tree accounting also bounds entries and
depth without following escaping links. Cache cleanup touches only the uniquely
created operation directory and checks its identity first.

## Contracts Added or Changed

- Schema-1 installation facts contain ID, recipe/version/target, relative
  payload, source, SHA-256 evidence, runtime identity, relative launch profiles,
  status and epoch-second creation/verification timestamps. They never override
  `lsp.preferred`, `lsp.servers` or file associations implicitly.
- Invalid schema, duplicates, escaping/absolute paths, oversized data and
  unverified ready entries block management. Preparing data never resumes or
  becomes selectable on startup.
- `lsp-install-server` joins the shared command registry. Live catalogs contain
  414 commands; configuration remains 219 entries. Ready selection performs
  another admission under the management locks and starts no acquisition/probe.
- Rust toolchain/source availability is reported separately. The operation
  never installs or upgrades a compiler. A release-channel checksum establishes
  artifact integrity, not independent publisher identity or semantic quality.

## Files of Interest

- `src/lsp/install/record.rs`, `storage.rs`, `tree.rs`: bounded facts, final paths,
  atomic publication and containment/accounting.
- `src/lsp/install/rust.rs`, `helper.rs`, `process.rs`, `probe.rs`: exact release
  acquisition and owned bounded helpers using the existing frame/protocol owners.
- `src/lsp/install/worker.rs`, `progress.rs`, `src/session/lsp/managed.rs`:
  native worker, phase observation, inventory and selection orchestration.
- `src/config/mutation.rs`, `edit.rs`: shared lock guard and restricted writer.
- `tests/lsp_installation/storage.rs`, `acquisition.rs`, `managed.rs`:
  persistence/process boundaries and both-profile native journeys.
- Command/binding catalogs, language-server reference/how-to and Unreleased
  describe the working command, prerequisites, storage and recovery choices.

## Validation

Focused development checks passed:

- `cargo check --locked`
- `cargo test --locked --test lsp_installation storage:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation settings:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation acquisition:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation managed:: -- --test-threads=1`
- `cargo test --locked --test commands palette::catalog_reference -- --exact`
- `cargo run --locked --quiet --example documentation-check`
- `cargo run --locked --quiet --example source-check -- --source src/session/lsp/managed.rs --source src/session/lsp/setup.rs --source src/lsp/catalog/inventory.rs --source src/lsp/install/progress.rs --source src/lsp/install/worker.rs`
- `git diff --check`

Storage checks cover admission, source conflicts, cross-process locking with
separate state roots, symlink escape, capacity and cancellation ownership.
Acquisition uses an owned curl fixture with real gzip/SHA-256 tools and a
controlled protocol payload. It covers missing tools, corrupt digest/gzip,
failed probe, missing integrity evidence, oversized asset, output overflow,
prior-ready preservation, empty-root exchange and forced cancellation of a
TERM-ignoring leader/descendant group. Native tests run both actual command-entry
profiles with owned XDG roots and verify selection conflicts, editing preservation,
ready reuse without helpers and corrupt-registry isolation. These fixtures do
not establish upstream Rust semantic behavior.

The first helper journey reached its finite deadline while parent descriptors
remained open; dropping those copies corrected EOF ownership. Native cancellation
checks were aligned with the approved C-g/both-profile and modal-Escape contract.
Focused checks then passed without weakening production or validation limits.
A temporary-storage quota interrupted command submission before native edits;
capacity was restored and fresh owned writes passed before continuation.

Pre-PR controller validation passed at
`ee39a681179a7c3ecf6316d20004c900f1b0e529`, receipt
`WJ6BNUUPPJYWBZTRYTZD2YCSCP`:

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo test --locked --test lsp_installation -- --test-threads=1`: passed.
- `git diff --check`: passed.

Canonical PR: https://forge.adrianpk.com/adrianpk/rev/pulls/224
Final controller receipt `IKLQJ3752CS64MESVHEZXAIP5A` passed the same four
commands for `11bc8142ffe1b0371b050cea91a1ea06df96d19f`. Canonical PR #224
merged that exact commit into dev. Slice 3 of 7 completed.
The combined delivery-set gate remains with the controller after all slices.

## Risks and Follow-ups

Real upstream Rust/Go semantic evidence, remaining recipes and lifecycle acceptance
remain assigned to their approved slices. Optional tools run with user permissions,
not in a security sandbox. Cleanup or durability failures are reported explicitly;
invalid facts are preserved for diagnosis rather than silently repaired.
