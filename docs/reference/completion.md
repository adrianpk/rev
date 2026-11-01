# Native completion

Learn one insertion in [Complete Names](../tutorials/user-guide/10-complete-names.md),
or use [Use completion](../how-to/use-completion.md) for invocation and settings.
Completion shares one session, filter and popup across native syntax and optional
language servers. Native syntax supplies observed declarations and finite
structural vocabulary. With LSP enabled and an eligible configured server,
completion requests semantic proposals for the current unsaved revision. A valid
empty response, no admitted items, unsupported capability, timeout or unavailable
server selects native fallback when native syntax or Org context is available.

Native completion opens automatically after an admitted identifier edit of at
least two Unicode characters or a language separator. Rust/C/Go and script
languages admit member separators; C also admits `->`. HTML admits tag delimiters
and attribute spacing, and CSS admits selector/declaration delimiters. JSON key
quotes/object separators and TOML key quotes/table delimiters also trigger in
admitted structural contexts. The default debounce waits
150 ms after the newest eligible native edit. `M-TAB` or
`M-x completion-at-point` requests immediately without a minimum prefix.
Terminal or desktop shortcuts may intercept M-TAB; the named command uses the
same handler. Candidates come from declarations in the current unsaved buffer
and bounded same-language open buffers, using their retained syntax owners. Native extraction scans no unopened files.

The compact, single-border list appears below or above the source cursor within
its pane. The first candidate is selected. Up/Down or C-p/C-n changes selection;
Tab or Return inserts it. Escape or C-g cancels. These keys keep ordinary editing
and Org behavior outside an active list. Unrelated movement, prompts, searches,
pane and buffer changes dismiss the list before normal dispatch. Identifier
edits and deletion schedule a fresh current context. Cancellation does not reopen
the list until another eligible text edit or explicit request. Bracketed paste
dismisses suggestions and does not schedule per-character work.

Resize retains a current selection and recomputes placement; a pane too small
for the border and a row hides the overlay. Source focus and document layout
stay in place. Selected rows use selection colors; borders use the active border
role. NO_COLOR keeps reverse video. `keys.completion` admits popup navigation,
acceptance and cancellation commands, including finite chord sequences. C-g
keeps its globally reserved `keyboard-quit` configuration value, interpreted as
popup cancellation; Escape always cancels literally in the active list.

Candidates are observed names and finite HTML/CSS vocabulary. Each row has a
compact type indicator, the name and a right-aligned type label when space
permits. Functions use `ƒ`, fields/properties `◆`, types `T` and variables `v`.
Indicators and type labels use the existing syntax palette roles, including on
the selected row. They require no Nerd Font. Narrow panes omit the type label
before reducing the visible name. LSP item kinds use the same presentation;
missing or unknown kinds use a neutral indicator without inventing a type.
Indicators and labels never become insertion text. Lowercase input matches without case
sensitivity; uppercase input requires matching case. Prefix matches precede
ordered subsequences, followed by current-buffer locality, declaration proximity,
source ordering and stable lexical ties.
Equal insertions are deduplicated. A single candidate still requires acceptance.
No usable candidates produce no popup. Automatic requests stay quiet in admitted
comments, strings and character literals by default; manual invocation bypasses
that suppression while retaining the same declaration coverage. Structural
HTML/CSS admission still excludes ordinary text and value strings. JSON/TOML
admission stays limited to keys and table headers even with literal suppression
disabled or a manual request. Quoted structural keys work with the default
`completion.in_strings = false`.

Acceptance replaces the full identifier around point with literal text through
the ordinary edit owner, as one undo operation. Read-only buffers and obsolete
view, point, revision, syntax or invocation identities cannot publish that edit.
Selection, cancellation and failed admission create no history entry. Native
completion does not expand snippets, execute commands or save a file.

Rust observes functions/methods, types/traits, modules, fields, constants and
local/parameter declarations. C observes functions, typedef/type/enum names,
fields, enumerators and local/parameter declarations. Go observes functions/
methods, types, fields, constants and local/parameter declarations. Observed
names do not imply imports, macro expansion, type checking or member resolution.
Local declarations and parameters stay within their lexical block/function;
locals from other functions or other buffers do not leak into the list. Bindings
after point are omitted. Fields remain observed names without receiver inference.

JavaScript and TypeScript observe functions, classes/types, import bindings and
aliases, members and local/parameter bindings. TSX uses a distinct parser with
JSX highlighting while retaining the `typescript` preference identity. TS and
TSX buffers reuse same-language names. Ordinary template text stays quiet by
default; `${...}` expressions retain code admission. Dollar/private script
identifiers and hyphenated HTML/CSS names use complete checked replacement ranges.

HTML observes tag and attribute names and adds finite common standard vocabulary
in those contexts, including an incomplete opening/closing delimiter. It stays
quiet in document text, attribute values, comments and raw script/style bodies.
CSS observes selector names and property declarations, adds finite standard
property names in declaration-name context, and offers observed custom properties
inside `var(...)`. Ordinary values and strings have no invented value vocabulary.
The compiled common subsets contain 72 tags, 48 attributes and 84 CSS properties,
based on the [HTML index](https://html.spec.whatwg.org/multipage/indices.html) and
[CSS property index](https://www.w3.org/TR/css-2024/#property-index).
These subsets do not model element-specific validity, DOM types or injected
languages. Missing or unsupported syntax degrades assistance while preserving editing.

## Preferences and work bounds

The complete [startup sample](config.toml) and [configuration catalog](defaults.md)
list all eight `completion` leaves and their finite ranges. Defaults are
`enabled = true`, `automatic = true`, `delay_ms = 150`, `min_prefix = 2`,
`in_comments = false`, `in_strings = false`, `max_candidates = 200` and
`popup_rows = 10`. `completion-mode` toggles completion for this session;
`completion-auto-mode` toggles automatic requests while retaining manual access.
Neither command writes settings or changes the startup-only loading contract.

JSON observes object keys, including nested objects. TOML observes pair-key
components (including inline/dotted keys) and complete table/array-table header
paths. Only current and admitted same-language open buffers contribute names;
no schema keys or value vocabulary are supplied. Escaped spellings decode to one
key identity for matching and deduplication. Acceptance replaces the complete key
literal with valid quoting/escapes; TOML uses bare keys when permitted. A table
completion replaces its full header path while preserving brackets. Dot-separated
paths keep quoted segments distinct from literal dots inside a key. Bounded
incomplete-key recovery uses the retained syntax tree and current token, so valid
independent observations remain usable around malformed source. Ordinary values,
array elements and comments do not become key contexts.

Native Org completion offers effective TODO states at the first heading word,
observed tags in a heading tag field, and observed property keys inside a native
property drawer. The file's valid TODO declaration takes precedence over
`org.todo.sequence`/`finished`. File/heading tags and TAGS declarations contribute
tag names; properties come from valid observed metadata. Incomplete property-key
slots preserve their native drawer header/placement fact without declaring new
names from malformed text.

After `#+`, the finite native directive list covers metadata, tasks, tags,
properties, source/example/export/comment blocks, names/captions, tables and
supported export attributes. After `[[`, admitted file/ID/attachment and web/mail/
news link types are offered. After `[[*`, current-document heading titles are
shown without TODO/priority/tag metadata; insertion encodes literal target bytes.
These proposals do not visit another document. Missing tag/property/directive/
link colons are supplied once when required; existing delimiters remain in place.
Heading targets preserve the surrounding link brackets.

Org structural punctuation and initial heading spacing can bypass the ordinary
prefix minimum. Ordinary prose, property/directive values, inline code/verbatim,
comments and literal block bodies stay quiet, including incomplete literal blocks.
The incomplete block-opening directive itself remains a directive position.
Automatic and manual requests share these context checks. Tab keeps its native
Org visibility action outside an active popup. A source-edit draft uses its
declared admitted language, so completion and draft undo precede the existing
checked one-transaction return to the Org source. Org uses the existing native
structure/task/tag/property/link recognizers and the same completion worker,
cancellation, source pins and work bounds. It requires no Org grammar.

Extraction admits at most 2 MiB and 4096 symbols per buffer, 32 observed buffers,
8 MiB total shared source snapshots, 16384 retained symbols, 8 MiB observation
payload and 512-byte labels. Current context precedes other sources; older
other-buffer observations are omitted first under pressure. One temporary worker
borrows each admitted buffer's parser and a shared rope/tree snapshot,
and performs filtering/ranking before returning disposable proposals. While the
parser is borrowed, the display keeps a read-only tree/query projection for
that exact revision. Debounce does not suppress ordinary syntax refresh. An
edit during borrowed work prepares current syntax independently; a retired
parser cannot overwrite it or paint old captures on new text. At most one worker and one newest pending request exist;
cancellation retains the slot until the old worker is reaped. Parse/query
callbacks check cancellation and a 50 ms work deadline. Limited results are
omitted without editing source. Source revisions, language/grammar generations, rename paths and open-buffer
availability are rechecked on arrival and acceptance, independently of worker
completion. No unopened file is scanned. A manual limit notice distinguishes
omitted observations from a complete no-match result.

Drawing observes retained candidates and performs no completion extraction.
The native loop waits for the debounce deadline, polls only while a worker is
owned, and has no completion timer while idle. Shutdown cancels and reaps owned
work. See [Syntax highlighting](syntax.md) for grammar admission/fallback and
[Bindings](bindings.md) for the complete command catalog.

Explicit [external grammar assets](syntax.md#external-grammar-assets) use the same
checked completion worker and candidate limits. Their finite declaration/scope
query profile supports languages outside the bundled catalog. Admission publishes
an immutable grammar generation; proposals from the previous generation cannot
be accepted. Failed admission retains bundled completion when available.

## Optional semantic source

LSP is disabled by default. Set `[lsp] enabled = true` in the existing TOML
configuration, or run `M-x lsp-mode` for this session. The Rust entry uses an
installed `rust-analyzer`; server entries declare their executable, argv,
languages and root markers under `[lsp.servers.<name>]`. No server is downloaded.
`lsp-status` observes the connection. `lsp-disconnect` excludes the current
canonical buffer across all its views; `lsp-connect` restores eligibility without
enabling a globally disabled client. `lsp-restart` replaces the current eligible
root after failure, preserving exclusions and other roots. Healthy servers stay
available after last-buffer close by default. Native completion remains available.
See [Language servers](language-servers.md) for lifecycle and protocol limits.

Semantic results use the same prefix/subsequence classes, using `filterText` or
label for matching and server sort keys/order within each class. Plain text,
TextEdit and InsertReplaceEdit primary replacements and supported list defaults
are admitted. The replacement extent is checked against point and prefix in the
negotiated UTF-8/UTF-16 units. Advertised trigger characters can request before
the minimum prefix; an incomplete list is requested again for changed input.
Selection alone never changes text. Acceptance uses ordinary canonical undo.

Acceptance resolves the selected item once when the server advertises resolve.
Only `additionalTextEdits` is advertised as a mutable resolve property; the primary
proposal remains fixed. List-default data and primary ranges accompany the
selected item. Resolution waits asynchronously; ordinary edits, movement and
completion cancellation retire it. A failed or unsupported resolve offers current
native fallback without applying a primary-only fragment.

Additional edits from the list or resolve must belong to the current document,
use valid negotiated positions and stay disjoint from the primary and each other.
At most 32 additional edits and 256 KiB total inserted text are admitted per item,
including the primary's existing 64 KiB limit. The whole batch publishes as one
ordinary undo/redo transaction, with the cursor following the primary insertion.
Read-only or stale state refuses the entire batch. Explicit snippets, commands,
cross-document edits and indentation adjustment remain unsupported. Connection, activation, revision,
view and invocation changes invalidate the old result. Cancellation and request
timeouts retire pending work; late responses cannot reopen the popup. The
configured `lsp.request_timeout_ms` defaults to 2000 and
`lsp.max_pending_requests` to 16 per connection, with one pending completion per
buffer. Semantic availability does not require a bundled grammar. The session's
source observation distinguishes semantic results and native fallback reasons;
transport readiness alone does not claim semantic candidates.
