# Org tables

| Command | Behavior |
| --- | --- |
| `org-table-create` | Enter columns x rows, for example `2x3`; blank input uses `5x2`. Create on a blank line with a separator after the first data row. |
| `org-table-align` | Align the selected native table while retaining the original cell text. |
| `org-table-next-field` / `org-table-previous-field` | Align, then visit the next/previous field, skipping separators. Next field appends a row when needed. |
| `org-table-next-row` | Align, then visit the same column in the next row, inserting before a separator or at the end when needed. At line boundaries, insert an ordinary newline. |
| `org-table-beginning-of-field` / `org-table-end-of-field` | Align and visit the current field's beginning/end. |
| `org-table-insert-row` | Insert an empty row above the current row. |
| `org-table-insert-hline` | Insert a separator below the current row. |
| `org-return` | Use next-row behavior in native tables and ordinary newline editing elsewhere. |

In Org context, TAB/S-TAB invoke table next/previous field behavior when point is
in a table. C-c C-c aligns the table; M-RET inserts a row below. RET runs
`org-return`, C-c followed by the vertical-bar key creates a table, and C-c -
inserts a separator. The same commands are available through M-x.

The first row sets indentation. Native cell values retain their original bytes,
including internal tabs, markup, entities and Unicode. Alignment uses the source
canvas glyph/tab policy and changes surrounding padding/delimiters; separators
span the actual columns. Numeric-rich columns align right. Explicit `<l>`, `<c>`
and `<r>` cookies override alignment; optional width spellings remain source.
Values are never truncated to fit a display width cookie.

Table work uses the existing single cancellable worker and configured job deadline.
C-g cancels prompts or active work. Publication rechecks source revision and view;
actual buffer transactions enforce read-only authority and provide one-operation
undo, shared source positions and ordinary saves. Navigation through an already
aligned table adds no history. Source changes retire stale fold observations.

The native table profile admits 8 MiB source/output, 10000 rows including
separators, 256 columns, 100000 rendered data cells and 4096 displayed columns per
cell. Ordinary text remains editable when a structured action reaches a limit.

[The Org table editor](https://orgmode.org/manual/Built_002din-Table-Editor.html)
defines the compatible delimiter, field-motion and separator forms. Names,
captions and formula source remain associated with the table. Row insertion in a
table with stored formulas rejects before publication because it requires checked
reference rewriting. Alignment and navigation never evaluate formulas.
