# Set Your Preferences

rev works with built-in defaults. This chapter changes one startup setting,
then shows a display preference you can change while editing.

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

## Change line numbers in a pane

Run `M-x gutter-set-line-numbers-relative`. Other lines show their distance
from the cursor; the current line keeps its absolute number.

Run `M-x gutter-set-line-numbers-absolute` to restore the default presentation.
These commands affect this pane for the session. They do not write settings.

For a persistent choice, change `mode` in the existing `[gutter]` table:

```toml
[gutter]
mode = "relative"
```

The [configuration reference](../../reference/defaults.md#startup-configuration)
and [complete sample](../../reference/config.toml) list every supported setting.
Use [Configure indentation by language](../../how-to/language-indentation.md)
when different file types need different values.

You have completed the User Guide. Continue with a
[task guide](../../how-to/index.md), look up a
[command](../../reference/commands.md), or return to
[all documentation](../../index.md).

[Previous](08-org-tables.md) · [User Guide](index.md)
