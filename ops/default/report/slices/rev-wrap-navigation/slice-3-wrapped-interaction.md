# Slice 3: Wrapped selection and gutter integration

Status: reviewing
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-03-wrapped-interaction`
PR: pending

## Purpose

Deliver the pointer, selection, search and gutter portion of WRP02 and WRP03
through the current source-to-visual fragment owner.

## Delivered Behavior

- Native pointer presses and drags resolve continuation cells to whole source
  graphemes. Word-break fill, oversized-glyph fallback and viewport padding do
  not invent points. Real EOL insertion cells retain their source identity.
- Wheel and due drag-autoscroll ticks move through visual fragments, including
  continuations of the final logical line. A visible point stays fixed; an edge
  crossing clamps it through canonical coordinates. Terminal fragments stop
  the owned deadline, and release/resize/source invalidation retain normal
  capture cancellation and revision checks.
- Linear selection and search underlines follow each fragment's logical source
  columns. Word-break padding stays unselected; a real selected source newline
  retains its trailing fill. Nested panels, tree geometry, both gutter sides
  and narrow panes retain text-only overlays and the existing syntax owner.
- Rectangles still extract and edit each logical source row/column exactly
  once, including tabs and wide/combining graphemes. Wrap boundaries add no
  copied newlines, kill-ring entries or history transactions.
- Gutter labels appear only on the first source fragment. Continuations,
  including at the viewport top, have blank number cells and no independent
  current-line role. Relative distances count source lines across folds.
- A retained EOL insertion-row anchor resolves into the final text fragment
  when a wider pane removes that extra display row, before another redraw.

## Implementation Notes

T3.1 extends the existing fragment mapping for pointer hits, viewport scrolling,
autoscroll destinations and search overlays. Source ranges, coordinates,
revision-qualified capture, linear regions and rectangle owners remain the
canonical authorities. Fragment methods share painted/highlight widths with
the existing document renderer; no second text store or painted hit map is
introduced.

T3.2 derives number labels from those same current fragments, retaining checked
gutter geometry and logical distance calculation. Fold/width/source changes
are observed afresh. The GUT03 baseline and consolidated copy now describe the
delivered continuation policy, and the gutter reference and Unreleased match it.

## Contracts Added or Changed

- Pointer and search geometry uses the current pane's admitted text width and
  source revision, rather than treating a display row as a source row.
- Viewport scrolling and its stop condition use visual fragments when wrapped;
  explicit no-wrap preserves horizontal clipping and logical viewport movement.
- Rectangle rows and gutter distances retain logical source identity.
- Continuation number fields stay blank under every current-line label form.
- Reflow retains a valid EOL source anchor when its insertion row disappears.

## Files of Interest

- `src/view/hit.rs`, `src/view/visual.rs`, `src/view/scroll.rs`,
  `src/view/wrap.rs`: current fragment hits, scrolling and retained anchors.
- `src/session/pointer/autoscroll.rs`: source-qualified edge placement and
  visual terminal-boundary cancellation through the existing capture owner.
- `src/display.rs`, `src/display/search.rs`, `src/display/gutter.rs`: shared
  fragment clipping and logical numbering over the existing frame projection.
- `tests/wrapping/interaction.rs`: native continuation events, source/history
  identity, rectangles, search, resize/capture and EOL-anchor regressions.
- `tests/gutter/wrapping.rs`, `tests/gutter/surfaces.rs`: both gutter sides,
  relative/absolute/current forms, folds, first-row continuations and actual
  terminal writer styling under nested/tree/narrow geometry.
- Existing pointer, rectangle, search and panel fixtures retain paired explicit
  wrap/no-wrap expectations for their historical horizontal contracts.

## Validation

- `cargo test --locked --test wrapping interaction::`: passed, eight native
  interaction regressions through the actual session and frame APIs.
- `cargo test --locked --no-fail-fast --test pointer --test rectangles --test search --test panels`:
  passed, 148 regressions during implementation.
- `cargo test --locked --test gutter`: passed, 36 gutter regressions.
- `cargo test --locked --test gutter wrapping::`: passed, two focused logical
  label/fold/width cases, including viewport-top continuations.
- `cargo test --locked --test search edges::multiline_resize`: passed after
  the EOL-anchor reflow correction, retaining exact source match/point identity.
- `cargo clippy --locked --lib -- -D warnings`: passed during implementation.

Development failures exposed historical no-wrap-only fixture expectations and
an autoscroll fixture whose horizontal pointer had not requested EOL. The
fixtures now pin explicit paired modes and an outside-edge clamp respectively.
They retain the original horizontal, Unicode, source/history and cancellation
assertions alongside the delivered wrapped traces.

Final configured formatter, wrapping/pointer/rectangles/search/gutter/panels
checks and `git diff --check` are recorded through the controller after the
PR-reference commit. The canonical PR carries those commands in configured
order. Focused source-quality, display and documentation evidence is recorded
separately. No integrated full gate is claimed by this report.

## Risks and Follow-ups

Slice 4 owns complete Org projection integration and bounded resumable layout
work. The shared fragment traversal still may rescan a long source-line prefix;
this slice adds no retained cache or idle scanner. Joint native/PTY acceptance
and final User Guide reconciliation remain with their recorded later owners.
