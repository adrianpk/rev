# Calculate a Table

Open `costs.org` with `C-x C-f`. Type or paste:

```org
| Item | Quantity | Price | Total |
|------+----------+-------+-------|
| Pencil | 2 | 1.50 | 0 |
| Notebook | 1 | 4.00 | 0 |
#+TBLFM: $4=$2*$3;%.2f
```

## Align the rows

Place the cursor inside a table row and press `C-c C-c`. rev aligns the
columns. Tab moves to the next field; Shift-Tab moves to the previous one.

## Calculate the totals

With the cursor still in the table, run `M-x org-table-recalculate`.
The Total cells become `3.00` and `4.00`. The formula multiplies the second
and third columns and displays two decimal places in the fourth.

Recalculation changes cell values. Press `C-c C-c` again if their new widths
need alignment, then save with `C-x C-s`.

Try changing the pencil quantity to `3` and recalculating. Its total becomes
`4.50`. Formulas run only when you request them; editing a value does not
automatically recalculate the table.

The [table reference](../../reference/org-tables.md) describes sums, rounding,
row and column editing, and import/export. rev supports a bounded numeric
formula language; it does not evaluate Emacs Lisp or the full Emacs Calc language.

Continue from the [Org mode guide](org/index.md) for capture, source blocks,
exports and the complete Org reference.

[Previous](07-org-tasks-and-agenda.md) · [Org mode guide](org/index.md)
