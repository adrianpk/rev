# Configure indentation by language

To use four spaces generally and two in Rust files, edit the existing
`[editor]` table in `~/.config/rev/config.toml` and add or update
`[language.rust]`:

```toml
[editor]
indent_style = "spaces"
indent_width = 4

[language.rust]
indent_width = 2
```

Save the configuration and start a new session with `rev main.rs`.
At the beginning of a line, Tab inserts two spaces. A new session opening
`notes.txt` still inserts four.

The built-in `.rs` association selects `rust`; no extra association is needed.
Other language settings inherit the global values unless explicitly changed.

For literal tabs instead, set the language's `indent_style = "tabs"`.
`tab_width` controls how wide existing tab characters appear. Neither option
converts existing spaces or tabs. For an explicit conversion, mark a region
and run `M-x tabify` or `M-x untabify`.

The [language rules](../reference/defaults.md#language-rules) list the built-in
filename associations and accepted overrides. For another configuration
location, see [startup configuration](../reference/defaults.md#startup-configuration).
