# Slice 6: Project buffer tab policies

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-06-buffer-display-policy`
PR: [#55](https://forge.adrianpk.com/adrianpk/rev/pulls/55)
Integrated: `9db4f5682730cb95dbe33bad260adf972aad8b1d`

## Purpose

Give each buffer one effective editor policy and use its tab width throughout
document geometry while retaining canonical text and independent view state.

## Delivered Behavior

- Installs resolved global/language editor leaves from a supplied immutable
  snapshot for scratch, created, loaded and visited buffers. Closing the final
  buffer retains the same configured global policy in its replacement.
- Projects literal tabs to the next buffer-specific stop, using the admitted
  1..32 width. Coordinates, viewport rows, cursor columns, vertical goals,
  selection, search marks and syntax geometry share that projection.
- Preserves byte/scalar domains, complete wide/combining/ZWJ clusters and CRLF
  ownership across policy changes and real rope chunk seams.
- Uses the same buffer policy across nested/shared views. A width change
  invalidates cached vertical goals before reveal or navigation. An unchanged
  display width preserves the goal even when indentation preferences change.
- Changes no text, mark, publication revision, saved-state identity, syntax
  state or history when replacing an effective policy. Prepared edits and
  undo/redo remain valid under their canonical source contracts.

## Implementation Notes

- T6.1: `8f8e39e0f2c134d3f8c8c9af014bee1c2538e753` adds buffer policy
  ownership, supplied-snapshot construction and tab-aware glyph coordinates.
- T6.2: `fe01eae99816159a732df635830a425e50e88982` adds view goal
  invalidation and shared-view/lifecycle coverage.
- T6.3: `fd6c061b651ff17ab21ee37dcc23b7de9555583c` adds mixed-unit,
  seam, replay, clipping and marked/search geometry cases
  and this report. All ten new cases pass across focused development runs.
- Buffer policies expose shared access only. Their widths come from admitted
  snapshots, so projection cannot divide by zero or accept an unchecked width.
- Coordinate borrows retain the buffer's current width. Private edit candidates
  carry that admitted width through coordinate construction and viewport mapping;
  source stamps continue to identify text revisions rather than presentation.
- View cache invalidation compares the previous tab width with the current
  buffer policy before using a goal. Logical points and top rows remain intact;
  normal reveal adjusts the viewport to the newly projected point when needed.
- Frames derive fresh rows and highlights on every projection. No row cache or
  syntax byte tree requires a policy-driven text revision or reparsing.
- Non-document UI text keeps the compiled default presentation width; it does
  not inherit whichever buffer happens to be selected.

## Contracts Added or Changed

- `Buffers::with_settings(Snapshot)` owns the frozen settings used when
  installing buffers, without configuration discovery or reads at construction.
- `Buffer::policy()` borrows its effective `Editor` record.
- `Buffer::apply_settings(&Snapshot)` resolves global/language leaves without
  file IO or canonical text/history mutation. It does not reload configuration.
- `Coordinates` and document `Glyphs` consume the buffer's admitted tab width.
  Private text candidates retain the same explicit projection parameter.
- Views invalidate their cached display goal lazily before reveal/navigation
  after a width change. Shared views retain their independent point and scroll.

## Files of Interest

- `src/buffer/policy.rs`, `buffer.rs`, `buffer/visits.rs`, `buffer/lifecycle.rs`:
  effective policy ownership and installation across buffer lifecycles.
- `src/coordinates.rs`, `coordinates/display.rs`: explicit width propagation
  and whole-grapheme terminal projection.
- `src/buffer/edit.rs`, `buffer/edit/history.rs`, `buffer/edit/views.rs`:
  coordinate construction for private candidates and history/view mapping.
- `src/view.rs`, `view/scroll.rs`: cached display-goal invalidation.
- `tests/coordinates/policy.rs`: bounds representatives, language inheritance,
  units, seams and canonical replay.
- `tests/display/policy.rs`, `tests/windows/policy.rs`: golden clipping,
  active search/region geometry, nested projection and view goals.

## Validation

Development checks:

- `cargo test --test coordinates policy::`: passed; two foundational cases.
- `cargo test --test windows policy::`: passed; three cases.
- `cargo test --test coordinates --test display policy::`: initially failed
  one search test's terminal-cursor assertion; five coordinate cases and the
  clipping case passed. The search prompt owns that cursor; the corrected test
  asserts document geometry through the view.
- `cargo test --test display policy::marked_geometry`: passed after correction.
- `cargo clippy --lib --test coordinates --test display --test windows -- -D warnings`: passed.

The controller recorded all eight ordered checks passing on clean head
`9db4f5682730cb95dbe33bad260adf972aad8b1d`, validation
`QODDPXB5OZODZAJXZJ75KLEGGG`: 24 coordinate, 20 display, 38 window and 92 editing
tests. Canonical PR #55 integrated that exact head into dev by Rebase +
Fast-forward. Required commands precede the affected editing regression target
and whitespace check:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test coordinates`
- `cargo test --test display`
- `cargo test --test windows`
- `python3 scripts/check-source.py`
- `cargo test --test editing`
- `git diff --check`

## Risks and Follow-ups

Indentation insertion and new-file formatting remain assigned to Slice 7.
Rectangle transfer semantics remain assigned to Slices 8-9; automatic policy
re-evaluation after successful path changes remains assigned to Slice 9.
Startup configuration loading remains assigned to Slice 20. Current terminal
startup retains compiled defaults until that activation.
