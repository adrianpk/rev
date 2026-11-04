# Slice 2: Lower Panel and Native Commands

Status: reviewing
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-02-panel`
PR: `#249`

## Purpose

Open a real interactive shell through native commands in the lower panel,
preserving source ownership, bounded resources and the stable process identity.

## Delivered Behavior

`terminal-open` creates or selects the instance's live default terminal.
`terminal-toggle` hides or restores the same session and remembered panel
height. Creation uses the selected directory context, then the current file's
parent or startup directory. Startup captures the configured executable or
user shell with `/bin/sh` fallback, and keeps argv literal. No child starts
until an explicit command.

Both command-entry profiles invoke the same handlers. Live characters reach
the child, including printable colon and Escape; host command entry, prefixes
and contextual cancellation retain their owners. C-g returns to the previous
editor focus. Host prompts remain above a separated child panel, and the native
buffer inventory takes presentation ownership when active.

Terminal buffers carry a typed session identity and an inert, permanently
read-only text resource. Native buffer selection focuses their panel without
rebinding a source view. File saves, edits, undo, bookmarks, completion, LSP,
Org and AI source commands cannot borrow the underlying source buffer's
authority. Killing a live terminal buffer confirms owned termination; refusal
retains it, and cleanup failure preserves ownership and its diagnostic.

## Implementation Notes

The seven `[terminal]` preferences use the existing layered startup loader,
origins, explicit array replacement and whole-candidate fallback. Defaults and
the packaged sample remain complete. One-shot `[shell]` policy stays independent.

The existing composition owner reserves terminal content, status, separator
and prompt rows while retaining source minima. Height clamps to admitted
geometry; an unavailable presentation rejects creation before allocating a
child. Tiny surfaces suspend presentation. Resize checks retain the old
emulator geometry on resource failure.

The display owner copies only admitted visible emulator cells, placing each
cell at its native column with structured colors and attributes. Partial wide
cells are blanked. Rejected host-width growth cannot allocate a host-sized
terminal cell vector. Blank padding stays in the ordinary bounded frame text.
Child bytes never become host escape passthrough. Coalesced PTY readiness wakes
the existing native signal queue; idle terminal work adds no polling deadline.

Confirmed buffer closure exposed process-observation pressure with 6210 visible
process records; a read-only Python scan took 110 ms. The owned Rust scan now
reuses a 4097-byte scratch buffer and parses its four identity fields without
per-record vectors. It retains the 100 ms observation deadline, 32768-entry and
256-group bounds, 4096-byte stat limit, 500 ms grace and 2 second final cleanup
deadline. The unchanged confirmed-close regression passes after this correction.

Native editor fixtures run in isolated child test processes so asynchronous
file-observer retirement and color caches cannot disturb the engine's exact
descriptor assertions. Positive color coverage removes `NO_COLOR` only in its
owned child. The custom-binding fixture configures the independent editor and
modal Normal maps explicitly.

Task commits: T2.1 `47660f76771a`; T2.2 `c81964d0fa2f`; T2.3 `7d6e91c04df7`.
Typed resource admission is coupled to panel creation so exposed commands never
publish writable terminal transcripts.

## Contracts Added or Changed

`buffer::Resource::Terminal` preserves buffer/session identity separately from
source text. `session::terminals` owns panel selection, default reuse, focus,
startup launch data and native IO service. Composition and display remain
derived owners. Source ingress, editing and provider commands retain their
ordinary behavior after returning source focus.

## Files of Interest

- `src/config/terminal.rs`: startup catalog, admission, origins and defaults.
- `src/session/terminals.rs` and `src/session/terminals/`: native panel owner,
  live input and source authority guards.
- `src/buffer.rs`, `src/buffer/lifecycle.rs` and `src/buffer/edit.rs`: typed,
  read-only terminal resources and deterministic source replacements.
- `src/panel/composition.rs` and `src/display/terminal.rs`: separated surfaces
  and bounded native cell rendering.
- `src/term/process.rs`: allocation reduction within unchanged cleanup bounds.
- `tests/integrated_terminal/panel.rs`: real-child native panel acceptance.

## Validation

Required validation attempt `BQ37AO7KJC5IK4Q63WR7DU7CIJ` on
`bd5aec14c2a8c2450df9c40d4416e4982712a430` passed formatting and strict
all-target Clippy, then failed the required test batch. All 163 command and
153 configuration tests passed. Seven engine/resource tests failed after
process observation exceeded its unchanged 100 ms bound; retained cleanup
descriptors then disturbed the failed-launch descriptor baseline. The original
descriptor and cleanup assertions remain intact.

Further bounded read-only scans observed 8571 process records and 109-127 ms
scan times even with allocation and parser variants. A kernel `getsid`
prefilter was tested locally, but rustix 1.1.5 panics for kernel tasks whose
session ID is zero. That experimental path was removed and is absent from the
canonical candidate. Continue only after restoring the host observation
prerequisite or authorizing a separate safe dependency correction. No scan,
cleanup, resource or gate limit has been increased.

The owned read-only reproducer `target/getsid-zero.rs` confirms `/proc/2/stat`
session zero, then exits 101 at rustix's `Pid::from_raw_unchecked` positive-PID
assertion. Its captured diagnostic is `target/getsid-zero.log`. The existing
dependency-source authorization covers alacritty_terminal 0.26.0; a separate
same-version rustix correction requires a new decision before application.

The catalog fixture now uses an owned quiet `/bin/sh` script for terminal
commands instead of user shell startup or asynchronous prompt timing. All
direct/named parity assertions remain. A new session-isolation assertion also
requires closing one PTY to leave another usable and release both descriptor
sets. These fixture changes require the unchanged runtime validation after
the prerequisite is restored.

Development evidence:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration terminal -- --test-threads=1`
  passed both preference tests, including all numeric/string/argv boundaries,
  layer origins, explicit empty replacement and whole-candidate fallback.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel:: -- --test-threads=1`
  passed four native panel scenarios across the two profiles: stable process
  reuse, source guards, effective bindings and geometry/resource admission.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel::source_guards -- --exact --test-threads=1`
  passed the unchanged confirmed-close regression after the observation fix.
- Focused source quality passed ten selected new/changed files: safe Rust,
  at most five parameters, nesting at most three and cyclomatic complexity at
  most ten.

Final acceptance uses the controller receipt for the exact clean PR HEAD and
this ordered required list. Its validation ID stays outside this report so
recording evidence does not change the candidate:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal --test configuration --test commands --test panels -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

## Risks and Follow-ups

Main-pane placement, additional session creation, complete input modes and mouse
handling, retained-output navigation and editor-wide shutdown acceptance remain
in their recorded later slices.
