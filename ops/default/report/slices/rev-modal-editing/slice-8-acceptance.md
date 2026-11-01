# Slice 8: Terminal acceptance, resources and guidance

Status: reviewing
Delivery set: rev-modal-editing
Plan: `ops/default/plan/rev-modal-editing.md`
Tracker: `ops/default/tracker/rev-modal-editing.md`
Branch: `feat/rev-modal-editing-08-acceptance`
PR: pending

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
No capability or delivery-set completion is claimed before the controller gate.

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

Final normal-release measurements, exact-HEAD outcomes and immutable receipt
identities are recorded through the controller and Acta after the PR-reference
commit. The configured script prints its MEASURE lines, including driver elapsed
time, source/count/block costs, executable bytes and separate process resources.
Its byte comparison uses the [recorded predecessor release](../rev-language-server-support/slice-6-acceptance.md)
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
or a universal size/latency ceiling. Final integration, the unchanged full gate,
all-report closure and capability completion remain controller-owned. Main,
release and public guide publication are separate actions.
