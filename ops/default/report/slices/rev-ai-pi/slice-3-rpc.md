# Slice 3: Pi RPC explanation

Status: delivered
Delivery set: rev-ai-pi
Plan: [Pi and Reload Delivery](../../../plan/rev-ai-pi.md)
Tracker: [Pi and Reload Tracker](../../../tracker/rev-ai-pi.md)
Branch: `feat/rev-ai-pi-03-rpc`
PR: `#231`

## Purpose

Explain a captured selection through an optional owned Pi RPC child while native
editing remains responsive and source text/history stay under existing ownership.

## Delivered Behavior

- `ai-explain` works through M-x and modal colon with a nonempty native linear
  selection. The envelope labels its Unicode range, buffer/revision, path, point,
  BOM and dirty state as captured editor data, separate from the instruction.
- Each invocation uses fresh input-only context and no previous conversation.
  Tools, automatic extensions, MCP, skills, templates and discovered context
  files are disabled. The visible status/result identifies this current policy.
- Pi starts lazily after explicit admission and a successful 1.3.0 version probe.
  Existing Pi authentication/model resolution remain with Pi; no native token,
  secret map, provider transport, implicit login or backend fallback is added.
- Prompt acceptance stays pending. A result requires correlated acceptance,
  agent_settled and a completed successful assistant message with exactly the
  version-1 text object. Partial, aborted, malformed, excessive and redirected
  output cannot become a successful explanation.
- The settled explanation uses the existing native read-only transient buffer.
  Local edits, disk bytes, dirty state and undo remain intact. A changed source
  or focused prompt keeps its current view; the result remains selectable.
- C-g and an owning modal Escape fence late records and cancel the sole operation.
  Prompt selectors keep their own cancellation. A stopping child retains its
  slot; editor exit stops/reaps only owned processes.

## Implementation Notes

Typed ai.pi leaves use the existing partial graph, validation, source provenance,
immutable snapshot and packaged template. Session capture uses native spans and
views. The existing result-buffer owner supplies read-only presentation without
another source rope, history engine or selection store.

One supervisor owns each fresh RPC child and its invocation epoch. Independent
scoped stdout/stderr readers continue while it writes. LF is the only delimiter;
records admit at most 2 MiB, queued protocol bytes at most 4 MiB/256 records
(including conservative delimiter accounting), and diagnostics a 64 KiB tail.
Logical input/result limits apply independently of encoded bytes. A partial
stdin write has one absolute 250 ms deadline, rather than renewed per-write
allowances. At most eight model turns are admitted; tool/UI requests fail under
this input-only grant. Diagnostics/provider errors are not copied into prompts,
settings or user-facing secret-bearing messages.

Cancellation sends clear_queue then abort, waits at most five seconds for the
admitted response/settled outcome, then closes stdin. Shutdown allows two seconds
before kill and a further two seconds for forced reap. Socket shutdown releases
readers even with inherited descriptors. RAII retains process ownership through
setup failures/unwind; failed OS cleanup blocks replacement children in the
owning editor session. Finite native input turns service active work after typing
batches; an inactive AI owner contributes no input timer or process.

Configuration landed in `8725da90411ada30b14fb5371c9be4dd06f7d8f7`.
T3.1 RPC ownership and T3.3 failure handling land together in
`9cc5443b89a5bb11f89e3b5cb769b099f2af3049`; they share the same finite process
lifecycle. T3.2 native command/result integration is
`0e81da925428984974e1f23251d1972566020a69`. The report introduction adds the
partial-write deadline and focused regression coverage.

## Contracts Added or Changed

PI01-PI04/PI07 and shared AI01/AI05/AI07/AI16 are implemented for the approved
selection-explanation increment. The command's visible input-only context remains
truthful until the recorded guarded-directory increment. Existing reload and
native input/selection/result ownership are preserved.

## Files of Interest

- `src/config/ai.rs`: typed configuration, partial projection/merge and bounds.
- `src/ai/pi{,/frame,/io,/protocol}.rs`: version admission, child/stream lifetime,
  LF limits, correlation, settlement, generated-result parsing and cancellation.
- `src/session/ai.rs`: native capture, epoch/cleanup fencing and result presentation.
- `src/session/{events,prompt,modal}.rs`: finite progress and cancellation ownership.
- `tests/{ai,pi}.rs`, `tests/ai/`: controlled process/session cases and a bounded
  installed-CLI offline probe that never submits a provider prompt.
- `docs/reference/{commands,bindings,defaults}.md`, `config.toml`: current callable
  scope, complete configuration/defaults and discoverable command inventories.

## Validation

Successful focused development commands:

- `cargo test --locked --test configuration ai:: -- --test-threads=1`
- `cargo test --locked --lib ai::pi:: -- --test-threads=1`
- `cargo test --locked --test ai dirty_and_readonly_sources -- --exact --test-threads=1`
- `cargo test --locked --test pi rejected_boundaries -- --exact --test-threads=1`
- `cargo test --locked --test pi deadlines -- --exact --test-threads=1`
- `cargo check --locked --all-targets`
- `python3 tests/ai/offline.py /home/adrian/.local/share/mise/installs/pi/1.3.0/pi/pi`
- `cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The controlled subprocess boundary covers acceptance versus settlement, response
ordering, Unicode LF framing, stderr backpressure, protocol/result/input limits,
invalid history/version/authentication, absent executables, tools/UI boundaries,
turn limits, prompt rejection, deadlines, blocked stdin, cancellation and owned
shutdown. Native sessions cover both profiles, captured dirty/read-only sources,
continued editing/undo, source preservation and prompt/focus ownership. Two
private unit cases check LF allocation and queue byte/count admission.

The real installed 1.3.0 CLI admitted fresh empty get_state sessions, closed with
exit zero and created no files in either owned temporary directory. Two fresh
starts took 471/444 ms, with peak RSS 146384/144624 KiB. Each 250 ms idle sample
emitted no output and used two CPU ticks. These are whole external-process
observations, not isolated editor overhead, warm retained-session measurements
or provider authentication/generation evidence.

Catalog validation passed 419 commands, 230 configuration entries, 68 pages and
414 local links/anchors. A development dirty-source case initially reused a
selection cleared by an edit; it now explicitly selects the unsaved text through
native commands, preserving the acceptance assertion. The catalog also identified
the missing binding inventory row; both inventories now include the working
command. Neither correction weakens checks, source assertions or resource limits.

Final clean-head evidence is recorded through the controller after PR-reference
follow-up, using exactly the four configured commands in order. The canonical
PR/controller receipt identifies that candidate and its results. No integrated
delivery-set gate is claimed by these focused checks.

## Risks and Follow-ups

Controlled peers establish native protocol/lifetime behavior, not provider
quality or an actual authenticated generation. The recorded joint acceptance
owns that bounded real-provider check. Directory discovery, retained directory
history and generated edit publication follow their existing approved slices.

## Integration

PR [#231](https://forge.adrianpk.com/adrianpk/rev/pulls/231) merged into dev on
2026-11-02T23:02:57Z at `eb2b45803856e7707ffc122ad697ef02d84a9fc7`.
Controller receipt `N77GD6JFCVJQGKPXJ7QCEPDOZA` passed the exact ordered
four-command slice validation at that clean head: format, strict all-target
Clippy, 7 AI/143 configuration/5 Pi tests, and git diff --check. No tests were
ignored. The canonical PR contract and committed/pushed references were verified.
The controller retains integration and the later unchanged delivery-set gate.
