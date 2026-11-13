# Edit Markdown

Write a short Markdown note, change its presentation, continue a list and edit a
table. This lesson uses a small local file and works offline. It is independent
of the Org learning path. Use the ordinary editing profile, or enter Insert
with `i` for text and contextual keys in the modal profile. Run named commands
with `M-x`; modal Normal also accepts `:` followed by the name and Return.

## Open a note

Create a practice directory and open a file:

```sh
mkdir -p /tmp/rev-markdown-lesson
cd /tmp/rev-markdown-lesson
rev notes.md
```

Enter this source and save with C-x C-s:

```markdown
# Field notes
## Observations

- First sample
- Second sample
    + Detail

3. Compare samples
4. Write the result

| Name | Value |
| :--- | ---: |
| 界é | `a\|b` |
```

The pane status identifies Markdown. The first two headings use `◉` and `○`
in distinct colors. Unordered items show `•` and `◦`; their indentation remains
visible. The ordered numbers and escaped pipe stay source text. No language
server is needed.

## Reveal the source

Run `markdown-pretty-mode`. The view shows `#`, `##`, `-` and `+` again, with
heading colors retained. Run `markdown-reset-presentation` to restore the default
pretty view. These commands change presentation without marking the file modified.

Select and copy a heading using the controls from [Move and Edit Text](02-edit-text.md).
Pasting it includes its original hashes. Undo that paste. Saving and reopening
also retains source markers, including the backslash in `a\|b`.

## Continue and move an item

Put point anywhere in `Second sample` and press Return. A new sibling appears
after that item's complete subtree, including `Detail`. Run `M-x undo` once to
restore the saved list. An accepted list action has one undo step. Repeat the
continuation when you want to add and type a third sample.

Return in `Compare samples` inserts a sibling numbered `4.`. The original later
item retains its existing number. On an empty item, Return leaves one list level.
Undo the continuation before the next exercise.

Place point in `Second sample`. `M-Right` indents the complete item and its child
under the preceding sibling. `M-Left` outdents it again. The exact command names
are `markdown-indent-item` and `markdown-outdent-item`. Use named commands when
your terminal reserves an arrow combination.

## Align and change a table

Put point in a table cell and run `markdown-table-align` or press C-c C-c.
Padding becomes consistent, the left/right alignment remains, and the Unicode
text and escaped pipe stay intact. TAB and Shift-TAB move between fields,
skipping the delimiter row. Return moves to the same field in the next row;
moving beyond the final body row adds a row.

Run `markdown-table-insert-row` in a body cell to add a blank row above it.
Meta-Return adds one below instead. Fill the new cells, then use
`markdown-table-delete-row` to remove that body row. The header and delimiter
remain protected.

Run `markdown-table-insert-column` to add a column before the current one.
Run `markdown-table-delete-column` in that new column to remove it. At least one
column must remain. Each source-changing table command, including alignment plus
navigation, has one undo step.

At a blank line outside a table or code block, run `markdown-table-create`.
Enter `2x1` for two columns and one body row. C-g cancels the prompt without an
edit. Save your note, quit with C-x C-c and reopen it to inspect the same Markdown
source and presentation.

The [Markdown reference](../../reference/markdown.md) lists the exact symbols,
startup preferences and list rules. [Markdown tables](../../reference/markdown-tables.md)
lists all keys, source-preservation rules and finite table limits.

[Markdown guide](markdown/index.md) · [User Guide](index.md)
