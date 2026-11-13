# Native Markdown

Files ending in `.md` or `.markdown` open in native Markdown mode. Recognition,
colors, presentation and structural editing work offline without a language
server, external grammar or plugin. `markdown-mode` selects it explicitly;
`text-mode` returns to ordinary text. Markdown is independent of Org mode.

For a guided exercise, read [Edit Markdown](../tutorials/user-guide/23-markdown.md).
For table placement, alignment, navigation and refusal rules, see
[Markdown tables](markdown-tables.md).

## Presentation and source

Native Markdown recognizes CommonMark headings, emphasis, strong text, links,
inline and fenced code, quotes and lists, plus GFM tables. Literal code retains
its source markers. A seventh hash does not introduce an ATX heading.

| Heading level | Pretty ATX marker | Default color |
| --- | --- | --- |
| 1 | ◉ | `#7aa2f7` |
| 2 | ○ | `#9d7cd8` |
| 3 | ✸ | `#c0caf5` |
| 4 | ✿ | `#9bb9f9` |
| 5 | ◆ | `#b59ce1` |
| 6 | ▶ | `#bcd0fb` |

Pretty presentation replaces only admitted ATX prefix cells, retaining their
width. Setext titles receive the level-one or level-two color; their `=` or `-`
underline stays literal. Unordered lists use `•`, `◦`, `▪`, repeating by admitted
nesting depth. Indentation and ordered numbers remain source text.

`markdown-pretty-mode` toggles the current view. `markdown-reset-presentation`
removes that override and uses the startup `markdown.pretty` value. Splits inherit
the effective presentation, then keep independent choices. These controls work
in read-only Markdown and add no undo entry or modified state. They refuse when
native Markdown is inactive. Copy and save always use canonical Markdown source,
including hashes, list markers, escapes, BOM and newline identity.

`syntax.enabled=false` suppresses colors while retaining structural presentation
and commands. `NO_COLOR` retains visible symbols, reverse selection and underlined
search cues. Wide and combining characters use the same source coordinates in
pretty/source views, wrapping and horizontal clipping.

## Lists and contextual editing

Use exact names through `M-x`, or `:` in modal Normal. Factory contextual keys
apply in non-modal editing and modal Insert. Explicit parent remaps override
factory Markdown bindings; explicit `keys.markdown` entries override the parent.
Completion popups and modal Normal, Visual and Replace retain their controls.

| Action | Command | Factory key |
| --- | --- | --- |
| Continue or leave an empty item | `markdown-continue-item` | Through Return or Meta-Return |
| Indent complete item subtree | `markdown-indent-item` | `M-Right` in a list |
| Outdent complete item subtree | `markdown-outdent-item` | `M-Left` in a list |
| Contextual next row, item or newline | `markdown-return` | `RET` |
| Insert below a table row, continue item or newline | `markdown-meta-return` | `M-RET` |

Continue inserts a sibling after the complete item, retaining its marker style
and advancing an ordered number. An empty item exits one level. Indent requires
a preceding sibling; outdent retains children and continuation text and places
the item after its parent's remaining subtree. Each accepted action publishes
one undo unit. Point and retained marks in other panes follow surviving source.
Read-only buffers, literal code and invalid structural context refuse edits.

Return dispatches to table navigation first, then list continuation, then an
ordinary newline. Meta-Return inserts a table body row below the current one,
continues a list item, or inserts an ordinary newline. The nine exact table names
and their factory keys are listed in [Markdown tables](markdown-tables.md).

## Startup settings and limits

The complete [configuration sample](config.toml) includes `markdown.enabled=true`,
`markdown.pretty=true`, all twelve `keys.markdown` factory entries and these
independent RGB leaves under `palette.markdown`:

| Leaf | Default |
| --- | --- |
| `heading` | `#7aa2f7` |
| `heading_2` | `#9d7cd8` |
| `heading_3` | `#c0caf5` |
| `heading_4` | `#9bb9f9` |
| `heading_5` | `#b59ce1` |
| `heading_6` | `#bcd0fb` |
| `emphasis` | `#9d7cd8` |
| `link` | `#7aa2f7` |
| `code` | `#9ece6a` |
| `quote` | `#565f89` |
| `list` | `#89ddff` |
| `table` | `#89ddff` |

`markdown.enabled=false` disables native admission; files remain editable as
Text. A lower syntax byte limit wins over the native 8 MiB source limit. Parsing
also admits at most 200,000 events, nesting depth 256 and two seconds of
cooperative worker time. Pending, canceled, stale or over-limit observations
cannot authorize structural edits. Unavailable presentation falls back to source.
Table actions have additional [finite layout limits](markdown-tables.md).

The [command inventory](commands.md), [binding catalog](bindings.md) and
[syntax reference](syntax.md#native-markdown-source) provide the complete editor
catalogs and observation lifecycle.
