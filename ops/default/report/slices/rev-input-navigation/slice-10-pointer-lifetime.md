# Slice 10: Own native pointer reporting and input state

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-10-pointer-lifetime`
PR: pending

## Purpose

Establish native pointer event admission and reversible terminal reporting
ownership before the recorded focus, scroll, drag and settings integrations.

## Delivered Behavior

- The existing crossterm event path admits primary press/drag/release and
  vertical wheel events with their zero-based terminal cells and modifiers.
  Hover, secondary buttons and horizontal scroll are ignored.
- Active prompt/decision owners retain their input and draft. Pointer events
  do not become document characters or keyboard commands.
- The terminal owner can reversibly acquire SGR coordinates and button-event
  tracking. Normal cleanup and Drop retain all undo obligations.
- Ordinary startup keeps reporting disabled until the working native handlers
  and controls from the recorded later slices are integrated.

## Implementation Notes

`pointer::Event` is a disposable projection of the existing decoder's event.
`Capture` records only a live view identity and revision-qualified source anchor;
there is no alternate text store, device layer, motion queue or idle timer.
The existing Input focus owner admits pointer outcomes only for editing.
Current Session feedback leaves these preparatory outcomes inert.

Reporting uses DEC 1006 and 1002. The pinned crossterm convenience enable also
requests 1003 all-motion and 1015 RXVT, so this owner emits only the two required
modes. It establishes format before tracking and restores tracking before format.
Undo is owned before a write can partially change the terminal. Acquisition
failure attempts immediate rollback; failed obligations remain for cleanup retry.
Terminal restoration still attempts every acquired operation and aggregates errors.

## Contracts Added or Changed

PTR01 uses the native decoder and existing input focus. PTR05 owns reversible
reporting without enabling hover or adding idle work.

## Files of Interest

- `src/pointer.rs`: admitted native operations and finite capture identity.
- `src/input.rs`: existing focus boundary and pointer outcomes.
- `src/terminal.rs`, `src/terminal/mouse.rs`: reporting and restoration owner.
- `tests/pointer.rs`: native admission and prompt isolation.

## Validation

- PASS: `cargo test --test pointer` checks eleven native event variants,
  preserved cells/modifiers and argument/decision isolation.

- PASS: `python3 scripts/check-source.py` retains unchanged safe-Rust and source-quality limits.

- PASS: `cargo test --lib terminal::mouse` checks exact minimal modes,
  idempotent disable/re-enable, partial-write acquisition rollback, all failed
  undo obligations, aggregated diagnostics and selective cleanup retry.
- PASS: `python3 scripts/check-pty.py --suite pointer-lifetime` checks eleven
  owned sessions: pre-entry guard, normal/toggle, each acquisition failure,
  failed rollback, each release failure, combined release failure, I/O and
  unwind. Termios, paste/cursor/attributes/screen cleanup, passive filesystem,
  descriptor closure and child reaping remain required.

BufWriter may resend a failed buffered command while attempting subsequent
cleanup. The retained-obligation assertion therefore observes fresh Drop resets
after the cleanup checkpoint, while separately checking that raw input was
already released and both operation diagnostics were retained. It does not infer
mode state from a write's return value or omit any restoration assertion.

## Risks and Follow-ups

The approved later slices integrate geometry, focus, scroll, selection and
startup/runtime controls before factory reporting activation. This preparatory
slice does not advertise those operations as available commands.
