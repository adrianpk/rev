# Slice 1: Checked file reload

Status: delivered
Delivery set: rev-ai-pi
Plan: [Pi and Reload Delivery](../../../plan/rev-ai-pi.md)
Tracker: [Pi and Reload Tracker](../../../tracker/rev-ai-pi.md)
Branch: `feat/rev-ai-pi-01-reload`
PR: `#229`

## Purpose

Reload a visiting file explicitly while retaining unsaved-text consent, checked
source identity, native history and all shared views.

## Delivered Behavior

- `revert-buffer` works through M-x and modal colon entry under either startup
  policy. It reads a fresh disk image and never writes the source.
- Dirty buffers require explicit discard consent bound to the captured revision
  and disk observation. Cancellation, typing, save-state changes, rebinding and
  closed-buffer results cannot publish stale text.
- Reload is one native undoable transaction, including BOM changes. Existing
  history, buffer identity, association, focus and read-only policy survive;
  shared points and viewports use the canonical edit mapper.
- Identical serialized images refresh disk evidence without an edit, history
  entry or cursor movement. Missing, invalid, nonregular and oversized sources
  preserve live text, history and saved evidence.
- Complete startup settings admit reload/ask and the approved debounce, byte and
  registration bounds. Explicit reload works now; event observation is Slice 2.

## Implementation Notes

The native file owner performs metadata, canonical-association and handle checks,
strict UTF-8/NUL/BOM ingress, a bounded complete read and after-read rechecks on
one owned worker. The final destination recheck also runs on that worker after
consent. NOFOLLOW/NONBLOCK admission rejects substituted links or nonregular
sources without opening an indefinitely blocking FIFO. Read admission is limited
to five seconds and at most two unstable-read retries. Late/canceled work retains
its single worker slot until exit and cannot publish. Idle sessions add no reload
deadline when no work exists.

The session rechecks current buffer state before prompting and publication.
Publication changes only the existing buffer rope through native prepared edits,
then acknowledges that state and disk observation as clean. Minimal grapheme-safe
prefix/suffix changes preserve unchanged anchors; BOM-only changes use the same
history owner. There is no alternate editing/history engine or runtime dependency.
The already locked rustix filesystem feature supplies safe open flags.

## Contracts Added or Changed

- EFC01: typed startup policy and bounds, include merge/provenance, whole-candidate
  rejection, complete sample and reference.
- EFC03-EFC05: bounded asynchronous ingress, fresh disk verification, explicit
  dirty discard decision and shared native command entry.
- EFC06: one native publication/history transition, clean-state acknowledgment,
  BOM undo/redo, shared view/anchor mapping and readonly refresh admission.
- EFC07: missing, invalid, oversized and changed associations retain the source.
- Automatic observation, notifications and measured kernel/idle behavior remain
  with the approved Slice 2 and joint acceptance scope.

## Files of Interest

- `src/file/reload.rs`: bounded, stable disk ingress and destination evidence.
- `src/buffer/reload.rs`, `src/buffer/edit/reload.rs`: revision-bound admission,
  native transaction preparation and checked clean-state publication.
- `src/session/reload.rs`, `src/window/points.rs`: one worker, consent, cancellation
  and all-view publication without file IO on the input/render thread.
- `src/config.rs`, `src/config/{merge,project,validate}.rs`: startup file settings.
- `tests/external_changes.rs`: real filesystem and native session coverage.
- `docs/reference/{commands,bindings,defaults}.md`, `config.toml`, `CHANGELOG.md`:
  command/policy reference, complete defaults and the usable reload outcome.

## Validation

Focused development checks passed:

- `cargo test --locked --test configuration reload_policy -- --test-threads=1`
- `cargo check --locked --all-targets`
- `cargo test --locked --test external_changes -- --test-threads=1`
- `cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The reload suite passed 11 tests (1.59 seconds). It uses real owned files for byte
limits, UTF-8/NUL rejection, missing/nonregular/link/permission failures, exact
serialized images, atomic replacement and checked save evidence. Controlled
interleavings cover disk/typing/save/closure/rebinding races. Native session
cases cover both profiles and policies, shared panes, focus, cancellation,
affirmative dirty consent and publication after a clean source becomes hidden.
These are not kernel-notification or real-provider observations.

Documentation passed 418 command identities, 223 configuration entries,
68 reachable pages and 414 local links/anchors. The all-target compile passed.
The initial incremental compile exposed fixture API/array-size and Ropey iterator
mismatches; corrected code and fixtures passed the checks above.

The initial clean-head controller batch at
`a25e827812dea9927248c594c48e8b7f443e314e` passed format and stopped at
Clippy's non-octal Unix permissions lint in the permission-denial fixture.
Changing the identical mode from `0` to `0o0` preserves the denial assertion;
no acceptance condition or configured check changed.

Controller receipt `IU5TOVX6QCM33PBROV4YJQWIC7` passed at clean candidate
`18dfa26c8f34c4f2d0548ad9bc33c5caaae77d1f`: format, strict all-target
Clippy, the configured combined tests (141 configuration, 11 reload, 75 files),
documentation catalogs and whitespace. Total batch time was 66.67 seconds.

Canonical PR admission requires the exact four-command configured prefix,
including final whitespace. It rejected the additional documentation command in
that list despite its successful result. Final evidence is therefore regenerated
after this report follow-up using exactly the four configured commands. The
successful documentation check remains separate evidence above; no check,
assertion or frozen configuration changes. The canonical PR/controller receipt
owns the final exact candidate and outcome. This is focused Slice 1 validation,
not the integrated delivery-set gate.

## Risks and Follow-ups

Automatic notification delivery, overflow/registration lifetime, idle/resource
measurements and the joint native/PTY acceptance remain in their approved slices.
No Pi process or model operation is part of this explicit reload increment.

## Integration evidence

Canonical PR #229 merged into dev at
`2f87ae673641d72abac12a985ec1630b2d542213` on 2026-11-02T21:38:02Z.
Final controller receipt `RM7KIWHX3TU37HLMBDT57F3D22` passed the exact four
configured commands at that clean head: format, strict all-target Clippy,
combined files/configuration/external_changes tests (227 passed) and whitespace.
The controller verified the canonical title/body/report introduction and owned
integration. This closes Slice 1 of 8; the integrated delivery-set gate is pending.
