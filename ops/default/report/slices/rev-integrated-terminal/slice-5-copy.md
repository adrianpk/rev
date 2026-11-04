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

Native modal word, object and line copy operators now resolve through native
coordinates and bounded per-copy find continuation. Screen alignment, buffer
and counted page navigation use the attachment's native view. Both native
rectangle commands and inclusive modal blocks retain their ordinary transfer
shapes. Modal search continuation and composed search operators still require
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

Initial controller validation `JYHDD3GI7HOIEREKLM2FV6FXPT` at
`893e82490db7bb282792c57e0d5c05e0908b7638` ran
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel:: -- --test-threads=1`:
17 passed; the new independent-copy fixture failed because screen clearing
retains older history. The corrected fixture searches the newer marker and
aligns its native row before comparing panes. The existing PTY assertions
passed unchanged, including input ownership and mouse reporting.

Source-quality diagnostics identified six control-flow violations in the new
copy dispatch, capture, input routing and rendering. Helpers now separate those
owners without raising limits. Fresh runtime and source validation is pending.

Focused controller validation `5DUFVYYYIFGO33Y7JBSOLISNTZ` passed at
`17b19b8a513efc61c8fe4af7f26cf3b5a8d71136` with this exact ordered list:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example source-check -- --file src/buffer/lifecycle.rs --file src/command/terminal.rs --file src/display/terminal.rs --file src/session/terminals/copy.rs --file src/session/terminals/copy/search.rs --file src/session/terminals/copy/snapshot.rs --file src/session/terminals/copy/state.rs --file src/session/terminals/input.rs --file src/session/terminals/placement.rs --file src/session/terminals/pointer.rs --file src/session/terminals/render.rs --file src/session/terminals/source.rs`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel:: -- --test-threads=1`
- `git diff --check`

All 18 focused PTY tests and all 12 source files passed. This is evidence for
the initial projection checkpoint; the later native operator and rectangle
changes require fresh validation.

## Risks and Follow-ups

T5.1 is in progress. T5.2 and T5.3 are pending. No Slice 5 PR is open.
