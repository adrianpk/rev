# Slice 2: External change observation

Status: reviewing
Delivery set: rev-ai-pi
Plan: [Pi and Reload Delivery](../../../plan/rev-ai-pi.md)
Tracker: [Pi and Reload Tracker](../../../tracker/rev-ai-pi.md)
Branch: `feat/rev-ai-pi-02-observation`
PR: `#230`

## Purpose

Observe live visiting files and apply the selected external-change policy without
polling document contents, taking input focus or replacing unsaved text.

## Delivered Behavior

- Clean visiting buffers refresh automatically under reload; ask keeps their
  text with a passive notice. Dirty buffers retain unsaved text under both
  policies, with pending conflict status and the existing overwrite guard.
- Native parent-directory notifications survive atomic replacement, direct
  writes and file deletion/recreation. Hidden buffers are observed; shared panes
  reuse one subscription and canonical publication. Read-only buffers remain so.
- Per-buffer notices name the affected buffer and revert-buffer action without
  opening prompts. New disk versions remain observable after dismissal.
  Canceling an automatic read keeps its external change pending.
- Same-byte images update checked evidence without text/history/cursor changes.
  Completed own-save/reload events coalesce; undo remains dirty and is protected.
- Native completion, bookmarks, Org derived state and LSP versions observe the
  existing document transaction. Stale semantic items cannot edit a reloaded image.

## Implementation Notes

The file owner uses already locked rustix fs/event APIs with no unsafe code,
new dependency or runtime. One notification worker owns inotify registration,
metadata and finite event batches; the sole reload worker retains complete
content reads, consent rechecks and publication admission from Slice 1.

Registrations cover canonical visiting paths through shared parent watches.
Subscription generations retire queued work after association/baseline publication
or closure.
The thread's control eventfd carries coalesced in-memory source updates. Both
notification and reload queues have at most 256 distinct identities; overflow
schedules metadata rechecks in batches of 16. Backpressure retains unacknowledged
observations and completes each rescan before another overflow pass. The main
reload queue preserves arrival order while coalescing repeated identities.

While a terminal is active, owned notifications wake Crossterm's existing native
SIGWINCH queue. Equal-size wake events do not execute resize/navigation/typing
transitions; changed-size terminal events retain native dispatch. Library sessions
never signal their host. Filesystem work gets one finite turn after native input
batches, including continuous typing. Idle observation blocks in kernel poll;
no filesystem timer, recursive project scan or second input consumer is added.

Main-thread subscription reconciliation compares borrowed source evidence on its
common input path. Watch registration and metadata stay on the worker. Expected
registration/source failures produce passive diagnostics and retain explicit
reload. Cancellation/deadline rejection retains one owned worker slot until exit;
late results cannot publish or start replacement-worker accumulation.

## Contracts Added or Changed

- EFC01/EFC02: startup policies drive real bounded per-buffer observation,
  including hidden/shared/readonly sources and rename-over saves.
- EFC03/EFC04: fresh complete ingress and native buffer/disk guards are reused;
  pending notices never acknowledge an external version as a completed save.
- EFC05/EFC06: passive native interaction, one native undo transition and common
  revision invalidation for views, completion, bookmarks, Org and LSP.
- EFC07/EFC08: missing/invalid/limited sources retain text/history; generations,
  finite queues/recheck batches and owned shutdown fence stale work and resources.

## Files of Interest

- `src/file/watch.rs`, `src/file/watch/worker.rs`: registration, kernel polling,
  bounded coalescing/rechecks, native wake and owned lifetime.
- `src/session/observation.rs`, `src/session/reload.rs`, `src/session/events.rs`:
  source reconciliation, policies, passive notices, cancellation and native input.
- `src/buffer.rs`, `src/buffer/reload.rs`, `src/display/status.rs`: per-buffer
  pending state and canonical clean publication/status without another text owner.
- `tests/external_changes/`: real kernel/session and reused bounded PTY coverage.
- `tests/lsp/{documents,edits}.rs`: actual canonical document versions and stale
  semantic popup retirement with controlled stdio peers.
- `docs/reference/{commands,defaults}.md`, `CHANGELOG.md`: current policy and
  reload guidance with one consolidated user-facing Unreleased outcome.

## Validation

Focused development checks passed:

- `cargo test --locked --test external_changes observation:: -- --test-threads=1`
- `cargo test --locked --lib file::watch:: -- --test-threads=1`
- `cargo test --locked --lib observation_wake -- --test-threads=1`
- `cargo test --locked --lib session::observation::tests::publication_retires_notices -- --exact --test-threads=1`
- `cargo test --locked --test external_changes pty::idle_native_wake -- --exact --nocapture --test-threads=1`
- `cargo test --locked --test lsp external_reload -- --test-threads=1`
- `cargo test --locked --test lsp edits::reload_retires_completion -- --exact --test-threads=1`
- `cargo run --locked --quiet --example documentation-check`
- `git diff --check`

Nine real kernel/session cases passed in 4.10 seconds. They cover both policies
and profiles, direct/atomic writes, dirty/ask notices and subsequent versions,
shared/hidden/readonly buffers, identical bytes, own saves, invalid/missing images,
recreation, registration limits/unavailability, cancellation and bookmark/Org
invalidation. The two worker tests cover controlled 300-source overflow/backpressure,
16-source rescan batches, queue coalescing and owned shutdown. The native event test
preserves one typing history group across an equal-size signal wake.

Both real Linux PTY profiles reloaded an external rename without input. With
10 ms configured debounce, observed write-to-frame latency was 47 ms non-modal and
46 ms modal. Each subsequent 350 ms idle window emitted no terminal output and
used zero CPU ticks; editor peak RSS was 29484 KiB and 30492 KiB respectively.
These are unoptimized debug fixture observations, not isolated feature memory
costs or production performance guarantees. Each child used the established
8-second/1-MiB owner, exited and restored its terminal attributes.

Controlled stdio LSP peers observed one native didChange version and no didSave
for disk reload. Undo advances the document version again. A preexisting semantic
popup is retired before resolve/accept can edit the new source. These controlled
peers do not establish an external server's semantic quality.

Documentation passed 418 command identities, 223 configuration entries,
68 reachable pages and 414 links/anchors. Final exact-head evidence is generated
through the controller after the PR-reference follow-up, using exactly the four
configured commands in order. The canonical PR/controller receipt owns that
candidate and result; no integrated delivery-set gate was run here.

The first controller batch at
`fe875dae54edb6c2ed2a7503d2db1f7a9951d239` passed format and stopped at
strict Clippy's drop_non_drop lint for the borrowed inotify reader. Removing
the explicit drop lets its borrow end at the last read, preserving the same
bounded event drain, overflow handling and owned descriptor lifetime. No check,
assertion, resource bound or policy changes. Final evidence is regenerated for
the corrected clean PR head.

Receipt `EISS6RRAVWXG64UZAK67OICMFP` passed all four configured checks at
`46a9f2d46183e37a01ae8b5cee590fe69f4a5b5f`: 21 external-change, 75 file and
46 LSP tests. Further bounded recovery assertions then exposed two publication
boundaries before final handoff. A real PTY lost observation after parent-directory
replacement and explicit reload; the owner now removes retired parent descriptors
and publishes the fresh observation baseline before input can block again. A
controlled late missing-notice test also failed after a completed native save;
baseline publications now advance subscription generations and reject that result.

The corrected controlled case passed, and both PTY profiles recovered the parent,
reloaded another external write without a keystroke, stayed idle and restored the
TTY (1.53 seconds total). The original 8-second/1-MiB child limits, both profiles,
exact document assertions and 350 ms idle admission remain unchanged. Source bytes,
checks and policy were not weakened. The final controller receipt is regenerated
for the resulting clean candidate.

## Risks and Follow-ups

Kernel overflow is covered through the bounded internal scheduler; no system-wide
watch exhaustion was forced. A missing-parent fixture exercises actual registration
failure, and explicit reload re-admits observation after the source appears.
Lost parent-directory watches report unavailable coverage until an admitted
association/baseline change retries registration. The later joint acceptance retains
broader multi-buffer/resource measurements under its approved scope.
