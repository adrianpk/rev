# Slice 15: Confirm overwrite and external changes

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-15-save-decisions`
PR: `#15`

## Purpose

Choose save destinations and confirm occupied or externally changed files without
losing edited text, prior associations or actual publication facts.

## Delivered Behavior

- `C-x C-w` and named `write-file` prompt for a destination. `C-x C-s` prompts
  when the active buffer is unnamed; otherwise it saves the visited destination.
- Occupied write-file targets require explicit yes/no consent. Changed, removed
  or unresolved visited destinations display a warning before overwrite or
  recreation. Enter never supplies implicit consent; no and `C-g` cancel.
- A question identifies the canonical destination and discloses a differing
  requested path. Saving a symlink destination replaces its resolved referent,
  preserving the link. A target owned by another live buffer is refused.
- Confirmation owns input focus and the echo cursor. Invalid answers retain
  the target question, resize preserves focus, and prompt/decision characters
  cannot edit the document. Cancellation preserves point, scroll and history.
- Consent applies to a captured target and observation. A changed destination,
  new visiting owner, native read-only restriction or changed source association
  invalidates the attempt before output; a fresh save must obtain fresh consent.
- Confirmed save-as retains its prior association on unpublished failure. Actual
  replacement moves the association even when later completion fails, preserving
  the warning and pending-save state. Complete saving acknowledges only the
  captured logical state; undo/redo retain truthful clean and dirty states.

## Implementation Notes

- T15.1: `06f8f340815ab3cd760d7fb38784014cf742e25c` adds observed decisions,
  focused commands and initial native evidence.
- T15.2: `ad26ededc8a9033df27eddb7242d607147320f3b` adds consent invalidation,
  publication failures, access and actual-executable journeys. It also places
  yes/no instructions before long destination text in the question.
- A confirmation privately owns the existing COW save capture; it exposes no
  generic force flag. Consuming consent uses the normal late admission and real
  filesystem publication owner without refreshing the captured observation.
- Metadata comparison remains conservative: device/inode, length, modification,
  change time and mode detect changes; no race-free filesystem authority is claimed.
  A retargeted requested alias cannot redirect an already captured canonical target.
- Session pending state distinguishes file arguments from save decisions. The
  existing argument limit, cancellation and repeat suppression remain in force.
- Existing checked temporary writes, permission checks, replacement, synchronization,
  cleanup and publication reconciliation remain common to ordinary and confirmed saves.

## Contracts Added or Changed

- `Buffers::request_save` returns a publishable `SaveRequest::Ready` or a private
  `Confirmation`. Preparation does not publish or relocate the buffer.
- `Buffers::confirm_save` consumes that specific confirmation, retaining all
  destination, live-owner, association and access guards. `confirm_save_with`
  exercises the same real output boundaries under native fault injection.
- `Command::WriteFile` joins common command naming and `C-x C-w`; session
  argument and decision completion route only to their pending save owner.
- `CHANGELOG.md` describes destination selection, confirmation and symlink saving.

## Files of Interest

- `src/buffer/save/decisions.rs`: target resolution, observation and consumed consent.
- `src/buffer/save.rs`: shared stable capture and guarded publication.
- `src/session/save.rs`, `src/session/file.rs`: destination arguments and save decisions.
- `src/session.rs`, `src/display/prompt.rs`: pending ownership, cancellation and echo focus.
- `src/input.rs`, `src/command.rs`: write-file key and canonical command registry.
- `tests/files/decisions.rs`, `tests/files/saving.rs`: state, consent and real filesystem failures.
- `scripts/check-pty.py`: unnamed, occupied, stale, changed, removed, symlink and collision journeys.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 147 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test files`: passed; forty-eight tests, zero skips. Includes
  argument/decision cancellation, invalid answers, tiny geometry, changed/removed
  targets, denied access, late restrictions, storage-full partial writes,
  save-as before/after replacement, unresolved observations, live ownership,
  captured states, clean/dirty history and canonical symlink consent.
- `python3 scripts/check-pty.py --suite terminal`: passed; twenty-six cases,
  zero skips. Real key journeys verify unnamed destination prompts, cancellation,
  occupied save-as, symlink preservation, external-change warnings, stale consent
  refusal, fresh confirmation, removed-path recreation, live-buffer collisions,
  continued editing, dirty undo and terminal restoration. Owned fixtures stop
  after their checks finish.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
