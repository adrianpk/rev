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
For configuration examples, see [Configure automatic delimiter editing](../how-to/configure-delimiter-editing.md).
Startup [pair preferences](defaults.md) resolve globally
and per language. `enabled`, `quotes`, `skip_closing` and `delete_empty`
independently control the delivered automatic effects. `wrap_selection` controls
automatic selection wrapping.

A positive prefix applies the typed operation that many times in one checked
undo unit. C-u 3 ( in an empty buffer produces ((())) with point after the three
openers. A zero count is inert; a negative count refuses. Overtyping confirmed
closers changes point only. Consecutive symmetric quote input closes its generated
mate before further adjacent quotes remain literal. Paste and accepted source
replacements insert literal bytes and do not generate mates.

Raw macro keys apply the current buffer's typing policy once during replay.
Generated mates are effects of that key, not additional recorded keys. Accepted
paste and completion steps replay their literal payloads, including unmatched
delimiters, without asking a completion provider again. AI result application,
plugin edits, external reload and programmatic replacements also keep their
literal source-edit behavior. Existing macro counts, counters, cancellation and
buffer-local undo groups retain their owners. Closer skipping adds no history;
redo restores the last source edit's point rather than later navigation.

With an active ordinary linear region, typing either side of an admitted pair
wraps the selected bytes once. Opening input leaves point after the generated
openers; closing input leaves it after all generated closers. The region is
deactivated. Both selection orientations and positive structural counts use one
undo step; undo restores the original point, mark and active orientation. Native
rectangular/block and modal Visual input keep their existing behavior.

M-x pair-wrap asks for one admitted opening or closing delimiter. Enter the
character, then press Enter; C-g cancels the prompt and retains the selection.
There is no default pair. Either choice uses opener placement. A positive prefix
chooses nested structural pairs. M-x pair-unwrap removes the innermost confirmed
pair enclosing point, including point on either delimiter. It preserves contents
and maps point/mark through one native edit. Both commands work when automatic
editing or its corresponding preference is disabled. They have no factory key.

Quote wrapping never escapes or reformats selected bytes. It refuses embedded
matching quotes, ordinary escape sequences, invalid single-line content and
ambiguous repeated quote counts. C/Go single-character quotes require one scalar;
Go backticks and ordinary TOML single quotes retain their raw-content rules.
Quote unwrapping refuses escape sequences and nested quote forms whose literal
interpretation cannot be proved safe. Unconfirmed helpers refuse before mutation;
uncertain ordinary structural typing retains the existing literal-input owner.

Native Org and Markdown heading presentation can remain available during body
typing and same-line wrap/unwrap while exact analysis refreshes. This display
continuity does not authorize structural actions from stale analysis. The native
32 KiB inspection and eight-row carry limits remain unchanged; paired edits share
one inspection budget and admit only matched delimiter insertions or removals
when two replacements share one source line.
Line-structure changes, larger batches and unproved literal contexts require fresh
analysis. Source spans, shared points and undo history remain native coordinates.

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
| `pair-highlight-mode` | Toggle this view between off and its last active delimiter style without writing settings. |
| `pair-highlight-style` | Choose depth, match, pulse or off delimiter presentation for this view. |
| `pair-wrap` | Choose an admitted delimiter and wrap the active linear region in one undo step. |
| `pair-unwrap` | Remove the innermost confirmed enclosing pair while preserving its contents. |
| `matching-delimiter` | Jump to the confirmed typed mate at or immediately before point without editing source. |
| `forward-pair` | Move forward across counted mode-defined groups without editing source. |
| `backward-pair` | Move backward across counted mode-defined groups without editing source. |
| `up-pair` | Move backward out of counted enclosing groups without editing source. |
| `down-pair` | Move forward into counted nested groups without editing source. |
| `forward-sexp` | Move forward across counted mode-defined expressions without editing source. |
| `backward-sexp` | Move backward across counted mode-defined expressions without editing source. |

The explicit matching-delimiter command is unbound. It uses the same confirmed
source facts, works on read-only source and ignores automatic-editing preferences.
It preserves an active linear mark, rejects numeric prefixes and refuses
unconfirmed endpoints without movement. See [Commands](commands.md#implemented-catalog)
for its entry policy and the distinct modal % behavior.

The unbound pair-highlight-mode and pair-highlight-style commands configure one
view's presentation independently of automatic editing. See
[Language rules](defaults.md#language-rules) for their startup defaults, prompt
admission and current rendering boundary.
