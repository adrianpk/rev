# Org support in rev

Org documents remain plain text. Headings, task states, properties, dates,
links, tables and source blocks are written in the document itself. Native
Rust code interprets the supported forms and supplies editing commands.
No Emacs process or package installation is needed.

Folds and agendas are views of that source. Folding hides text in a particular
pane without deleting it. An agenda collects task entries from selected sources;
visiting an entry returns to its real heading. Unsaved text in an open source
buffer takes precedence over the disk version.

Capture is different from ordinary unsaved editing: finishing a capture inserts
the draft into its chosen destination and saves it. The template and destination
are preferences, not an automatically discovered project configuration.

## Tables

Native formulas cover numeric arithmetic, aggregates and rounding. They are
useful for small tables without embedding a spreadsheet engine or an Emacs Lisp
runtime. A stored formula is source text until an explicit recalculation.
Alignment adjusts the layout independently of calculation.

The supported language has explicit limits. It does not evaluate the full
Emacs Calc language, Lisp formulas or arbitrary code. Ordinary table text
remains editable when a formula is unsupported. The
[table reference](../reference/org-tables.md) defines the accepted forms.

## Source blocks

A source block can simply document code. A separate editing buffer makes its
body easier to change, while tangling writes declared blocks into code files.
Neither operation runs the code.

Execution is a separate, optional operation. It requires configured external
programs and the applicable confirmation; it is not needed for notes, task
planning, tables, source editing or tangling. See
[Org source blocks](../reference/org-source-blocks.md).

Native exports likewise render document content; optional PDF compilation and
publishing tools have their own configuration and invocation.

Start with [Write Org Notes](../tutorials/user-guide/06-org-notes.md).
The [Org control catalog](../reference/org-controls.md) links the full delivered
set: outlines, tasks, agenda, capture, clocks, links, attachments, tables,
exports, source blocks and feeds. It describes rev's supported Org behavior,
not every capability of Emacs Org mode.
