# Use completion

Put the cursor in a name and run `M-x completion-at-point`, or press `M-TAB`
when the terminal delivers it. This requests suggestions immediately. With the
default settings, eligible typing also requests them after 150 ms and at least
two Unicode characters. Language separators can admit a request earlier.

Use Up/Down or `C-p`/`C-n` to choose, Tab/Return to insert, and Escape/`C-g` to
cancel. Native acceptance replaces the complete name around point as one undo operation.
An enabled language server can supply a checked primary replacement range in the
same popup; empty or unavailable semantic results use native fallback.
Cancellation, selection and a refused edit leave the text and history unchanged.
Outside a visible list, these keys retain their normal editing actions.

## Change automatic requests

Run `M-x completion-auto-mode` to toggle automatic requests for this session
while keeping manual invocation. Run `M-x completion-mode` to toggle completion
entirely. Neither command writes your configuration.

For a persistent choice, edit the existing `[completion]` table in
`~/.config/rev/config.toml` and restart rev. When `XDG_CONFIG_HOME` is set, use
`rev/config.toml` beneath that directory. For example:

```toml
[completion]
enabled = true
automatic = false
delay_ms = 150
min_prefix = 2
in_comments = false
in_strings = false
max_candidates = 200
popup_rows = 10
```

This is an excerpt to replace the existing table, not a second table to append.
The [complete sample](../reference/config.toml) includes popup keys;
the [configuration catalog](../reference/defaults.md) gives allowed ranges.

## Find the expected context

Use declarations in Rust, C, Go, JavaScript or TypeScript/TSX, tag/attribute names
in HTML, selectors/properties in CSS, or keys/tables in JSON/TOML. Native Org
offers task states, tags, property keys, directives, link types and heading
targets in their structural positions. Ordinary prose and JSON/TOML values
do not produce names. Comments and ordinary strings stay quiet automatically
by default. Bracketed paste does not request completion per character.

If no list appears, check the language status, context and session toggles.
An immediate manual request can report omitted observations when source or
symbol limits apply. Suggestions use current and bounded same-language open
buffers; opening a project directory does not index it. No match leaves editing
available. See [Native completion](../reference/completion.md) for exact contexts,
ranking, fallback and limits, or [Register grammar assets](register-grammar.md)
for another explicitly configured language.

For a guided first insertion, follow
[Complete Names](../tutorials/user-guide/10-complete-names.md).
