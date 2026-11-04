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

T5.1 captures a per-attachment read-only copy projection while PTY output
continues. Native point, region, incremental search, confirmed query continuation,
word/reverse/count searches, copy operators, text objects and rectangle transfer
address that frozen text. Copying retains the selection and viewport. Return
resumes live input without sending Return. C-g cancels search, clears an active
selection, then leaves copy mode. Stronger host prompts retain first refusal.

C-c C-t enters copy mode. C-c C-c sends one interrupt; C-q C-c bypasses the
prefix. An unclaimed second key forwards the deferred C-c with that key.
Local pointer inspection freezes its output and resumes following at the tail
or on accepted live input. Explicit copy mode retains keyboard ownership.
Placement moves transfer the copy state; separate attachments capture and
navigate independently without duplicating the child.

## Implementation Notes

Private native buffers reuse coordinates, regions, rectangle transfer, literal
matching and frame projection. They never enter the visiting buffer registry,
providers or source undo history. The registered terminal resource retains its
empty text storage and stable process identity.

Native input consumes a search prompt before delivering its acceptance action;
the copy owner accepts its retained checked result. Invalid acceptance restores
the origin. Cancelled or reassigned searches cannot consume another host prompt.
Unchanged Visual state preserves pending native prefixes.

The first local pointer capture maps the VT cell actually observed before
switching to native copy layout. A stationary release retains that observation
when native grapheme layout joins multiple VT cells. Subsequent pointer motion
uses the frozen native coordinates. Failed capture does not publish legacy
selection metadata or replace source point.

## Contracts Added or Changed

A capture admits at most 1 MiB of UTF-8 text. Up to 32 attachment projections
share an 8 MiB editor charge: twice the captured bytes, physical-row mappings
and 256 KiB per projection. The fixed charge increased from the initial 64 KiB
estimate to conservatively cover native coordinate metadata and two bounded
query observations. The editor and text caps remain fixed. One bounded candidate
may be prepared before admission; rejection preserves existing views and live
state. The projection preserves physical rows and spacing, filters control
characters and keeps wide/combining transfers on native grapheme boundaries.
Emulator retention, IO bounds and process cleanup remain separately owned.

## Files of Interest

- `src/session/terminals/copy/`: bounded snapshot, native view, pointer, operator and search ownership.
- `src/session/terminals/copy.rs`: admission, commands, transfers and attachment lifetime.
- `src/buffer/lifecycle.rs`: private read-only projection construction.
- `src/display/terminal.rs`: native copy projection beside live VT cell rendering.
- `tests/integrated_terminal/copy.rs`: real-PTY stability, independent copies, native keys, Unicode and admission rejection.

## Validation

Controller receipt `NBFYD5DMKMI6RM4NRLBO7KFQLI` passed at clean implementation
HEAD `976b0dcb5ade370b536dbfb22a7b8a7b6e3fa6b4` with this exact ordered list:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example source-check -- --file src/buffer/lifecycle.rs --file src/command/terminal.rs --file src/display/terminal.rs --file src/session/terminals/copy.rs --file src/session/terminals/copy/navigation.rs --file src/session/terminals/copy/operator.rs --file src/session/terminals/copy/pointer.rs --file src/session/terminals/copy/query.rs --file src/session/terminals/copy/search.rs --file src/session/terminals/copy/snapshot.rs --file src/session/terminals/copy/state.rs --file src/session/terminals/input.rs --file src/session/terminals/placement.rs --file src/session/terminals/pointer.rs --file src/session/terminals/render.rs --file src/session/terminals/source.rs`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel:: -- --test-threads=1`
- `git diff --check`

All 21 focused panel PTY tests passed, including the 15 existing tests unchanged
and six copy tests. All 16 selected Rust source files passed the repository
complexity, parameter and nesting rules. The checks cover both input profiles,
continued output, pane independence, placement transfer, native operators/search,
linear/line/block and wide/combining transfers, Return/C-g exits, per-capture
and editor-wide rejection, rejected pointer capture and joined-emoji selection.

The joined-emoji regression first failed at `e8b8548a1123f8c062db533f8b69c299a898a500`
under receipt `FWQ5IRNBFT4KA2EAWIW7HW2OY3`: a click on the displayed X copied a
space. The correction preserves the observed VT cell and passes the same
assertion. Native search acceptance also follows the actual input-consumption
order; no assertion, resource cap or frozen gate was weakened.

These are focused T5.1 checks. The required ordered Slice 5 validation is pending
completion of T5.2 and T5.3 and the final PR head.

## Risks and Follow-ups

T5.2 terminal yank/paste, clear/send and line/cursor commands are pending.
T5.3 prompt/error/link evidence navigation and the finite send inventory are
pending. No Slice 5 PR is open.
