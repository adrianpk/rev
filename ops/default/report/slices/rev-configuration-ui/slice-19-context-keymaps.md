# Slice 19: Dispatch transient and tree bindings

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-19-context-keymaps`
PR: [#68](https://forge.adrianpk.com/adrianpk/rev/pulls/68)

## Purpose

Dispatch supplied minibuffer and tree keymaps through the existing prompt,
search, command, inventory, confirmation and filesystem action owners.

## Delivered Behavior

- Supplied snapshots activate all three independent context maps. Transient input
  selects the minibuffer map while retaining the underlying editor/tree workspace.
  Exact reassignment replaces only its normalized chord; other aliases inherit.
- The bounded sequence owner handles editor, prompt and tree prefixes uniformly.
  Pending labels expose normalized chords. Mismatches consume their suffix;
  prompt acquisition/acceptance, cancellation, paste and workspace transitions
  clear all chords. Escape retains its finite deadline and expiry semantics.
- Prompt aliases retain grapheme editing, scalar limits and atomic argument
  consumption. M-x aliases preserve completion cycling, accepted-command history
  and its navigation draft. Find-file aliases edit and accept the existing path
  argument. Search repeats/acceptance and literal replacement tabs keep their
  original state machines.
- Inherited minibuffer keys preserve their audited stage distinctions, including
  inventory arrows versus command-history Control-p/n. Explicit aliases select
  the named stage action. Prompt actions cannot reach document editing handlers.
- C-g remains reserved in every context. Decisions admit only native fresh explicit
  answers and cancellation aliases. Only cancel sequences may wait in a decision;
  a mismatched y/n suffix cannot become consent. Paste, held answers and mapped
  acceptance cannot confirm replacement, deletion or recursive deletion.
- Tree aliases use existing selection, open, refresh, create, rename and delete
  owners. Navigation can repeat, while mutations, prefix completion, prompt
  acceptance and workspace transitions require fresh presses. Tree focus rejects
  document text/paste; canceled transient operations restore the retained tree.

## Implementation Notes

- T19.1: `d025410560191c202f5e59d3e786d82aa3fea936` generalizes bounded map
  dispatch, routes minibuffer aliases to existing stage operations and tree aliases
  to existing command identities, and removes obsolete fixed binding decoders.
- T19.2: `71406c2e227812c86157bffbbaff408e9b34421a` guards decision admission,
  enables bounded cancellation sequences without consuming consent suffixes, and
  tests replacement/filesystem decisions and reserved cancellation.
- T19.3: `6ef075a601e28560fe344ecc3111b6c0134c2121` adds the remaining native workflow
  regressions and documents supplied context dispatch.
- Map entries retain whether a chord is explicitly configured. Inherited entries
  preserve existing stage-specific key distinctions; custom targets translate
  directly to the existing owner's canonical operation without synthesizing an
  event or re-entering lookup.
- Initial workflow assertions were corrected to match verified native behavior:
  backward search excludes the current match, creation requires a selected
  directory, and decision feedback retains the pending question. No underlying
  search or filesystem admission behavior was changed.

## Contracts Added or Changed

- `Input::with_settings` dispatches the complete admitted editor/minibuffer/tree
  maps. The default constructor dispatches the same compiled maps and defaults.
- One sequence owner enforces the existing eight-chord limit in every context.
  Confirmations inspect only cancellation prefixes and never map acceptance to
  affirmative answers.
- Action-based repeat admission covers prompt aliases and tree navigation while
  retaining fresh acceptance, confirmation and mutation boundaries.
- Existing Command and prompt-only target names remain unchanged; aliases add no
  new editing actions, filesystem operations or M-x catalog entries.

## Files of Interest

- `src/input/dispatch.rs`, `context.rs`, `input.rs`: bounded resolution, active map,
  stage delegation, repeat admission and explicit decision guards.
- `src/input/keymap/map.rs`: configured-entry identity and cancel-prefix lookup.
- `src/input/key.rs`, `tree.rs`: normalized key and existing repeat suppression.
- `tests/commands/context_keymaps.rs`, `context_keymaps/guards.rs`,
  `context_keymaps/flows.rs`: eighteen configured behavior regressions.
- `docs/reference/bindings.md`, `terminal-input.md`: supplied dispatch and phases.

## Validation

Development checks:

- `cargo test --test commands --test tree --test search`: passed after context routing; 185 native regressions.
- `cargo test --test commands context_keymaps::`: passed; eighteen configured behavior tests.
- `cargo test --test commands palette::context_reference`: passed; documented defaults retain the audited target catalogs.
- `cargo clippy --lib --test commands --test tree --test search -- -D warnings`: passed after context routing.
- `cargo clippy --lib --test commands -- -D warnings`: passed after all configured workflow tests.
- `cargo run --quiet --example source-check -- --source src/input/dispatch.rs --source src/input/context.rs --source tests/commands/context_keymaps.rs`: passed.
- `cargo run --quiet --example source-check -- --source src/input/context.rs --source src/input/dispatch.rs --source src/input/keymap/map.rs --source tests/commands/context_keymaps/guards.rs`: passed.
- `cargo run --quiet --example source-check -- --source tests/commands/context_keymaps/flows.rs --source tests/commands/context_keymaps/guards.rs --source tests/commands/context_keymaps.rs`: passed; nesting at most two.

Final evidence `ZG7VQKTLCDBWMCIRGAXJPQMUJS` passed all nine ordered checks
on clean head `c26ba1b857f90c8baf2cb6cf2b3e75c40879a3ec`: 362 tests, zero skips.
The controller integrated PR #68 into dev at that same commit.
Acta merge event: `01M5YZMECFWEPVPCMGFBGA550B`.
Required repository checks precede focused configuration and editing regressions:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test commands`
- `cargo test --test tree`
- `cargo test --test search`
- `python3 scripts/check-source.py`
- `cargo test --test configuration`
- `cargo test --test editing`
- `git diff --check`

## Risks and Follow-ups

Executable startup discovery remains assigned to Slice 20. This slice consumes
immutable supplied snapshots and does not add configuration rereads or watching.
Terminal report distinguishability and Escape timing keep their existing bounds.
