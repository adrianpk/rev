# Slice 23: deliver native Org relative and countdown timers

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-23-timers`
PR: pending

## Purpose

Deliver native relative/countdown timer controls under the
[approved plan](../../../plan/rev-org-mode.md#slice-23-deliver-native-org-relative-and-countdown-timers)
and [tracker](../../../tracker/rev-org-mode.md#slice-23-deliver-native-org-relative-and-countdown-timers).

## Delivered Behavior

T23.1 adds compiled start/reset, countdown entry, insertion, timed description
items, pause/continue, stop and inspection. Familiar Org chords share these
handlers; the host agenda context admits countdown with semicolon. Relative
values use a monotonic instant and retain fractional elapsed time across pause.
Countdown entry accepts unsigned minutes or H:MM:SS, with a 64-byte/365-day
bound. Zero defaults stay unarmed. Replacement asks yes/no; cancellation and
invalid editable drafts preserve the existing timer. Relative and countdown
modes require stopping before changing kind.

Timer source insertion uses ordinary checked transactions, CRLF/Unicode and
shared-view history. M-RET continues an actual timed description item. Literal,
unclosed, read-only and incompatible list source reject before auto-starting a
timer. Source undo does not rewind session time.

## Implementation Notes

One session timer retains its mode, precise accumulated/remaining duration,
optional monotonic anchor/deadline and generation. No wall clock, zone, source
identity or persisted timer database defines elapsed time. Generation checks
prevent stale countdown drafts from changing a replaced timer. The countdown
deadline and one-shot core expiry are ready for native event integration in T23.2.

`org.timer.default` is typed, layered configuration with source provenance;
its default "0" creates no timer and no startup IO. The ordinary Org list owner
supplies bounded structural context and the normal canonical edit owner supplies
publication, cursor mapping and history.

Behavior references: [Org timers](https://orgmode.org/manual/Timers.html) and the
[Org timer implementation](https://raw.githubusercontent.com/emacs-mirror/emacs/master/lisp/org/org-timer.el).

## Contracts Added or Changed

- ORG22: native session-relative/countdown data and compiled controls.
- ORG18: ordinary source/history admission for timed notes and lists.

## Files of Interest

- `src/org/time/timer.rs`: monotonic state, pause/resume and one-shot expiry.
- `src/session/timers/`: checked prompt and source orchestration.
- `src/org/list/timer.rs`: actual timed description recognition/continuation.
- `tests/org_time/timers.rs`: independent arithmetic and resource contracts.

## Validation

T23.1 development checks passed:

- `cargo test --test org_time --test org_editing --test configuration timer --no-fail-fast`:
  three time, two native editing and one configuration cases, zero skips.
- `cargo test --test org_editing timed_description`: source/undo continuation.
- `cargo test --test commands org_`: ten routing/binding/catalog cases, zero skips.
- `cargo clippy --lib --test org_time --test org_editing --test configuration --test commands -- -D warnings`.
- `python3 scripts/check-config-reference.py`: 121 schema entries, 46 RGB roles,
  181 default bindings and 15321 packaged template bytes agree.
- Changed-source quality passed at unchanged safe-Rust, parameter, nesting and
  cyclomatic limits.

Checks caught a prefix-result test assumption, a redundant mutable binding and
routing complexity. Fixtures use the existing input API and owning helpers;
command eligibility and source-value preparation now have separate owners.
No assertions, deadlines or resource limits were weakened.

## Risks and Follow-ups

Native deadline integration and terminal lifecycle acceptance remain in T23.2
and T23.3. The delivery-set full gate has not run.

T23.1 commit: `37ee7a7ac5c3`.

T23.2 integrates the selected countdown deadline with the existing native wait
owner. A queued stop wins at the timeout boundary before expiry is serviced.
Relative, paused, stopped and expired states retain indefinite idle waiting.
Delayed expiry emits one in-editor notice and removes its deadline. Ordinary
frames derive the selected-pane status value without scheduling redraws.
Small/suspended frames retain their native clipping and cursor bounds.

Prompt/draft and quit/save questions remain intact when expiry occurs. A status
done marker remains until new input. Normal completed exit and the terminal
run boundary both discard the timer; no canceled timer can later wake the host.
Terminal acquisition/restoration remain with the existing terminal owner.

T23.2 development checks passed:

- `cargo test --lib timer_`: native inactive/relative/paused/stopped wait case.
- `cargo test --lib countdown_input_priority`: actual Org stop chord wins queued
  input and timeout-boundary cases; an uncancelled delayed wake signals once.
- `cargo test --test display --test recovery --test org_editing timer --no-fail-fast`:
  two display, two recovery and two editing cases passed, zero skips.
- `cargo clippy --lib --test org_time --test org_editing --test configuration --test commands --test display --test recovery -- -D warnings`.
- Changed-source quality passed at original limits. A precommit whitespace check
  found an extra trailing blank line in the defaults reference; it is removed.

The native recovery fixture now uses the existing compiled `Command::Quit`
identity. Countdown controls are tested directly without treating an armed
countdown as background structural work in a settling helper; all original
source, deadline and cancellation assertions remain.
