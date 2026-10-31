# Slice 1: Optional owned server transport

Status: reviewing
Delivery set: rev-language-server-support
Plan: [Optional LSP Delivery](../../../plan/rev-language-server-support.md)
Tracker: [Optional LSP Tracker](../../../tracker/rev-language-server-support.md)
Branch: `feat/rev-language-server-support-01-transport`
PR: [#203](https://forge.adrianpk.com/adrianpk/rev/pulls/203)

## Purpose

Deliver the optional connection and configuration layer for LSP01-LSP02 and
LSP04-LSP05, after the completed native completion predecessor.

## Delivered Behavior

LSP starts disabled. Enabling through startup TOML or `M-x lsp-mode` activates
an installed configured server for the current visiting file. File visits and
buffer/view activation reuse a connection for the admitted server entry and
canonical root. Nearest configured markers select the root; an explicitly
opened containing tree bounds lookup and supplies the fallback. Empty marker
lists use the containing directory. No grammar allowlist restricts languages.

`M-x lsp-status` reports disabled, unavailable, starting, transport-ready,
missing executable, failed, limited or stopping state. Missing and failed
connections remain retained until disable/enable, preventing automatic respawn.
The connection cap includes those retained failures. Native editing and
completion remain independent of transport availability.

## Implementation Notes

One supervisor owns each directly launched executable/argv and its stdio.
Worker readers and a bounded writer keep pipe waits and JSON decoding outside
terminal dispatch. Content-Length measures UTF-8 bytes. Headers admit at most
16 KiB; bodies obey the configured ceiling, at most 8 MiB. Each direction
admits 64 queued messages and 16 MiB; stderr drains separately with a 64 KiB tail.
Reserved control slots let failure and shutdown bypass a saturated inbound queue.

Initialization negotiates UTF-8/UTF-16 and full/incremental synchronization,
rejecting unavailable encoding/sync contracts. Healthy idle workers block on IO
or a condition variable. Initialization and shutdown have finite deadlines;
disable requests cleanup without waiting on terminal dispatch. Exit closes
owned sockets, terminates a remaining owned child and reaps it. All workers
receive shutdown before any join, so independent deadlines run concurrently.

The supported server-request subset returns admitted configuration/root data,
acknowledges progress-token creation, declines applyEdit/showMessageRequest and
returns MethodNotFound for unknown methods. It cannot mutate source or execute
server-supplied commands. The initialization profile advertises no snippets or
dynamic registration.

## Contracts Added or Changed

Typed `lsp` preferences declare enablement, lifetime policy, finite limits and
at most 32 named server entries with languages, executable, argv, root markers,
initialization options and settings. Entries inherit by leaf across existing
configuration sources. New entries require a command and languages after
inheritance; NUL launch strings, traversal markers and non-finite JSON data reject.
The default Rust entry uses an already installed `rust-analyzer`.

The packaged sample, live-schema catalog and command/binding references include
the new declarations and the two working native controls. Document attachment
and last-buffer lifetime policy are completed by the subsequent planned layers.

## Files of Interest

- `src/lsp.rs`, `src/lsp/`: connection identity, roots, framing, queues and ownership.
- `src/config/lsp.rs`, `src/config/lsp_validation.rs`: typed settings and admission.
- `src/config/lsp_merge.rs`: source inheritance and provenance.
- `src/session/lsp.rs`: activation, nonblocking disable and native status/control.
- `tests/lsp.rs`, `tests/lsp/`: bounded local process and native-command fixtures.
- `tests/configuration/lsp.rs`: invalid inputs, inherited entries and unbundled languages.
- `docs/reference/config.toml`, `docs/reference/defaults.md`: complete sample/catalog.

## Validation

Successful development checks:

- `cargo check --locked --offline`: passed.
- `cargo test --locked --test lsp --test configuration --test files --test commands`:
  passed the initial 380 selected tests before final control/limit coverage.
- `cargo test --locked --test lsp`: passed six transport/root/budget tests.
- `cargo test --locked --test lsp native_controls`: passed the native-control fixture.
- `cargo clippy --locked --lib -- -D warnings`: passed.
- `cargo run --locked --quiet --example documentation-check`: passed 328-command
  inventories, 211 configuration entries and 63 reachable documentation pages.

Focused source-quality checks cover all changed production owners, including
crate-root safety and the existing parameter, nesting and complexity bounds.
Final exact-HEAD commands are recorded through the controller after the canonical
PR-reference follow-up. The controller owns integration and its later full gate.

Coverage includes split/combined UTF-8 frames, malformed/oversized headers and
bodies, invalid JSON, EOF, stderr bursts, failed initialization, unsupported
encoding/sync, deadlines, missing executables, disabled activation, root bounds,
canonical aliases, connection budgets, disable/enable and graceful shutdown/exit.
Owned local fixture processes are reaped after their checks.

T1.1 is commit `dc8b3428da348d5f259c69134b7a742739fe8785`.
T1.2 is commit `37dff9caf62315049ee65fb98348548a52dd956c`, which introduces this report with native activation/status and configuration tests.

## Risks and Follow-ups

The local peer establishes deterministic transport behavior. Real installed
Rust-server interoperability remains in Slice 5. Document synchronization,
buffer exceptions, semantic candidates and resolved edits remain in Slices 2-5;
combined acceptance remains in Slice 6.
