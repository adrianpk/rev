# Slice 21: switch and recover Org task clocks

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-21-clock-lifecycle`
PR: `#122`

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

T21.3 adds opt-in checked XDG clock state and bounded recent-task actions.
Persistence defaults off; opening a selected missing store creates no directory.
Startup reads the bounded state file and asks before visiting its one pinned
source. Recovery separately checks file observation, canonical path, actual
heading/CLOCK text, native structural context and the selected start instant.
A matching file observation cannot establish source or time equivalence.

Recent locators default to 32, admit 0..256 and retain exact source revisions.
Zero disables them. Recent goto/re-clock reject stale sources and remove closed
buffers. State stores the captured start and acknowledged operation instant,
including seconds, while compatible CLOCK text retains minute precision.

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

State admits 65536 bytes, 4096-byte path/source fields and a 1048576-byte explicit
source visit. It uses the existing owned temporary/output boundary and checked
atomic replacement, distinguishing failed replacement from published-but-unresolved
completion. File saves refresh the checked locator. State checkpoint failure never
rolls back a published canonical clock operation or rewrites external state.
No state polling or source-directory startup scan is introduced.

The existing TOML dependency has no writer feature. A finite state-schema encoder
quotes strings and emits checked integers without adding dependencies or changing
Cargo features. Native round trips cover escaped paths and Unicode. The ordinary
source file still requires explicit saving; recovery retains a missing/unsaved
interval as unresolved instead of fabricating source text.

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

T21.2 commit: `a1df24f58a3c04d43d0578c30caf1c5c0447df58`.

T21.3 development checks passed:

- `cargo test --test org_io`: eleven native persistence, mismatch, source-context,
  publication-failure and restart cases passed, zero skips.
- `cargo test --test org_editing --test recovery --test configuration clock`:
  24 editing, three recovery and three configuration cases passed, zero skips.
- `cargo clippy --all-targets --all-features -- -D warnings`.
- `python3 scripts/check-config-reference.py`: 120 schema/catalog entries,
  46 RGB roles, 171 default bindings and 14887 packaged template bytes agree.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tasks`:
  eight owned sessions passed, zero skips, including RGB/NO_COLOR startup/keep/
  restart recovery with real XDG state, exact CLOCK/duration/source saves,
  terminal restoration and owned cleanup. Four additional restart sessions
  establish the real persistence boundary without multiplying core variants.
- Changed-source quality passed at the original thresholds.

Development checks exposed the disabled TOML writer feature, a redundant return,
and complexity boundaries. The finite encoder and separate admission/identity/
source responsibilities preserve the dependency and resource contracts. A literal
state fixture originally used an unescaped heading, which resynchronizes blocks
under the approved source contract. Its comma-escaped heading now exercises the
intended literal context with unchanged rejection/source/state assertions.
Recovery end selection also retains a captured current instant at sub-minute
precision, avoiding a rounded earlier second for zero-minute restart intervals.

T21.3 commit: `998e7d26d93bbb7604dfd4c6962859892c862005`.

[Canonical PR #122](https://forge.adrianpk.com/adrianpk/rev/pulls/122) targets dev.
Final required controller validation binds the final clean PR head in this order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_io --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

Final focused evidence also covers all affected editing/window, Org editing,
command, configuration and display targets, reference agreement and eight PTYs.

## Risks and Follow-ups

The delivery-set full gate has not run.

## Integration

Canonical PR #122 merged into dev at
`55f5a763eac4817f1eb8560211beed27ece30455`; live Forgejo and Git agree.
Required controller receipt `74YUR2Z7JMGBCWZ6BDBWNKUXF4` passed all five ordered
checks, including 95 tests. Focused receipt `KDLOGSJDKZERJ3EPAAZ32HPBLV` passed
623 affected tests, configuration reference agreement and eight owned task and
restart PTYs. Both receipts bind that exact clean head; no checks were skipped.
