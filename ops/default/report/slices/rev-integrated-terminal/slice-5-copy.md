# Slice 5: Retained Output and Transfer

Status: reviewing
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-05-copy`
PR: [#252](https://forge.adrianpk.com/adrianpk/rev/pulls/252)

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

T5.2 provides retained-text yank and checked yank-pop, atomic frontend paste
under the active bracketed-paste mode, separate clear-screen/scrollback commands,
frozen line/cursor controls, child-side undo and the complete twelve explicit
send identities. C-y/M-y use retained text in live input; literal-next preserves
access to their child encodings. Explicit input commands require leaving copy
mode. Clearing history releases that session's captures and preserves its screen.

T5.3 navigates actual bounded OSC 133 prompt reports and recognized retained
compiler locations. Positive file/row/column evidence visits an existing regular
source through the terminal's creation context; missing or invalid evidence
retains the source. Explicit named and pointer hyperlink activation uses the
captured URI during frozen inspection, with the existing literal viewer argv and
scheme policy. Both 474-command inventories include every finite send identity.

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

Paste receipts grant replacement only while the exact paste remains wholly
queued. Any positive write, later input or protocol reply revokes the engine
receipt. Commands and retained-text generations additionally constrain native
yank-pop. No child-side erasure is inferred. Candidate UTF-8 plus framing shares
the existing pending-input cap and publishes atomically. Escape/control bytes
other than line breaks/tabs are rejected before queue mutation. Cursor reset
checks that the live text is represented by the frozen projection before moving.

The prompt owner retains at most 1024 reports. Full-grid scrolling adjusts
positions and evicts records beyond history. Resize, complex grid transforms and
unrepresentable pending-wrap positions invalidate or decline evidence. A frozen
copy retains up to 1024 prompt points and 1024 hyperlink spans. The 64 KiB link
budget reserves span storage before copying URI bytes, within the existing
256 KiB per-copy metadata charge. Missing captured links never fall back to a
changed live grid. Error locations recognize `path:line[:column]: error:` with
positive decimal positions, an existing regular file and represented native
source coordinates. Paths remain literal data; no shell cwd is inferred.

## Files of Interest

- `src/session/terminals/copy/`: bounded snapshot, native view, pointer, operator, search and captured evidence ownership.
- `src/term/paste.rs` and `src/session/terminals/transfer.rs`: atomic input receipts and retained-text authority.
- `src/term/prompts.rs`: bounded reported prompt positions and invalidation.
- `src/session/terminals/copy/errors.rs`: checked compiler-location visits.
- `tests/integrated_terminal/transfer.rs`, `evidence.rs` and `paste.rs`: real-PTY paste, controls, retained evidence and capacity assertions.
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

Controller receipt `MCZY75A524KKYSJWPA7Z35TKP4` passed at clean T5.2 HEAD
`a5daf0519d9f53abf38c85a73a271f002d579d1b` with this exact ordered list:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example source-check -- --file src/command/terminal.rs --file src/kill.rs --file src/session/terminals.rs --file src/session/terminals/input.rs --file src/session/terminals/source.rs --file src/session/terminals/transfer.rs --file src/session/terminals/control.rs --file src/session/terminals/copy/control.rs --file src/session/terminals/copy/snapshot.rs --file src/term.rs --file src/term/session.rs --file src/term/paste.rs`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

All 34 terminal tests passed, including unchanged resource/FD assertions and four
new transfer/control cases. All twelve selected source files passed, and both
command inventories contain all 470 implemented commands. Actual child bytes
cover both profiles, normal/application modes, bracketed/plain paste, literal
controls, copy input protection, clear-history separation and cursor rejection.
The queue fixture preserves its 4096-byte cap, atomic size/control rejection and
exact 16-byte delivery assertion. Its startup printf now preserves a literal
shell escape sequence rather than embedding a Rust NUL; it additionally asserts
negotiated bracketed mode. Receipt `7R5IXHB7NKFOVTSQAG7XESSHJP` records that
regression passing before the documentation inventory was completed.

T5.3 source checks passed all fourteen selected files at
`1b394b3eb6b299bd2a3a7868ac5bdd2f4a3acb1e` in retained batch
`7M6OAKZ2CGI5UH2AY2LCJ5427X`. That batch passed 37 of 38 terminal tests; the
link fixture passed its percent-encoded URI as a printf format and produced
spaces. Correction `77416996db98` passes the payload as data, preserving the exact
URI, activation policy and frozen/live distinction. Receipt
`AVBV5N2H6WLBQW7CXB2MFIKZRH` passed the corrected link journey, the 474-command
catalogs and diff checking.

Receipt `CMCFOMPGVG7Z2XCBTMMBJSNSU6` passed at
`c9fbc6df3c1c674a3e3261147485547de20cca9b` with this exact ordered list:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal engine::prompt_retention -- --exact --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel::evidence:: -- --test-threads=1`
- `git diff --check`

These four evidence tests cover both profiles, actual history eviction, the
1024-report cap, unchanged frozen boundaries during continued output, validated
named/pointer link activation, 64 KiB copy-link admission exhaustion, earlier
link preservation, genuine error locations and invalid/missing file/row/column
source retention. The prior resource/FD assertions remain unchanged.

The controller retains the exact final PR-head receipt after its reference
follow-up commit. Its required ordered commands are:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal --test search --test rectangles -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

## Risks and Follow-ups

Yank-pop replacement ends when input leaves the owned queue; it does not erase
arbitrary child state. Prompt and link navigation report missing admitted
evidence. Error locations use the creation context and represented source
coordinates. Canonical integration remains controller-owned.
