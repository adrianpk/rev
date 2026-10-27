# Slice 16: Apply bounded history and prompt preferences

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-16-bounded-preferences`
PR: pending

## Purpose

Apply admitted history, linear kill-ring and prompt limits through their existing
owners while preserving whole transactions, saved-state identity and cancellation.

## Delivered Behavior

- Completed edits enforce the configured whole-group count and soft logical
  fragment byte target. Oldest groups evict first; the newest coherent group
  remains reversible even when it alone exceeds the target.
- Applying a buffer policy retains existing undo/redo traversal until the next
  completed edit. Branch publication discards redo and prunes whole groups;
  pruning never changes the logical saved identity or marks text clean.
- Session construction applies positive linear ring capacity and directional
  coalescing. New entries evict the oldest when full; capacity one still yanks
  and rotates. Disabled coalescing retains adjacent kills separately. Rectangular
  text keeps its independent owner.
- Prompt acquisition, keys, whole pastes, completion and command-history recall
  share the admitted Unicode scalar bound. Errors retain draft, cursor, focus
  and navigation state, and C-g remains reachable. Search/query-replace literal
  text uses the same limit, including separators and tab insertion.
- Command history retains distinct accepted invocations up to its configured
  limit, promotes repeats and evicts oldest entries. Cancellation retains the
  history; rejected input adds nothing. All retained state stays in session memory.

## Implementation Notes

- T16.1: `c9c6abca1aaf167768bc8810939bd7a3ea0199f7` stores admitted history
  policy beside the canonical stacks and applies it at completed edit pruning.
- T16.2: `8da79cb8dcfd7cc0dce6e98f8d891b75469f8aff` initializes session ring
  and input policy, threads scalar limits into every text prompt, and publishes
  completion/recall state only after bounded replacement succeeds.
- T16.3: This report's introduction commit adds configured behavior regressions
  and documents bounds, units, eviction, deferred pruning and prompt recovery.
- Existing revision-qualified edit admission, typing grouping, history transfer,
  precomposed kill fragments and grapheme cursor repair remain the owners of
  their respective operations. No timer participates in grouping.

## Contracts Added or Changed

- `Input::with_settings` accepts an admitted immutable snapshot; `Input::default`
  retains compiled preferences. `Session::new` uses its buffer registry snapshot
  for input and kill-ring construction.
- `Buffer::apply_settings` admits the future retention policy without dropping
  currently published records. Completed edits enforce it coherently.
- Prompt limit feedback is now `Prompt input limit exceeded`, accurate for every
  admitted scalar limit. A short query may invoke a longer fixed catalog name,
  but completion or recall cannot insert that name past the prompt text bound.
- The history payload target is soft and does not describe process memory.

## Files of Interest

- `src/buffer.rs`, `buffer/policy.rs`, `buffer/edit.rs`: canonical history policy.
- `src/kill.rs`, `src/session.rs`: ring bounds and snapshot initialization.
- `src/input.rs`, `input/prompt.rs`, `input/palette.rs`, `input/palette/history.rs`:
  bounded prompt ownership and atomic completion/recall.
- `tests/editing/preferences.rs`, `tests/rectangles/preferences.rs`: whole groups,
  compound rectangles, saved identity, capacity-one rotation and kill directions.
- `tests/commands/preferences.rs`: Unicode, completion, history and cancellation.
- `docs/reference/commands.md`, `prompts.md`: supplied retention/prompt semantics.

## Validation

Development checks:

- `cargo test --test editing retained_`: passed; compiled-default whole groups.
- `cargo clippy --lib -- -D warnings`: passed.
- `cargo test --test commands prompt_`: passed.
- `cargo test --test commands palette::`: passed.
- `cargo test --test editing kills::`: passed.
- `cargo test --test search edges::`: passed.
- `cargo clippy --lib --test commands --test search -- -D warnings`: passed.
- `cargo test --test editing preferences::`: passed; four configured behavior tests.
- `cargo test --test commands preferences::`: passed; seven configured behavior tests.
- `cargo test --test rectangles preferences::`: passed; compound rectangle under both targets.
- `cargo clippy --lib --test editing --test commands --test rectangles -- -D warnings`: passed.
- `cargo run --quiet --example source-check -- --source tests/editing/preferences.rs --source tests/commands/preferences.rs --source tests/rectangles/preferences.rs`: passed; nesting at most two.

The controller records final evidence on the clean PR head after its
PR-reference follow-up commit. Required commands precede the focused shared
search/query-replace input regressions:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test editing`
- `cargo test --test commands`
- `cargo test --test rectangles`
- `python3 scripts/check-source.py`
- `cargo test --test search`
- `git diff --check`

## Risks and Follow-ups

Executable startup configuration loading remains assigned to Slice 20; supplied
library snapshots exercise these policies now. Scalar input limits can make
long catalog names unavailable for completion/recall while their short queries
remain invocable. Runtime configuration reload and session persistence are not
introduced. The soft fragment target always admits the newest whole group.
