# Slice 4: Live Terminal Interaction

Status: drafting
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-04-interaction`
PR: pending

## Purpose

Route native input and present interactive child tools through the existing
terminal emulator, host context and cell renderer.

## Delivered Behavior

Live terminal keys bypass source and modal editing in both profiles. Return,
Tab, deletion, arrows, Home/End, paging and F1-F20 use classic VT encodings,
including modified movement, child application-cursor and newline modes.
Reported keypad events respect application-keypad mode. C-x retains its host
prefix; C-c and modal C-w reach the child. M-x opens native commands and C-g
cancels a stronger host prompt or prefix before returning to the editor.
C-q and terminal-send-next-key forward exactly one normalized key literally,
including protected host keys; releases do not consume that request.

Child cursor shape and blink state travel as native frame metadata. Host
prompts and source editing retain their own presentation; terminal restoration
continues through the existing hosting-terminal owner.

## Implementation Notes

The keyboard adapter emits bounded complete sequences to the existing atomic
input queue. Extended keyboard negotiation remains disabled. A single armed
literal key is scoped to the current session and cleared on source return.
Dependencies, resource limits and process ownership remain unchanged.

## Contracts Added or Changed

The native command catalog admits terminal-send-next-key with a focused
terminal requirement. Child controls cannot accidentally start unrelated host
prefixes. A frame exposes optional native terminal cursor style metadata.

## Files of Interest

- `src/session/terminals/keyboard.rs`: classic VT keyboard encoding.
- `src/session/terminals/input.rs`: host priority and literal-next routing.
- `src/display/terminal.rs`: native cursor presentation.
- `tests/integrated_terminal/interaction.rs`: exact real-PTY input checks.

## Validation

Focused attempt `NA76K3A36DM2CK62VG3QC4QL2Q` exposed delayed Escape
cancellation returning from the child as well as closing the host prompt. The
expiration path now preserves the prior transient owner through dispatch.
The existing focus assertion remains unchanged.

Attempt `FLGGPRJOJ4AK2ERS2PCMBNIPFB` passed the raw-PTY cases after that
correction, then found two command-dispatch complexity violations. Input and
placement dispatch now delegate directory entry to its existing small owner.

Pending focused clean-head controller validation and the ordered Slice 4
commands. The raw-PTY fixture covers both startup profiles and ordinary versus
application modes, protected-key forwarding and host transient cancellation.

## Risks and Follow-ups

T4.2 rendering/tool validation and T4.3 pointer/link admission remain active
within this slice. Stable retained copy projections and paste transfer remain
in their recorded subsequent slice.
