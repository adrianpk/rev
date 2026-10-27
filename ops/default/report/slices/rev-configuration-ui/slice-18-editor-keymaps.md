# Slice 18: Dispatch configurable editor bindings

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-18-editor-keymaps`
PR: [#67](https://forge.adrianpk.com/adrianpk/rev/pulls/67)
Integrated: `acebc81731bb936cb14d5b13f6060546d167ca7c`

## Purpose

Dispatch supplied editor keymaps through existing Rust command handlers while
preserving inherited aliases, input phases, transient ownership and canonical edits.

## Delivered Behavior

- Library input/session construction copies the admitted maps. Editor chords
  resolve through the complete effective editor map before ordinary text fallback;
  exact replacement cannot fall through to an old hardcoded handler.
- Omitted entries retain independent inherited aliases. Omitting an override in
  a new snapshot restores its built-in. Invalid candidates supply complete defaults
  and cannot partially activate a valid alias from the rejected candidate.
- Custom sequences retain at most eight normalized chords and display their full
  pending label. Complete sequences emit the existing Command identity once;
  mismatches consume the prefix and suffix without inserting document text.
- C-g, paste, prompt acquisition/acceptance and workspace ownership changes clear
  all pending chords. Escape keeps its finite Meta policy and expiry clears the
  whole sequence. Resize alone retains a pending editing prefix.
- Repetition follows the resolved action. Single-chord editing/motion aliases
  retain native repetition; prefix completions and session/prompt transitions
  require fresh presses. Releases do not dispatch. Every Enter alias requires a
  fresh prompt acceptance after normalization.
- Literal Unicode/control-looking paste bypasses bindings and uses one canonical
  edit. Read-only refusal, points, revisions, history and saved-state identity use
  the same owners as M-x. Editor mappings do not change native prompt/tree keys.

## Implementation Notes

- T18.1: `a6ecace9ee64235fe8ec324fa63d418d4985efe7` routes editor input through
  admitted maps, adds normalized pending labels and action-based repetition, and
  centralizes prefix cleanup across existing input acquisition paths.
- T18.2: `2422f5d109f74ebdd0b818854a45368b367c1711` preserves exact replacement
  and native aliases, synchronizes workspace ownership after direct commands and
  resize, and covers prompt/workspace prefix reset and context isolation.
- T18.3: `3967342f2753d44ec19a5f528df94a852010f2d6` covers phase boundaries, maximum
  sequences, literal paste, invalid candidates, read-only admission and M-x parity.
- A focused regression reproduced held Control-m/j accepting a prompt because
  repeat admission inspected the raw key. Phase admission now checks the
  normalized code, and all Enter aliases preserve the existing fresh-key rule.
- Pending chords belong to Input, separate from canonical text. Normalized runtime
  sequences admit only supported key codes before ordered map lookup. Existing
  built-in prefix identities remain available to native hosts.

## Contracts Added or Changed

- `Input::with_settings` consumes admitted editor maps as well as prompt/history
  policy. The default constructor uses complete compiled maps.
- `Input::prefix_label` exposes the full normalized pending label; `Prefix::Sequence`
  identifies a custom prefix without changing existing built-in prefix identities.
- Runtime sequence lookup is bounded by the same eight-chord limit as parsing.
  Unsupported native key codes never enter the closed sequence ordering domain.
- `Command` owns repeat admission for configured editor actions. Input normalizes
  aliases before context phase guards; active prompt/tree handlers retain routing.
- Session tracks workspace ownership at event, direct-command and resize boundaries.
  Ownership changes clear pending input and suppress held suffixes.

## Files of Interest

- `src/input/editor.rs`, `src/input.rs`: editor dispatch, repeats and prefix lifecycle.
- `src/input/keymap/sequence.rs`, `src/input/key.rs`: bounded runtime lookup and labels.
- `src/command.rs`: repeatable existing action identities.
- `src/input/meta.rs`, `paste.rs`, `palette.rs`, `inventory.rs`, `search.rs`, `replace.rs`:
  shared pending-state cleanup at existing acquisition/cancellation boundaries.
- `src/session.rs`, `session/events.rs`: workspace synchronization and prefix feedback.
- `tests/commands/editor_keymaps.rs`: configured input through native owners.
- `docs/reference/bindings.md`, `terminal-input.md`: supplied editor dispatch semantics.

## Validation

Development checks:

- `cargo test --test commands keymap::`: passed; normalized syntax and native defaults.
- `cargo test --test commands editor_keymaps::`: passed; 15 configured behavior tests.
- `cargo test --test commands key_phases`: passed.
- `cargo test --test commands prompt_acquisition`: passed.
- `cargo test --test commands terminal::`: passed; eleven Meta and phase regressions.
- `cargo test --test commands editor_keymaps::acceptance_aliases`: initially failed, reproducing held normalized Enter acceptance; passed within the corrected configured suite.
- `cargo clippy --lib --test commands -- -D warnings`: passed.
- `cargo run --quiet --example source-check -- --source src/input/editor.rs --source src/input.rs --source src/input/keymap/sequence.rs --source src/session/events.rs --source tests/commands/editor_keymaps.rs`: passed.
- `cargo run --quiet --example source-check -- --source src/input.rs --source tests/commands/editor_keymaps.rs`: passed after the phase correction; nesting at most two.

Final evidence `6MGAJQQHEPCP2DO2K2K4H2AR5X` passed all nine ordered checks
on clean head `acebc81731bb936cb14d5b13f6060546d167ca7c`: 344 native tests,
zero skips. The controller integrated PR #67 into dev at that same commit.
Acta merge event: `01M5YYS776WR2PCKDYPFA5EC46`.
Required commands precede focused snapshot, search/query-replace and tree checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test commands`
- `cargo test --test editing`
- `python3 scripts/check-source.py`
- `cargo test --test configuration`
- `cargo test --test search`
- `cargo test --test tree`
- `git diff --check`

## Risks and Follow-ups

Configured transient/tree dispatch remains assigned to Slice 19; their native
routing is preserved here. Executable startup discovery remains assigned to
Slice 20. Immutable supplied snapshots activate editor aliases now; this slice
adds no reload or file-watching behavior.
