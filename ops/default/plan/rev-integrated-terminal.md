# Integrated Terminal Delivery

Status: Approved
Date: 2026-11-03
Delivery set: rev-integrated-terminal
Slice strategy: behavior-first
Approved: owner, 2026-11-03; explicit request to prepare and start one autonomous Looper delivery of the approved terminal specification
Planning baseline: `32648d7875663d1cdae4e677c83185bc74e89cfc`
Spec: [Integrated Terminal Sessions](../spec/editor-integrated-terminal.md)
Model: [Integrated Terminal Model](../spec/editor-integrated-terminal-model.md)
Tracker: [Integrated Terminal Tracker](../tracker/rev-integrated-terminal.md)
Concern: [Pending Work Index](../README.md)
Full gate: `cargo fmt --all -- --check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal --test configuration --test commands --test windows --test panels --test display --test pointer --test core --test files --test recovery --test shell_commands --test external_changes -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib process -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-integrated-terminal.sh && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh && git diff --check`
Active slice: Slice 7 - Native and PTY acceptance

## Outcome and authority

Deliver TRM01-TRM11 in one retained Looper run through dev. The terminal opens
in a lower panel by default, can open directly in the main buffer area, and can
move between those placements without restarting its process. Multiple sessions
are created on demand and use ordinary buffer navigation. The lower panel shows
one selected session at a time.

The startup request authorizes the preparation commit/push, eight recorded
slice worktrees, one canonical PR per slice, controller-owned Rebase +
Fast-forward integration into dev, the fixed final gate and documentation
closure. Looper owns execution progress, canonical merges and validation
receipts. Do not request individual maintainer slice merges.

Release tagging, main promotion, mirror publication, global installation and
website deployment remain separate operations. Do not include pending tickets
or broaden AI, LSP, Org or external-extension authority in this run.

## Strategy and existing owners

Each slice delivers real behavior at its boundary and passes its recorded
checks. Slice 1 is a necessary native adapter increment, exercised with a real
PTY and shell before commands are exposed; it must not publish placeholder
handlers. Later slices extend that same owner rather than replace it.

Use existing buffer/view identities, window layout, command/prompt registry,
effective keymaps, retained-text transfer, process cleanup and observation wake
mechanisms. src/terminal.rs already owns the hosting terminal and restoration;
keep that owner distinct from a new src/term.rs and src/term/ interactive-session
owner. Fit actual repository boundaries rather than create a new framework.
Terminal-backed buffers are typed navigation resources, not writable file ropes.
Source save, edits, undo, bookmarks, completion, LSP, Org and AI entry points must
check their existing source admission. Ordinary source behavior must survive.

Use native command names from the approved specification. Non-modal M-x and
modal colon invoke the same commands in host contexts. Live child input owns
printable colon and Escape unless a host transient owns that event. Preserve
protected host recovery and literal-next forwarding in both profiles.
The finite terminal-send-* inventory covers return, tab, escape, backspace,
delete, left, right, up, down, page-up, page-down and space, alongside
terminal-send-next-key. Do not create arbitrary command families or stub
unrecognized prompt/error navigation.

## Dependency and IO decision

Pin alacritty_terminal to =0.26.0 with default-features = false. Its published
manifest declares Rust 1.85 and edition 2024, compatible with the repository's
Rust 1.98. The serde feature is unnecessary because live terminal state is not
serialized. Record the resolved transitive delta and binary-size change.
No additional direct runtime or development crate is authorized; reuse the
crate's exported VT/PTY facilities and existing std/rustix facilities.

Inspected upstream sources are the [crate manifest](https://docs.rs/crate/alacritty_terminal/0.26.0/source/Cargo.toml)
and its [event-loop implementation](https://docs.rs/crate/alacritty_terminal/0.26.0/source/src/event_loop.rs).
The crate offers Term/grid, tty::Pty, child/file access, resize and child-event
facilities. Its EventLoop uses an unbounded mpsc channel and pending write list.
Do not treat that queue as a bounded editor input path. Reuse the PTY/engine
where compatible and own bounded IO, backpressure and wake delivery in rev.
Coalesce replaceable wake/resize observations; never drop arbitrary bytes from
an accepted key sequence or split VT stream to meet a limit.

Keep safe Rust in rev. Child terminal bytes must reach the emulator and native
cell renderer, never the hosting terminal directly. Do not launch desktop
Alacritty or add a font/window backend. Advertise only implemented capabilities.

## Initial resource and preference contract

These are bounded engineering defaults, not measured performance claims.
Slice 1 verifies the allocation sketch and records actual cell/extra storage;
Slice 7 measures the integrated candidate. Preserve finite limits and report
resource rejection truthfully. Do not silently increase limits to pass a gate.

| Preference under [terminal] | Initial default | Accepted range and effect |
| --- | --- | --- |
| program | empty string | Resolve the startup user's interactive shell, then /bin/sh; a configured executable remains structured data. |
| args | empty array | At most 32 strings and 16 KiB combined; no implicit -c or string evaluation. |
| panel_height | 10 | 3-200 rows; clamp to admitted screen geometry while preserving a usable source area, or reject without changing the old layout. |
| max_sessions | 8 | 1-32 owned live/exited sessions; explicit closure releases capacity. |
| scrollback_lines | 2000 | 0-20000 requested retained rows, additionally constrained by finite cell/storage admission. |
| max_pending_input_bytes | 262144 | 4096-1048576 bytes per session, including checked paste and accepted control input. |
| max_pending_output_bytes | 262144 | 4096-1048576 bytes per session before parsing; apply PTY backpressure instead of unbounded buffering. |

Use the complete startup TOML catalog, sample, origins and all-or-default failure
behavior. Do not create persistent sessions or automatically launch shells at
startup. Shell-command preferences remain a separate table and policy.

Hard safety admission must include the live grid and retained extras, not only
line count: at most 1048576 cells per session, 2097152 cells across the editor,
4096 bytes per retained child control/title/link string and 64 combining scalars
per cell. Account for queued bytes and copy projections separately. Reject or
contain an over-limit session without losing unrelated source state. Publish
the actual supported fallback for oversized child effects. Take at most 64 KiB
of IO per service turn across sessions with round-robin progress; keep native
input/rendering responsive and use readiness wakes rather than an idle timer.

Copy projections are private read-only native buffers, outside the buffer
registry and source providers. Bound each capture to 1 MiB of UTF-8 text and
at most 32 attachments. Charge twice the UTF-8 bytes, physical-row mapping
storage and 256 KiB of native metadata per projection against an 8 MiB editor
budget. One candidate capture may be prepared before admission; failure retains
all existing projections and live emulator state. Physical rows and spacing
remain addressable by native linear, line and block coordinates. Removing an
attachment or terminal releases its projection; a placement move transfers
its frozen view without creating a new child.

Closure uses owned process-group signaling and waiting. Start with a 500 ms
grace period and a finite final reap deadline of 2 seconds; normal child exit
retains bounded final output and actual status. A cleanup failure must remain
visible and owned, never labeled successful or resolved by killing unrelated
processes. Geometry admission and capacity failure must roll back atomically.

Owner-approved incident 2 correction, 2026-11-04: Slice 2 moves process
discovery, group signaling, waiting and reaping to one owned cleanup worker per
admitted session. Replace the implementation-only 100 ms whole-host scan cutoff
with the existing 2 second requested-close deadline, measured from submission;
retain the 500 ms grace, 32768-entry/256-group/4096-byte observation bounds and
Unix-session ownership of background job groups. Publish pending cleanup and
failures truthfully, retaining buffer identity and capacity until success.
Natural exit drains bounded final output and preserves actual status. Running
commands and loops have no runtime deadline. Dependencies, descriptor/resource
assertions, frozen gate, configuration and execution budgets remain unchanged.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Native engine and PTY ownership | `feat/rev-integrated-terminal-01-engine` | `feat(terminal): own bounded Rust terminal sessions` | `ops/default/report/slices/rev-integrated-terminal/slice-1-engine.md` |
| Slice 2 | Lower panel and native commands | `feat/rev-integrated-terminal-02-panel` | `feat(terminal): open interactive shells in the lower panel` | `ops/default/report/slices/rev-integrated-terminal/slice-2-panel.md` |
| Slice 3 | Buffer placement and multiple sessions | `feat/rev-integrated-terminal-03-placement` | `feat(terminal): move live sessions between panel and buffers` | `ops/default/report/slices/rev-integrated-terminal/slice-3-placement.md` |
| Slice 4 | Live terminal interaction | `feat/rev-integrated-terminal-04-interaction` | `feat(terminal): route native input and render interactive tools` | `ops/default/report/slices/rev-integrated-terminal/slice-4-interaction.md` |
| Slice 5 | Retained output and transfer | `feat/rev-integrated-terminal-05-copy` | `feat(terminal): navigate and copy retained terminal output` | `ops/default/report/slices/rev-integrated-terminal/slice-5-copy.md` |
| Slice 6 | Session lifecycle and failure containment | `feat/rev-integrated-terminal-06-lifecycle` | `feat(terminal): preserve sessions and confirm owned shutdown` | `ops/default/report/slices/rev-integrated-terminal/slice-6-lifecycle.md` |
| Slice 7 | Native and PTY acceptance | `feat/rev-integrated-terminal-07-acceptance` | `test(terminal): verify shells tools and bounded session resources` | `ops/default/report/slices/rev-integrated-terminal/slice-7-acceptance.md` |
| Slice 8 | User guidance and joint integration | `feat/rev-integrated-terminal-08-guidance` | `docs(terminal): explain integrated terminal workflows` | `ops/default/report/slices/rev-integrated-terminal/slice-8-guidance.md` |

## Slice 1: Native engine and PTY ownership

Prerequisite: approved preparation on synchronized dev and no active Rev owner.
Requirements: TRM04, TRM08, TRM10.

- T1.1: Add the pinned minimal engine and bounded native session/PTY/process owner; establish typed identities, finite queues, grid/extra accounting, readiness and cleanup without exposing command stubs.
- T1.2: Exercise a real shell through a PTY, parsed text/colors/cursor/alternate screen, input, resize, child exit, invalid launch and cleanup; introduce tests/integrated_terminal.rs with meaningful shared fixtures.
- T1.3: Record dependency scope, baseline and candidate release size, allocation sketch and startup/idle observations; preserve the host terminal and existing one-shot process owners.

## Slice 2: Lower panel and native commands

Prerequisite: Slice 1 integrated into dev.
Requirements: TRM01, TRM06-TRM08, TRM10.

- T2.1: Admit the complete [terminal] preference catalog, defaults, sample and candidate validation; retain structured shell argv, startup directory context and no startup process launch.
- T2.2: Open a real interactive terminal in the default lower panel, reuse its stable default session, show/hide with remembered size/focus and render cells through native display ownership.
- T2.3: Admit typed terminal-backed buffers through native navigation and command catalogs; guard source-only file/edit/completion/LSP/Org/AI actions without changing ordinary source buffers, and update Unreleased when the user-visible path works.

## Slice 3: Buffer placement and multiple sessions

Prerequisite: Slice 2 integrated into dev.
Requirements: TRM01-TRM03, TRM07-TRM08.

- T3.1: Implement direct terminal-new-buffer and bidirectional move commands; retain process/emulator/buffer identity, working directory, alternate-screen state and output, with atomic geometry failure.
- T3.2: Create multiple sessions only on explicit invocation; implement independent names, selection/list/rename and one selected lower-panel session, retaining hidden sessions and the instance-global default.
- T3.3: Support ordinary pane views with one resize authority and independent copy/viewport state, checked focus changes, tiny-screen recovery, cwd entry commands and other-pane selection; verify no duplicate child on move/toggle.

## Slice 4: Live terminal interaction

Prerequisite: Slice 3 integrated into dev.
Requirements: TRM04, TRM06-TRM07, TRM10-TRM11.

- T4.1: Route printable keys, child control keys, Return/Tab/deletion/arrows/paging, cursor shape and application modes in both startup profiles; preserve host recovery/prefix commands, literal-next and contextual transient cancellation.
- T4.2: Render real shell and full-screen TUI screen/style/cursor/alternate-screen/resize changes without source repaint glitches, raw escape passthrough, focus stealing or a busy idle loop.
- T4.3: Route child-enabled mouse/wheel input and native local selection/scrolling, Shift bypass and explicit validated link activation; deny child clipboard/host effects and keep printable colon/Escape under live child authority.

## Slice 5: Retained output and transfer

Prerequisite: Slice 4 integrated into dev.
Requirements: TRM06-TRM07, TRM11.

- T5.1: Provide bounded scrollback, stable per-view copy mode, selection/search, copy with retained viewport, tail-follow suspension/resumption and the specified Return/C-g exits while output continues.
- T5.2: Implement native terminal yank/yank-pop with checked replaceability and bracketed paste, separate clear-screen/clear-scrollback operations, line/cursor navigation and explicit send/undo operations.
- T5.3: Complete prompt/error/link navigation using actual admitted evidence such as OSC 133 and recognized locations; report unavailable evidence honestly. Complete the finite terminal-send-* command inventory without placeholders.

The admitted paste replacement policy owns only the exact preceding paste still
wholly queued in rev. Any successful write, later accepted input, protocol reply,
unrelated command or changed retained-text generation revokes replacement.
This policy does not infer child-side line editing from cursor position or text.
Paste candidates, including active bracket framing, share the configured pending
input cap; rejection publishes no partial bytes. Copy cursor reset requires a
current-text representation check before moving the frozen point.

Prompt evidence retains at most 1024 actual OSC 133 A reports per session and
follows ordinary full-grid scrolling. Unsupported grid transforms, resize and
unrepresentable pending-wrap positions invalidate or decline evidence. Frozen
copy metadata includes up to 1024 prompt points and 1024 hyperlink spans within
a 64 KiB link allocation budget, covered by the existing 256 KiB per-copy fixed
charge. Recognized `path:line[:column]: error:` locations require positive
positions and an existing regular file in the creation context; no evidence
means no source jump or URI fallback to a changed live grid.

## Slice 6: Session lifecycle and failure containment

Prerequisite: Slice 5 integrated into dev.
Requirements: TRM02-TRM03, TRM05, TRM07-TRM09, TRM11.

- T6.1: Keep visibility separate from process lifetime, retain actual exited status/output, replace an exited default without erasing its buffer and release capacity only after explicit closure.
- T6.2: Confirm live-buffer termination and editor-wide session shutdown once, preserve declined choices, own process-group cleanup/reaping and handle failure without losing unsaved source or unrelated sessions.
- T6.3: Exercise queue/grid/extra/session admission, output floods, blocked input, incomplete controls, rapid resize/close, failed relocations and native shutdown/error paths under finite deadlines and fair readiness service.

## Slice 7: Native and PTY acceptance

Prerequisite: Slice 6 integrated into dev.
Requirements: TRM01-TRM11 together.

- T7.1: Add examples/integrated-terminal-acceptance.rs and scripts/check-integrated-terminal.sh using existing Rust/Linux PTY support and isolated fixtures; exercise real shells and installed lazygit in a disposable local repository without network or user Git state.
- T7.2: Verify both profiles, multiple sessions, panel/main-buffer movement, continued child state, alternate screen, focus/literal-next, mouse, copy/paste, protected source admission and complete host restoration through real terminal journeys.
- T7.3: Measure binary delta, idle/active CPU, RSS, bounded retained output and queue/flood behavior, process/FD cleanup and fairness. Add focused source-quality coverage, share fixture setup and record real observations instead of guessed performance claims.

## Slice 8: User guidance and joint integration

Prerequisite: Slice 7 integrated into dev.
Requirements: TRM01-TRM11 and complete user-facing discovery.

- T8.1: Add a short ordinary terminal chapter to the editor learning path, a task guide and exact command/configuration/input/lifecycle reference; explain panel versus buffer, moving a live lazygit session, multiple terminals and returning to the editor with small visual examples.
- T8.2: Keep Markdown the source of truth for Spore, update indexes/catalog checks and both learning-path navigation, document shell-command/filter distinction and finalize user-facing Unreleased notes without a version bump.
- T8.3: Reconcile accepted baseline/model clauses and all requirement evidence, run the focused slice checks and prepare clean dev for the controller-owned combined gate; do not claim completion before that receipt.

## Exact validation contract

Every listed command is ordered and required. Use the same low-footprint
profile for all Cargo work:

`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`

Each slice runs cargo fmt --all -- --check, that profile with cargo clippy
--locked --all-targets -- -D warnings, the slice-specific tests below, that
profile with cargo run --locked --quiet --example documentation-check, and
git diff --check last. The generated controller configuration must contain
these full literal commands in this exact order, not symbolic aliases.

| Slice | Required test command after the shared profile prefix | Additional command before documentation/whitespace |
| --- | --- | --- |
| Slice 1 | cargo test --locked --test integrated_terminal -- --test-threads=1 | None. |
| Slice 2 | cargo test --locked --test integrated_terminal --test configuration --test commands --test panels -- --test-threads=1 | None. |
| Slice 3 | cargo test --locked --test integrated_terminal --test windows --test panels --test core -- --test-threads=1 | None. |
| Slice 4 | cargo test --locked --test integrated_terminal --test display --test pointer -- --test-threads=1 | Profile + cargo test --locked --test modal contexts:: -- --test-threads=1. |
| Slice 5 | cargo test --locked --test integrated_terminal --test search --test rectangles -- --test-threads=1 | None. |
| Slice 6 | cargo test --locked --test integrated_terminal --test files --test recovery --test shell_commands --test external_changes -- --test-threads=1 | Profile + cargo test --locked --lib process -- --test-threads=1. |
| Slice 7 | cargo test --locked --test integrated_terminal -- --test-threads=1 | Profile + bash scripts/check-integrated-terminal.sh. |
| Slice 8 | cargo test --locked --test integrated_terminal --test commands --test configuration -- --test-threads=1 | Profile + bash scripts/build-user-guide.sh, after documentation-check. |

## Full gate and cost

Run one selected integration gate after eight integrations. It protects the
changed session, resource-buffer, layout, rendering, input, configuration and
source-admission contracts; it does not chain historical feature journeys or
run every Org/LSP/AI test. New terminal acceptance must cover those features'
source exclusions. Batch native regressions together with one thread and reuse
the Cargo artifacts. New checks use Rust and Bash, not Python.

The exact gate is the Full gate metadata line above.
The first dependency/release build is expected to dominate; allow minutes for
the warm native batch and PTY/resource scenarios, and record actual timing in
Slice 7. No hours-long soak or production timeout belongs in routine acceptance.
Finite scenario deadlines must not be increased to hide hangs.

## Final gate, recovery and closure

After all eight slices are integrated, the controller owns the exact combined
gate on clean dev. At most two bounded correction PRs may repair affected
terminal acceptance under the same frozen commands, run, conversation, budgets
and reports. Use branch prefix fix/rev-integrated-terminal-validation and PR
title fix(terminal): repair approved integrated terminal acceptance, retaining
Slice 8's report. Do not add slices, dependencies or Looper fixes, weaken
assertions, change resource bounds silently or bypass a merge checkpoint.

The local bounded-v1 policy permits two diagnostics per family, six per run,
two failed repairs per family and two environmental retries, with ticket paths
excluded. Allowed repair paths must be limited to this delivery's native
terminal/session/buffer/layout/input/display/configuration owners, directly
affected tests, terminal acceptance/guidance and these operational documents.

Only these documentation paths may change in final closure:

- ops/default/plan/rev-integrated-terminal.md
- ops/default/tracker/rev-integrated-terminal.md
- ops/default/report/slices/rev-integrated-terminal/slice-1-engine.md
- ops/default/report/slices/rev-integrated-terminal/slice-2-panel.md
- ops/default/report/slices/rev-integrated-terminal/slice-3-placement.md
- ops/default/report/slices/rev-integrated-terminal/slice-4-interaction.md
- ops/default/report/slices/rev-integrated-terminal/slice-5-copy.md
- ops/default/report/slices/rev-integrated-terminal/slice-6-lifecycle.md
- ops/default/report/slices/rev-integrated-terminal/slice-7-acceptance.md
- ops/default/report/slices/rev-integrated-terminal/slice-8-guidance.md
- ops/default/spec/editor-integrated-terminal.md
- ops/default/spec/editor-integrated-terminal-model.md
- ops/default/spec/rev-editor.md
- ops/default/spec/rev-editor-model.md
- ops/default/README.md

Mark the selected specification/model and plan/tracker Completed only after the
controller's successful exact-candidate gate. Record all delivered reports,
gate identity and source/documentation closure difference. Stop at clean,
synchronized dev; main, public mirrors and tagged publication are not this run.

## Storage discipline

Use timestamped worktrees under the standard project worktree root, each with
its own Cargo target; never share mutable build output with another branch.
Check for at least 1 GiB free before builds and use one build job without
incremental/dev/test debug artifacts. Newly created targets for this run may be
cleaned after their slice is integrated and its worktree is clean with no live
build or validator. Preserve source worktrees, reports, validation receipts and
unrelated targets. Do not change global configuration or clean other projects.
