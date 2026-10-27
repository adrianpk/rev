# Slice 24: Complete and navigate file paths

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-24-path-navigation`
PR: [#73](https://forge.adrianpk.com/adrianpk/rev/pulls/73)

## Purpose

Connect bounded filesystem observations to native find-file input and the shared
vertical completion panel, with selected/shared completion and directory traversal.

## Delivered Behavior

- Find-file observes the visiting file's parent, or cwd for an unnamed buffer.
  Its blank draft filters that explicit base; relative text resolves against it.
  The query appears above bounded vertical candidates with directory markers,
  semantic selection colors and independent path identities.
- Up/Down and C-p/C-n select candidates without changing the query. Existing
  configurable minibuffer previous/next and completion aliases use the same owner.
  The selected row scrolls into view within the configured candidate-row limit.
- Tab completes a selected or unique name, or extends a shared prefix at whole
  grapheme boundaries. Directory completion retains its slash and observes that
  component; editing back to a parent restores the parent's candidates.
- Relative, absolute, `./`, `../`, `~/`, space and Unicode path spellings retain
  their editable prefix through completion. Symlink entries remain observed leaves.
- Prefix edits within one component reuse cached observations. Only directory
  transitions and explicit Tab refresh disk state. A refreshed selection follows
  its name when sort positions change; a removed choice reports an error.
- Incomplete lists reserve a diagnostic row within the configured limit. They
  cannot imply unique/shared completion from a subset; explicit observed choices
  remain completable. Unsupported names never become lossy completion text.
- Cursor-only movement, refused input and resize retain selection and the draft.
  Zero candidate rows preserve list origin. Prompt bounds reject a long completion
  atomically. C-g discards the interaction and restores the retained workspace
  view, qualified point, buffer identity and history without visiting candidates.
- File/write prompt distinctions remain explicit. Write/save Tab retains its
  path-entry guidance; find-file uses the provider. Existing typed-path ingress
  and empty-path rejection continue through the canonical file owner.

## Implementation Notes

- T24.1: `324c635437f760dfd2679607e4aec12aff24baf9` connects the find-file base,
  retained navigation and shared query/candidate geometry to native events.
- T24.2: `531930ec9a4115c234df5a11345d97bb9f3c33e5` adds unique/shared/selected
  completion, directory traversal and refreshed-selection identity checks.
- T24.3: `e698d323a802d46d890dafdce40879c02b4c94b5` adds path-spelling,
  incomplete-list, prompt-bound, resize, alias and actual
  RGB/NO_COLOR terminal journeys, updates references and records the slice report.
- Input owns the bounded path prompt and key decoding. Session owns its pending
  Navigation and event-driven observation. Frame reads retained labels and list
  state; frame preparation reveals only the list while completion owns focus.
- Composition::completion is the common geometry owner for command and path
  lists. Both use one candidate-row renderer and the existing selection palette.
- Existing file/tree prompt tests identify the Echo row rather than assuming it
  is the last terminal row. Their cursor, buffer and cancellation assertions remain.

## Contracts Added or Changed

- UI03: initialized find-file base, vertical path candidates, literal path spelling,
  selected/shared Tab completion, directory descent and editable parent navigation.
- UI04: retained filtering/rendering without IO, explicit bounded refresh,
  incomplete-list warnings and exact identity separate from safe display labels.
- CFG12-CFG13: admitted minibuffer actions dispatch through the active path owner
  while retaining prompt bounds, cancellation, prefix and event-phase guards.
- `Navigation` provides the concrete file-owned draft, snapshot, selection and
  origin API. Native file opening remains delegated to the existing ingress owner.

## Files of Interest

- `src/file/completion/navigation.rs`, `navigation/complete.rs`: retained state,
  explicit refresh and whole-grapheme completion.
- `src/input/path.rs`, `src/session/path.rs`, `session/file.rs`: native ownership.
- `src/display/completion.rs`, `src/panel/composition.rs`: shared vertical surface.
- `tests/path_completion/panel.rs`, `navigation.rs`, `edges.rs`: 14 new path tests.
- `tests/commands/context_keymaps/flows.rs`: native path aliases.
- `tests/files.rs`, `tests/tree.rs`: retained file/tree cursor contracts.
- `scripts/check-path-navigation.py`: two owned terminal sessions.
- Prompt/binding/provider references and `CHANGELOG.md`: delivered behavior.

## Validation

Successful development checks:

- `cargo test --test path_completion panel::`: initial native base, geometry and cached-rendering cases passed.
- `cargo test --test path_completion navigation::`: five completion/refresh/directory/Unicode cases passed.
- `cargo test --test path_completion panel::native`: two native descent/selection cases passed.
- `cargo test --test path_completion edges::`: four spelling, resize, incomplete-list and prompt-bound cases passed.
- `cargo test --test commands argument_`: argument focus, feedback and continuation passed.
- `cargo test --test commands command_continuation`: focused alias continuation passed.
- `cargo test --test commands path_tab`: find-file versus write/save Tab behavior passed.
- `cargo test --test commands find_file_aliases`: native prompt editing/completion aliases passed.
- `cargo test --test commands path_candidate_aliases`: configured path candidate aliases passed.
- `cargo test --test panels`: 19 panel regressions passed.
- `cargo test --test files find_argument`: native file prompt cursor/ingress passed.
- `cargo test --test files prompt_geometry`: Unicode/tiny geometry and original empty-path rejection passed.
- `cargo test --test tree command_focus`: retained tree/editor focus and path cursor passed.
- `cargo clippy --lib --test path_completion --test commands --test files --test tree -- -D warnings`: passed.
- `cargo run --quiet --example source-check -- --source src/input.rs --source src/session.rs --source src/file/completion/navigation.rs --source src/session/path.rs --source src/display/completion.rs --source src/session/file.rs --source src/input/path.rs --source tests/path_completion/panel.rs`: passed.
- `cargo run --quiet --example source-check -- --source src/file/completion/navigation/complete.rs --source src/session/path.rs --source tests/path_completion/navigation.rs --source tests/path_completion/panel.rs`: passed.
- `cargo run --quiet --example source-check -- --source src/session/path.rs --source src/session/file.rs --source tests/path_completion/edges.rs --source tests/commands/context_keymaps/flows.rs --root tests/files.rs --root tests/tree.rs`: passed; nesting at most two.
- `python3 scripts/check-path-navigation.py`: two RGB/NO_COLOR terminal sessions passed, zero skips; source and home fixture bytes remained unchanged.

Final readiness uses controller evidence for the clean PR head after the
PR-reference follow-up commit. Required checks precede focused file/tree and
terminal regressions in this exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test path_completion`
- `cargo test --test commands`
- `cargo test --test panels`
- `python3 scripts/check-source.py`
- `cargo test --test files`
- `cargo test --test tree command_focus`
- `python3 scripts/check-path-navigation.py`
- `git diff --check`

## Risks and Follow-ups

Selected-candidate Return, directory Return and failed-ingress prompt retention
remain assigned to Slice 25. This slice delivers navigation/completion and retains
the existing typed-path opening handler.
