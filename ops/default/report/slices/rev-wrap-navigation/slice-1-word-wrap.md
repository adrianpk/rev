# Slice 1: Word-aware source display

Status: delivered
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-01-word-wrap`
PR: `#174`
Integrated commit: `366efcd384b163e69478c3bf4bda918bd1d7547a`

## Purpose

Deliver the source-preserving display foundation for WRP01, WRP02 and WRP06:
word-aware wrapping at each editing pane's available text width, enabled by
default and controllable through the existing configuration and command owners.

## Delivered Behavior

- Long source lines continue at space/tab boundaries. Unbroken tokens advance
  at complete graphemes; combining sequences remain intact. Tabs retain logical
  stops, and oversized glyphs use the existing blank-cell clipping fallback.
- Each pane derives its own fragments after gutter geometry. Cursor reveal
  follows the source point through resizing and visual continuations. A word
  moved to the next row owns its starting point; exact-width EOL has an
  insertion row. Zero-width and suspended panes preserve canonical state.
- `editor.soft_wrap` defaults to true. Explicit false restores horizontal
  presentation. `M-x visual-line-mode` toggles one buffer for all its views;
  newly created buffers retain the frozen startup default.
- Presentation changes leave source revisions, dirty state and undo history
  unchanged. Native save/undo coverage preserves literal tabs, CRLF and Unicode.

## Implementation Notes

T1.1: `aa7c739f30ba097955193c43852508716cf6017e` introduces the shared fragment
owner and pane rendering/reveal. T1.2:
`9946e539aaa39b1e81cc78f40e2e339f08cb6536` integrates configuration, the native
command, reference catalogs, Unreleased and native regression coverage.

Fragments carry the buffer stamp, logical line, checked scalar span, logical
cell span and padded display text. Layout streams glyphs with one pane-width
of lookahead; returned rows are limited to viewport demand. Reveal retains at
most the pane's text-row count while approaching point. No fragment cache or
whole-document visual-line index is retained. Syntax paints stop at fragment
boundaries, leaving word-break padding unstyled.

The packaged template includes the reference TOML asset directly. Both runtime
and documentation therefore expose the same complete sample bytes; startup
tests independently check the compiled default, explicit false and include
precedence rather than relying on this shared provenance alone.

## Contracts Added or Changed

- Global startup Boolean `editor.soft_wrap`, default true; language-local wrap
  settings remain unsupported and are rejected through whole-candidate loading.
- Native command `visual-line-mode`, admitted for an editing buffer and
  registered in both command and ordinary binding catalogs.
- Buffer-owned runtime presentation override, independent from source edits.
- Source-qualified visual fragments and a source anchor for wrapped viewports.

## Files of Interest

- `src/view/wrap.rs`, `src/view.rs`: checked fragments and source-point reveal.
- `src/display.rs`, `src/display/syntax.rs`: pane rendering and fragment paints.
- `src/buffer/policy.rs`, `src/session/wrap.rs`: buffer policy and command routing.
- `src/config.rs`, `src/config/merge.rs`, `src/config/project.rs`: startup setting.
- `tests/wrapping.rs`: real layout, session commands, split policy, save and undo.
- `tests/display.rs`, `tests/display/policy.rs`, `tests/display/org_colors.rs`:
  paired factory-wrap and explicit no-wrap assertions, including syntax edges.
- `docs/reference/`, `CHANGELOG.md`: public setting/command catalogs and outcome.

## Validation

- `cargo test --locked --test wrapping`: passed, six native tests; the observed
  warm test execution took 0.17 seconds, excluding compilation.
- `cargo test --locked --test commands palette::catalog_reference`: passed.
- `cargo test --locked --test wrapping --test display --test configuration --test commands`:
  passed, 339 tests in the first controller batch at
  `b32f67bb702e9ed411ebb7d38d8aa65d58ec500e`.
- `cargo clippy --locked --lib -- -D warnings`: that batch failed on the new
  iterator's loop form. Replaced it with the requested `while let` form before
  final validation; the batch diagnostics remain in the controller ledger.
- The earlier display run identified five obsolete no-wrap expectations. They
  now assert both startup policies; the command-catalog failure was corrected
  by documenting the new binding. Assertions were retained for both modes.

The controller passed canonical receipt `Y7GHL424WJD6IBUZMYYYXSN7K7` against
`366efcd384b163e69478c3bf4bda918bd1d7547a` with these exact ordered commands:

- `cargo fmt --all -- --check`: passed.
- `cargo test --locked --test wrapping --test display --test configuration --test commands`: passed, 339 tests.
- `git diff --check`: passed.

Supplemental receipt `J6QBLUQYPVO2BSTYEB2QOYP537` passed library Clippy,
documentation checks (58 pages, 330 local links) and `git diff --check` at the
same candidate. Canonical PR #174 integrated that exact head through the
controller's Rebase + Fast-forward path. These are slice checks; the integrated
delivery gate remains controller-owned.

## Risks and Follow-ups

The approved later slices own visual-line motion/kills, pointer/search/selection
and continuation gutter mapping, complete Org integration, and measured bounded
long-line reflow. This slice retains no cache; reaching a distant position within
one long logical line can still rescan its prefix. Slice 4 owns resumable work
ceilings and long-line/input measurements before final acceptance.
