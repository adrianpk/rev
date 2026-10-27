# Slice 10: Own native pointer reporting and input state

Status: reviewing
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
Confirmed activation is separate from pending undo. A failed reset makes mode
state uncertain; re-enable first finishes cleanup, then acquires a fresh pair.
A native regression reproduced and corrected the former assumption that a retained
undo flag established active tracking.

## Contracts Added or Changed

PTR01 uses the native decoder and existing input focus. PTR05 owns reversible
reporting without enabling hover or adding idle work.

## Files of Interest

- `src/pointer.rs`: admitted native operations and finite capture identity.
- `src/input.rs`: existing focus boundary and pointer outcomes.
- `src/terminal.rs`, `src/terminal/mouse.rs`: reporting and restoration owner.
- `tests/pointer.rs`, `tests/pointer/input_state.rs`: admission and keyboard/prompt isolation.
- `src/terminal/mouse/tests.rs`: mode commands and failure/retry transitions.
- `examples/terminal_probe/pointer.rs`, `scripts/configuration_ui/pointer_lifetime.py`: actual controlling-PTY decoder and lifetime checks.
- `docs/reference/terminal-input.md`: reporting boundary and deferred activation.

## Validation

Development checks passed:

- `cargo test --test pointer`: five groups cover fourteen event variants,
  preserved cells/modifiers, prompt/decision isolation, C-x continuation,
  unchanged Escape deadlines, once-only native expiry, following text and C-g.
- `cargo test --lib terminal::mouse`: five fault groups cover minimal modes,
  idempotence, partial writes, failed rollback, selective undo retry and
  fresh acquisition after uncertain cleanup.
- `cargo test --test pointer --test commands --test panels --test recovery`:
  190 native cases passed.
- `cargo test --lib --test display --test pointer`: 46 native cases passed,
  including the existing bounded ordered-input/history tests and all display
  regressions.
- `python3 scripts/check-pty.py --suite pointer-lifetime`: thirteen owned
  sessions passed. Eleven exercise pre-entry, normal/toggle, acquisition,
  rollback, release, combined cleanup, I/O and unwind. Actual SGR reports
  cover press/drag/release/wheel, modifiers, coordinates above 223, unsupported
  gestures and keyboard continuation. The ordinary binary verifies deferred
  reporting with typing, undo, prefix cancellation and exit.
- `python3 scripts/check-source.py`: unchanged safe-Rust and source-quality limits.

PTY owners retain the existing eight-second deadline and 1-MiB output limit,
owned HOME/XDG roots, raw-input and full terminal cleanup assertions, passive
filesystem, child reaping and descriptor closure. Idle decoder observation emits
no output or process CPU ticks during the existing 120 ms sample. No runtime
queue, timer or scheduling ceiling changes. The warm thirteen-session fixture
cost is under one second locally, excluding a cold build.

BufWriter may resend a failed buffered command while attempting subsequent
cleanup. The retained-obligation assertion observes fresh Drop resets after a
cleanup checkpoint, separately checking prior raw release and both diagnostics.
It does not infer successful mode cleanup from a write's return value.

Final clean-head canonical validation is recorded by the controller after PR
reference commits, in this exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test commands --test panels --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

The canonical PR and controller evidence identify the final candidate. Focused
exact-head evidence additionally checks the library/display and owned PTY suite.

## Risks and Follow-ups

The approved later slices integrate geometry, focus, scroll, selection and
startup/runtime controls before factory reporting activation. This preparatory
slice does not advertise those operations as available commands.
