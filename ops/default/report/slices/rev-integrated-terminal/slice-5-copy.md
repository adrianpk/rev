# Slice 5: Retained Output and Transfer

Status: drafting
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-05-copy`
PR: pending

## Purpose

Navigate and transfer retained terminal output without borrowing source text
or changing the running child's ownership.

## Delivered Behavior

The initial implementation captures a per-attachment read-only copy projection
while PTY output continues. Native point, region and incremental search address
that frozen text; copying retains the selection and viewport. Return resumes
live input without sending Return. C-g cancels search, clears an active selection,
then leaves copy mode. Stronger host prompts retain first refusal.

C-c C-t enters copy mode. C-c C-c sends one interrupt; C-q C-c bypasses the
prefix. An unclaimed second key forwards the deferred C-c with that key.
Local pointer inspection freezes its output and resumes following at the tail
or on accepted live input. Explicit copy mode retains keyboard ownership.

## Implementation Notes

Private native buffers reuse coordinates, regions, rectangle transfer, search
and frame projection. They never enter the visiting buffer registry, providers
or source undo history. The registered terminal resource retains its empty
text storage and stable process identity. Placement moves transfer the copy
state, while separate attachments capture and navigate independently.

Native modal operator composition and complete copy navigation still require
completion. Terminal yank, replacement checks, clear/send commands and evidence
navigation remain pending under T5.2 and T5.3.

## Contracts Added or Changed

A capture admits at most 1 MiB of UTF-8 text. Up to 32 attachment projections
share an 8 MiB editor charge: twice the captured bytes, physical-row mappings
and 64 KiB per projection. One bounded candidate may be prepared before
admission. Rejection preserves existing projections and live state. The
projection preserves physical rows and spacing and filters control characters.
Emulator retention, IO bounds and process cleanup remain separately owned.

## Files of Interest

- `src/session/terminals/copy/`: bounded snapshot, native view and search ownership.
- `src/session/terminals/copy.rs`: admission, commands, transfers and attachment lifetime.
- `src/buffer/lifecycle.rs`: private read-only projection construction.
- `src/display/terminal.rs`: native copy projection beside live VT cell rendering.
- `tests/integrated_terminal/copy.rs`: real-PTY stability, independent views and rejection.

## Validation

Runtime validation pending. Test compilation initially found an unavailable
inventory accessor in the new fixture; the fixture now checks the registered
terminal's empty text and source stamp directly.

## Risks and Follow-ups

T5.1 is in progress. T5.2 and T5.3 are pending. No Slice 5 PR is open.
