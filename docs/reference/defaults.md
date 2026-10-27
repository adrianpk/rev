# Scope and defaults

## Runtime and invocation

The executable is `rev`. It runs in a terminal and accepts zero arguments or
one ordinary file path. With no path it opens an unnamed `*scratch*` buffer.
A nonexistent path opens an empty visiting buffer without creating disk entries.
The initial tree root is the file's parent, or the working directory without a
file argument. The tree starts hidden.

Linux terminals are the primary target. Ordinary execution is offline and uses
compiled-in defaults and one fixed syntax palette. [Terminal input](terminal-input.md)
states the keyboard reports and Escape behavior.

## Text and files

| Area | Default |
| --- | --- |
| Encoding | Strict UTF-8. Invalid UTF-8 and NUL-containing binary input reject before installing a buffer. |
| BOM and separators | Preserve one recognized leading UTF-8 BOM, existing LF/CRLF and final-newline presence. Mixed separators remain mixed. |
| Newline insertion | Follow the buffer's prevailing LF/CRLF style; new text defaults to LF. Literal paste retains supplied separators. |
| Movement | Whole graphemes; buffer positions, UTF-8 bytes and display cells remain separate. |
| Display | No soft wrap. Horizontal scrolling reveals point. Tabs use four-cell stops; control text uses visible notation. |
| Status | Buffer label, modified/read-only state, one-based logical line/display column and language mode. |
| Visits | Canonical paths reuse one live buffer, including symlink aliases. Display names stay distinct. |
| Save | Stage in the destination directory, flush/synchronize, recheck the observed destination and replace it, then synchronize/observe completion. |
| Permissions | Copy an existing destination's access bits; new files use owner read/write access (`0600`). |
| Conflicts | Occupied write-file destinations and detected external changes require explicit fresh consent. A symlink save writes its resolved target. |
| Missing paths | Saving creates a file only in an existing parent. Removed visited text remains live; explicit saving can recreate the file. |
| Publication errors | Distinguish no publication from a published file whose completion remains unresolved. Only completed saves satisfy required close/exit saving. |

Each editing window retains its own point and viewport and references shared
buffer text. Closing a window retains buffers; closing a modified buffer asks
for save/discard/cancel. Closing the final buffer creates a fresh empty buffer.

## History and retained text

Undo/redo belongs to each buffer. Adjacent ordinary typing groups until another
command or save; paste, accepted replacement, region kill and rectangle edits
form coherent units. A new edit after undo discards redo. Saving retains
history and records the current logical clean state; undo/redo can return to it.
History retains up to 512 groups with a soft 64-MiB fragment budget, always
retaining the newest complete group. Pruning must not make different text clean.

The linear ring keeps 32 entries. Consecutive directional kills append/prepend
in document order; intervening commands break coalescing. A yank-pop applies
only to the current yank sequence and its owning view. Resize alone preserves
that sequence. Rectangle rows use a separate store. Copying short source lines
adds no absent trailing text; insertion adds only required prefixes and rows.

Search is literal and does not wrap implicitly. A query without uppercase
characters ignores Unicode case; uppercase queries match case-sensitively.
Search cancellation restores its starting view. Accepted query-replace
occurrences are independent undo steps. See [Search and replacement](search.md).

## Interaction bounds

Prompt input is bounded at 4096 Unicode scalars, with whole-payload rejection
instead of truncation. Command history retains up to 128 distinct accepted
names in session memory. File/buffer arguments and decisions own input until
accepted or canceled. Return and pasted text cannot grant save/delete consent.

A separately reported Escape has a 100-ms Meta deadline; `C-g` cancels the
active interaction. Tiny or zero geometry retains text, views, pending input
and history while suspending/clipping presentation. Normal editing panes need
three text columns and a text/status row pair; nested splits use their combined
minimum. The tree hides below 32 terminal columns and uses at most 24 columns
or one third of the frame when shown.

## Product boundary

The built-in language support is Rust syntax highlighting, with editable plain
text for other files or syntax failures. This baseline has no user configuration
language, plugins, macros, package runtime, LSP, language completion, automatic
indentation, OS clipboard bridge, regex/project search, soft wrap or session
persistence. The command catalog describes the implemented editing subset.

Use [Bindings](bindings.md), [Focused prompts](prompts.md),
[Failure recovery](recovery.md) and [File tree](file-tree.md) for exact behavior.
