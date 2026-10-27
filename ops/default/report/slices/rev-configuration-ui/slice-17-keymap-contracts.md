# Slice 17: Validate context keymaps

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-17-keymap-contracts`
PR: [#66](https://forge.adrianpk.com/adrianpk/rev/pulls/66)
Integrated: `09c26d8951fbbdf63eedaf245288cf9aa1a9fc0f`

## Purpose

Admit complete editor, minibuffer and tree keymaps before publication, using
fixed Rust action identities and the same normalized chord vocabulary as input.

## Delivered Behavior

- Parse one through eight normalized chords with explicit named keys, modifiers,
  printable scalars and compatible Escape/Meta pairs. Runtime and configuration
  share Control, NUL, Shift-Tab and Alt/Meta alias normalization.
- Audit complete native context defaults. Editor targets retain their exact
  M-x `Command` identities; four context-only prompt actions name existing
  acceptance, completion and previous/next stage operations.
- Compile independent inherited maps. Exact normalized overrides replace the
  inherited entry; later source spellings replace earlier aliases and their
  provenance. Unspecified bindings survive, and omitting overrides restores them.
- Reject invalid syntax, unknown/unavailable targets, same-source normalized
  duplicates and complete-map command/prefix conflicts. Diagnostics identify
  context, implicated sequences and original key/value spans when available.
- Publish all three immutable maps together through `Snapshot::keymaps()`.
  Rejected candidates retain complete built-ins, including unrelated settings.
  Valid previous snapshots remain usable after a later candidate is rejected.
- Reserve C-g for keyboard-quit in every context. Reject longer sequences that
  contain cancel because the native cancellation owner consumes them first.
  Empty, boolean and sentinel values cannot remove a binding.

## Implementation Notes

- T17.1: `9e7d05767b363d99cf26eb6b649dab21634a2b20` defines the common binding
  catalog, audits native defaults and adds bounded sequence parsing. Native
  editor routing and alias tests verify the catalog against current handlers.
- T17.2: `921c5038bb8d56c732a7f1b1d2e2e650e53e369b` compiles complete maps,
  validates context availability and prefixes, preserves normalized layer
  replacement/provenance and publishes the admitted maps with the snapshot.
- T17.3: `4a758356c05bfbb0aea9e0d482d0d6effc4ae289` protects cancellation, tests rejected
  replacements and terminal collisions, and documents every default mapping and
  stage-specific operation.
- Ordered normalized keys make prefix validation a scan of adjacent entries
  after map construction. The private sequence representation admits only the
  documented keys; its explicit ordering does not assume foreign terminal types
  implement Rust's Ord trait.
- Development checks caught unsupported foreign Ord derivation, redundant Vec
  use, oversized alias matches and one Clippy match form. The corrected code uses
  explicit sequence ordering, fixed alias tables and bounded concern helpers.

## Contracts Added or Changed

- `command::Binding` resolves existing commands and four `PromptAction` values.
  Prompt-only names remain outside the editor/M-x catalog and cannot install
  executable bodies. Context availability is checked before map publication.
- `input::keymap::Sequence`, `Context`, `Map` and `Effective` expose immutable
  parsing, lookup, prefix and catalog contracts. Raw spellings serve diagnostics;
  normalized identities own lookup, duplicate detection and replacement.
- Configuration parsing returns precise source spans for source-local errors;
  merged conflicts retain the implicated source/key through leaf provenance.
- The conservative terminal vocabulary merges physical aliases even when an
  enhanced report could distinguish some of them. Unsupported/intercepted
  physical events cannot be promised by configuration.

## Files of Interest

- `src/command/binding.rs`: fixed command and prompt binding identities.
- `src/input/key.rs`, `input/keymap/sequence.rs`: shared normalization and syntax.
- `src/input/keymap/defaults.rs`, `availability.rs`, `map.rs`, `cancellation.rs`:
  inherited maps, context admission, conflicts and reserved cancel.
- `src/config/validate.rs`, `merge.rs`, `resolve.rs`: atomic snapshot admission.
- `tests/commands/keymap.rs`, `tests/configuration/keymaps.rs`: native defaults,
  alias limits, layering, diagnostics, cancellation and rejected publication.
- `docs/reference/bindings.md`, `terminal-input.md`: vocabulary and complete defaults.

## Validation

Development checks:

- `cargo test --test commands keymap::`: passed; four parser/native-default tests.
- `cargo test --test configuration scopes::`: passed.
- `cargo test --test configuration keymaps::`: passed; eight map/admission tests.
- `cargo test --test configuration binding_values`: passed with a valid chord replacing the old placeholder key.
- `cargo test --test commands terminal::`: passed; eleven Meta, phase and cancellation regressions.
- `cargo test --test commands reference`: passed; global and per-context documentation catalogs plus matching prompt regressions.
- `cargo run --quiet --example source-check -- --source tests/commands/palette.rs`: passed.
- `cargo clippy --lib --test commands -- -D warnings`: passed after correcting fixed-array use.
- `cargo clippy --lib --test configuration --test commands -- -D warnings`: passed.
- `cargo run --quiet --example source-check -- --source src/input/key.rs --source src/input/keymap/cancellation.rs --source src/input/keymap/map.rs --source tests/configuration/keymaps.rs --source tests/commands/keymap.rs`: passed; nesting at most two.

The first controller attempt `NUSJYRY6UUSG2DHIL2OSCOESYY` at
`baf501d3b5d96c9954d8c13c6ff7db4c55bc1ae1` passed formatting, Clippy and all
63 configuration tests. Command validation found the old documentation oracle
counted shared targets across independent context tables as duplicates. The
corrected oracle preserves the global catalog check and adds exact, unique
per-context default target coverage; its focused check passed.

Exact-head controller evidence `CZSYPTGNKC3GWGVR5WHMHZCJSA` passed all eight
commands below at `09c26d8951fbbdf63eedaf245288cf9aa1a9fc0f`: 233 native tests,
zero skips, formatting, Clippy, source and whitespace checks. PR #66 merged by
Rebase + Fast-forward into `dev` at that head. Slice 17 of 30 completed.
Required commands precede focused shared search/query-replace and tree regressions:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration`
- `cargo test --test commands`
- `python3 scripts/check-source.py`
- `cargo test --test search`
- `cargo test --test tree`
- `git diff --check`

## Risks and Follow-ups

Configured dispatch remains assigned to Slices 18-19, and executable startup
loading remains assigned to Slice 20. The admitted maps do not activate user
bindings in this contract slice. Stage-specific actions keep their existing
prompt/decision/inventory owners; map validity never grants destructive consent.
