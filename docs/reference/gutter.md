# Line numbers

Editing views show one-based logical line numbers on the left with no required
configuration. The number field is right-aligned, reserves at least four digits,
has no left padding and one right padding cell, and ends with `│`.
The terminal renderer also admits a one-cell ASCII `|` fallback.

The current logical line uses a distinct color and bold emphasis. Bold remains
available when terminal colors are disabled. Empty documents show line 1;
a final editable empty line is numbered, while blank viewport padding beyond
EOF has no label. CRLF counts as one separator; tabs, wide glyphs and combining
characters do not create extra logical lines.

Width follows the complete buffer's line count, so crossing 9999/10000 lines
grows the field before drawing text and cursor. Horizontal scrolling moves only
text. Tab stops and reported logical columns remain relative to document text.
If the complete field cannot fit beside at least three text columns, numbers
hide until geometry permits them again.

Numbers are presentation. Copying a region or rectangle, searching, replacing,
saving and undo use canonical document text; labels and separators never become
file bytes, history entries or dirty-state changes. Borders, status rows, prompts,
completion candidates and the file tree keep their own presentation bounds.

## Native view policy

The native `gutter::Policy` also supports relative mode. Each noncurrent label
shows its logical distance from that view's point. With point on line 42, lines
40 through 44 show `2, 1, 42, 1, 2`. Focus changes do not move the relative origin
of an inactive pane sharing the same buffer.

In either mode, the current-line form can be absolute (the default), zero or
hidden. Only its label changes; its semantic current-line role and number-field
geometry remain. Relative movement never changes digit width, which follows
the complete source line count rather than visible distances.

Each view keeps its policy while switching buffers and recomputes labels for
the new source. Splits inherit both effective values and override provenance by
value; subsequent updates and reset affect only the addressed view. Hiding and
restoring the field retain the selected mode. Closing a view releases its own
policy, and closing a buffer rebinds its live views through ordinary buffer
lifetime rules.

These native policy values are available to Rust view consumers. Factory terminal
sessions currently display absolute numbers; startup TOML and M-x gutter controls
are not exposed yet.
