# Automatic Delimiter Editing

Ordinary source typing inserts a matching parenthesis, bracket or brace in a
proved code or prose context and leaves point between them. An existing unmatched
closer supplies the mate instead of receiving a duplicate. Typing an admitted
existing closer moves past it without a source change or undo group. Angles are
literal. Modal Insert and Replace use the same policy; Normal and Visual keep
their command owners. Replace overwrites physical input characters only.

Quotes depend on the recognized language, even with syntax colors disabled.
Double quotes require an eligible boundary; plain text apostrophes and backticks
remain literal. Rust apostrophes, unproved raw/triple strings, script templates,
comments, quoted contents and embedded literal regions retain conservative literal
input. HTML quotes require an attribute-value boundary. Adjacent quote runs and
word interiors do not acquire speculative mates.

Use M-x auto-pair-mode to toggle automatic editing for the current buffer and all
its panes. The override survives language changes and never saves configuration.
Startup [pair preferences](defaults.md) resolve globally
and per language. `enabled`, `quotes` and `skip_closing` independently control
the delivered typing effects. Wrapping and paired deletion have separate settings.

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
