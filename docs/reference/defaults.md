# Scope and defaults

## Runtime and invocation

The executable is `rev`. It runs in a terminal and accepts zero arguments or
one ordinary file path. With no path it opens an unnamed `*scratch*` buffer.
A nonexistent path opens an empty visiting buffer without creating disk entries.
The initial tree root is the file's parent, or the working directory without a
file argument. The tree starts hidden.

Linux terminals are the primary target. Ordinary execution is offline and uses
compiled-in defaults unless startup configuration supplies supported overrides. [Terminal input](terminal-input.md)
states the keyboard reports and Escape behavior.

Editing views show [absolute logical line numbers](gutter.md) with current-line
emphasis and a stable four-digit minimum field. Numbering does not change text
columns, copied data or saved bytes. Narrow views preserve their text area.

Native mouse support starts enabled: primary clicks focus/place point, the wheel
scrolls the pointed pane or tree without changing keyboard focus, and a primary
drag selects canonical text with bounded source-edge autoscroll. Use
`M-x mouse-mode` to toggle only this session. The startup step is three logical
rows; [terminal input](terminal-input.md#pointer-reporting-ownership) documents
supported reports, cleanup and terminal-owned selection bypass.

## Startup configuration

The [complete English reference sample](config.toml) contains every supported
global default, all 54 active RGB roles and all eight default context maps. Its
optional include, language and association examples are commented out. First
startup installs these complete defaults; a customized file may retain only the
overrides you need.

The executable reads `$XDG_CONFIG_HOME/rev/config.toml` once before creating
buffers or input owners. An unset, empty or relative `XDG_CONFIG_HOME` uses
`$HOME/.config/rev/config.toml`; HOME must be absolute for that fallback.
Data, state and cache locations resolve independently through their XDG variables
with HOME defaults `.local/share/rev`, `.local/state/rev` and `.cache/rev`.
When no entry exists at the configuration path, normal editor startup creates
only its missing parents and the complete packaged sample, then validates it
through the ordinary loader. It does not need installed reference files or a
checkout. Data, state and cache directories remain absent.

Publication exposes a complete synchronized file without replacing a concurrent
winner. Every existing entry is preserved, including empty/invalid files,
directories and dangling symbolic links. Pure location, parsing, loading and
preparation helpers remain read-only. A provisioning failure shows its operation
and source in `Configuration: ...` and retains complete usable built-ins.

A present unreadable or invalid source shows `Configuration: ...` in the echo area and uses complete
built-ins, including palette, formats and bindings. Diagnostics identify the
source and key, with a line/column when available. Fix the source and start a new
session to apply it. No settings from a rejected candidate become active.

For example, this TOML overrides indentation, literal-tab display and one editor
binding; all omitted settings and independent aliases retain their defaults:

```toml
[editor]
indent_style = "spaces"
indent_width = 2
tab_width = 8

[keys.editor]
"M-j" = "next-line"
```

Sources use TOML 1.1, schema version 1 (also when omitted). Optional top-level
`include = ["editing.toml", "colors.toml"]` loads paths relative to their declaring
source in order, before the parent's leaves. Only explicit includes participate;
project-local files and variable-looking paths receive no implicit discovery or
shell expansion. Every source and the merged candidate validate before use.

The session retains one immutable snapshot, including policies for buffers opened
later. Editing or deleting a source does not change the running session; restart
to read it again. Existing files retain their observed BOM/newlines, while new
buffers use configured format defaults. [Bindings](bindings.md#configuration-keymap-contracts),
[syntax](syntax.md#palette), [prompts](prompts.md#supplied-prompt-preferences) and
[tree preferences](file-tree.md) describe the existing configurable owners.

## Source precedence and limits

Settings resolve in this order:

1. Compiled built-in defaults.
2. Explicit includes, depth-first in declaration order. Each child's includes
   apply before that child. A later sibling source overrides an earlier sibling.
3. The declaring source's own leaves, with the selected user root last.
4. The selected language's editing leaves, after all global layers.

Only explicitly present leaves override earlier values. False, zero and empty
arrays are values; an omitted leaf or empty concern table inherits siblings.
Arrays replace rather than append. Maps merge by key, and keymap layers replace
an earlier spelling by normalized sequence identity. Includes are source-local
traversal metadata, not a merged list used for another discovery pass.

Relative include paths resolve against the declaring source's selected path;
absolute local paths are also accepted. Tilde, variables and shell expressions
are not expanded. A source can be included more than once outside the active
stack; each occurrence counts toward the limits. Symlink/hard-link aliases on
the active stack are cycles and reject. Limits are fixed, not configurable:

| Limit | Bound |
| --- | --- |
| Include depth | 16 edges; the root is depth zero. |
| Source occurrences | 64, including the root and repeated acyclic includes. |
| One source | 256 KiB of UTF-8 TOML bytes. |
| Complete graph | 2 MiB across source occurrences. |

Sources must be regular files. Explicit read-only loading of an absent optional
root is silent; native startup first attempts provisioning as described above. A missing
explicit include, directory, unreadable file, invalid UTF-8, malformed/duplicate
TOML key, unknown option, wrong type, invalid bound/color/enum or conflicting
keymap rejects the complete candidate. A bad included value cannot be hidden by
a later valid override. The editor remains usable with complete built-ins and
one source/key diagnostic. No settings source is executed or repaired; creation
is confined to the absent implicit root during normal startup.

See [Native Org controls](org-controls.md) for capability, command, context,
preference and palette coverage.

## Configuration value catalog

The tables below and the [Org configuration catalog](org/configuration.md)
together describe every accepted schema leaf or map. All settings are
read once at startup. Global values apply throughout that session; the four
`language.<id>` leaves are the only language overrides. Integers and finite
floats have distinct types; use `0.5`, not `0`, for a float share. Widths describe
terminal display columns/cells, never bytes or Unicode scalar positions.

### Source metadata and maps

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `version` | Integer, exactly 1 | `1` | Per source; omitted means schema 1. |
| `include` | Array of nonempty local path strings | `[]` | Per source; explicit ordered include graph, with fixed limits above. Empty paths, NUL and URL forms reject. |
| `language` | Map of language IDs to partial tables | `{}` | IDs start with `[a-z]`, then contain only `[a-z0-9-]`; no grammar installation is implied. |
| `files.associations` | Map of exact basenames or `*.suffix` to language IDs | `{}` | User rules override built-in recognition; see language rules below. |
| `keys.editor` | Map of chord sequences to existing editor command names | Compiled map | Editing focus; all 71 entries are active in the sample. |
| `keys.minibuffer` | Map of chord sequences to admitted prompt actions/commands | Compiled map | Prompt focus; all 22 entries are active in the sample. |
| `keys.tree` | Map of chord sequences to admitted tree commands | Compiled map | Tree focus; all 21 entries are active in the sample. |

Only the fixed implemented [commands and context actions](bindings.md) are
accepted as keymap values. No value unbinds a key or supplies code or consent.
Sequences contain 1..=8 normalized chords. The conservative terminal aliases,
reserved C-g, prefix conflicts and stage admission rules apply to every layer.
The sample declares the complete default targets explicitly; omit its key tables
to inherit native stage-specific distinctions, or copy only desired overrides.

### Line numbers

Startup values initialize each editing view. Runtime [gutter commands](gutter.md)
change only that view's overrides and never reload or write this snapshot.

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `gutter.visible` | Boolean | `true` | Initial number visibility, subject to protected text geometry. |
| `gutter.mode` | String: "absolute" or "relative" | `"absolute"` | Logical line calculation from each view's own point. |
| `gutter.current_line` | String: "absolute", "zero" or "hidden" | `"absolute"` | Current-row label, independent of calculation mode. |
| `gutter.side` | String: "left" or "right" | `"left"` | Number field placement inside its editing pane. |
| `gutter.minimum_digits` | Integer 1..=20 | `4` | Minimum reserved digits; full source line count may require more. |
| `gutter.left_padding` | Integer 0..=4 | `0` | Cells before digits, excluding the separator. |
| `gutter.right_padding` | Integer 0..=4 | `1` | Cells after digits, excluding the separator. |

### Mouse controls

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `mouse.enabled` | Boolean | `true` | Initial native pointer reporting; M-x mouse-mode changes only the session override. |
| `mouse.scroll_lines` | Integer 1..=100 | `3` | Logical rows per wheel report; each drag-edge tick also clamps to the current text-area height. |

### Editing, presentation and retention

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `editor.soft_wrap` | Boolean | `true` | Word-aware wrapping at each pane's available text width. M-x visual-line-mode toggles one live buffer for all its views; false retains horizontal scrolling. No source or history changes. |
| `editor.indent_style` | String: "spaces" or "tabs" | `"spaces"` | Buffer policy; Tab emits the selected indentation form. Language-overridable. |
| `editor.indent_width` | Integer 1..=32 | `4` | Buffer policy; indentation-stop display columns, independently of tab display width. Language-overridable. |
| `editor.tab_width` | Integer 1..=32 | `4` | Buffer policy; literal-tab display cells. Shared views use the same width. Language-overridable. |
| `scroll.page_overlap` | Integer 0..=1000 | `2` | Session; overlap in page rows, clamped so a usable pane advances at least one row. |
| `windows.split_ratio` | Finite float 0.1..=0.9 | `0.5` | Session; preferred first-child share after the separator, clamped by both recursive subtree minima. |
| `minibuffer.candidate_limit` | Integer 1..=32 | `8` | Session; maximum visible command/path candidate rows, including any list diagnostic, clipped to geometry. |
| `minibuffer.match` | String: "prefix-then-substring", "prefix" or "substring" | `"prefix-then-substring"` | Session; M-x matching only. File candidates retain literal component-prefix matching. |
| `minibuffer.history_limit` | Integer 1..=4096 | `128` | Session; distinct accepted command names in memory, newest first. No persisted history. |
| `minibuffer.input_limit` | Integer 1..=65536 | `4096` | Session; Unicode scalars per prompt, including paste, completion and history recall. Excess rejects wholly. |
| `history.group_limit` | Integer 1..=65536 | `512` | Per buffer; maximum retained coherent undo/redo groups. Pruning never splits an operation. |
| `history.payload_target_bytes` | Integer 1..=1073741824 | `67108864` | Per buffer; soft logical fragment-payload target in bytes, not resident memory. The newest complete group survives excess. |
| `kill_ring.capacity` | Integer 1..=4096 | `32` | Session; linear entries, separate from retained rectangle rows. |
| `kill_ring.coalesce` | Boolean | `true` | Session; consecutive compatible directional kills join; unrelated commands still end the chain. |
| `tree.visible` | Boolean | `false` | Initial show preference; showing never steals editor focus. Narrow geometry can temporarily hide the pane. |
| `tree.width` | Integer 1..=512 | `24` | Session; preferred tree columns, constrained by its frame share and complete editor geometry. |
| `tree.max_fraction` | Finite float 0.1..=0.5 | `0.3333333333333333` | Session; maximum terminal-width fraction for the tree. The divider is separate. |
| `tree.root` | String: "auto", "cwd" or "file-parent" | `"auto"` | Initial root: auto/file-parent use the startup file's parent, falling back to cwd without a file; cwd keeps the working-directory root. |
| `tree.show_hidden` | Boolean | `true` | Session; admit dot-prefixed tree entries. This does not filter find-file candidates. |
| `tree.directories_first` | Boolean | `true` | Session; tree directories sort before leaves, then by filename bytes. This does not change find-file ordering. |
| `syntax.enabled` | Boolean | `true` | Session; enable derived syntax styling where a bundled grammar is selected. Text remains editable on fallback. |
| `syntax.max_bytes` | Integer 0..=1073741824 | `0` | Session; UTF-8 byte cutoff. Zero means no configured cutoff; equality is admitted and a larger buffer uses plain presentation. |
| `files.newline` | String: "lf" or "crlf" | `"lf"` | Creation default for new buffers only; existing separators and final-newline state are preserved. |
| `files.bom` | Boolean | `false` | Creation default for one leading UTF-8 BOM; existing files retain their observed BOM state. |

Indentation, tab display, syntax and retained history use each buffer's policy
across shared windows. Startup source changes do not update existing or later
buffers in that session. A successful rename or published write-file reassociation
re-evaluates language preferences from the same snapshot; a failed or canceled
path change retains the old policy. Reassociation never converts text or resets
history, revision or saved-state identity. [Rectangles](rectangles.md) retain copy
versus kill tab semantics across source/destination policies.

### Language rules

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `language.<id>.indent_style` | String: "spaces" or "tabs" | Inherit global | Selected language's buffer policy; replace only this editing leaf. |
| `language.<id>.indent_width` | Integer 1..=32 | Inherit global | Selected language's indentation-stop display columns. |
| `language.<id>.tab_width` | Integer 1..=32 | Inherit global | Selected language's literal-tab display cells. |
| `language.<id>.grammar` | String: "rust" or "plain" | Rust for rust; plain otherwise | Selected language's grammar; no external parser download or installation. |

Association keys are literal basenames or a leading `*.suffix`, including
compound suffixes. Other wildcards, separators and controls reject. Matching is
case-sensitive: any user match precedes all built-ins; within each class, exact
basename wins, then the longest suffix. Recognition uses the visiting basename,
retaining the first alias when canonical reuse finds an existing buffer; unique
buffer-label suffixes do not participate. No language table is required for an
identity. Unmatched or unnamed text is `plain`; only the `rust` identity defaults
to the bundled Rust grammar. Other identities can use indentation without a
parser, or explicitly select either admitted grammar.

### Palette roles

Every value is a string of exactly `#RRGGBB` with six ASCII hexadecimal digits;
hexadecimal letter case is accepted. Each present leaf overrides only that role.
All palette roles are global, with no language-scoped color override.

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `palette.background` | RGB string | `"#1a1b26"` | Document and ordinary prompt background. |
| `palette.foreground` | RGB string | `"#c0caf5"` | Document foreground; ordinary tree text inherits it. |
| `palette.cursor` | RGB string | `"#c0caf5"` | Terminal cursor when the terminal owner supports color/reset. |
| `palette.selection` | RGB string | `"#283457"` | Document and vertical candidate selection background. |
| `palette.selection_text` | RGB string | `"#c0caf5"` | Selected document/candidate foreground. |
| `palette.search` | RGB string | `"#3d59a1"` | Current search-match background. |
| `palette.search_text` | RGB string | `"#c0caf5"` | Search-match foreground; underlining remains. |
| `palette.status` | RGB string | `"#16161e"` | Active status background. |
| `palette.status_text` | RGB string | `"#a9b1d6"` | Active/inactive status foreground. |
| `palette.status_inactive` | RGB string | `"#1a1b26"` | Inactive status background. |
| `palette.border` | RGB string | `"#3b4261"` | Inactive separator foreground. |
| `palette.border_active` | RGB string | `"#7aa2f7"` | Active separator foreground. |
| `palette.tree` | RGB string | `"#16161e"` | Tree background. |
| `palette.tree_selected` | RGB string | `"#283457"` | Selected tree entry background. |
| `palette.tree_selected_text` | RGB string | `"#c0caf5"` | Selected tree entry foreground. |
| `palette.message` | RGB string | `"#c0caf5"` | Normal echo/message foreground. |
| `palette.warning` | RGB string | `"#e0af68"` | Warning foreground. |
| `palette.error` | RGB string | `"#db4b4b"` | Error foreground. |
| `palette.gutter.background` | RGB string | `"#1a1b26"` | Number-field background. |
| `palette.gutter.foreground` | RGB string | `"#565f89"` | Ordinary logical labels. |
| `palette.gutter.current_line` | RGB string | `"#7aa2f7"` | Current-line label; bold emphasis remains without color. |
| `palette.gutter.separator` | RGB string | `"#3b4261"` | Number/text separator foreground. |
| `palette.syntax.keyword` | RGB string | `"#9d7cd8"` | Keyword captures. |
| `palette.syntax.string` | RGB string | `"#9ece6a"` | String/escape captures. |
| `palette.syntax.comment` | RGB string | `"#565f89"` | Comment/documentation captures. |
| `palette.syntax.type` | RGB string | `"#2ac3de"` | Type/constructor captures. |
| `palette.syntax.function` | RGB string | `"#7aa2f7"` | Function/method/macro captures. |
| `palette.syntax.number` | RGB string | `"#ff9e64"` | Constant, boolean and numeric captures. |
| `palette.syntax.punctuation` | RGB string | `"#89ddff"` | Grammar-provided punctuation/operator captures. |


Selection takes precedence over search, which takes precedence over syntax;
row boundaries reset colors/attributes. Unknown syntax groups inherit document
foreground. A nonempty `NO_COLOR` suppresses RGB and retains selection reverse
video/search underlining. Terminal true-color fidelity, intercepted key reports
and cursor OSC support remain terminal capabilities. Cursor color is untouched
for unknown terminal families or NO_COLOR; normal/error/unwind cleanup resets
an attempted cursor color to the terminal default. It does not reconstruct a
previous application's cursor color. See [Terminal input](terminal-input.md#palette-and-cursor-restoration).

## Command, panel and file-completion flows

- [M-x](commands.md) keeps a query above a bounded vertical command list, with
  selection, Tab completion and separate M-p/M-n session history. Return invokes
  the selected or exact/nonempty matching command once; C-g cancels.
- Editing panes, tree and completion rows share composition rules. The tree has
  no editing-window identity, so `C-x o` cycles editing panes only. Candidate rows
  borrow space without scrolling retained document views; tiny/zero geometry
  retains prompts, selection and view identities. No panes are created by completion.
- [Find-file](prompts.md) starts at the visiting file's parent or cwd. Tab completes
  selected/unique names or a shared component prefix; accepted directories descend.
  Return uses an explicit selection, otherwise the freely typed filename, including
  a new name. Canonical aliases reuse dirty buffers; new names remain in memory
  until save. Errors retain the draft and original buffer. Observation is bounded
  to 4096 entries/512 KiB per component and never creates files or traverses child
  symlinks recursively. Rendering/filtering reuse the retained snapshot.

The sample and consistency checks establish schema/default agreement. Parsing a
sample does not prove rendered colors, physical key availability, interaction
geometry or runtime acceptance; those have separate native and terminal checks.

## Text and files

| Area | Default |
| --- | --- |
| Encoding | Strict UTF-8. Invalid UTF-8 and NUL-containing binary input reject before installing a buffer. |
| BOM and separators | Preserve one recognized leading UTF-8 BOM, existing LF/CRLF and final-newline presence. Mixed separators remain mixed. |
| Newline insertion | Follow the buffer's prevailing LF/CRLF style; new text defaults to LF. Literal paste retains supplied separators. |
| Indentation | Tab inserts spaces to the next four-column stop. Literal pasted tabs retain their bytes and use four-cell display stops. |
| Movement | Whole graphemes; buffer positions, UTF-8 bytes and display cells remain separate. |
| Display | No soft wrap. Horizontal scrolling reveals point. Tabs use four-cell stops; control text uses visible notation. |
| Status | [Pane-local groups](status.md): label, state, access and one-based logical line/display column on the left; language mode at the right edge. |
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

Explicit [tabify/untabify](commands.md#whitespace-conversion) commands convert
only the admitted marked region. Loading settings and changing Tab insertion
policy never retabulate existing text.

## Interaction bounds

Prompt input is bounded at 4096 Unicode scalars, with whole-payload rejection
instead of truncation. Command history retains up to 128 distinct accepted
names in session memory. File/buffer arguments and decisions own input until
accepted or canceled. Return and pasted text cannot grant save/delete consent.

A separately reported Escape has a 100-ms Meta deadline; `C-g` cancels the
active interaction. Tiny or zero geometry retains text, views, pending input
and history while suspending/clipping presentation. Normal editing panes need
three text columns and a text/status row pair. Each split reserves one shared
separator cell on its split axis; nested splits use their combined minimum.
Separators use continuous Unicode lines and connected junctions, without an
outer frame around each pane. The tree hides below 32 terminal columns and uses at most 24 columns
or one third of the frame when shown.

## Product boundary

Native mode activation includes `.org` and `.org_archive` files, with
[native Org source coloring](syntax.md#native-org-source). Rust uses the
bundled grammar; other files and syntax failures retain editable plain
presentation. TOML startup preferences configure existing actions. This
version has no plugins, macros, package runtime, LSP,
language completion, automatic
indentation, OS clipboard bridge, regex/project search, soft wrap or session
persistence. The command catalog describes the implemented editing subset.

Use [Bindings](bindings.md), [Focused prompts](prompts.md),
[Failure recovery](recovery.md) and [File tree](file-tree.md) for exact behavior.
