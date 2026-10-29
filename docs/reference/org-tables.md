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
Width-only cookies do not count as data when choosing numeric alignment. Values
are never truncated to fit a display width cookie. Creating between existing
tables retains blank boundaries so their rows remain separate tables.

Table work uses the existing single cancellable worker and configured job deadline.
C-g cancels prompts or active work. Publication rechecks source revision and view;
actual buffer transactions enforce read-only authority and provide one-operation
undo, shared source positions and ordinary saves. Navigation through an already
aligned table adds no history. Source changes retire stale fold observations.

The native table profile admits 8 MiB source/output, 10000 rows including
separators, 256 columns, 100000 rendered data cells and 4096 displayed columns per
cell. Leading combining marks use the real padding context, and field destinations
remain complete grapheme boundaries. Ordinary text remains editable when a structured action reaches a limit.

[The Org table editor](https://orgmode.org/manual/Built_002din-Table-Editor.html)
defines the compatible delimiter, field-motion and separator forms. Names,
captions and formula source remain associated with the table. Alignment and
navigation never evaluate formulas.

Row and column structure commands are available through M-x. In a table,
M-Up/M-Down move the current row (including separators), and M-Left/M-Right
move its column in every data row. M-S-Left removes a column; M-S-Right inserts
an empty column before the current one. M-S-Up kills a row or separator and
M-S-Down inserts above. Moving beyond a table boundary rejects. The final
column cannot be deleted; use row removal to remove an entire table.

`org-table-sort-lines` (C-c ^) sorts data rows between the nearest separators.
Select a data field first. With an active region, the mark determines the sort
column and both endpoint rows are included; the region must stay inside one
separator group. The prompt admits a/A for stable Unicode case-insensitive
alphabetical order, or n/N for finite binary64 numeric order. Uppercase selects
reverse order, retaining equal-key order. Empty numeric fields sort as zero;
other nonnumeric fields reject the whole operation. No expression is evaluated.

`org-list-to-table` converts contiguous, flat single-line native list items into
a one-column table. Literal cell text is retained; nested/continued items,
blank-separated items and literal vertical-bar delimiters reject.
`org-table-to-list` converts one-column tables without separators into ordinary
unordered items. Other table shapes reject before mutation. Original bullets
and spacing return through ordinary one-operation undo.

Shared points and marks inside moved cell text follow the retained source.
The canonical buffer owner verifies the original identity, candidate bytes and
disjoint retained ranges before publication; a proposed anchor cannot grant
mutation authority. Other positions use normal checked edit affinity.

Structural edits rewrite supported absolute `TBLFM` assignment targets and
operands: `$N`, `@N`, `@N$M`, and same-form inclusive ranges. Data row numbers
ignore separators. Changed coordinates follow the original logical row/column;
unchanged coordinate spelling and all other formula bytes remain intact. A range
must retain every original target consecutively and in order. Inserting through
or reordering within a range rejects, even if its endpoints still exist.

Checked rewrites admit finite numeric literals, arithmetic `+ - * / ^`,
parentheses, and `sum`, `mean`, `min`, `max`, `round`, `vsum`, `vmean`, `vmin`,
`vmax` calls, with `::` separating assignments. They do not evaluate formulas.
Relative/current, named, hline, remote, Lisp, Calc-specific or malformed forms
reject structural editing before any source mutation. Removed/out-of-table
reference targets reject instead of losing formula source. Removing a whole
formula-bearing table or converting it to a list also rejects. Alignment and
ordinary field navigation preserve unsupported formula source unchanged.
Reference scans retain the table source limit and admit at most 100000 tokens
and 64 parenthesis levels under the same cancellation/deadline checks.
