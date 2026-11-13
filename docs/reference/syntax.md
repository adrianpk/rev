# Syntax highlighting

By default, case-sensitive visiting basenames select bundled Rust (`.rs`),
C (`.c`, `.h`), Go (`.go`), HTML (`.html`, `.htm`), CSS (`.css`), JavaScript
(`.js`, `.mjs`, `.cjs`), TypeScript (`.ts`, `.tsx`), JSON (`.json`) and TOML
(`.toml`, including `Cargo.toml`) grammars and queries.
`.tsx` selects the distinct TSX parser while retaining the `typescript` settings
identity. User associations and the selected language table can override recognition.
Canonical reuse retains the first visiting alias; display labels such as
`main.rs<2>` do not determine the language. No runtime grammar installation,
network access or external language service is required. Other identities default
to plain `Text` grammar presentation; [native Org source](#native-org-source)
uses its own coloring. See the [complete language rules](defaults.md#language-rules).

The status line names the selected grammar only for a prepared syntax tree of
the current buffer revision. TypeScript and TSX have distinct mode labels. A failed or missing query uses `Text`; incomplete source
remains editable and can still have valid captures. Syntax failures cannot
reject a text edit or discard history. Renaming or saving to another extension
updates recognition using the actual associated path. A missing `.rs` destination
can use Rust presentation without creating a disk file.

## Supplied syntax preferences

Library sessions use their admitted snapshot's `syntax.enabled`, `max_bytes` and
per-language `grammar`. Bundled `rust`, `c`, `go`, `html`, `css`, `javascript`,
`typescript`, `tsx`, `json`, `toml` and fallback `plain` choices are supported;
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
| Types, constructors, tags, attributes and properties | `palette.syntax.type` | `#2ac3de` |
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
Tree-sitter parsing builds no complete document String. Native Markdown uses
the bounded worker snapshot described below.

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

## Native Org source

Admitted `.org` files and `M-x org-mode` use the native Org source recognizers,
independent of Rust grammars and external language services. `text-mode` drops
Org styles. `syntax.enabled` and the UTF-8 byte cutoff also apply to Org;
`org.enabled` controls mode availability. The status remains `Org` while source
observation is pending, incomplete or canceled, since mode and derived styling
have separate ownership.

`palette.org` supplies independent colors for headings, keywords, drawer
boundaries, properties, planning lines, block boundaries, literal code, comments,
list markers, table rows, formula directives, emphasis, links, targets, footnotes,
entities and malformed structural delimiters. The [complete sample](config.toml)
and [Org palette catalog](org/configuration.md#palette-roles) list every active
Org leaf and RGB default.
Nested inline captures take precedence over their enclosing heading/table style.
Code/verbatim inline objects and source, example, export and comment blocks keep
literal context. Their text never becomes an outer heading or inline object.
Coloring does not evaluate formulas, expand entities or hide source delimiters.

One temporary session-owned worker observes visible Org buffers from shared rope
snapshots. Split views reuse their buffer's result. Editing remains available
while work is pending; old styles disappear immediately on revision change.
Results must match the canonical buffer/revision before publication. C-g cancels
owned work, and hiding its buffer or disabling its mode/policy retires it. Native
input polls at 10ms only while a job is owned; idle sessions have no Org deadline.
The canceled revision is not retried on each redraw; a new edit can recover.

The default observation profile admits 8 MiB, 200,000 source nodes/captures and
256 structural levels. It reports incomplete coverage when a bound stops work;
unsupported or unobserved source stays readable. Index suffixes retain their
unaffected prefix after edits. Structural/inline ownership is re-observed from
the full admitted snapshot so delimiter edits cannot inherit obsolete context.
This does not promise suffix-only semantic work. Native cancellation is checked
between bounded parser steps and around finite capture sorting; no permanent
parser thread or file scan runs while idle. Library users can explicitly prepare
and install a `ColorScan` result before standalone `Frame::project`.

Colors use the same grapheme/tab/wide-cell projection, selection/search precedence
and NO_COLOR fallback as Rust. NO_COLOR keeps original source and delimiters,
selection reverse video and search underlining without RGB output. Task-state,
tag and timestamp text retain their enclosing heading and planning roles.

## Native Markdown source

Files ending in `.md` or `.markdown` select native Markdown through the existing
association resolver. User associations take precedence. `markdown-mode` also
selects it explicitly; `text-mode` suspends it without changing text or history.
`markdown.enabled = false` disables native entry and automatic activation.
The status stays `Markdown` while analysis is pending, canceled or over a bound.
Native Markdown works offline; Marksman and other language servers remain optional.

CommonMark context and the GFM table extension provide source-backed recognition.
Emphasis and strong emphasis, links and images, inline/fenced/indented code,
quotes, lists and tables use the six `palette.markdown` colors in the
[configuration catalog](defaults.md#markdown-semantic-colors). Nested inline
colors override enclosing block colors. Literal code never recognizes nested
Markdown markup. Headings retain their source prefixes and ordinary text color.
There is no source decoration or structural editing command in this increment.

A temporary session-owned worker takes one bounded contiguous source snapshot
per admitted revision. The profile admits at most 8 MiB, 200,000 parser events
and retained nodes, 256 structural levels and two seconds of cooperative work.
A lower nonzero `syntax.max_bytes` wins. Cancellation is checked during snapshot
copying, before/after parser construction and between events; parser-internal
steps remain bounded by the admitted source. A limit or cancellation discards
the entire observation. That revision is not retried on every draw.

Results must match the current buffer, revision and mode generation. Edits,
undo and reload invalidate old colors immediately; leaving the mode or hiding
its buffer retires pending work. Split views reuse one observation. C-g cancels
owned work. There is no permanent parser thread and no Markdown polling deadline
when idle. `syntax.enabled = false` disables colors while retaining independent
structural recognition and the Markdown mode.

Source bytes, BOM/newlines, read-only state, points, marks, dirty state and undo
history remain canonical. Shared grapheme projection, selection/search
precedence and NO_COLOR behavior apply. The private parser adapter uses the
approved default-off dependencies and retains their exact upstream
[MIT notices](../../THIRD-PARTY-NOTICES.md).

## Library fold projection

Native Org `View::set_folds` accepts up to 4096 revision-qualified source ranges.
Nonempty ranges hide complete logical lines; an unterminated final line may end
at EOF. Empty ranges are ignored. Overlapping/adjacent ranges form one hidden
interval. Partial lines, foreign/stale source, inactive Org mode and hiding the
view's current point are rejected before replacing existing visibility. This
API changes neither canonical source nor history and works on read-only buffers.

Each view retains its own map. Splits copy the choice through shared immutable
offsets; revealing or clearing one view leaves its peers unchanged. `View.top`
remains a canonical logical row, `source_row` resolves a local displayed row and
`screen_row` returns no position for hidden or above-viewport source. Cursor,
gutter labels, syntax, selection, search and mouse hits use those same rows.
Hidden text stays part of canonical regions and copied/saved source. Complete
Unicode glyphs, tabs, horizontal clipping and blank EOF cells retain their
ordinary coordinate rules.

Vertical/page/wheel/autoscroll input advances visible rows. A targeted source
navigation or search reveals its containing hidden interval before displaying
the point. The [native Org visibility controls](org-mode.md) retain hierarchy
choices and reveal only the target entry and ancestor path. Canceled search
restores the original view choice. Canonical text
publication, undo and redo release obsolete fold offsets in all affected views.
Standalone stale maps also fall back to visible source; stale view points remain
invalid until the caller supplies a current canonical point. Tiny geometry
preserves source scroll and does not create a row alias or idle deadline.

## External grammar assets

Register a compatible precompiled native library and both UTF-8 queries in the
startup TOML. An external registration can replace a bundled source or admit an
unbundled language. Existing file associations choose the language identity;
for example, `.py` already selects `python`. The four leaves are required together:

```toml
[language.python.external]
parser = "python-0.25.0/parser.so"
symbol = "tree_sitter_python"
highlights = "python-0.25.0/highlights.scm"
completions = "python-0.25.0/completions.scm"
```

Relative paths resolve under the resolved XDG data root's `rev/grammars`
directory; absolute paths are also accepted. Relative parent escapes and symlinks
outside that root fail admission. Paths must name regular local files. The
symbol must begin `tree_sitter_` with a nonempty ASCII letter/digit/underscore
suffix and fit 128 bytes. Register at most 32 languages; each nonempty query
must fit 1 MiB. Registrations are frozen at startup; no asset directory is scanned.

First use queues admission on one temporary worker. Library IO/loading and
query compilation run outside terminal input and drawing. The parser ABI and
both queries must validate before that generation becomes visible. While pending,
and after a failed admission, the bundled source remains available if present;
an unbundled language stays editable as `Text`. Successful external syntax uses
`External` status (`Org Src External` in a declared-language source draft).
A completed failure is cached, even if its files are subsequently corrected.
There is no asset polling timer after admission finishes.

Native libraries are user-trusted code, not a sandbox. Admission cannot contain
arbitrary native-code faults. The upstream safe loader retains a successfully
loaded mapping for the process lifetime, including when later query validation
fails. Rev never unloads or replaces it in process. Install assets into a new
versioned directory, update the registration, and restart to activate that version.
Never overwrite a mapped library in place. Rev does not compile, download or
install grammars; supply compiled assets before starting it.

### External query profile

Highlight capture families are `keyword`, `string`, `escape`, `comment`, `type`,
`constructor`, `tag`, `attribute`, `property`, `function`, `constant`, `number`,
`punctuation` and `operator`. Dot-qualified names use their first component.
Unrecognized captures reject the complete registration.

Completion captures are exactly `function`, `type`, `module`, `constant`,
`binding`, `parameter`, `field`, `scope`, `comment` and `string`. At least one
of the seven declaration kinds is required. Declaration captures name actual
source nodes, not identifier occurrences or generated text. Their text becomes
the literal candidate; the existing label, symbol, snapshot and work limits apply.
The nearest explicitly captured ancestor `scope` bounds a declaration's visibility.
Capture a body/block when the function name itself should remain visible outside
that function. Unlike bundled adapters, the external profile does not infer
language-specific scope node names, structural keys or HTML/CSS catalogs.

The optional `comment` and `string` captures identify literal regions for the
existing automatic completion policy; ordinary Tree-sitter comment/string node
names are also recognized. Manual requests retain their existing explicit
invocation policy. For example, a Python completion query can use:

```scheme
(function_definition name: (identifier) @function)
(assignment left: (identifier) @binding)
(block) @scope
(comment) @comment
(string) @string
```

Tree-sitter's built-in text predicates are supported. Unknown predicates,
property tests/settings, invalid node names, unknown captures, malformed queries
and incompatible parser ABIs reject the complete registration. These checks
preserve the same bundled/plain fallback without changing canonical text/history.
