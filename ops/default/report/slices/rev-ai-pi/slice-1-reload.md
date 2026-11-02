# Slice 1: Checked file reload

Status: reviewing
Delivery set: rev-ai-pi
Plan: [Pi and Reload Delivery](../../../plan/rev-ai-pi.md)
Tracker: [Pi and Reload Tracker](../../../tracker/rev-ai-pi.md)
Branch: `feat/rev-ai-pi-01-reload`
PR: pending

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

After the PR-reference follow-up, the controller records final evidence against
the exact clean PR HEAD with the configured format, strict Clippy and combined
files/configuration/external_changes tests in order, followed by focused
checks and `git diff --check`. The canonical PR and controller receipt own that
candidate/outcome; these focused results do not claim the integrated full gate.

## Risks and Follow-ups

Automatic notification delivery, overflow/registration lifetime, idle/resource
measurements and the joint native/PTY acceptance remain in their approved slices.
No Pi process or model operation is part of this explicit reload increment.
