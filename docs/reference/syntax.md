# Syntax highlighting

Files whose canonical destination ends in the case-sensitive `.rs` extension use
the bundled Rust grammar and highlight query. No runtime grammar installation,
network access or external language service is required. Unsupported extensions,
unnamed buffers and unavailable syntax use plain `Text` presentation. Display
labels such as `main.rs<2>` do not determine the language.

The status line shows `Rust` only for a prepared syntax tree of the current
buffer revision. A failed or missing query uses `Text`; incomplete Rust source
remains editable and can still have valid captures. Syntax failures cannot
reject a text edit or discard history. Renaming or saving to another extension
updates recognition using the actual associated path. A missing `.rs` destination
can use Rust presentation without creating a disk file.

## Supplied syntax preferences

Library sessions use their admitted snapshot's `syntax.enabled`, `max_bytes` and
per-language `grammar`. Only `rust` and `plain` grammar choices are supported;
associations identify a language independently of its grammar. Disabling syntax
or exceeding a positive UTF-8 byte cutoff drops derived styling without
rejecting edits or changing history. Equality is admitted and zero means no
configured cutoff. Shrinking below the cutoff starts parsing from current text.
A canceled parse is not retried on every redraw of that same revision.
Renames and published write-file destinations re-resolve grammar from the same
snapshot; rejected operations retain their old policy.

## Palette

Document syntax uses the semantic RGB palette, with these built-in Tokyo Night
night colors. The library's registry composition reads overrides from its
admitted configuration snapshot; a standalone `Frame::project` uses defaults
unless supplied a palette with `Frame::with_palette`.

| Captures | Palette role | RGB default |
| --- | --- | --- |
| Keywords | `palette.syntax.keyword` | `#9d7cd8` |
| Strings and escapes | `palette.syntax.string` | `#9ece6a` |
| Comments and documentation comments | `palette.syntax.comment` | `#565f89` |
| Types and constructors | `palette.syntax.type` | `#2ac3de` |
| Functions, methods and macros | `palette.syntax.function` | `#7aa2f7` |
| Constants, booleans and numeric literals | `palette.syntax.number` | `#ff9e64` |
| Grammar-provided punctuation and operators | `palette.syntax.punctuation` | `#89ddff` |

Each present RGB leaf replaces only its own role. Unknown query capture groups
retain the ordinary document foreground. The bundled query determines which tokens
receive captures; highlighting does not invent language coverage from character
matching. Selected cells use the selection text/background colors before
syntax; search matches use their own text/background colors and underlining.
Selections take precedence when the two overlap. Colors reset at row boundaries,
and composed status, tree, separator and prompt spans keep their own roles.
The standard `NO_COLOR` environment preference suppresses RGB sequences while
retaining reverse-video selection and search underlining. RGB fidelity depends
on the terminal's true-color support.

Executable startup applies the admitted user palette, grammar associations and
syntax policy. Missing or rejected configuration uses complete built-ins.
The [terminal input reference](terminal-input.md#palette-and-cursor-restoration)
describes cursor color capability fallback and restoration.

## Source and display coordinates

Queries read borrowed rope chunks and return byte ranges qualified by the
buffer's revision. Presentation maps captured bytes through complete graphemes
to visible terminal cells. Tabs, wide clusters, control rendering and horizontal
clipping retain the ordinary editor coordinate rules; clipped grapheme fragments
remain blank. A styled row contains the same padded text as an unstyled row.

## Incremental revisions

Every admitted mutation prepares exact Tree-sitter input edits alongside its
rope candidate. Ranges use the base revision and publish in descending source
order. Byte offsets and LF-based row/byte columns account for multibyte text,
CRLF, replaced line breaks and each inserted fragment. Typing contributions,
paste, rectangle rows, undo and redo all use the same publication boundary.
Canceled or rejected preparation leaves the existing tree unchanged.

Publication adjusts the retained tree's coordinates and immediately invalidates
its captures. Several accepted edits can accumulate before rendering. A later
refresh reuses the tree only when its coordinates match the current revision;
otherwise it starts from current source. Drawing refreshes at most once per
rendered revision, outside atomic publication, from borrowed rope chunks.
No complete document String is built for parsing.

An incomplete parse can still supply current captures. A canceled or failed
parse discards the tree and parser continuation, keeps the accepted text/history
and uses Text status. That revision is not retried on every redraw. A subsequent
accepted revision can recover from current text and resume incremental reuse.
The library's progress-aware refresh accepts Tree-sitter ParseOptions for a
bounded parser attempt; cancellation cannot roll back an accepted edit.

Queries cover the visible source interval of each document row. Each request
allows at most 4096 in-progress matches and 65536 returned styles; exceeding a
bound discards that request's styles. Drawing and capture generation never
change document bytes, dirty/save state, point, mark or undo history.
