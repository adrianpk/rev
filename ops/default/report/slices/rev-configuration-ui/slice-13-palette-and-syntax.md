# Slice 13: Apply semantic RGB palette roles

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-13-palette-and-syntax`
PR: pending

## Purpose

Resolve the existing bundled syntax captures through the admitted semantic RGB
palette while preserving canonical document, policy and history ownership.

## Delivered Behavior

- All 25 pinned Tokyo Night night defaults remain complete and independently
  overridable. New coverage replaces each leaf in turn and verifies every
  omitted sibling retains its inherited value.
- Registry composition carries the global snapshot palette into each derived
  frame. Standalone projections use defaults or an explicitly supplied palette.
  Palette selection never depends on the current path or language.
- The seven existing capture groups render configured RGB foregrounds rather
  than hardcoded ANSI indices. Capture grouping, byte ranges, incremental parse
  behavior, plain fallback and row resets retain their existing contracts.
- Linear and rectangular selections suppress syntax foregrounds on selected
  glyphs. Colors resume when leaving a selection inside a captured token.
- Rust/plain destination changes retain dirty text, marks, revisions, logical
  saved identity and undo/redo; they discard only derived syntax presentation.

## Implementation Notes

- T13.1: `bf851d6483286cb6836ac87b159f18c8b4e20d99` connects the complete
  palette to derived frames and checks independent overrides for every role.
- T13.2: `e86b30c254970d3a06a6635bdf9caeb5327b67c5` resolves capture styles
  to RGB at emission, prioritizes selection, and updates the syntax reference,
  Unreleased syntax note and existing PTY color expectations.
- T13.3 adds real capture-to-output comparisons for every syntax override,
  standalone/composed frame equivalence, partial-token region precedence and
  dirty Rust/plain transitions followed by undo and redo.
- Configuration parsing, validation, merging and typed defaults keep their
  existing owners. The frame holds one disposable palette copy; neither buffers
  nor syntax trees gain a second settings owner or terminal color state.

## Contracts Added or Changed

- `Frame::with_palette` supplies a global palette for standalone projection;
  `Frame::compose` and native workspace composition use their registry snapshot.
- Syntax classification stays semantic. Terminal RGB resolution belongs to
  display and resets at row boundaries. `NO_COLOR` retains its existing effect.
- Selected cells take presentation precedence over syntax without modifying
  capture metadata, source coordinates or the shared document.

## Files of Interest

- `src/display.rs`, `display/compose.rs`, `src/buffer.rs`: palette projection.
- `src/display/syntax.rs`, `selection.rs`, `src/syntax/style.rs`: RGB emission.
- `tests/configuration/resolution_contract.rs`: 25 independent role overrides.
- `tests/syntax/palette.rs`, `tests/display/palette.rs`: output and state truth.
- `scripts/check-pty.py`, `docs/reference/syntax.md`, `CHANGELOG.md`: syntax colors.

## Validation

Development checks:

- `cargo test --test configuration single_palette_leaf`: passed; 25 role cases.
- `cargo test --test syntax visual_only`: passed.
- `cargo test --test syntax palette::`: passed; seven overrides and path transitions.
- `cargo test --test display selected_syntax`: passed; linear/rectangle token cases.
- `cargo clippy --lib --test configuration --test syntax -- -D warnings`: passed.
- `cargo clippy --test syntax --test display -- -D warnings`: passed.
- `cargo run --quiet --example source-check -- --source src/display.rs --source src/buffer.rs --source src/display/compose.rs --source src/display/selection.rs --source src/display/syntax.rs --source src/syntax/style.rs --source tests/configuration/resolution_contract.rs`: passed.
- `cargo run --quiet --example source-check -- --source tests/syntax/palette.rs --source tests/display/palette.rs`: passed.

The controller stores final evidence on the clean PR head after its PR-reference
follow-up commit. Required commands precede focused real-PTY syntax regressions:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration`
- `cargo test --test syntax`
- `cargo test --test display`
- `python3 scripts/check-source.py`
- `python3 -c 'from pathlib import Path; import runpy, subprocess; subprocess.run(["cargo", "build", "--bin", "rev"], check=True); suite = runpy.run_path("scripts/check-pty.py"); [suite[name](Path.cwd()) for name in ("rust_syntax", "syntax_fallback", "incremental_rectangles")]'`
- `git diff --check`

## Risks and Follow-ups

RGB fidelity depends on terminal true-color support. UI role consumers and
cursor restoration remain assigned to Slice 14; selection still uses reverse
video with syntax suppressed. Executable startup snapshot loading remains
assigned to Slice 20; the running executable currently uses compiled defaults.
