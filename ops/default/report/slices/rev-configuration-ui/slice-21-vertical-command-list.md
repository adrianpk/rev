# Slice 21: Present vertical M-x candidates

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-21-vertical-command-list`
PR: [#70](https://forge.adrianpk.com/adrianpk/rev/pulls/70)

## Purpose

Present the M-x query above bounded vertical candidates using the existing panel,
command, input and palette owners while preserving the underlying editing view.

## Delivered Behavior

- M-x uses the reserved command region for its query followed by one command per
  row. The configured candidate limit (default eight, admitted 1..=32) is clipped
  after recursive editing/status minima and the shared separator.
- Matching applies the admitted prefix-then-substring, prefix or substring mode.
  Existing alphabetical registry ordering remains stable within each rank.
  An empty result displays an explicit unselected `No matching commands` row.
- Candidate selection and scroll origin belong to Palette independently of the
  editable prompt and the original Tab filter. Later Tab-completed selections
  stay visible. Query/history edits reset list state; resize does not change the
  draft or selection, and zero candidate rows retain the scroll origin.
- The selected row consumes semantic selection text/background roles, including
  reverse-video fallback under NO_COLOR. Query cursor and clipping use the shared
  whole-grapheme display owner. The prompt remains usable without candidate rows.
- Completion borrows presentation rows without scrolling the retained editing or
  tree workspace. Cancellation restores the same view, qualified point, intended
  column and recenter state. Nested pane identities and buffer history stay put.
- Rendering derives labels from the existing fixed command registry. It performs
  no filesystem observation or command dispatch. Existing completion, history,
  argument acquisition and cancellation handlers remain authoritative.

## Implementation Notes

- T21.1: `5ff7d648583fa5da25a08c6c87b1e9149ec9bea4` connects configured matching
  and row limits to native command composition, shared prompt placement and
  vertical candidate rendering, replacing joined inline hints.
- T21.2: `46cb306acc8409ad2297b23defed8c557c58c4a7` retains independent list
  selection/scroll, reveals the selected row and preserves zero-row suspension.
- T21.3: `e8d7dd186c509f969df483d0963b578a349af94a` adds geometry/Unicode/limit and actual
  terminal regressions and updates the command documentation.
- Composition::command repositions the query ahead of the auxiliary panel inside
  the same reserved extent. Editing minima, tree bounds and shared separators
  keep their existing owners. Composition size includes the full completion area.
- Frame projection borrows immutable Palette state. Input applies geometry to the
  retained list before projection; only the palette owner updates scroll origin.
- Lifecycle tests locate the query by its Echo role rather than assuming it is
  the final row. Existing document/cancellation assertions remain in place.

## Contracts Added or Changed

- Palette consumes admitted minibuffer matching and row-limit preferences, exposes
  its selected index and origin, and requests one empty-state row when needed.
- Composition::command supplies query-before-candidates geometry; zero available
  candidate rows retain the existing usable prompt reservation.
- Band::Completion distinguishes candidate/empty rows. Frame uses existing panel
  translation, safe glyph clipping and semantic selection colors.
- Session frame preparation preserves live workspace view state while M-x owns
  input and reveals only the retained command list against current geometry.

## Files of Interest

- `src/input/palette.rs`, `palette/history.rs`: matching, selection and scroll.
- `src/panel/composition.rs`: command region ordering and extent.
- `src/display/completion.rs`, `prompt.rs`, `display.rs`: vertical rows and cursor.
- `src/session.rs`, `session/tree.rs`: native composition and retained workspace.
- `tests/commands/vertical.rs`, `vertical/geometry.rs`: nine command-list regressions.
- `tests/panels/workspace.rs`: command extent/minima geometry regression.
- `scripts/check-command-list.py`: two RGB/NO_COLOR controlling-terminal sessions.
- Command/prompt references and `CHANGELOG.md`: delivered presentation behavior.

## Validation

Development checks:

- `cargo test --test commands --test panels --test display`: command/display suites passed after routing; the panel lifecycle assertion was updated for the query's new row and then passed separately.
- `cargo test --test commands vertical::`: passed; nine configured behavior tests.
- `cargo test --test commands palette::catalog_reference`: passed.
- `cargo test --test commands palette::context_reference`: passed.
- `cargo test --test panels rendering::workspace_lifecycle`: passed.
- `cargo test --test tree preferences::memory_only_draw`: passed, including retained no-filesystem-read behavior.
- `cargo test --test panels workspace::command_region`: passed.
- `cargo clippy --lib --test commands --test panels --test tree -- -D warnings`: passed after initial composition.
- `cargo clippy --lib --test commands -- -D warnings`: passed after list retention.
- `cargo clippy --lib --test commands --test panels -- -D warnings`: passed after geometry cases.
- `cargo run --quiet --example source-check -- --source src/input/palette.rs --source src/panel/composition.rs --source src/display/completion.rs --source src/display/prompt.rs --source src/session.rs --source src/session/tree.rs --source tests/commands/vertical.rs`: passed.
- `cargo run --quiet --example source-check -- --source src/input/palette.rs --source src/input/palette/history.rs --source src/session.rs --source src/display/completion.rs --source tests/commands/vertical.rs`: passed.
- `cargo run --quiet --example source-check -- --source tests/commands/vertical.rs --source tests/commands/vertical/geometry.rs --source tests/panels/workspace.rs`: passed; nesting at most two.
- `python3 scripts/check-command-list.py`: passed; two terminal sessions, zero skips.

The controller records final evidence on the clean PR head after its PR-reference
follow-up commit. Required commands precede the focused tree/workspace and actual
terminal regressions:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test commands`
- `cargo test --test panels`
- `cargo test --test display`
- `python3 scripts/check-source.py`
- `cargo test --test tree`
- `python3 scripts/check-command-list.py`
- `git diff --check`

## Risks and Follow-ups

Candidate navigation, separate Meta history bindings and selected Return semantics
remain assigned to Slice 22. This slice preserves existing Tab cycling and history
keys while delivering their vertical presentation and retained list state.
