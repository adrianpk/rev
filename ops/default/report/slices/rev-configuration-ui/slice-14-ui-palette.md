# Slice 14: Render UI palette and restore terminal colors

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-14-ui-palette`
PR: [#63](https://forge.adrianpk.com/adrianpk/rev/pulls/63)

## Purpose

Apply every semantic UI palette role through native composition and restore
cursor/text colors through the terminal lifecycle, including partial failures.

## Delivered Behavior

- Document foreground/background, active/inactive status, focus borders, tree
  surfaces/selection, regions, search, prompts, warnings and errors consume the
  admitted palette. Mixed rows retain each panel's own semantic color spans.
- Selected text/background override syntax and search colors; current search
  matches override syntax while retaining underlining. Row boundaries reset
  colors/attributes, preventing leakage into neighboring panels and prompts.
- Session outcomes supply message severity explicitly, including failed saves,
  ingress, tree operations and partial-operation warnings. Prompts return to
  their ordinary role without changing handler text or interaction semantics.
- Compatible terminals receive typed cursor RGB through OSC 12. OSC 112 resets
  the terminal default cursor color during cleanup. Unknown TERM or NO_COLOR
  skips cursor mutation; monochrome selection retains reverse video.
- Failed set/reset writes retain cleanup ownership. Restoration still attempts
  every obligation and releases raw input before Drop retries failed output.
  Rendering stays event-driven and adds no capability query or blink timer.

## Implementation Notes

- T14.1: `eea4858ae3c2b16e174091f53020867ef0f81c1c` adds compact semantic
  spans to existing projected rows, resolves overlay precedence, and carries
  explicit message severity from existing handlers.
- T14.2: `7a665f6f4394a487d8344290b866a1e934727f91` adds owned cursor
  color cleanup, bounded capability fallback, real PTY probes and color docs.
- T14.3: `c57478dedb74bedf16e17e6e2c9fad34289cfff9` decodes the actual writer's SGR/cursor output into effective cell colors
  to test mixed panels, tree focus, overlays and safety. It completes explicit
  severity handling for tree/save outcomes and adds executable default coverage.
- The palette remains snapshot-owned; frames retain disposable derived values.
  Panel bounds and syntax capture ownership are unchanged. No per-cell style
  raster, retained render cache, new dependency or layout owner is introduced.

## Contracts Added or Changed

- `Frame::message_kind` receives an explicit message severity. Semantic spans
  follow native blits while selection/search/capture ranges retain their common
  clipped cell coordinates.
- Color precedence is selection, search, syntax, then the surface's ordinary
  role. Tree selection uses the tree text/background pair. NO_COLOR retains
  non-color region/search attributes.
- `Terminal::set_cursor_color` requires alternate-screen ownership, takes only
  admitted RGB channels and owns reset before writing. Reset failure cannot
  prevent text/mode cleanup and remains retryable through Drop.

## Files of Interest

- `src/display/style.rs`, `compose.rs`, `selection.rs`, `tree.rs`: native styles.
- `src/session.rs`, `session/save.rs`, `session/tree/`: explicit message severity.
- `src/terminal.rs`, `examples/terminal-probe.rs`: cursor lifetime and faults.
- `tests/display/palette.rs`, `tests/support/surface.rs`: effective cell assertions.
- `scripts/check-ui-palette.py`: isolated normal/error/capability PTY cases.
- `docs/reference/terminal-input.md`, `syntax.md`, `CHANGELOG.md`: visible behavior.

## Validation

Development checks:

- `cargo test --test display selected_syntax`: passed.
- `cargo test --test display draw_boundary`: passed.
- `cargo test --test display palette::`: passed; five cell-color/safety tests.
- `cargo test --test display safe_colors`: passed after correcting a mistaken test character count to exact source preservation.
- `cargo test --test syntax palette::`: passed; seven overrides and path transitions.
- `cargo clippy --lib --test display -- -D warnings`: passed.
- `cargo clippy --lib --example terminal-probe -- -D warnings`: passed.
- `cargo run --quiet --example source-check -- --source src/display/style.rs --source src/display/compose.rs --source src/display/selection.rs --source src/session.rs --source tests/support/surface.rs`: passed.
- `cargo run --quiet --example source-check -- --source src/terminal.rs --source examples/terminal-probe.rs`: passed after splitting probe setup/dispatch to retain the complexity bounds.
- `cargo run --quiet --example source-check -- --source tests/display/palette.rs --source tests/support/surface.rs`: passed.
- `cargo run --quiet --example source-check -- --source src/session/tree/delete.rs --source src/session/tree/rename.rs --source src/session/save.rs`: passed.
- `python3 scripts/check-ui-palette.py`: passed the initial 11 PTY cases; executable-default coverage was then added.
- `python3 -c 'import runpy; suite = runpy.run_path("scripts/check-ui-palette.py"); suite["restoration"]("normal", 0); suite["restoration"]("color-stop", 1)'`: passed after adding active text-color probes.
- `python3 -c 'import runpy; suite = runpy.run_path("scripts/check-ui-palette.py"); suite["application"]()'`: passed; actual executable defaults and cleanup.

The first exact-head validation stopped at an inherited reverse-video assertion
in display policy coverage. Selection assertions in the affected display,
linear-region, rectangle and search tests now inspect effective RGB cells;
NO_COLOR reverse-video behavior remains covered by the isolated PTY case.
Focused corrections passed:

- `cargo test --test display marked_geometry`
- `cargo test --test editing multiline_surface`
- `cargo test --test rectangles clipped_surface`
- `cargo test --test search selected_surface`
- `cargo run --quiet --example source-check -- --source tests/display/policy.rs --source tests/editing/regions.rs --source tests/rectangles.rs --source tests/search.rs --source tests/support/surface.rs`

Final controller validation `GPQFHWTX4LUK6AS3RIPAXXV2VY` passed all thirteen
commands on clean PR head `3ba6bb0ecd825a4f3e75081ec5b056821fa6ff5d`: 87 native
tests (display 25, panels 18, recovery 18, syntax 23 and three focused cases),
fifteen PTY cases and zero skips. PR #63 integrated through Rebase + Fast-forward
into dev at that same commit. Commands, in canonical order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test display`
- `cargo test --test panels`
- `cargo test --test recovery`
- `python3 scripts/check-source.py`
- `cargo test --test syntax`
- `cargo test --test editing multiline_surface`
- `cargo test --test rectangles clipped_surface`
- `cargo test --test search selected_surface`
- `python3 scripts/check-ui-palette.py`
- `python3 -c 'from pathlib import Path; import runpy; suite = runpy.run_path("scripts/check-pty.py"); [suite[name](Path.cwd()) for name in ("rust_syntax", "syntax_fallback", "incremental_rectangles")]'`
- `git diff --check`

## Risks and Follow-ups

TERM recognition is an attempt policy, not capability proof: terminals or
multiplexers can ignore/block OSC and may approximate RGB. Cleanup resets the
terminal's default cursor color; it does not reconstruct a pre-entry custom
cursor color. These limits and NO_COLOR fallback are documented in the terminal
reference. Executable startup snapshot loading remains assigned to Slice 20.
