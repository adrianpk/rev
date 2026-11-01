# Slice 8: Terminal acceptance, resources and guidance

Status: delivered
Delivery set: rev-modal-editing
Plan: `ops/default/plan/rev-modal-editing.md`
Tracker: `ops/default/tracker/rev-modal-editing.md`
Branch: `feat/rev-modal-editing-08-acceptance`
PR: `#219`
Merged commit: `b7bdb59c52f58a62d105e0d0aa8c894ff47cdd48`
Canonical validation: `XVYCAJS2UTJT2VXNUKEPJC3HUD`, 464 tests and five configured checks passed
Focused validation: `EQ4GVBYOFJDOAO4QWXSBS734OB`, Clippy, source checks and Spore build passed

## Purpose

Validate startup profiles and shared native workflows through owned source,
installed-server and real-terminal boundaries, with measured work and complete
user guidance before the integrated delivery-set gate.

## Delivered Behavior

- The optional modal User Guide chapter teaches startup choice, state entry,
  navigation, composed edits, selections, jumps, panes, repeat and shared colon
  commands/completion with visible exercises. Emacs remains the introductory
  default. Markdown, chapter navigation, references and the Spore tree agree.
- A native Rust journey checks immutable profile isolation, operators, all three
  Visual shapes, native undo, jumps, dot, colon/M-x arguments and popup ownership.
  An installed rust-analyzer operates on an unsaved field in Replace, preserving
  state, one-step undo, disk contents and owned server cleanup.
- Independently configured Emacs/Vim executables run through the existing Linux
  controlling-PTY driver. Decoded input covers native completion, ordinary modal
  edits, Visual cancellation, colon/Meta command entry, pane focus, C-o/Tab, dot,
  Replace and a finite burst. Cursor styles/reset, idle silence, exact disk
  source and terminal restoration are asserted.
- Resource observations cover 16,384 source lines with Unicode/CRLF and u32-max
  counts, arithmetic overflow without publication, a 4,096-row logical block,
  128-event native/terminal bursts, executable bytes and separate driver/server
  RSS/CPU. Normal release flags, existing limits and production input batching
  remain unchanged.

## Implementation Notes

T8.1 is `caa894e3734bea595656cda7b6be07dfca8e1f01`. T8.2-T8.3 share the
acceptance entrypoint/script and are introduced together in
`e41d88d80d02adf7b1255e09c6c052efad1da0cc`; the aggregate timing output is corrected
in `6543539ffff6ca2d13446330aec7897a8613f786`. T8.4 adds the requirement map,
final pane preservation, ownership notes and this report.

The shell wrapper builds the normal release editor and Rust acceptance example
with locked dependencies and two jobs. It inventories production Rust and the
actual crate roots before executing the journey. HOME/XDG stores, source files,
Cargo state and the real server project are owned fixtures. It acquires no tools
or dependencies. The installed server is an acceptance prerequisite, with no
change to the disabled-by-default runtime setting.

The shared PTY observer exposes only a read-only query over its existing bounded
stream. No parser, source cursor, event queue or terminal lifetime is duplicated.
Both PTYs retain eight seconds and 1 MiB, the installed-server child retains its
60-second outer guard and 35-second readiness window, and native work/cleanup
uses the existing five-second bound. Only owned children are reaped.

Undo restores the native initiating point after Visual publication. Compound
fixtures therefore issue explicit 0/gg0 before a new exercise instead of assuming
that undo moves to the selection's top-left corner. Their original text, shape
and history assertions remain intact. A separately decoded Escape precedes undo;
packed Escape-u is a terminal Meta chord. Source-quality findings were resolved
by separating workflow phases and removing needless borrows, without lint
exceptions or weakened checks.

## Contracts Added or Changed

The [plan](../../../plan/rev-modal-editing.md),
[tracker](../../../tracker/rev-modal-editing.md),
[specification](../../../spec/editor-modal-editing.md) and
[paired model](../../../spec/editor-modal-editing-model.md) retain MOD01-MOD14
and the unchanged integrated gate. Baseline R01/R22-R24/R33 and CFG12-CFG15
reconciliation remains in implementation commits; historical receipts are intact.
The controller subsequently passed the integrated gate recorded below.

| Requirement | Concrete coverage |
| --- | --- |
| MOD01 Startup choice | `tests/modal.rs`, configuration startup tests, native isolation and both independently started PTY profiles. |
| MOD02 Configuration isolation | `tests/modal/keymaps.rs`, configuration whole-candidate/default-sample tests and native immutable-snapshot checks. |
| MOD03 Editing states | Native operators/repeat, Insert and Replace completion, PTY state transitions and cursor shapes. |
| MOD04 Logical navigation | `tests/modal/navigation.rs`, `viewport.rs`, `local_targets.rs` and source-bounded u32-max resource traversal. |
| MOD05 Composed operators | `tests/modal/operators.rs`, `objects.rs`, `operator_safety.rs` and native/PTY word deletion with exact undo. |
| MOD06 Visual geometry | `tests/modal/visual.rs`, `visual_safety.rs`, three native shapes and the 4,096-row Unicode/CRLF block measurement. |
| MOD07 Shared colon entry | `tests/modal/contexts.rs`, command-catalog tests, native colon/M-x argument equivalence and decoded PTY selector/pane controls. |
| MOD08 Context ownership | `tests/modal/search.rs`, `contexts.rs`, `context_safety.rs`, source-draft save/abort and popup-local cancellation. |
| MOD09 Cancellation and Escape | `tests/modal/boundaries.rs`, `completion.rs`, `context_safety.rs` and separately decoded Escape versus packed Meta input in PTYs. |
| MOD10 State maps | `tests/modal/keymaps.rs`, complete supported sample/configuration checks and the independent Emacs/Vim PTY settings. |
| MOD11 Native presentation | `tests/modal/presentation.rs`, display tests, narrow native resize, PTY source cursor, three cursor styles and reset. |
| MOD12 Bounds and preservation | Count overflow, large source/block, native and PTY 128-event bursts, native/server/PTY resource observations and first-party source quality. |
| MOD13 Checked jumps | `tests/modal/jumps.rs`, `jump_safety.rs`, native jump/undo ownership and decoded C-o/Tab and post-split backward traversal. |
| MOD14 Replacement and repeat | `tests/modal/repeat.rs`, `repeat_safety.rs`, native/PTY dot grouping and installed semantic acceptance in Replace with one undo. |

The complete supported sample and both inventories retain 409 commands and 218
configuration entries. The editor path now has eight chapters; the independent
Org path and existing assets/routes remain intact. Unreleased links the new
modal learning path without changing release metadata.

## Files of Interest

- `docs/tutorials/user-guide/11-modal-editing.md`, its index/neighbor and
  `publication/userguide/index.sptree`: the guided modal path.
- `examples/modal-acceptance.rs` and `modal_acceptance/native.rs`: native profile,
  editing, selection, command and completion workflows.
- `examples/modal_acceptance/pty.rs` and the shared
  `completion_acceptance/pty.rs`: decoded input, native cursor observations,
  retained terminal limits and cleanup.
- `examples/modal_acceptance/semantic.rs`: owned offline Rust project, actual
  installed semantic response, atomic acceptance and server lifetime.
- `examples/modal_acceptance/resources.rs` and `scripts/check-modal.sh`: source-
  bounded counts/block work, burst/idle measurements, normal release and first-
  party source inventory.
- `examples/documentation-check.rs`: the eight-chapter editor learning path and
  combined live command/configuration catalogs.

## Validation

Exploratory checks passed:

- `cargo build --locked --bin rev --example modal-acceptance --example documentation-check`:
  initial cold debug build completed in 3m 03s; later focused rebuilds passed.
- `cargo clippy --locked --example modal-acceptance --example documentation-check --example userguide-publication -- -D warnings`:
  passed after the timing output correction.
- `bash -n scripts/check-modal.sh`: passed.
- `target/debug/examples/documentation-check`: 67 reachable pages, 409 commands
  and 218 configuration entries passed before final coverage/report additions.
- The existing source-check executable passed all 778 production Rust files,
  production/acceptance roots and changed example modules; the later pane
  assertions passed focused source quality as well.

Owned debug entrypoints passed native editing, installed semantics and both
real PTY workflows, including the final pane/burst sequence. A debug semantic
observation reached the unsaved field in 343,243 microseconds with separate
server peak RSS 31,168 kB; both PTY idle observations were zero CPU ticks over
350 ms. These are debug observations, not normal-release measurements.

Both final batches passed at the merged commit above, after the PR-reference
commit, with immutable controller receipts and Acta records. Canonical checks
passed 464 tests, all native/installed-server/PTY assertions, source quality for
778 production files and the documentation check for 67 pages, 401 local links,
409 commands and 218 configuration entries. The focused Spore build rendered
the new chapter; its downloadable sample matched the source byte for byte.

The normal-release journey recorded these observations:

| Work or process | Observed result |
| --- | --- |
| u32-max counts over 16,384 logical lines, 311,296 bytes | 211,226 microseconds; overflow rejected without an edit. |
| Unicode/CRLF logical block, 4,096 rows and three columns | 26,902 microseconds; one atomic delete and exact undo. |
| Native finite burst, 128 events | 23,257 microseconds; exact point, source and history. |
| Executable | 19,351,552 bytes; 420,392 bytes above the recorded predecessor, ratio 1.022206. |
| Native driver | Peak RSS 24,636 kB; zero CPU ticks during 350 ms idle. |
| Installed Rust semantics | Readiness 266,231 microseconds; driver peak RSS 15,992 kB, separate server 31,224 kB and three CPU ticks. |
| Emacs and Vim PTYs | Peak RSS 15,528 and 15,508 kB; each recorded zero CPU ticks and no output during 350 ms idle. |
| Complete modal acceptance journey | 2,598 milliseconds, excluding compilation and the other validation commands. |

The byte comparison uses the [recorded predecessor release](../rev-language-server-support/slice-6-acceptance.md)
of 18,931,160 bytes with the same normal release profile and two build jobs.

The focused batch checks changed examples and the actual Spore build before the
canonical ordered slice batch. These checks do not run the integrated all-target
delivery-set gate.

```sh
cargo clippy --locked --example modal-acceptance --example documentation-check --example userguide-publication -- -D warnings
cargo run --locked --quiet --example source-check -- --root examples/documentation-check.rs
bash scripts/build-user-guide.sh
git diff --check
```

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test configuration --test commands --test display --example documentation-check --example userguide-publication
bash scripts/check-modal.sh
cargo run --locked --quiet --example documentation-check
git diff --check
```

## Risks and Follow-ups

Measurements apply to the recorded Linux host, Rust/Cargo 1.98.0 and installed
rust-analyzer 0.3.3089-standalone. They do not establish arbitrary-server support
or a universal size/latency ceiling. All eight slices are integrated, their
reports are delivered and the unchanged controller full gate passed. Main,
release and public guide publication are separate actions.

## Delivery-set closure

Slice 8 of 8 completed. Canonical PRs #212-#219 are merged into dev and all eight
reports are delivered. The controller passed the unchanged full gate at
`e046746acab9514117520eaa68b454f7145f6212`, immutable receipt
`2MIARCIE3HXRZVLKHS5FZLPIJB`. Its 45 top-level test targets passed 2,114 tests;
five isolated child-test executions also passed. Strict Clippy, source quality
for 778 production files, native/installed-server/PTY acceptance, the 67-page
documentation check and actual Spore build all passed.

```sh
cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-modal.sh && cargo run --locked --quiet --example documentation-check && bash scripts/build-user-guide.sh && git diff --check
```

The initial attempt `P6QOTKHGBXKPUIOEAVHH5TXFU5` failed on unresolved modal imports
and fields. After regeneration of only the package build artifacts, the
checkpoint's explicitly authorized retry passed the unchanged gate at the same
candidate. No source, acceptance assertion or gate command changed.

The integrated normal-release journey recorded these results, separately from
the earlier Slice 8 measurements above:

| Work or process | Integrated gate observation |
| --- | --- |
| Large counts / Unicode logical block / 128-event native burst | 222,301 / 29,587 / 33,845 microseconds, with exact preservation/undo assertions. |
| Executable / native driver | 19,351,552 bytes, delta 420,392 and ratio 1.022206 to the recorded baseline; driver peak RSS 24,588 kB. |
| Installed Rust semantics | Readiness 369,501 microseconds; driver peak RSS 16,132 kB, separate server 30,600 kB and four CPU ticks. |
| Emacs / Vim PTYs | Peak RSS 15,508 / 15,324 kB; each retained source and restored terminal state. |
| Idle work | Native driver and both PTYs recorded zero CPU ticks over 350 ms; PTYs emitted no output. |
| Modal journey | 2,824 milliseconds, excluding compilation and the other gate commands. |

The log records compilation steps of 27.89 seconds for Clippy, 1m 35s for test
targets and 2m 17s for the normal-release acceptance binaries. These are build
step timings, not the entire gate duration. The generated modal chapter and
byte-exact downloadable configuration were verified after the successful gate.
The owned server was reaped, both PTYs restored their terminals and fixture
owners removed their stores. No active owned fixture remained at closure.

The plan, tracker, specification/model and pending-work index now record
completion using this exact-candidate evidence. Only configured documentation
closure paths differ afterward; runtime/build inputs, earlier receipts and
deferred item 10 remain unchanged.
