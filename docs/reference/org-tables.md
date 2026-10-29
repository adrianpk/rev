# Org tables

| Command | Behavior |
| --- | --- |
| `org-table-create` | Enter columns x rows, for example `2x3`; blank input uses `5x2`. Create on a blank line with a separator after the first data row. |
| `org-table-align` | Align the selected native table while retaining the original cell text. |
| `org-table-recalculate` | Apply the first stored `TBLFM` to the selected table as one checked undo unit. |
| `org-table-recalculate-cell` / `org-table-recalculate-column` | Publish only the selected field or column; compute assigned dependencies privately. |
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
`vmax` and `abs` calls, with `::` separating assignments. Compatible `;fN`
and `;%.Nf` fixed decimal formats remain literal source. They do not evaluate formulas.
Relative/current, named, hline, remote, Lisp, Calc-specific or malformed forms
reject structural editing before any source mutation. Removed/out-of-table
reference targets reject instead of losing formula source. Removing a whole
formula-bearing table or converting it to a list also rejects. Alignment and
ordinary field navigation preserve unsupported formula source unchanged.
Reference scans retain the table source limit and admit at most 100000 tokens
and 64 expression recursion levels under the same cancellation/deadline checks.

The native formula syntax owner also recognizes current coordinates (`$0`, `@0`)
and signed relative coordinates (`$+1`, `@-1$-2`) as distinct typed forms. Omitted
coordinates retain their current-row/column meaning. Inclusive ranges retain
both endpoints; data-row resolution and numeric evaluation are separate work.
Structural rewriting still requires absolute coordinates and proves that every
original range target survives consecutively and in order.

Arithmetic syntax associates powers to the right. Powers bind above unary signs;
multiplication binds above division, and both bind above addition/subtraction.
Subtraction and division associate left. Thus `2^3^2` parses as `2^(3^2)`,
`-2^2` as `-(2^2)`, and `8/2*2` as `8/(2*2)`, following the
[Org arithmetic reference](https://orgmode.org/manual/Formula-syntax-for-Calc.html).
The native subset admits no Calc modes, Lisp, named/remote references or
executable function extensions. Aggregate aliases `sum/mean/min/max` retain
their existing `vsum/vmean/vmin/vmax` identities; `abs` takes one argument and
`round` takes one or two. Decimal precision is bounded to 12 places by default,
with an admitted maximum of 15. The parser returns byte-positioned syntax,
unsupported, limit or cancellation errors and never computes result values.

Diagnostic ranges are half-open UTF-8 bytes relative to the original formula
value, with an empty EOF range for a missing operand or delimiter. Malformed
numeric literals and overflowing coordinates identify the whole numeric token.
Missing numeric coordinates are syntax errors; named/hline/remote/Lisp forms
remain explicitly unsupported. Decimal precision accepts unsigned digits only.
No parser observation can authorize an edit of a newer buffer revision.

`org::formula::Program::evaluate` computes private results over an immutable
`eval::Grid`. Row numbers exclude separators; a ragged missing field is empty.
Column formulas skip rows above the first separating hline. Explicit field
formulas override column formulas; the last definition of the same field or
column wins. Relative coordinates resolve from each output cell. Range endpoints
define an inclusive rectangle, regardless of endpoint direction.

The evaluator uses finite binary64 values. Empty input is zero; aggregate calls
omit text, while scalar arithmetic rejects it. An aggregate with no numeric
inputs returns zero for `sum` and rejects `mean`, `min` and `max`. Dependency
values retain binary64 precision until explicit `round` calls. Default output
uses at most 12 significant digits; `;fN` and `;%.Nf` select fixed decimal text.
Rounding uses nearest with halfway values away from zero, including negative
values. `round(value, places)` admits signed integer decimal precision; negative
precision rounds to tens, hundreds and so on. Binary64 representation still
applies to decimal input and intermediate arithmetic.

The numeric policy defaults to 100000 grid cells, 1000000 work steps and 12
decimal places; ceilings are 1000000 cells, 10000000 steps and 15 significant or
decimal digits. Cancellation and worker deadlines share the existing token.
Cycles, invalid references, division by zero, non-finite input/results and
exhausted limits reject the entire private result set. Evaluation does not edit
canonical source or create history; recalculation publication is a separate
transaction layer.

Explicit recalculation publishes changed result fields only, preserving formula
text, unrelated values, padding and separators. Every destination must be an
existing source field. A ragged missing input still counts as zero; a missing
output field rejects the complete operation. The operation rechecks the captured
source and initiating view, uses ordinary point/mark affinity, honors read-only
authority and adds no history for identical results. One undo restores all prior
result fields; redo restores the checked calculation. Opening, saving, coloring,
alignment and navigation do not evaluate stored formulas.

In Org, C-c * runs whole-table `org-table-recalculate`. The same command and
cell/column variants are available through M-x or configured Org bindings.
C-c C-c on a `#+TBLFM` line applies that line to its owning table. Other formula
lines remain alternative programs; ordinary table recalculation uses the first
line, following [Org's multiple-formula behavior](https://orgmode.org/manual/Editing-and-debugging-formulas.html).
C-c C-c in table data retains alignment behavior. Cell/column recalculation
requires a data field, rejects an absent formula target, and evaluates only the
selected outputs and their transitive assigned dependencies. Unselected cells
remain byte-identical, including dependency result fields. Unsupported syntax
anywhere in the selected program rejects before evaluation.

`org.tables.significant_digits` selects 1..15 output digits (default 12).
`org.limits.formula_cells`, `formula_steps`, `formula_depth` and
`formula_decimal_places` constrain grid size, work per syntax/numeric stage,
syntax nesting and decimal precision. Their defaults and ceilings are listed
in the configuration reference. The existing job deadline applies to the whole
operation. Empty numeric input and half-away rounding retain the fixed native
profile. Preferences are captured with the initiating buffer's configuration.

`org-table-import` reads an explicitly selected `.csv` or `.tsv` file into a
blank line in the current Org buffer as one undo unit. Paths are literal and
relative to the Org file's directory (or the working directory for an unvisited
buffer). An open canonical input buffer supplies its unsaved text. Import adds
no file visit and does not save the Org document.

`org-table-export` writes the selected table's data rows to an explicit `.csv`
or `.tsv` path. Separators, names, captions and formulas stay in Org source;
export never evaluates formulas or modifies/saves the source. Existing output
files require `yes` confirmation tied to their observed version. Files owned by
live buffers, including aliases, cannot be export destinations. Publication uses
the ordinary checked temporary-file replacement and reports unresolved completion
truthfully. Exported records use LF and quote every field, doubling literal quotes;
TSV uses the same quoting with a tab delimiter.

Delimited input admits UTF-8 with an optional leading BOM, LF/CRLF records,
quoted delimiters, doubled quotes, empty cells and Unicode. Each row must have
the same number of fields. Embedded cell line breaks (including Unicode line/paragraph separators), literal vertical bars,
controls other than internal tabs, and leading/trailing spaces or tabs reject
before mutation because native table values cannot retain them losslessly.
Malformed quoting and ragged input/export reject without lossy recovery.
Interchange retains the native table row/column/cell/source/output bounds.

Property columns and explicit dynamic reports are described in
[Org property columns](org-columns.md).
