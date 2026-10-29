# Org property columns

`M-x org-columns` opens a read-only property projection for the current subtree,
or the whole file when point precedes its first heading. Enter visits the exact
canonical heading; `g` refreshes from current source and `q` closes the projection.
Opening, closing and refreshing columns do not write properties or add history.
The view uses unsaved canonical text, retains source revision/heading provenance,
and marks incomplete clock evidence. It does not select additional agenda files.

Use an inherited heading `COLUMNS` property or a top-level file declaration:

```org
#+COLUMNS: %ITEM %TODO %OWNER %EFFORT{:} %POINTS{+} %CLOCKSUM
```

The native profile admits 1..16 distinct `%PROPERTY` fields and optional `{+}`
numeric or `{:}` duration sums on stored properties. Names have 1..64 ASCII
letters, digits or underscores and start with a letter. Widths, custom titles,
other summaries and executable formatters reject explicitly. The default is
`%ITEM %EFFORT{:} %CLOCKSUM %CLOCKOPEN %CLOCKUNRESOLVED`.

Property sums derive parent values from direct children, working from leaves
upward; the parent's own stored value is not added again. Leaf values retain
their source meaning, empty numeric/duration input is zero, and non-finite or
invalid input rejects the operation. Duration sums display hours and minutes
using configured native duration units. Summaries never write property drawers.

`CLOCKSUM` includes all admitted recorded intervals in each subtree, including
its parent heading's intervals. It uses the operation's configured time zone,
without a calendar date filter. `CLOCKOPEN` and `CLOCKUNRESOLVED` count incomplete
or unresolved records. Reports disclose open, unresolved and overlapping records
even when their status columns were omitted. Unknown/malformed intervals never
become fabricated elapsed time.

`M-x org-columns-insert-dblock` inserts a native report at an admitted blank line
or updates a selected `columnview` block. `org-dblock-update` and contextual
C-c C-c on its opening line update the existing recognized body explicitly:

```org
#+BEGIN: columnview :id global :format "%ITEM %EFFORT{:} %CLOCKSUM"
#+END:
```

Supported options are `:id local` (current subtree, the default) or `global`
(whole file), `:format "..."` and `:maxlevel 1..256`. Unspecified format uses
the source declaration. Unsupported options, ID/file lookups, malformed options,
unknown blocks and values that cannot be represented as native table cells
reject before publication. An inserted block defaults to global scope.

Updates retain the exact opening/closing lines and unrelated source. The whole
body replacement is one checked undo unit, uses the source's newline format,
and rechecks canonical source/view/read-only authority. Identical updates add
no history. Opening/saving a file does not update reports. Clock tables retain
the existing explicit interval/date-range behavior.

Column work shares the existing single cancellable report worker and configured
job deadline. The existing report scan admits at most 262144 source UTF-8 bytes,
4096 headings/records and 1 MiB generated output. Other source/query limits still
apply. C-g cancels owned work; rejected reports leave ordinary editing available.

The [Org column attributes](https://orgmode.org/manual/Column-attributes.html)
and [columnview blocks](https://orgmode.org/manual/Capturing-column-view.html)
define the compatible source forms. The native profile above states the admitted
subset and keeps computed parent properties separate from canonical source.
