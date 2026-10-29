# Org document editing

| Command | Behavior |
| --- | --- |
| `org-footnote-action` (`C-c C-x f`) | Follow the note/reference at point; otherwise create a numeric reference and a definition under Footnotes. |
| `org-footnote-new` | Create a numeric reference and definition. |
| `org-footnote-renumber-fn:N` | Renumber numeric labels in first-reference order, retaining named labels. |
| `org-emphasize` (`C-c C-x C-f`) | Prompt for `* / _ + = ~`; wrap the active region, or insert a marker pair with point between them. A space removes markers from a fully selected emphasis. |
| `org-insert-entity` | Insert a native named entity as `\name{}`. Unknown names reject without edits. |
| `org-edit-affiliated-keyword` | Enter NAME, CAPTION or ATTR_BACKEND, then its literal value. Replace a uniquely owned value or add the keyword to the selected element. |

Footnote labels follow the native Org syntax, including named and anonymous inline
definitions. Missing or duplicate definitions reject without source changes.
Creation chooses an unused positive numeric label. A reference at column zero
receives a leading space so it cannot become a definition. Numeric renumbering
rejects unreferenced numeric definitions instead of guessing their order.

Edits preserve unrelated source, BOM and newline format and use one ordinary undo
group. Shared views map through the canonical buffer transaction. Navigation
retains a checked return point for `org-mark-ring-goto`. Read-only buffers permit
navigation and reject source publication.

Observations use the native 8 MiB source, 200000-node and 256-depth profile, the
single cancellable Org worker and configured job deadline. Footnotes retain at
most 4096 records with 1024-byte labels. Entity names have 64 bytes; affiliated
values have 4096 bytes without controls. Incomplete observations reject. No
idle work, filesystem access, viewer or program execution is introduced.

[Org footnotes](https://orgmode.org/manual/Creating-Footnotes.html) and
[emphasis](https://orgmode.org/manual/Emphasis-and-Monospace.html) define the
compatible source forms and default bindings. Entities remain source; this
command does not replace them with display glyphs or run an exporter.
