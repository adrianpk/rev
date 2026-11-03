# Slice 3: Native interaction and lifecycle

Status: reviewing
Delivery set: rev-shell-commands
Plan: [Native Shell Commands Delivery](../../../plan/rev-shell-commands.md)
Tracker: [Native Shell Commands Tracker](../../../tracker/rev-shell-commands.md)
Branch: `feat/rev-shell-commands-03-interaction`
PR: `#244`

## Purpose

Connect native shell commands to effective keymaps, prompt history and both
editing profiles, with checked interaction ownership and finite job lifecycle.

## Delivered Behavior

- Non-modal M-! and M-| invoke command and region display. Exact C-u M-! and
  C-u M-| sequences select insertion and filtering. Effective overrides remain
  authoritative, Org's longer C-u route survives, and modal C-u remains a motion.
- The native shell prompt retains at most 32 admitted command strings in session
  memory. M-p/M-n recall entries and restore the current draft. Cancellation,
  invalid text and unlaunched requests never add history or edit the source.
- M-x and colon share the existing six command handlers. Characterwise and
  linewise Visual ranges survive colon; block input rejects explicitly.
- Inner argument, search, selector, completion popup and modal Escape ownership
  precede job cancellation. Source C-g or the named cancel command stops the
  owned process group; another source's C-g leaves the invocation running.
- A running or stopping invocation rejects another request without queuing.
  Later input revokes completion-message authority even at the same point;
  completion retains output without replacing the user's newer interaction.
- Output reopening uses its retained native identity. Closing it reports
  unavailable. A source's new visiting association revokes automatic edits.
- Cancelled quit preserves the child and dirty source. Completed native save or
  discard decisions stop descendants and reap the leader before returning Exit.
- Native input polling and frame production observe completion. Active status
  displays running/stopping alongside timer activity. Completed work leaves no
  shell worker or periodic shell deadline, including during repeated idle frames.

## Implementation Notes

- `T3.1`: `7a898d22c8e867cf4dbfd2ab76f7b98a104d434e` adds effective keys,
  the bounded shell-specific native prompt history and actual command cases.
- `T3.2`: `9e408f818be84c70ba87fa06a821ba061c7a1a5f` fences completion echo
  after later interactions and exercises shared invocation and owned lifecycle.
- `T3.3`: `fd88363c0e76fdde404288d13c3ba82a737303cc` completes frame/status/idle integration,
  reconciles references and revises the existing Unreleased capability entry.
- Shell history is separate from named-command history. It records only commands
  admitted by the session after successful operation acquisition, with no disk
  persistence and no execution during recall or catalog discovery.
- Completion polling occurs before derived frame layout budgets so canonical
  publication never inherits a projection-only yield. There is no idle shell
  worker, independent render clock or permanent shell input timer.
- The existing active-pane activity renderer now composes timer and shell labels
  through one owner and retains disk-change notice text. Narrow panes keep their
  ordinary bounded status rendering.

## Contracts Added or Changed

- Complete the approved non-modal key sequences and 32-command session history.
- Revoke completion echo on later native input without revoking the original
  checked source target or redirecting publication.
- Observe ready shell results through the embedding frame API as well as native
  input deadlines, and display the active operation's actual cancellation state.
- Preserve all six command identities, five startup preferences, existing
  dependencies and Cargo.lock. Reconcile command/binding references and the
  single existing Unreleased behavior description.

## Files of Interest

- `src/input/shell.rs`, `src/input/keymap/defaults.rs`: native prompt history and
  effective exact key sequences, with ordinary prompt editing and cancellation.
- `src/session/shell.rs`, `src/session/events.rs`, `src/session.rs`: notification
  authority, shared operation ownership and frame completion observation.
- `src/session/status.rs`, `src/session/timers/events.rs`,
  `src/display/activity.rs`: shared active-pane activity presentation.
- `tests/shell_commands/interaction.rs`, `tests/shell_commands/lifecycle.rs`,
  `tests/shell_commands/status.rs`: real-child native interaction and idle cases.
- `tests/commands/shell.rs`, `tests/modal/shell.rs`: effective keymap/Org/modal
  contracts, Visual filtering and cancellation through the existing input owners.
- `docs/reference/commands.md`, `docs/reference/bindings.md`, `CHANGELOG.md`:
  exact delivered command interaction and visible behavior.

## Validation

Successful focused development checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands shell:: -- --test-threads=1`: 2 tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands interaction:: -- --test-threads=1`: 2 tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands lifecycle:: -- --test-threads=1`: 8 tests before the additional other-source case.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands lifecycle::other_source_cancel -- --test-threads=1`: 1 test.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts::shell:: -- --test-threads=1`: 2 tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands status:: -- --test-threads=1`: 2 tests.
- `git diff --check`: passed.

These tests use real sh/sort/tr, actual owned leaders and descendants, native
buffer/undo/file association owners and ordinary save/quit consent. The quit
driver drains existing post-save file observation after asserting immediate
process cleanup; it does not treat unrelated native deadlines as shell workers.
Visual tests use the admitted TOML section and native preserved motion column.
Block admission asserts the explicit linear-selection rejection.

After PR-reference commits and push, the final clean head receives the configured
ordered Slice 3 checks through `looper validate`. The canonical PR and controller
receipt record exact-candidate success; this report does not claim the integrated
delivery-set gate.

## Risks and Follow-ups

The next approved slice owns joint Linux PTY/resource acceptance and approachable
user guidance. Active labels are omitted when a narrow pane cannot fit them;
explicit native cancellation and output commands remain available. Shell bounds,
checked publication and conservative disk observation retain their existing
contracts. No dependency or Cargo.lock change is introduced.
