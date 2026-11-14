---
id: TKT-20261113200442
title: Preserve Org presentation across structural edits
status: open
kind: bug
severity: unclassified
priority: unclassified
scope: ui
tags: org-mode, rendering, flicker
source: chat
reported_at: 2026-11-13T20:04:42Z
commits: [d6d38d3fffc5701d1514f75f9a1c7ff5a96780f0, af716022be8dc962cb28645e8f5e4de2ba496efa]
---

# Preserve Org presentation across structural edits

## Observed behavior

Creating a new Org heading or list item can still expose a temporary literal
presentation before current structural analysis finishes. Unchanged headings
can lose their symbols, visual margins or heading-level colors during that
interval. Ordinary admitted edits within one line now preserve presentation.

## Delivered baseline

The completed [presentation continuity specification](../../spec/editor-presentation-continuity.md)
and its [acceptance report](../../report/slices/rev-presentation-continuity/slice-5-acceptance.md)
deliver checked display carry for single-change, same-line body/title edits.
That correction is integrated into dev and passed its controller-owned gate.

Edits containing CR/LF, multi-change transactions and structural or uncertain
contexts deliberately retire carry. Their literal fallback is outside the
completed correction's initial scope. This ticket retains the remaining
multiline/structural presentation concern; it does not reopen same-line delivery.

## Reproduction

1. Open an Org document with nested headings and body/list content.
2. Enable pretty headings and visual indentation. Type within a recognized
   title or ordinary body line and confirm that presentation remains stable.
3. Insert a new heading or list item, adding a line break. Observe unchanged
   heading symbols, margins and colors until structural analysis completes.
4. Repeat with only pretty headings enabled and only indentation enabled.

## Expected behavior

Line insertion and structural editing keep unaffected heading presentation
continuous while displaying current text. Changed structure updates coherently
without inventing headings inside literal contexts or freezing an incorrect
level. Cursor, selection, wrapping and pointer targets describe the source
actually displayed. Canonical stars, whitespace and editing semantics remain
unchanged.

## Engineering boundary

The current same-line mapping cannot be reused for changed row counts without
new validity rules. Any extension must account for line shifts, affected
structural context and convergence before retaining presentation. Old and new
block/drawer boundaries both matter. Display evidence must remain separate
from the exact current observations that authorize structural commands.

Keep preparation, publication, cancellation and resource ownership explicit.
Do not hide the transition by delaying all input, accepting unchecked stale
offsets or treating synchronized terminal output as structural continuity.
The design for multiline/structural continuation remains to be specified.

## Acceptance and validation

- Observe every intermediate native/terminal update when creating headings
  and list items, joining/splitting lines and pasting multiline source.
- Keep unaffected symbols, margins and heading faces continuous while letting
  genuinely changed text and structure reflow through checked current mappings.
- Cover heading-depth changes, undo/redo, source/example blocks and drawers,
  including boundary deletion/corruption, Unicode and narrow wrapped panes.
- Preserve source bytes, history, copy/save, selection/search precedence,
  current structural-command authority and independent view policies.
- Preserve cancellation, incomplete/over-limit fallback and bounded work/state;
  compare accepted mappings with fresh full analysis and report actual costs.
- Retain the completed same-line continuity cases and unaffected source modes.

Capture and version the remaining concern only. No successor delivery or
implementation is started by this ticket.
