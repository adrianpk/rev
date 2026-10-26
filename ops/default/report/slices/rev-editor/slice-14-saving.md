# Slice 14: Publish safe same-directory saves

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-14-saving`
PR: `#14`
Integrated: `50fd66b75b57f4a1ad54ebd4be15f47e7bea2ea2`

## Purpose

Save captured text through checked same-directory replacement and reconcile the
actual publication, logical save point and destination without losing edits.

## Delivered Behavior

- `C-x C-s` and named `save-buffer` save the active visiting buffer. Output
  retains exact UTF-8, BOM, LF/CRLF and final-newline bytes. Complete saves
  report the destination and permit ordinary exit when all buffers are clean.
- A private COW capture fixes the source revision, logical state, format and
  destination. A complete save acknowledges that state; newer edits stay dirty,
  while undo can return to the captured clean state. History remains available.
- Output uses an exclusively created owner-private temporary file in the target
  directory. Checked writes, flush, access-bit preservation and file sync precede
  replacement; directory sync and a fresh observation complete the save.
- Errors before replacement preserve the destination and prior association,
  save point and dirty state. Errors after replacement retain the actual target
  and publication fact, record a warning and keep saving pending. An unresolved
  publication cannot satisfy quit admission, even at an otherwise clean state.
- Explicit absent destinations can receive their first native save association.
  Association, index and label changes follow actual publication, including a
  published but unresolved result. Unpublished attempts do not relocate them.
- Changed destinations, occupied alternate targets, another live visiting owner,
  read-only buffers and denied OS write access are refused before output.
  Canonical referent saving preserves visiting symlink entries.
- Error echoes put publication status before long destination paths. Owned
  temporary cleanup is checked and secondary cleanup failures remain reported.

## Implementation Notes

- T14.1: `c9d8c39b05901db5b80c2fba1bfd60d8af5493fd` implements captures,
  filesystem publication, reconciliation and commands. Command naming ownership
  is completed through `f16523df10507278497ad7d8cc6454ee096119bc`, keeping
  action lookup and delegated navigation families within the source limit.
- T14.2: `7bdc7fa8bed7bd3db00340f9b1cc877b56b36885` adds fault, state,
  association, cleanup and real-executable save evidence.
- A capture owns one stable rope snapshot, not another authoritative document.
  Both live serialization and captured output stream borrowed rope chunks through
  the same format emitter; no whole-file String enters saving.
- Conservative observations retain device/inode, length, mtime, ctime and Unix
  mode. Admission rechecks association, native access and live destination ownership;
  the output owner checks the recorded disk observation and actual write access.
- Exclusive temporary creation retries at most 32 name collisions. Existing Unix
  rwx bits are retained; new files receive owner read/write access. Cleanup touches
  only the exclusively created path, with a final best-effort Drop attempt.
- Successful rename establishes publication before any later fallible step.
  Directory-sync and observation errors are both retained. Unsupported required
  synchronization yields an unresolved result rather than a completion claim.
- Reconciliation moves the save point only on complete success. Warnings keep
  unresolved publication dirty independently of logical history state. A retry
  with a known disk observation can complete; an unknown observation remains
  guarded against an unconfirmed overwrite.
- Source checks pass safe Rust, five parameters, nesting at most three and
  cyclomatic paths at most ten.

## Contracts Added or Changed

- `Buffers::prepare_save` creates a checked `Save` capture for the associated
  destination or an explicit unoccupied new target. `publish_save` consumes it,
  rechecks admission, performs output and reconciles actual results.
- `file::Outcome` exposes `NotPublished`, `Complete` and `Uncertain` publication,
  target and error facts. Only `is_complete` can acknowledge successful saving.
- `file::Output` supplies real filesystem boundaries with narrow native fault
  overrides. Returning a replacement error means no replacement; post-replacement
  failures belong to completion/observation boundaries.
- `Location` retains conservative observations. `Buffer::save_warning` exposes
  unresolved publication, which contributes to modified/quit admission.
- `Command::SaveBuffer` joins named dispatch and `C-x C-s`; error messages and
  `CHANGELOG.md` describe usable saving and pending-save protection.

## Files of Interest

- `src/buffer/save.rs`: capture, late admission and association/save-point reconciliation.
- `src/file/save.rs`: checked staging, replacement and explicit completion outcomes.
- `src/file/temporary.rs`, `src/file/output.rs`: exclusive ownership and I/O boundaries.
- `src/file/observation.rs`, `src/file/location.rs`: conservative destination observations.
- `src/session/save.rs`, `src/command.rs`, `src/input.rs`: common save command and echoes.
- `tests/files.rs`, `tests/files/saving.rs`: exact format, history, fault and ownership cases.
- `scripts/check-pty.py`: real save, failed-save recovery and guarded exit journeys.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 136 tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test files`: passed; thirty-seven tests, zero skips. Faults
  inject partial write/storage-full, flush, file sync, replace, directory sync,
  observation and cleanup errors at real owned filesystem boundaries. Cases
  check original or published bytes, both post errors, first/relocated association,
  index, exact logical save state, newer edits, read-only/access admission,
  external changes, stale captures, live collisions and symlink preservation.
- `python3 scripts/check-pty.py --suite terminal`: passed; twenty-four cases,
  zero skips. Added journeys check actual save keys, exact BOM/CRLF bytes,
  permissions, saved undo states, failed-save exit refusal, missing-parent retry,
  external-change refusal and continued editing. Owned fixtures stop after checks.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
