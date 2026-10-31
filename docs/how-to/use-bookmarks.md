# Save and revisit named locations

Open a file in an editing pane and move to a position you want to keep.
If the file tree has focus, press `C-g` to return to editing first.

## Name a position

Press `C-x r m`, type `checkpoint` and press Enter. In `notes.txt`, the prompt
and subsequent feedback look like this:

```text
Set bookmark (default notes.txt): checkpoint
Bookmark set
```

An empty answer uses the displayed basename. Names are case-sensitive and
shared across files, so use distinct names for different locations. Setting an
existing name replaces its position. Use `C-x r M` to refuse replacement:

```text
Bookmark name already exists
```

`C-g` cancels without changing the record. A file-backed buffer can be modified,
read-only or visiting an intended new file. An unnamed scratch buffer cannot
supply a bookmark. Setting a bookmark does not save or modify source text.

## Return to a name

Press `C-x r b`, enter `checkpoint` and press Enter. Up/Down selects a name;
Tab fills the highlighted name without accepting it. Successful feedback is
`Bookmark destination`, or `Bookmark resolved by checked context` after a
checked relocation. The initial completion highlight alone does not accept a name.

Live locations follow edits and source undo. A removed point becomes stale;
undo can restore it, or setting the name again can replace it. After a restart,
rev checks the stored context before opening the location. Ambiguous, missing
or unavailable evidence leaves your current view intact. `C-g` cancels pending
resolution. Save source changes separately with `C-x C-s`; bookmarks cannot
recreate discarded or unsaved source text.

Use `M-x bookmark-next` and `M-x bookmark-previous` for resolved locations in
the current file. They wrap and step through separate names at the same point.

## List or remove names

Press `C-x r l` to open the read-only `*Bookmarks*` view in the selected pane.

- Up/Down or `n`/`p` moves between names; Enter visits the selected location.
- `d` marks a name for deletion; `u` unmarks it. Both advance to the next row.
- `x` deletes the marked names. `q` returns and discards unexecuted marks.

The list shows observed status and file paths; it does not open every target.
If the collection changed elsewhere, the next action refreshes the list and
clears staged marks. Select and mark again before deleting. For one name,
`M-x bookmark-delete` provides a completion prompt.

## Keep the collection across sessions

Ordinary quit saves changed bookmarks after source save/discard choices.
`M-x bookmark-save` saves them immediately and reports `Bookmarks saved`.
It never saves source files. The default destination is
`$XDG_DATA_HOME/rev/bookmarks.toml`, with the usual HOME fallback.

For explicit-only saves, add this to your configuration and restart:

```toml
[bookmarks]
save = "manual"
```

Manual mode leaves unsaved bookmark changes unsaved on quit. `"on-change"`
checkpoints explicit bookmark mutations and updates when you save or rename
a file; typing alone does not write the store.

If storage changed elsewhere, a save refuses to overwrite it. Inspect the
stored data and use `M-x bookmark-reload` after correcting the problem; it asks
before discarding unsaved bookmark changes. A failed automatic quit checkpoint
offers `retry`, `discard` or `cancel`. Discard exits without saving the pending
bookmark changes; cancel or `C-g` keeps the session open. Repeating save alone
does not authorize replacing an unknown or conflicting store.

## Control gutter markers

Resolved source lines show `●` by default, independently of number labels.
Several names on one line share one glyph. Wrapped continuations, folded bodies
and filler remain blank; jumping reveals a folded target.

`M-x bookmark-markers-mode` toggles markers for the current pane. Splits inherit
that pane's visibility, then remain independent. Hiding markers never deletes
names. For different startup defaults, change the existing configuration tables:

```toml
[bookmarks]
markers = false
marker = "*"

[palette.gutter]
bookmark = "#e0af68"
```

The glyph must be one printable non-control grapheme of terminal width one.
Narrow panes omit the marker before reducing available number geometry or the
protected text area. See [Line numbers](../reference/gutter.md) and the
[bookmark reference](../reference/commands.md#named-bookmarks) for exact rules,
storage bounds and the complete command surface.
