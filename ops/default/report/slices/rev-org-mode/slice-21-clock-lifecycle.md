# Slice 21: switch and recover Org task clocks

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-21-clock-lifecycle`
PR: pending

## Purpose

Deliver checked session clock transitions and explicit recovery under the
[approved plan](../../../plan/rev-org-mode.md#slice-21-switch-and-recover-org-task-clocks)
and [tracker](../../../tracker/rev-org-mode.md#slice-21-switch-and-recover-org-task-clocks).

## Delivered Behavior

T21.1 switches task clocks using one captured instant: close the previous interval
and open the selected heading after validating both source buffers. Selecting the
same active heading leaves its record unchanged. Completing a clocked task closes
its interval together with the keyword, planning and selected history/note edits.
`org.clock.out_when_done` defaults true. Cancelling a note leaves source and clock
unchanged; accepting it uses the captured completion time.

T21.2 asks close or keep before quitting or closing the active source buffer,
then retains normal per-file save/discard/cancel behavior. Explicit
`org-resolve-clocks` observes the selected entry and requires a numbered interval
and resume/close/cancel/keep choice. It never guesses identity from stale text.
Selected starts and ends use the captured zone; folds expose actual offset
choices and gaps reject. Close asks for the accepted end, excluding rejected idle
time. Invalid end drafts remain editable. Source/view changes cannot publish an
old recovery draft. Undo/redo can be followed by fresh explicit source adoption.

## Implementation Notes

The existing buffer publication owner now admits source, points, marks and
history capacity separately before synchronous publication. Clock operations
prepare at most two buffers and all actual shared views, including a hidden
participant with no initiating view. No participant publishes until every source,
point, viewport map and history allocation is admitted. Canonical publication
still owns text/history; no duplicate editable document is introduced.

Clock completion coalesces a drawer insertion with its adjacent CLOCK replacement.
Zero-minute removal retains a drawer when task history is inserted into it.
Source identity remains revision-qualified; stale source rejects before either
close or open publishes. Files remain subject to ordinary explicit saves.

Recovery retains at most 64 incomplete records in a 262144-byte entry, with a
4096-byte CLOCK line cap. It preserves start spelling after checking the selected
instant against parsed source. Source, end-time and zone validation precede every
edit; no failed or cancelled step silently creates a running clock or counts time.
The [Org idle-time reference](https://orgmode.org/manual/Resolving-idle-time.html)
provides the explicit resolution and accepted-time behavior.

## Contracts Added or Changed

- ORG19: one active session clock, checked switching and coherent completion.
- ORG18: bounded source preparation and ordinary edit/history ownership.

## Files of Interest

- `src/session/outline/clocks/`: switching, completion and synchronous publication.
- `src/buffer/edit.rs`, `src/window/points.rs`: shared publication admission.
- `src/session/outline/tasks/repeat.rs`: captured completion through note entry.
- `tests/org_editing/clock_switching.rs`: native switching and completion boundaries.

## Validation

T21.1 development checks passed:

- `cargo check --quiet`.
- `cargo test --test org_editing clock`: 16 selected tests, zero skips.
- `cargo test --test configuration org_clocks`: two policy cases, zero skips.
- `cargo test --test editing --test windows`: 96 edit and 41 shared-window tests,
  zero skips; approximately 15 seconds warm, selected because publication admission changed.
- `cargo clippy --lib --test org_editing --test configuration -- -D warnings`.
- `python3 scripts/check-config-reference.py`: 118 schema/catalog entries,
  46 RGB roles, 169 default bindings and 14779 packaged template bytes agree.
- Changed-source quality passed at the original safe-Rust, parameter, nesting
  and complexity limits.

A policy-off fixture initially expected the Org clock provider's timestamp while
using the ordinary task logger. Its task clock is now explicitly injected too;
production timestamp and policy behavior were unchanged.

T21.1 commit: `b22d7b7236875c57860d43ed2ce3368ff7000c88`.

T21.2 development checks passed:

- `cargo test --test org_editing --test recovery clock -- --nocapture`: 20 selected
  editing and three quit/close recovery cases passed, zero skips.
- `cargo test --test org_time clock_resume_source`: source/selected-instant and
  forged-offset admission passed.
- `cargo test --test org_editing clock_recovery_limit`: exact 64/65 record boundary passed.
- `cargo test --test org_editing clock_recovery_missing`: source/end spring gaps
  preserved source and the invalid editable draft.
- `cargo test --test commands org_visibility`: command/key parity passed.
- `cargo test --test commands catalog_reference`: complete compiled catalog agrees.
- `cargo clippy --test org_editing --test org_time --test commands --test recovery -- -D warnings`.
- `python3 scripts/check-config-reference.py`: 118 schema/catalog entries,
  46 RGB roles, 170 default bindings and 14816 packaged template bytes agree.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tasks`:
  four owned RGB/NO_COLOR editing/read-only PTYs passed, zero skips. The existing
  journey now covers task switching, canceled departure, explicit keep/close,
  exact CLOCK/source save contents and complete undo.
- Changed-source quality passed at the original limits.

Development checks caught a date-entry receiver mismatch, transient completion
routing overlap and complexity boundaries. Existing date entry and native
rendered prompt assertions now exercise the owning APIs; parsing and recovery
responsibilities were split while preserving all bounds and assertions.

## Risks and Follow-ups

T21.3 remains in progress. The delivery-set full gate has not run.
