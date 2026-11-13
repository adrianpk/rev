# Markdown tables

Native Markdown supports GFM tables offline. A table requires a header and a
matching delimiter row. Left (`:---`), right (`---:`), center (`:---:`) and
unspecified (`---`) alignment are retained. Outer pipes are optional on input.
Escape a cell's pipe as `\|`, including inside inline code. Editing keeps the
original cell source, including backslashes, entities and inline markup.

Use these named commands from `M-x` or modal `:`. Factory editing keys apply
in non-modal editing and modal Insert. Explicit parent remaps override factory
Markdown routes; explicit `keys.markdown` entries take priority over the parent.

| Action | Command | Factory key |
| --- | --- | --- |
| Create on a blank line | `markdown-table-create` | `C-c \|` |
| Align source padding | `markdown-table-align` | `C-c C-c` in a table |
| Next field | `markdown-table-next-field` | `TAB` in a table |
| Previous field | `markdown-table-previous-field` | `S-TAB` in a table |
| Same field in next row | `markdown-table-next-row` | `RET` through `markdown-return` in a table |

Create asks for explicit **columns x body rows**, for example `2x1`. `2x0`
creates a header and delimiter without body rows. The blank line may have up to
three leading spaces; creation refuses literal code and an active selection.
Creation isolates neighboring paragraphs and tables with blank lines and checks
that the proposed source admits the requested table. Cancel the prompt with
`C-g` to retain the source.

Alignment writes consistent outer pipes and source padding, preserving delimiter
alignment and cell contents. Short body rows gain empty fields. A body row with
extra cells refuses editing because GFM rendering would discard those cells.
Mismatched header/delimiter rows and literal code do not admit table actions.

Navigation skips the delimiter row. Next-field at the last field appends a body
row. Next-row retains the column and appends at the last row. Previous-field at
the first header cell stays there. An already aligned move adds no history.
Each source change, including alignment plus navigation, has one undo step.
Retained source points in other panes follow their cells. Save retains the
buffer's BOM, newline style and final-newline identity.

Read-only buffers refuse table actions. Pending, canceled or stale observations
cannot authorize them. `C-g` cancels an action before publication. Actions check
buffer revision, view and point before publishing. The native profile bounds
source/output to 8 MiB, total rows to 10,000, columns to 256, cells to 100,000
and each cell to 4,096 displayed columns. Existing Markdown byte, event, depth
and worker limits also apply; a lower bound wins.

See [syntax and presentation](syntax.md#native-markdown-source),
[bindings](bindings.md) and [command inventory](commands.md).
