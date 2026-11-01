# Slice 5: Shared lifecycle and real Rust interoperability

Status: delivered
Delivery set: rev-language-server-support
Plan: [Optional LSP Delivery](../../../plan/rev-language-server-support.md)
Tracker: [Optional LSP Tracker](../../../tracker/rev-language-server-support.md)
Branch: `feat/rev-language-server-support-05-lifetime`
PR: [#207](https://forge.adrianpk.com/adrianpk/rev/pulls/207)

## Purpose

Complete LSP01 and LSP04-LSP06 shared lifetime ownership and bounded LSP02
failure handling, including actual installed Rust-server interoperability.

## Delivered Behavior

`lsp-restart` retires the selected eligible root's connection and document
identities before starting a replacement. Other roots keep their connections;
manual buffer exclusions and global activation remain unchanged. Reattached
buffers send their latest unsaved canonical text. Aliases and split views still
share one process per configuration/root. Retired resolve replies cannot publish
into the replacement connection, even when the source bytes are unchanged.

Healthy connections remain available with no input timer after last-buffer close
by default. Optional last-buffer shutdown, disable and exit use the existing
finite shutdown/exit, forced termination and reap owner. Missing/failed entries
remain retained across close/reopen and completion requests until explicit
restart or disable/enable; the last-buffer policy cannot silently reopen a crash
restart loop. Native editing and fallback continue after EOF or queue failure.

Worker completion retires initialized and queued document leases and clears
synchronization evidence, including failed initialization. Snapshot resize and
retirement serialize their byte lease while preserving the 8 MiB/document and
32 MiB aggregate limits. Concurrent cleanup cannot leak capacity, subtract twice
or republish an obsolete synchronized version.

Configuration/folder/progress requests receive the admitted bounded responses.
Server applyEdit requests are denied; command execution and capability
registration return MethodNotFound. Progress/diagnostic notifications grant no
publication authority and create no native undo entry or file write.

## Implementation Notes

The installed-server test runs in a separate test process with owned HOME, XDG,
Cargo-home and target directories and offline Cargo. A dependency-free Rust crate
has `disk_field` on disk but `unsaved_field` in its canonical buffer. The driver
requires a Semantic source containing the unsaved field, accepts it through the
shared completion path, verifies one additional history transaction, undoes to
the unsaved text, and checks that disk bytes remain unchanged. It observes one
owned rust-analyzer PID and verifies its removal after session exit. The parent
owns a separate process group and a 60-second outer timeout for fixture failures;
normal semantic readiness has a 35-second bound. No server download or skipped
success substitutes for the installed executable.

Observed versions:

- `rust-analyzer 0.3.3089-standalone`
- `rustc 1.98.0 (88d9e12ae 2026-08-18) (Arch Linux rust 1:1.98.0-1)`
- `cargo 1.98.0 (797e8a9bc 2026-08-05) (Arch Linux rust 1:1.98.0-1)`

The real server negotiated UTF-8 incremental synchronization and completion.
Its initial valid-empty responses selected native fallback while the workspace
loaded; the third request returned the required semantic unsaved field in the
focused run (0.42 seconds for the isolated test). This field completion did not
advertise resolve. Resolved imports and cancellation remain proven by the owned
protocol fixture, without claiming that the real field result proves them.

The close/reopen driver confirms the native kill-buffer prompt before asserting
a fresh document generation. The real-server driver waits for Semantic source
identity as well as the expected field; a matching native candidate is not proof
of server readiness. These corrections preserve the source, history and resource
assertions rather than weakening them.

## Contracts Added or Changed

The native command catalog adds `lsp-restart`; binding/reference inventories and
the existing Unreleased completion entry describe it. No new configuration or
persistent connection store is introduced. Existing finite queue, frame,
request, snapshot and shutdown limits remain unchanged.

## Files of Interest

- `src/lsp/attachment.rs`, `src/session/lsp.rs`: scoped restart and failed-state retention.
- `src/lsp/documents.rs`, `src/lsp/inbox.rs`: initialized/queued lease cleanup and concurrency bounds.
- `src/command.rs`, `src/command/lsp.rs`: native restart registration.
- `tests/lsp/lifetime.rs`, `tests/lsp/lifetime.sh`: owned lifecycle, server authority and blocked-I/O fixtures.
- `tests/lsp/real.rs`: isolated installed-server semantic/undo/disk/cleanup proof.
- `tests/lsp/edits.rs`: pending resolve retirement during explicit restart.

## Validation

Successful focused development checks:

- `cargo test --locked --test lsp lifetime:: -- --nocapture`: five cases passed.
- `cargo test --locked --lib lsp:: -- --nocapture`: five resource/lease/control cases passed.
- `cargo test --locked --test lsp real::installed_rust -- --nocapture`: mandatory installed-server case passed.
- `cargo test --locked --test lsp edits::retired_resolves -- --nocapture`: cancellation matrix including explicit restart passed.
- `cargo clippy --locked --test lsp --test completion --lib -- -D warnings`: passed.
- `cargo run --locked --quiet --example documentation-check`: 331 commands,
  211 configuration entries and 63 reachable pages passed.
- `cargo run --locked --quiet --example source-check -- --file src/lsp.rs --file src/lsp/attachment.rs --file src/lsp/documents.rs --file src/lsp/inbox.rs --file src/command/lsp.rs --file src/command.rs --file src/session/lsp.rs`: safe Rust, parameters at most five, nesting at most three and complexity at most ten passed.

Controlled cases include initialization stall, ignored shutdown, blocked writes,
80 canonical replacements under queue pressure, EOF without respawn, idle
retention, root-local restart, unchanged independent roots and buffer exclusions,
exact 64-message/16 MiB queue controls, snapshot accounting and concurrent
retirement. Existing tests cover optional last-buffer shutdown, pending request
cancellation and configured unbundled-language completion without a grammar.
All fixture processes are owned and reaped; tests never stop unrelated servers.

The final clean PR head is validated through the controller after the
PR-reference follow-up, using precisely the configured command order:

- `cargo fmt --all -- --check`
- `cargo test --locked --test lsp --test completion --test recovery --test files --test windows --test configuration`
- `git diff --check`

T5.1 is commit `8081de017dc5a2f5d4bb2e5d3e56a6725f57f2c7`.
T5.2 is commit `846071a8e318814a6062a065c092e267e6b8cf5e`, which introduces this report with the installed-server and restart-cancellation
proof. Canonical integration and its exact-head receipt remain controller-owned.

## Risks and Follow-ups

The installed field-completion result establishes the recorded server/toolchain
and profile on the isolated Rust fixture. Joint executable acceptance, resource
and size measurements, and complete optional-LSP guidance remain in Slice 6.
EOF and bounded protocol fixtures do not claim arbitrary-server compatibility.

## Integration

Canonical PR [#207](https://forge.adrianpk.com/adrianpk/rev/pulls/207) merged
through Rebase + Fast-forward into dev at
`9dcc1ad7c418a9a8b02e84d86acaae8112646ad9`. Controller receipt
`5SO6BE73DXTHRQBETQUHCAMLU4` passed all three configured commands and
368 tests at that clean head. The controller recorded the merge in Acta.
