# Slice 22: Navigate and execute command candidates

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-22-command-selection`
PR: [#71](https://forge.adrianpk.com/adrianpk/rev/pulls/71)
Integrated: `0bb5aadf82d19849857d0dea63c62dd58f7ee511`

## Purpose

Navigate the vertical M-x list independently of session history, complete the
selected name and invoke its existing command handler through the native input
and argument owners.

## Delivered Behavior

- Up/Down and C-p/C-n move the candidate selection without rewriting the query.
  Movement stops at either endpoint. The selected row remains visible through
  list scrolling and resize, including suspension when no candidate rows fit.
- M-p/M-n recall accepted command history and restore the draft independently
  of candidate movement. Existing recency, deduplication and retention apply.
- Tab fills the selected name without executing or cycling it. Its original
  matching set remains available for subsequent candidate navigation.
- Return invokes an explicitly selected candidate; otherwise it accepts an
  exact typed name or the first match for a nonempty query. An untouched empty
  query reports `No command entered`. Explicit navigation or completion can
  choose from the unfiltered list. No-match navigation/completion retains the
  prompt; no-match Return reports `Unknown command` and restores the workspace.
- Actual query edits and history recall reset selection and scroll. Cursor-only
  movement, no-op deletion, empty paste and refused input retain list state.
- Selected commands use the same native handlers and argument stages as named
  commands and bindings. Held/released acceptance cannot consume another stage
  or dispatch twice. C-g discards command entry and restores the retained editor
  or tree focus, selection, view, buffer stamp and history.
- Configurable candidate, completion, acceptance and history aliases use those
  same owners. The fixed editor/M-x command catalog remains at 62 commands.

## Implementation Notes

- T22.1: `9e7f3ebb897adc59c32c8a23feb732c187a3ab4a` separates candidate and
  history input, completes the selected name and routes selected acceptance.
- T22.2: `97410b79146cfb26605381ea31099b95116f8609` distinguishes text changes
  from cursor/no-op outcomes and tests explicit, exact, empty and unmatched input.
- T22.3: `629b4aa7151f531faf25e39e90cf6246eb57294a` adds custom-context,
  selected argument, cancellation and Escape-history
  regressions, and extends the two owned RGB/NO_COLOR terminal journeys.
- Palette retains an explicit-choice flag separately from its default visual
  highlight. Navigation can override a previously completed exact name without
  changing the draft. Text changes retire that choice before matching again.
- History actions are context-only Rust identities. Custom aliases translate to
  the native Meta history operation without another key lookup or event dispatch.
  Other prompt and decision stages retain their existing admission behavior.

## Contracts Added or Changed

- UI02: separate candidate/history navigation, selected completion/dispatch,
  explicit empty/no-match semantics, retained geometry and workspace cancellation.
- CFG12-CFG13: minibuffer maps additionally admit `minibuffer-history-previous`
  and `minibuffer-history-next`, inherited on M-p/M-n. Candidate actions retain
  `minibuffer-previous` and `minibuffer-next` on Up/Down and C-p/C-n.
- Baseline R23: selected M-x invocation reaches the shared handler exactly once,
  continues native arguments and preserves repeat/cancellation guards.

## Files of Interest

- `src/input/palette.rs`, `palette/history.rs`: independent selection and history.
- `src/command/binding.rs`, `src/input/context.rs`, `keymap/defaults.rs`: targets
  and canonical operation routing.
- `tests/commands/selection.rs`, `context_keymaps/flows.rs`: eight new native
  regressions with explicit state, phase, argument and cancellation assertions.
- Existing palette, preference and vertical tests: updated interaction contracts.
- `scripts/check-command-list.py`, `scripts/check-pty.py`: native terminal input
  for candidate navigation, Meta history and selected argument dispatch.
- Command/binding references and `CHANGELOG.md`: delivered interaction behavior.

## Validation

Successful development checks:

- `cargo test --test commands --test configuration`: 120 command and 70 configuration tests passed after routing.
- `cargo test --test commands selection::`: five selection/history regressions passed.
- `cargo test --test commands candidate_aliases`: custom candidate/history targets passed.
- `cargo test --test commands selected_`: selected completion, native/aliased file arguments and editor/tree cancellation passed.
- `cargo clippy --lib --test commands --test configuration -- -D warnings`: passed after routing.
- `cargo clippy --lib --test commands -- -D warnings`: passed after final native regressions.
- `cargo run --quiet --example source-check -- --source src/command/binding.rs --source src/input/context.rs --source src/input/palette.rs --source src/input/palette/history.rs --source tests/commands/palette.rs --source tests/commands/preferences.rs --source tests/commands/vertical.rs`: passed.
- `cargo run --quiet --example source-check -- --source src/input/palette.rs --source tests/commands/selection.rs`: passed.
- `cargo run --quiet --example source-check -- --source tests/commands/context_keymaps/flows.rs --source tests/commands/selection.rs`: passed; nesting at most two.
- `python3 scripts/check-command-list.py`: two owned terminal sessions passed, zero skips; selected native write-file saved exact fixture bytes.

Final readiness evidence belongs to the clean PR head after the PR-reference
follow-up commit. The controller validation ledger records that head and these
ordered commands. Evidence `4VLTMTFP4447WUE5MDYDLQATXT` passed all eight
checks at clean head `0bb5aadf82d19849857d0dea63c62dd58f7ee511`: 217 native
tests and two terminal sessions, zero skips. PR #71 integrated into dev at the
same head. Acta merge event: `01M5Z227PFE13YDWPTRSM0N4RR`.

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test commands`
- `cargo test --test panels`
- `python3 scripts/check-source.py`
- `cargo test --test configuration`
- `python3 scripts/check-command-list.py`
- `git diff --check`

## Risks and Follow-ups

None recorded.
