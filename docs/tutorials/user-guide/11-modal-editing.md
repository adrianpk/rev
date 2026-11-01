# Choose Modal Editing

Learn the optional modal profile, with Vim-like editing, using a small file.
The earlier chapters use the default non-modal profile, with Emacs-like editing,
where ordinary text enters directly. Both profiles share files, commands,
selections, completion and undo.

## Select the startup profile

Save your files and quit. Open `~/.config/rev/config.toml` and change the existing
`[editor]` table:

```toml
[editor]
interaction_profile = "modal"
```

Keep the other entries in that table. With `XDG_CONFIG_HOME`, use
`rev/config.toml` beneath that directory instead. Save and restart rev with a new
practice file:

```sh
rev modal-practice.txt
```

The status line shows `NORMAL`, and typing ordinary letters does not enter text.
The startup choice applies to every buffer and source draft in this session.
Changing the file does not switch a running editor. To return to non-modal
editing, set `interaction_profile = "non-modal"`, save and restart. Omitting the
setting also selects non-modal editing.

## Enter text and return to Normal

Press `i`, type the following lines, then press Escape:

```text
one two
three four
```

The status changes from `INSERT` to `NORMAL`; the native cursor stays at its
checked position. `C-[` has the same editing transition. Use `a` to insert after
the current grapheme, `I` at the first text on a line, `A` at its end, or `o`/`O`
to open a line below/above. `R` enters `REPLACE`: typing overwrites whole
graphemes on this line and inserts at its end without consuming the next line.
Escape returns to Normal.

In Normal, use `h/j/k/l` or the arrows to move. `j/k` follow logical source lines
even when a line wraps. `0` and `$` reach line boundaries, `w/b/e` move by words,
and `gg/G` reach the beginning/end of the file. A count applies to one operation:
`2j` moves two logical lines, and `2G` goes to line two.

## Compose an edit and undo it

Press `gg0` to reach the start of the first line, then `dw`. The file begins with
`two`: delete and the word target form one operation. Press `u` to restore
`one two` in one step. `C-r` redoes that operation; press `u` again to restore it.

Use `c` with a target to change text and enter Insert, or `y` to copy it. `dd`,
`cc` and `yy` act on complete lines. `diw` deletes the current word; `ci"` changes
inside a quoted pair. Counts can compose: `2d3w` selects six word targets before
one checked edit. A missing target or a read-only buffer leaves the text intact.

For a repeat exercise, enter `iX`, press Escape, then `j0.`. Both lines now start
with `X`. Dot resolves the last supported change at the current target. Press
`u` twice to restore the original file. Insert/Replace text and each repeated
change use ordinary undo groups. Oversized changes keep their full text and undo
but may be unavailable to dot.

## Select a shape

At `gg0`, press `v`, then `l`. Visual Character includes the current and destination
graphemes. `y` copies the selection and returns to Normal; `p` pastes retained
text. Use `u` to reverse a paste.

`V` selects complete logical lines. `C-v` selects a block in logical grapheme
columns: extend it with motions, then use `y`, `d` or `c`. Tabs count as one
logical grapheme in this modal block; non-modal rectangles keep their display-cell
edges. `o` exchanges the active end. Press the same shape key again or Escape
to leave Visual without an edit.

## Search, revisit and arrange panes

Type `/two`, press Enter, then use `n` or `N` for the next/previous match. Search
is literal, uses smart case and does not wrap. `?` searches backward; `C-g`
cancels a search preview and restores its origin.

Nonlocal moves such as `2G`, accepted searches and file visits enter the pane's
checked jump history. In Normal, `C-o` revisits an older location and `C-i` a
newer one. Many terminals report `C-i` as Tab; an active prompt or completion
popup keeps its own Tab action. In Insert, `C-o` runs one complete Normal command
and resumes insertion when its source and pane remain compatible.

In Normal or Visual, use the `C-w` prefix to arrange panes: `s` splits below,
`v` splits right, `h/j/k/l` changes focus, `c` closes this pane, `o` keeps only
this pane, and `=` balances the existing divisions.

## Use shared commands and completion

Press `:`, type `goto-line`, then Enter. The ordinary line-number prompt opens;
enter `1` and press Enter. Colon opens the same vertical command selector as
`M-x`, with canonical names, matching, history, candidates and argument prompts.
For example, `:save-buffer` invokes the normal save command. Use full names;
`:w`, `:q`, Ex ranges and trailing arguments are not command forms here.

Cancelling the selector with `C-g` retains a compatible Visual selection. A
changed source or selected view retires it safely. Tree, search, argument prompts,
bookmarks and local Org controls retain their own input meanings. Insert and
argument prompts treat colon and slash as text. Org source drafts use this
startup profile with their existing save/abort controls.

Follow [Complete Names](10-complete-names.md) in a Rust practice file. Enter
Insert with `i`, or Replace with `R`, before typing an eligible name. Automatic
native or optional language-server suggestions appear only in those text-entry
states. Manual `M-x completion-at-point` remains available. Popup arrows/C-p/C-n
select, Tab or Return accepts, and Escape/C-g cancels locally. After cancellation,
a later editing Escape returns to Normal. Checked completion and imports undo in
one step; retired replies cannot reopen a popup or edit a different target.

Save with `:save-buffer` or `C-x C-s`, then quit with `C-x C-c`. Both profiles use
the same unsaved-file confirmation. See the [binding reference](../../reference/bindings.md#modal-startup-profile)
for the finite supported operations and [defaults](../../reference/defaults.md)
for startup and state-map configuration.

You have completed the User Guide. Continue with a
[task guide](../../how-to/index.md) or look up a
[command](../../reference/commands.md).

[Previous](10-complete-names.md) · [User Guide](index.md)
