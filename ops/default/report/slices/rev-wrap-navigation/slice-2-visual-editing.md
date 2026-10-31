# Slice 2: Emacs visual-line editing

Status: reviewing
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-02-visual-editing`
PR: pending

## Purpose

Deliver WRP07 and the navigation part of WRP03: native visual-row motion and
line editing through the existing source projection, kill ring and history.

## Delivered Behavior

- Up/Down and C-n/C-p traverse wrapped visual rows, preserving terminal-cell
  goal columns through short rows, tabs and wide/combining graphemes.
- C-a/C-e address visual row boundaries. C-k follows Emacs wrap-whitespace and
  hard-token boundary behavior while remaining an ordinary undoable source
  deletion. Repeated kills retain the normal directional chain, and yank
  restores exact source bytes. Existing read-only rejection remains unchanged.
- Paging and center/top/bottom recentering use visual rows. Goto-line still
  addresses logical source lines. Explicit no-wrap keeps logical line commands.
- Pane-width and wrap-policy changes invalidate visual goals before their next
  use, including commands issued before redraw. Split points remain independent.
- Shared edits and history map each viewport's source anchor before canonical
  publication. A combining insertion at an anchor reprojects its derived
  boundary without rejecting an otherwise valid edit.

## Implementation Notes

T2.1 introduces shared visual-fragment navigation and kill routing. T2.2 adds
visual paging/recenter, width-qualified goals and mapped viewport anchors.
The original task commits are recorded in the tracker.

The view resolves current fragments at admitted gutter/text geometry. It keeps
source positions distinct from relative visual columns and uses the existing
coordinate owner for whole-grapheme destinations. Neighbor traversal retains
only the current/preceding fragment, including across visible logical lines.
No whole-document visual-row index or new text store is introduced.

Viewport mapping now returns both the resulting source row and a left-affine
grapheme-safe anchor. Ordinary edits, undo and existing multi-source publication
owners use that same map without adding viewport entries to source history.
Existing selected-newline fill is retained at a source EOL; word-break padding
does not become selected source text.

## Contracts Added or Changed

- Selected native line commands follow the effective buffer wrap policy.
- Visual goal columns belong to their observed pane width and wrap policy;
  indentation-only changes retain valid goals.
- Derived viewport anchors map with left affinity through canonical edits.
- R07 and its retained display-default table now describe the approved wrap
  and visual-line behavior. The model, command/binding references and Unreleased
  match these delivered changes; historical gate candidates remain intact.

## Files of Interest

- `src/view/visual.rs`, `src/view/scroll.rs`, `src/view.rs`: visual destinations,
  viewport movement and width-qualified goals over the existing fragment owner.
- `src/session/motion.rs`, `src/session/kill.rs`: native command orchestration.
- `src/window/points.rs`, `src/buffer/edit/views.rs`, `src/session/source_batch.rs`:
  shared transactional viewport mapping.
- `tests/wrapping/visual.rs`: native aliases, source/history/kill invariants,
  resize/split goals, paging/recenter, logical goto and anchor seams.
- `tests/coordinates.rs`, `tests/windows.rs`, `tests/editing/regions.rs`:
  retained contracts with explicit wrap and no-wrap assertions.

## Validation

- `cargo test --locked --test wrapping`: passed, including seven visual-editing
  tests and six source-display tests; observed warm execution was 0.50 seconds.
- `cargo test --locked --test wrapping --test windows --test coordinates`:
  passed, 78 tests.
- `cargo test --locked --test editing regions::`: passed, seven regressions.

The configured final formatter, wrapping/editing/windows/core/coordinates tests
and `git diff --check` are recorded by the controller after the PR-reference
commit. Additional source-quality, display and documentation checks use a
separate exact-HEAD receipt. The canonical PR carries the configured command
list in its exact order. No integrated delivery gate is claimed here.

Emacs 31.1 was observed in an owned, fresh `-Q -nw` terminal fixture with twelve
text cells. These source offsets and removed bytes were checked against native
Rev fixtures at the same text width:

| Source and starting offset | C-e offset | C-k text |
| --- | --- | --- |
| `one two three four five` at 1 | 7 | `ne two ` |
| Same source at 7 | 7 | One space |
| `abcdefghijklmnopqrstuvwx` at 0 | 11 | `abcdefghijk` |
| Same source at 11 | 11 | `l` |
| Real source EOL | Same source EOL | LF, or the preserved whole CRLF separator |

The [GNU Emacs command source](https://github.com/emacs-mirror/emacs/blob/master/lisp/simple.el)
defines the visual-edge and wrap-character behavior. Rev retains its approved
read-only admission and grapheme-safe coordinate rules. The reference probe is
development evidence, not an added runtime or gate dependency.

## Risks and Follow-ups

Slice 3 owns complete pointer/search/selection and continuation-gutter mapping.
Slice 4 owns full Org projection integration and bounded resumable long-line
work with measured input/resize ceilings. Neighbor lookup can still rescan a
long source-line prefix; this slice adds no retained cache or idle scanner.
