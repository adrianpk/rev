# Slice 1: Startup profile and Insert state

Status: reviewing
Delivery set: rev-modal-editing
Plan: `ops/default/plan/rev-modal-editing.md`
Tracker: `ops/default/tracker/rev-modal-editing.md`
Branch: `feat/rev-modal-editing-01-profile`
PR: pending

## Purpose

Admit a startup-only interaction profile and native Normal/Insert editing while
preserving default Emacs input and shared editing owners.

## Delivered Behavior

- `editor.interaction_profile` selects Emacs or Vim once. Omission retains Emacs;
  invalid profiles or mappings reject the entire candidate into native defaults.
- Vim starts in Normal, rejects unmatched text/paste and admits i/a/I/A/o/O.
  Insert uses native edits, Unicode coordinates and ordinary history. Escape/C-[
  returns to Normal without decrementing or replacing the native point.
- Vim state factories are independent of Emacs overrides. Local Org/component
  maps retain priority; C-g and M-x recovery remain available. Changing the
  selected native view or buffer retires incompatible text-entry state.
- The selected pane shows NORMAL/INSERT, with N/I below six cells. Optional
  block/bar cursor feedback owns its reset on exit, including failed writes.
- The complete provisioned sample, configuration/default catalogs, commands,
  bindings, terminal/status references, baseline clauses and Unreleased agree.

## Implementation Notes

The existing input owner retains the immutable profile, modal state and native
view/buffer identity. State commands share the canonical command catalog and
session dispatch. Line opening publishes one prepared edit with an explicit
native cursor; read-only failure preserves text, point, state and history.
Logical targets reuse checked coordinates and the existing cooperative layout
queue. First-text entry preserves combining clusters, tabs and line separators.
Cursor shape is optional presentation; status remains the observable state.

T1.1 is commit `219a5dfac8b3`; T1.2 is `4a70307ef11f`; T1.3 is
`516f918e9d5e`. T1.4 and its focused boundary corrections are introduced with
this report. The tracker records task completion; canonical PR/controller
receipts own integration and exact-candidate validation.

## Contracts Added or Changed

MOD01-MOD03 and the Slice 1 portions of MOD09-MOD11 are implemented.
R01, R24, R33 and CFG12-CFG15 recognize startup profiles and state-local input;
the paired models retain native ownership. Historical gate identities remain
unchanged. Four later-state keymap tables have empty factories and do not add
unimplemented commands or runtime profile switching.

## Files of Interest

- [Plan](../../../plan/rev-modal-editing.md) and [tracker](../../../tracker/rev-modal-editing.md).
- `src/config.rs`, `src/config/merge.rs`, `src/config/validate.rs`: typed startup admission.
- `src/input/modal.rs`, `src/input/keymap/vim.rs`: selected state and local map resolution.
- `src/session/modal.rs`, `src/session/edit.rs`: native state entry and atomic line opening.
- `src/display/status.rs`, `src/terminal/cursor.rs`: mandatory state and reversible optional cursor feedback.
- `tests/modal.rs`, `tests/modal/`: owned configuration, input, source and presentation coverage.

## Validation

Successful development checks:

- `cargo test --locked --test modal --test configuration --test commands`: passed
  with 8 modal, 139 configuration and 161 command tests before presentation.
- `cargo test --locked --test modal`: passed with all 11 modal tests after
  presentation and source-boundary corrections.
- `cargo run --locked --quiet --example source-check -- --source src/config.rs --source src/input/keymap.rs --source src/input/keymap/defaults.rs --source src/session.rs --source src/session/modal.rs`:
  passed after correcting changed dispatch/target complexity.

The final clean PR candidate uses the ordered commands below. Looper records
its exact HEAD and outcomes before handoff; the canonical PR carries that
candidate's validation list. The report introduction precedes that receipt.

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test configuration --test core --test commands --test display --test windows
cargo test --locked --lib terminal::cursor::tests::restoration
cargo run --locked --quiet --example source-check -- --source src/command.rs --source src/command/modal.rs --source src/config.rs --source src/config/merge.rs --source src/config/project.rs --source src/config/validate.rs --source src/display.rs --source src/display/status.rs --source src/input.rs --source src/input/context.rs --source src/input/dispatch.rs --source src/input/keymap.rs --source src/input/keymap/availability.rs --source src/input/keymap/defaults.rs --source src/input/keymap/vim.rs --source src/input/meta.rs --source src/input/modal.rs --source src/input/org.rs --source src/session.rs --source src/session/edit.rs --source src/session/layout.rs --source src/session/modal.rs --source src/session/mode.rs --source src/terminal.rs --source src/terminal/cursor.rs --source tests/configuration/resolution_contract.rs --root tests/modal.rs --source tests/modal/keymaps.rs --source tests/modal/presentation.rs
cargo run --locked --quiet --example documentation-check
git diff --check
```

## Risks and Follow-ups

The exploratory repository-wide `python3 scripts/check-source.py` also found
existing violations in unchanged examples/tests, including
`examples/bookmark_acceptance/journey.rs` and `examples/userguide-publication.rs`.
Changed Rust files receive the focused source check above after their corrections.
Real-terminal modal acceptance and the remaining modal states/grammar follow the
approved later slices; this report does not claim their completion.
