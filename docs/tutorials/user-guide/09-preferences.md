# Set Your Preferences

rev works with built-in defaults. This chapter changes startup settings and
shows display preferences you can change while editing.

## Change indentation

Normal first startup creates `~/.config/rev/config.toml` if it is absent.
Save your work and quit with `C-x C-c`. In the terminal, open the configuration:

```sh
rev ~/.config/rev/config.toml
```

Find the existing `[editor]` table and change its `indent_width` to `2`:

```toml
[editor]
indent_width = 2
```

Keep the other settings you want; this is an excerpt, not a second table to
append. Save and quit, then open `notes.txt` again. At the start of a line,
Tab now inserts two spaces with the default `indent_style = "spaces"`.
Existing text is not converted.

Settings load at startup. Saving the configuration does not reload it into
an already running session. If you set `XDG_CONFIG_HOME`, the file is
`rev/config.toml` beneath that directory instead.

## Choose how long lines display

Wrapping is enabled by default. Run `M-x visual-line-mode` to toggle wrapping
for the current buffer and all panes showing it. Each pane reflows at its own
text width; resizing keeps source positions. The toggle does not write settings
or create an undo step.

For horizontal scrolling at startup, change `soft_wrap` in the existing
`[editor]` table and restart:

```toml
[editor]
soft_wrap = false
```

With wrapping disabled, `C-n`/`C-p`, `C-a`/`C-e` and `C-k` use logical lines.
The same choice applies to visible Org text, including tables and source blocks;
folded text stays hidden. Use the separate [Org guide](org/index.md) for those
structured editing commands.

## Change line numbers in a pane

Run `M-x gutter-set-line-numbers-relative`. Other lines show their distance
from the cursor; the current line keeps its absolute number.

With the cursor on line 2, the number column changes like this:

```text
Absolute    Relative
1 │         1 │
2 │         2 │
3 │         1 │
```

Run `M-x gutter-set-line-numbers-absolute` to restore the default presentation.
These commands affect this pane for the session. They do not write settings.

For a persistent choice, change `mode` in the existing `[gutter]` table:

```toml
[gutter]
mode = "relative"
```

## Choose a bookmark symbol

Set a bookmark with `C-x r m`, type `checkpoint` and press Enter. Its source line
shows a blue square `■` before the left line number. Several names on that line
share the same marker. Hiding line numbers keeps it visible.

If your terminal uses a font with Nerd Fonts support, change the existing
`[gutter]` table and restart to use the flat bookmark ``:

```toml
[gutter]
nerd_fonts = true
```

Rev does not detect or install a font. Leave this false for the ordinary square.
Copying text never includes the marker or the numbers. For settings created by
an older version, follow the [bookmark migration guidance](../../reference/gutter.md#migrate-older-bookmark-settings)
to remove an ignored custom marker string or replace an explicit old color.

The [configuration reference](../../reference/defaults.md#startup-configuration)
and [complete sample](../../reference/config.toml) list every supported setting.
Use [Configure indentation by language](../../how-to/language-indentation.md)
when different file types need different values.

Next: [Complete Names](10-complete-names.md).

[Previous](05-file-tree.md) · [User Guide](index.md)
