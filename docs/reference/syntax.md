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

## Palette

The fixed palette applies only to document foregrounds. Terminal backgrounds
remain unchanged. Unknown query capture groups retain the default foreground.

| Captures | Foreground | ANSI palette index |
| --- | --- | --- |
| Keywords | Muted magenta | 176 |
| Strings and escapes | Green | 114 |
| Comments and documentation comments | Gray | 244 |
| Types and constructors | Cyan | 81 |
| Functions, methods and macros | Blue | 75 |
| Constants, booleans and numeric literals | Amber | 215 |
| Grammar-provided punctuation and operators | Light gray | 250 |

The bundled query determines which tokens receive captures. Highlighting does
not invent extra language coverage from character matching. Region reverse video
and search underlining remain independent of foreground colors. Tree labels,
status lines, prompts, dividers and buffer inventory keep their normal styling.
Foreground resets at row boundaries prevent color from leaking into those areas.
The standard `NO_COLOR` environment preference suppresses foreground sequences
without changing recognition, text or selection behavior.

## Source and display coordinates

Queries read borrowed rope chunks and return byte ranges qualified by the
buffer's revision. Presentation maps captured bytes through complete graphemes
to visible terminal cells. Tabs, wide clusters, control rendering and horizontal
clipping retain the ordinary editor coordinate rules; clipped grapheme fragments
remain blank. A styled row contains the same padded text as an unstyled row.

The buffer retains parser, tree and query state between redraws. Rendered buffers
refresh at most once per new revision, outside atomic text publication, using
borrowed source chunks.

Queries cover the visible source interval of each document row. Each request
allows at most 4096 in-progress matches and 65536 returned styles; exceeding a
bound discards that request's styles. Drawing and capture generation never
change document bytes, dirty/save state, point, mark or undo history.
