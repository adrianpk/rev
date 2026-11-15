# Configure automatic delimiter editing

Use these controls when you want different automatic punctuation behavior for
one buffer or language. Start with the [editing lesson](../tutorials/user-guide/02-edit-text.md)
to try insertion, deletion and wrapping.

## Toggle the current buffer

Run `M-x auto-pair-mode`. The toggle affects every pane showing that buffer and
survives language changes until the buffer closes. Run it again to toggle back.
It does not write configuration. Explicit `M-x pair-wrap` and `M-x pair-unwrap`
remain available when automatic editing is off.

## Set startup preferences

Edit your startup configuration, normally `~/.config/rev/config.toml`, and restart
Rev. All five preferences default to true. For example, keep ordinary mates but
disable automatic selection wrapping and quotes:

```toml
[pairs]
enabled = true
wrap_selection = false
delete_empty = true
skip_closing = true
quotes = false
```

`enabled` controls automatic editing as a whole. The other preferences independently
control wrapping an active linear selection, Backspace between adjacent empty
delimiters, moving past a confirmed closer and language-eligible quote editing.
Paste, completion and accepted generated text always remain literal.

## Override one language

Add only the leaves you want to change. This example disables automatic editing
in Rust and keeps structural editing without automatic quotes in Markdown:

```toml
[language.rust.pairs]
enabled = false

[language.markdown.pairs]
quotes = false
```

Omitted language leaves inherit the global preferences. Language identification
still applies with syntax colors disabled. A buffer's explicit runtime toggle
takes precedence over its resolved `enabled` setting. Unknown names and wrong
types reject the configuration candidate through the normal startup diagnostic;
see [configuration loading](../reference/defaults.md#startup-configuration).

## Wrap or remove a pair explicitly

Select an ordinary linear region, run `M-x pair-wrap`, enter an admitted opening
or closing delimiter, then press Enter. `C-g` cancels without changing the selection.
Point finishes after the opener. To remove the innermost confirmed enclosing pair,
place point inside it or on either delimiter and run `M-x pair-unwrap`.
One undo restores the original source and selection. Neither helper escapes or
reformats content; unsafe quote transformations refuse before changing text.

Read [Automatic Delimiter Editing](../reference/delimiter-editing.md) for the quote
matrix, count semantics, literal-input policy and fixed work limits. These commands
have no factory key assignment; [custom bindings](../reference/bindings.md#configuration-keymap-contracts) can assign one.
