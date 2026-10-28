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

## Per-view modes and controls

Use `M-x gutter-set-line-numbers-relative` to show relative numbers in the
active editing view. `M-x gutter-set-line-numbers-absolute` restores absolute
calculation and shows numbers. Each noncurrent label
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

`M-x gutter-toggle-line-numbers` hides or restores the field while retaining its
mode. `M-x gutter-cycle-line-number-mode` alternates modes without changing
visibility. `M-x gutter-cycle-current-line` cycles absolute, zero and hidden;
`M-x gutter-toggle-line-number-side` alternates left/right. These commands have
no additional factory keybinding; editor keymaps can bind their exact names.
They require editing focus, and C-g cancels M-x without changing policy.

`M-x gutter-inspect-numbering` reports effective values and whether each mutable
leaf comes from startup or an explicit view override. Width and padding retain
the startup source. `M-x gutter-reset-numbering` clears this view's overrides,
restoring its immutable startup policy, including visibility. Other views and
text remain unchanged. No command reloads, edits or persists configuration.

## Startup settings

The optional `[gutter]` table sets `visible`, `mode`, `current_line`, `side`,
`minimum_digits`, `left_padding` and `right_padding`. Its factory values are
`true`, `"absolute"`, `"absolute"`, `"left"`, `4`, `0` and `1`, respectively.
Minimum digits admit 1..20 and padding 0..4 cells. Width always accommodates
the complete source count. `[palette.gutter]` supplies `background`, `foreground`,
`current_line` and `separator` as six-digit RGB values.

See the [complete sample](config.toml) and [value catalog](defaults.md#line-numbers)
for all supported values. Includes and root leaves merge independently. Invalid
preferences reject the complete candidate. Editing TOML affects the next startup;
runtime reset continues using the running session's snapshot.

In a folded native Org view, displayed labels retain original logical source
line numbers across hidden ranges. Relative labels measure logical source-line
distance from point, and current-line emphasis follows that same source identity.
Number/text geometry and label width still use the canonical document line count.
