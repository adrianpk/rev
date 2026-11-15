# Automatic Delimiter Editing

Ordinary source typing inserts a matching parenthesis, bracket or brace in a
proved code or prose context and leaves point between them. An existing unmatched
closer supplies the mate instead of receiving a duplicate. Typing an admitted
existing closer moves past it without a source change or undo group. Angles are
literal. Modal Insert and Replace use the same policy; Normal and Visual keep
their command owners. Replace overwrites physical input characters only.

Backspace between an adjacent empty admitted pair removes both ends in one
native undo step. Whitespace or other content prevents paired deletion. A closing
delimiter otherwise receives ordinary deletion. Active regions, read-only source
and native grapheme boundaries keep their existing owners: a combining character
attached to a delimiter is never split to remove the pair. Uncertain, malformed
or bounded-out context keeps ordinary Backspace.

Quotes depend on the recognized language, even with syntax colors disabled.
Escaped quotes and word interiors stay literal. Quote admission is conservative:

| Profile | Admitted ordinary quotes | Literal or uncertain boundaries |
| --- | --- | --- |
| Plain and other undeclared languages | Double | Apostrophes and backticks. |
| Rust | Double | Apostrophe character/lifetime ambiguity and unproved raw strings. |
| C | Double and single | Escapes, comments and unproved prefixed forms. |
| Go | Double, single and backtick raw strings | Escapes and comments; only proved raw-string boundaries. |
| JavaScript, TypeScript and TSX | Double and single | Templates, interpolation, regular-expression ambiguity and unproved JSX. |
| JSON | Double | Single quotes and backticks. |
| TOML | Ordinary double and single | Triple and unproved multiline strings. |
| CSS | Double and single | Comments and unproved string contexts. |
| HTML | Attribute-value double and single | Other markup, embedded script/style and comments. |
| Org and Markdown | Prose double | Inline code/verbatim, code blocks and unproved embedded language. |

Insertion, skipping and empty-quote deletion require the current recognized
profile and source proof. Existing syntax can strengthen literal exclusion;
stale syntax and external grammar colors do not define additional quote rules.

Use M-x auto-pair-mode to toggle automatic editing for the current buffer and all
its panes. The override survives language changes and never saves configuration.
Startup [pair preferences](defaults.md) resolve globally
and per language. `enabled`, `quotes`, `skip_closing` and `delete_empty`
independently control the delivered automatic effects. `wrap_selection` reserves
the separate selection-wrapping behavior.

A positive prefix applies the typed operation that many times in one checked
undo unit. C-u 3 ( in an empty buffer produces ((())) with point after the three
openers. A zero count is inert; a negative count refuses. Overtyping confirmed
closers changes point only. Consecutive symmetric quote input closes its generated
mate before further adjacent quotes remain literal. Paste and accepted source
replacements insert literal bytes and do not generate mates.

Qualification inspects at most 256 KiB per request, admits depth 256 and retains
at most 2 MiB of pair metadata per buffer. Counted work shares that budget.
Pending work yields through the native input queue, with at most 256 lexical steps
per poll and a four-millisecond scheduling target. C-g cancels before publication;
source, point, mode or policy changes invalidate pending proof. Exhausted counted
work leaves source and history unchanged. Uncertain ordinary typing stays literal.

## Command metadata

| Name | Description |
| --- | --- |
| `auto-pair-mode` | Toggle automatic delimiter editing for this buffer and its views without writing settings. |
