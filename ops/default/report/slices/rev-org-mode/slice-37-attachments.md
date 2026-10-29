# Slice 37: manage native Org heading attachments

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-37-attachments`
PR: [#138](https://forge.adrianpk.com/adrianpk/rev/pulls/138)

## Purpose

Deliver checked native attachment operations under the
[approved plan](../../../plan/rev-org-mode.md#slice-37-manage-native-org-heading-attachments)
and [tracker](../../../tracker/rev-org-mode.md#slice-37-manage-native-org-heading-attachments).
[Org attachments](https://orgmode.org/manual/Attachments.html) supplies ID/DIR,
copy-default dispatcher and attachment-link behavior.

## Delivered Behavior

T37.1 delivers `C-c C-a`, named copy/move/hard-link, list/open, explicit DIR
selection and retained attachment results. Effective DIR takes precedence over
ID; new attachments can propose a UUID-backed native data directory. Every file
mutation requires an explicit method/destination confirmation, including existing
file replacement. Declining or cancelling before publication leaves files and
heading unchanged. Successful publication adds ATTACH and any generated ID in
one source undo group, then stores a compatible attachment link.

## Implementation Notes

Native property/tag owners prepare actual revision-qualified edits. The shared
attachment location resolver also serves existing attachment links. Native and
external opening reuse the real checked visit and configured viewer paths; an
explicit attachment selection re-observes heading data without fabricating an
inline link or a buffer identity.

Linux directory descriptors pin admitted parents. Symlink/non-regular entries
reject; staged new-file publication cannot overwrite a concurrent entry. Move
syncs its destination before removing the checked input. File effects and
metadata publication have separate retained outcomes. Cancellation after an
actual effect cannot discard that effect when the worker is reaped. Source
undo never claims to undo disk effects; metadata is saved through ordinary Save.

Bounds retain the existing single cancellable Org worker, configured Org file
byte/time limits, 64 path components, 255 filename bytes, 4096 listed files,
1 MiB of names and 16 operation results. No idle scanner or permanent worker is
introduced. These are conservative native file observations rather than a
filesystem-wide transaction or a hard deadline for blocked kernel IO.

## Contracts Added or Changed

- ORG24: explicit heading attachment dispatcher, locations and real file effects.
- ORG15: compatible stored attachment links and checked native/viewer following.

## Files of Interest

- `src/org/attachment/`: locations, observations, pinned parents and file effects.
- `src/session/attachment/`: typed prompts, source transactions and retained results.
- `src/org/job.rs`: effect-preserving reaping while retaining pure-job cancellation.
- `src/org/links/`: shared actual-source attachment/viewer selection.
- `docs/reference/org-attachments.md`: commands, bounds, decisions and recovery.

## Validation

- `cargo test --test org_io attachments -- --nocapture`: four real-file workflows
  pass for binary copy/move/hard-link, ID/DIR selection, one-group source undo,
  collision refusal/acceptance, native visits and confirmed viewer arguments.
- `cargo clippy --lib --test org_io --test commands -- -D warnings`: passed.
- `python3 scripts/check-config-reference.py`: 155 schema/catalog leaves,
  54 RGB roles, 241 bindings and 19177 exact packaged bytes agree.
- Changed-source quality passes at unchanged limits.

T37.1/report introduction: `bc4bf57976aebb6198fcdecaf1077143b4c45e7e`.

T37.2 checks:

- `cargo test --lib --test org_io attachments -- --nocapture`: eight tests pass,
  including actual write failure, destination/source directory sync failure,
  input-removal failure, changed file/target/directory observations, cancelled
  worker reaping and stale/read-only/cancelled heading publication. Actual files,
  retained generated IDs and cleanup outcomes are asserted independently.
- `cargo clippy --lib --test org_io -- -D warnings`: passed.
- Changed-source quality passes at unchanged limits.

Only concrete write, directory-sync and input-removal boundaries are injectable.
Publication, path identities, collision checks, observations and acknowledgements
remain native. A newly owned hard-link staging name is created directly; existing
candidate names never grant cleanup authority. Existing read-only destinations
reject before replacement. Complete joined paths retain UTF-8/control/byte limits.

T37.2: `37a7774bb3f551bc4c30b7fc77e194a287164d9b`.

T37.3 checks:

- `cargo test --lib --test org_io attachments -- --nocapture`: 13 tests pass.
  Additional coverage includes input/parent/target symlinks, unreadable files,
  read-only replacements/headings, exact listed names, invalid IDs and byte/record
  bounds. A heading at EOF without a newline coalesces its new tag/drawer
  insertion into one checked replacement and one undo group. Native resource access-time observations remain unchanged during
  source visiting/indexing and point movement; explicit listing then reads the
  owned directory. File contents, history and editing state remain asserted.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-attachments`:
  two owned RGB/NO_COLOR sessions pass collision refusal, restored-permission
  retry, actual binary publication, ordinary metadata undo/redo/save, native
  attachment listing/opening/return, stored link insertion/undo and cancellation.
  Exact BOM/CRLF source bytes and the entire fixture file set are checked.
  Processes exit and are reaped; quiet inactive work has no polling output.
- `cargo clippy --lib --test org_io -- -D warnings`: passed.
- Changed-source quality passes at unchanged limits.

The representative attachment journey adds two short owned terminals. Variants
remain in native tests. Existing journeys, deadlines, byte limits and the full
gate are unchanged.

T37.3: `11ac14a7fb0296047da8eea36b78160ff8ffdbab`.

Canonical PR #138 targets dev. Final controller receipts bind the clean pushed
head after this PR-reference commit, with required commands in approved order.
Focused evidence covers the shared job/source, command, window and display
contracts, configuration reference and owned attachment/link terminal journeys.

## Risks and Follow-ups

The full delivery-set gate remains controller-owned.
