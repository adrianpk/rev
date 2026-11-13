# Scope and defaults

## Runtime and invocation

The executable is `rev`. It runs in a terminal and accepts zero arguments or
one file or directory path. With no path it shows the native four-action
[startup screen](contexts.md#startup-screen). New Buffer enters standalone work;
an explicit path bypasses the screen.
A nonexistent file path opens an empty visiting buffer without creating disk entries.
An explicit directory target opens its visible, first-level-expanded tree.
Otherwise the tree starts hidden unless configured visible; file visits use their
parent as the automatic root, or launch cwd under the explicit cwd preference.
A chosen directory root takes precedence and survives later file visits.

Linux terminals are the primary target. Ordinary execution is offline and uses
compiled-in defaults unless startup configuration supplies supported overrides. [Terminal input](terminal-input.md)
states the keyboard reports and Escape behavior.

Editing views show [absolute logical line numbers](gutter.md) with current-line
emphasis and a stable four-digit minimum field. Numbering does not change text
columns, copied data or saved bytes. Narrow views preserve their text area.

Native mouse support starts enabled: primary text clicks focus/place point;
tree entry clicks select and activate by default. Set
`mouse.tree_activate_on_click = false` for selection/focus only. The wheel
scrolls the pointed pane or tree without changing keyboard focus, and a primary
drag selects canonical text with bounded source-edge autoscroll. Use
`M-x mouse-mode` to toggle only this session. The startup step is three logical
rows; [terminal input](terminal-input.md#pointer-reporting-ownership) documents
supported reports, cleanup and terminal-owned selection bypass.

## Editing profile

`editor.interaction_profile = "non-modal"` is the default, with Emacs-like
editing: ordinary text inserts directly. Choose `"modal"` for Vim-like Normal,
Insert, Replace and Visual editing. Restart after changing it; the profile is
immutable for the session and has no command or language override. Modal editing
starts in Normal. Use i/a/I/A/o/O to enter Insert and Escape/C-[ to return to Normal.
The [binding reference](bindings.md#modal-startup-profile)
lists the currently implemented state commands. Invalid profile or keymap
settings reject the whole candidate into complete non-modal defaults. Follow
[Choose Modal Editing](../tutorials/user-guide/11-modal-editing.md) for a small
startup and editing exercise.

Editor prefix input is available by default through C-u, Meta digits and
Meta minus in non-modal editing and modal Insert/Replace. Normal/Visual C-u
keeps half-page movement. Pending arguments are transient and add no startup
preference; see [Command Prefix Arguments](commands.md#command-prefix-arguments)
for the complete defaults, policies and limits.

## Shell command preferences

These global preferences are frozen at startup. Commands run only after explicit
native invocation and accepted command input. The [shell reference](shell-commands.md)
defines stdin/stdout, checked publication and cancellation; the
[User Guide](../tutorials/user-guide/15-shell-commands.md) shows a sort-and-undo exercise.

| Setting | Type | Default | Behavior |
| --- | --- | --- | --- |
| `shell.program` | String | `""` | Empty selects the nonempty startup SHELL value, then /bin/sh. Otherwise one executable, at most 4096 UTF-8 bytes without NUL. |
| `shell.args` | Array of strings | `["-c"]` | Whole-array replacement; 1..32 entries, at most 4096 bytes each and 32768 total, without NUL. Accepted command text is appended as one final argument. |
| `shell.timeout_seconds` | Integer 1..=600 | `600` | Finite execution deadline, followed by owned process-group cleanup. |
| `shell.max_input_bytes` | Integer 1..=16777216 | `8388608` | Maximum captured region input in UTF-8 bytes. |
| `shell.max_output_bytes` | Integer 1..=16777216 | `8388608` | Aggregate stdout plus stderr bytes; excess fails without admitting partial output. |

## Git preferences

These global preferences are frozen at startup. Loading them does not discover
a repository or launch Git. All values inherit by leaf through ordinary includes;
an invalid source or complete candidate selects the complete built-in defaults.

| Setting | Type | Default | Behavior |
| --- | --- | --- | --- |
| `git.program` | String | `"git"` | Nonempty literal installed executable, at most 4096 bytes without NUL; no shell expansion. |
| `git.observation_timeout_seconds` | Integer 1..=60 | `10` | Deadline for one repository observation, including its capability and HEAD checks. |
| `git.operation_timeout_seconds` | Integer 1..=600 | `60` | Deadline for a changing repository operation. |
| `git.transport_timeout_seconds` | Integer 1..=600 | `120` | Separate deadline for transport. |
| `git.max_status_paths` | Integer 1..=65536 | `1024` | Complete retained path records; reached admission produces explicit truncation. |
| `git.max_path_bytes` | Integer 4096..=4194304 | `262144` | Aggregate exact path bytes retained in an observation. |
| `git.history_page_size` | Integer 1..=1024 | `128` | Revisions in an ancestry page. |
| `git.max_history_pages` | Integer 1..=16 | `4` | Retained ancestry pages per context. |
| `git.max_refs` | Integer 1..=16384 | `256` | Complete retained ref records. |
| `git.max_remotes` | Integer 1..=256 | `32` | Complete retained remote records. |
| `git.max_output_bytes` | Integer 4096..=16777216 | `1048576` | Aggregate stdout and stderr capture for one job. |
| `git.max_detail_bytes` | Integer 4096..=8388608 | `524288` | Detail bytes; must not exceed max_output_bytes. |
| `git.max_diagnostic_bytes` | Integer 1024..=1048576 | `65536` | Retained diagnostic bytes; must not exceed max_output_bytes. |
| `git.max_process_records` | Integer 1..=32 | `8` | Active and recent records; live ownership cannot be evicted. |
| `git.max_process_record_bytes` | Integer 65536..=16777216 | `2097152` | Aggregate retained process bytes; must be at least max_output_bytes. |

All Git capacities, including temporary replacements and parser scratch, also
share a fixed 32 MiB editor-wide admission budget. At most four repository
contexts remain retained. Reached admission preserves existing observations
and active ownership. Count preferences cannot override these hard limits.

## Interactive terminal preferences

These global preferences are frozen at startup and separate from one-shot
shell commands. Loading configuration does not launch a terminal process.
Arguments remain structured values; no command string or implicit `-c` is added.

| Setting | Type | Default | Behavior |
| --- | --- | --- | --- |
| `terminal.program` | String | `""` | Empty selects the nonempty startup SHELL value, then /bin/sh. Otherwise one executable, at most 4096 UTF-8 bytes without NUL. |
| `terminal.args` | Array of strings | `[]` | Whole-array replacement, including an empty array; at most 32 strings and 16384 combined UTF-8 bytes without NUL. |
| `terminal.panel_height` | Integer 3..=200 | `10` | Preferred terminal content rows, clamped to valid geometry that preserves usable source panes. |
| `terminal.max_sessions` | Integer 1..=32 | `8` | Owned live and exited sessions; explicit closure releases capacity. Cell admission can reject a session before this count is reached. |
| `terminal.scrollback_lines` | Integer 0..=20000 | `2000` | Requested retained rows, additionally constrained by cell and extra-storage admission. |
| `terminal.max_pending_input_bytes` | Integer 4096..=1048576 | `262144` | Complete input sequences are accepted atomically into this bound. |
| `terminal.max_pending_output_bytes` | Integer 4096..=1048576 | `262144` | Maximum pre-parser output; direct bounded reads apply PTY backpressure without an accumulating output queue. |

The [terminal reference](integrated-terminal.md#startup-settings) explains these
startup fields, live-session admission and the separate shell-filter settings.

## Startup configuration

The [complete English reference sample](config.toml) contains every supported
global default, all active RGB roles and the profile, state and local context maps. Its
optional include, language and association examples are commented out. First
startup installs the separate [commented startup examples](startup.toml), with no
active overrides. Omitted settings and bindings inherit the installed version's
defaults, including later updates. Uncomment only preferences you want to fix.
Existing configuration files retain their exact contents; remove or comment an
old explicit binding to inherit its current default again.

The executable reads `$XDG_CONFIG_HOME/rev/config.toml` once before creating
buffers or input owners. An unset, empty or relative `XDG_CONFIG_HOME` uses
`$HOME/.config/rev/config.toml`; HOME must be absolute for that fallback.
Data, state and cache locations resolve independently through their XDG variables
with HOME defaults `.local/share/rev`, `.local/state/rev` and `.cache/rev`.
When no entry exists at the configuration path, normal editor startup creates
only its missing parents and the packaged commented examples, then validates it
through the ordinary loader. It does not need installed reference files or a
checkout. Data, state and cache directories remain absent.

Command ranking loads the existing user-wide state window without creating a
file or directory. After a named invocation it may create the state root and
save the latest 256 invocations to `command-history.toml`. This fixed usage
window is separate from the configured distinct-name M-p/M-n history below.
See [Command usage state](commands.md#command-usage-state) for the versioned
format, 64 KiB bound, concurrent publication and nonfatal failure behavior.

The [saved macro library](macro-library.md) reads `macros/library.toml` at startup
without execution or directory creation. Only explicit save/delete writes this
private concern; ordinary naming, replay and exit never save it implicitly.

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

### Org initial visibility

`org.startup_folded` defaults to `showeverything`, preserving fully visible
ordinary Org files. Select `overview` for top-level headings, `content` for all
headings without bodies, or `showall` for visible ordinary text with drawer
bodies hidden. Recognized document STARTUP tokens override configuration. The
[initial visibility reference](org-mode.md#initial-visibility) explains one-time
per-view admission, reset, drawer-only controls and structural protection.

### Org heading presentation

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `org.headings.pretty` | Boolean | `true` | Decorate current native Org heading prefixes; false retains readable source stars. |
| `org.headings.bullets` | Array of marker strings | `["◉", "○", "✸", "✿", "◆", "▶"]` | Whole-array replacement; 1..32 entries, at most 32 UTF-8 bytes each and 1024 total. Each entry is one printable non-whitespace grapheme of terminal width one. Duplicates are valid. |
| `org.headings.hide_leading_stars` | Boolean | `true` | Hide the first N-1 stars in a pretty level-N heading. With visual indentation enabled their cells stay blank; when disabled they occupy no cells. False retains those stars. |
| `org.headings.cycle_bullets` | Boolean | `true` | Cycle markers for deeper levels; false repeats the final configured marker. |

Source positions and the original separator/title remain canonical. With
`org.indent=true`, pretty presentation preserves source-prefix cells within the
added visual margin. With `org.indent=false` and hidden leading stars, it
collapses those cells so all heading markers start at the left margin; showing
source stars restores their literal width and can change wrapping.
The `org.indent` preference adds visual margins to headings and their body text;
see [Org configuration](org/configuration.md). Use `org-indent-mode`,
`org-indent-enable` or `org-indent-disable` to override it for one live buffer
without writing TOML. All its views share that choice; closing discards it.
Literal or unverified prefixes remain source.
Heading recognition remains active with syntax coloring disabled, but respects
`org.enabled`. The terminal owns font coverage and shaping; use `pretty = false`
or simpler one-cell markers when a symbol is unavailable. rev installs no fonts
and does not rewrite programming operators to imitate ligatures.
Use `org-pretty-headings-mode` to override only the active view's pretty flag;
`org-reset-heading-presentation` returns to its immutable startup value.
Both use M-x or modal colon and leave this configuration unchanged.

Heading colors use an independent eight-face cycle in pretty and source
presentation. `palette.org.heading` controls face 1; `palette.org.heading_2`
through `palette.org.heading_8` control the remaining faces. Defaults follow
Tokyo Night blue, purple and lighter variants; level 9 repeats face 1.
See [heading colors](org-mode.md#heading-colors) for all RGB defaults and
selection/search precedence. Disabling syntax coloring leaves markers active.

### Source metadata and maps

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `version` | Integer, exactly 1 | `1` | Per source; omitted means schema 1. |
| `include` | Array of nonempty local path strings | `[]` | Per source; explicit ordered include graph, with fixed limits above. Empty paths, NUL and URL forms reject. |
| `language` | Map of language IDs to partial tables | `{}` | IDs start with `[a-z]`, then contain only `[a-z0-9-]`; no grammar installation is implied. |
| `files.associations` | Map of exact basenames or `*.suffix` to language IDs | `{}` | User rules override built-in recognition; see language rules below. |
| `keys.completion` | Map of popup chord sequences to completion commands | Compiled map | Active cursor popup; all 7 entries are active in the sample. Escape is a reserved literal cancellation. |
| `keys.editor` | Map of chord sequences to existing editor command names | Compiled map | Non-modal editing focus; all 123 entries are active in the sample. |
| `keys.vim_normal` | Map of chord sequences to canonical commands | Compiled map | modal Normal editing; state entry, navigation/counts/search, operators, transfer, jump history, Replace, dot repetition, colon commands and native pane controls. The sample declares all 181 entries. |
| `keys.vim_insert` | Map of chord sequences to canonical commands | Compiled map | modal Insert editing; independent native factories and one-Normal-command C-o. The sample declares all 124 entries. |
| `keys.vim_replace` | Map of chord sequences to canonical commands | Compiled map | modal Replace editing; native modified controls, whole-grapheme overwrite and underline cursor. The sample declares all 123 entries. |
| `keys.vim_visual_character` | Map of chord sequences to canonical commands | Compiled map | Visual Character navigation, inclusive selection and shared operators. The sample declares all 163 entries. |
| `keys.vim_visual_line` | Map of chord sequences to canonical commands | Compiled map | Visual Line navigation, complete logical lines and shared operators. The sample declares all 163 entries. |
| `keys.vim_visual_block` | Map of chord sequences to canonical commands | Compiled map | Visual Block navigation and atomic block delete/change/copy with logical grapheme columns. The sample declares all 159 entries. |
| `keys.minibuffer` | Map of chord sequences to admitted prompt actions/commands | Compiled map | Prompt focus; all 22 entries are active in the sample. |
| `keys.git_refs` | Map of chord sequences to native ref commands | Compiled map | Ref inspection, explicit fetch, reviewed upstream push and confirmed local branch/tag actions in both profiles; all 49 entries are active in the sample. |
| `keys.git_worktree` | Map of chord sequences to native worktree commands | Compiled map | Exact linked-worktree identities, checked Status visits and reviewed create/lock/unlock/prune/remove in both profiles; all 27 entries are active in the sample. |
| `keys.git_stash` | Map of chord sequences to native stash commands | Compiled map | Exact stash selection, read-only detail and checked saved-file create/apply/pop/branch/drop in both profiles; all 26 entries are active in the sample. |
| `keys.git_conflict` | Map of chord sequences to conflict inspection commands | Compiled map | Independent conflict inspection, native result visits, checked saved resolution and optional read-only explanation in both profiles; all 26 entries are active in the sample. |
| `keys.git_sequence` | Map of chord sequences to native sequence commands | Compiled map | Independent Git Operation navigation and checked integration/revision controls and manual bisect in both profiles; all 34 entries are active in the sample. |
| `keys.git_inspect` | Map of chord sequences to read-only Git inspection commands | Compiled map | Revision, reflog, blame and historical bytes in both profiles; all 22 entries are active in the sample. |
| `keys.git_history` | Map of chord sequences to admitted Git navigation commands | Compiled map | History in both profiles, with reviewed ordered cherry-pick/revert; all 29 entries are active in the sample. |
| `keys.git_commit` | Map of chord sequences to native composer commands | Compiled map | Inherits editor keys for commit/tag messages and prepared literal argument composition in both profiles; all 4 entries are active in the sample. |
| `keys.macro_definition` | Map of chord sequences to native definition commands | Compiled map | Inherits ordinary editor text input in both profiles; all 3 entries are active as local overrides in the sample. |
| `keys.macro_menu` | Map of chord sequences to native macro menu commands | Compiled map | Read-only ring component; all 29 entries are active in the sample, including native selection and recovery. |
| `keys.git` | Map of chord sequences to admitted Git commands | Compiled map | One native Git interaction owner in both profiles; all 72 entries are active in the sample. |
| `keys.bookmark_list` | Map of chord sequences to admitted bookmark-list commands | Compiled map | Read-only bookmark view; all 16 entries are active in the sample. Prompts retain their own input maps. |
| `keys.tree` | Map of chord sequences to admitted tree commands | Compiled map | Tree focus; all 33 entries are active in the sample. |

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
| `gutter.nerd_fonts` | Boolean | `false` | Startup choice of compiled symbols for bookmarks and decorated tree entries: ordinary when false, Nerd Fonts when true; no font detection or installation. |
| `gutter.mode` | String: "absolute" or "relative" | `"absolute"` | Logical line calculation from each view's own point. |
| `gutter.current_line` | String: "absolute", "zero" or "hidden" | `"absolute"` | Current-row label, independent of calculation mode. |
| `gutter.side` | String: "left" or "right" | `"left"` | Number field placement inside its editing pane. |
| `gutter.minimum_digits` | Integer 1..=20 | `4` | Minimum reserved digits; full source line count may require more. |
| `gutter.left_padding` | Integer 0..=4 | `0` | Cells before digits, excluding the separator. |
| `gutter.right_padding` | Integer 0..=4 | `1` | Cells after digits, excluding the separator. |

The [gutter reference](gutter.md#bookmark-markers) describes fixed bookmark
symbols and outside placement. Its [migration guidance](gutter.md#migrate-older-bookmark-settings)
covers ignored legacy `bookmarks.marker` strings and explicit old palette colors.

### Mouse controls

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `mouse.enabled` | Boolean | `true` | Initial native pointer reporting; M-x mouse-mode changes only the session override. |
| `mouse.scroll_lines` | Integer 1..=100 | `3` | Logical rows per wheel report; each drag-edge tick also clamps to the current text-area height. |
| `mouse.tree_activate_on_click` | Boolean | `true` | Startup tree policy; false selects/focuses only. Global mouse disablement suppresses both selection and activation. |

### Optional Pi transport

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `ai.commands` | Map of at most 64 complete literal definitions | `{}` | Callable ai-name records; input/context/output and optional Pi model. See [Reusable AI commands](commands.md#reusable-ai-commands). |
| `ai.pi.command` | Nonempty string, at most 4096 bytes, no NUL | `"pi"` | Trusted executable name/path; arguments are supplied by the adapter without a shell. |
| `ai.pi.model` | Optional nonempty string, at most 256 bytes, no NUL | Unset | Pi model reference; absent uses Pi's resolved model. |
| `ai.pi.persist_sessions` | Boolean | `true` | Directory-history policy; input-only operations always use a fresh ephemeral child. |
| `ai.pi.idle_timeout_ms` | Integer 0..=3600000 | `900000` | Directory-child idle lifetime; input-only children close after each operation. |
| `ai.pi.request_timeout_ms` | Integer 1000..=1800000 | `300000` | Generation deadline followed by bounded cancellation/shutdown. |
| `ai.pi.max_input_bytes` | Integer 1..=4194304 | `1048576` | Complete encoded editor snapshot/instruction envelope. |
| `ai.pi.max_result_bytes` | Integer 1..=1048576 | `262144` | Logical generated UTF-8 result limit, separate from protocol frame limits. |

### Experimental plugin declarations

| Setting | Type / bounds | Default | Meaning |
| --- | --- | --- | --- |
| `plugins.enabled` | Boolean | `false` | Admit the explicit local experimental package catalog; disabled startup reads no manifests. |
| `plugins.manifests` | Array of 0..=64 local paths | `[]` | Replace the manifest list; resolve paths against the source assigning the array. |
| `plugins.startup` | Array of 0..=8 distinct package IDs | `[]` | Explicit startup activation requests for admitted packages; empty keeps lazy command/trigger activation. |
| `plugins.keybindings` | Array of 0..=512 binding objects | `[]` | Explicit command, keys, context and scope with optional buffer selectors; user choices precede package defaults within that scope. |

See [experimental plugin declarations](plugins.md) for complete manifest,
selector, lifecycle and limit contracts, or the [optional plugin lesson](../tutorials/user-guide/plugins/index.md).

### Optional server transport

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `lsp.enabled` | Boolean | `false` | Start optional configured stdio servers for activated files; lsp-mode toggles the session. |
| `lsp.shutdown_on_last_buffer` | Boolean | `false` | True shuts down a healthy connection when its last buffer detaches; false retains it until disable/exit. |
| `lsp.request_timeout_ms` | Integer 1..=600000 | `2000` | Milliseconds allowed for an outbound write or subsequent client request. |
| `lsp.initialize_timeout_ms` | Integer 1..=600000 | `10000` | Milliseconds to initialize a newly owned connection. |
| `lsp.shutdown_timeout_ms` | Integer 1..=600000 | `2000` | Milliseconds for shutdown/exit before owned-child termination and reaping. |
| `lsp.max_message_bytes` | Integer 1..=8388608 | `8388608` | Maximum incoming or outgoing JSON body bytes. |
| `lsp.max_servers` | Integer 1..=8 | `8` | Maximum retained server/root connections, including failed entries. |
| `lsp.max_pending_requests` | Integer 1..=16 | `16` | Maximum pending completion/resolve requests per connection; one completion per buffer. |
| `lsp.servers` | Map of at most 32 server entries | `{ rust = { languages = ["rust"], command = "rust-analyzer", args = [], root_markers = ["Cargo.toml"], initialization_options = {}, settings = {}, env = {} } }` | Named entries merge by leaf; the Rust default needs an installed executable. |
| `lsp.preferred` | Map from language ID to compatible server name or `false` | `{}` | Explicit server choice or native-only; absent choices keep lexical selection. `true` and unknown or incompatible names reject the complete candidate. |

Each `lsp.servers.<name>` entry declares `languages` (nonempty language array),
`command` (nonempty executable), `args` (literal string array), `root_markers`
(relative marker paths without traversal), `initialization_options` and `settings`
(finite JSON-compatible TOML data). New entries need command and languages after
source inheritance. Empty argument/marker arrays explicitly replace earlier values.
`env` is a literal string map, empty by default, merged by key across includes.
It overrides the inherited process environment only for that server. Admit at
most 64 entries, 128-byte nonempty names without NUL or `=`, 8 KiB per value
without NUL, and 64 KiB total name/value bytes. Diagnostics omit values.
An explicit `lsp.preferred.<language>` selects one compatible server or `false`
for native-only. Otherwise the first matching server name in lexical order wins. No grammar allowlist,
download, environment expansion or shell evaluation applies.

Root lookup chooses the nearest configured ancestor marker. An explicitly opened
containing tree bounds lookup and supplies the fallback; files outside that tree
fall back to their own containing directory. Empty marker arrays use that directory.
Canonical roots share a single process across file aliases. Queues retain at most
64 messages and 16 MiB per direction; stderr retains a separate 64 KiB tail.
The transport negotiates UTF-8/UTF-16 and requires document open/close plus full or incremental sync to admit an unsaved document. Eligible visiting
buffers publish unsaved text through didOpen/didChange and retire it through
didClose; synchronization never saves a file. Successful saves send only the
negotiated save notification. Wire columns count encoding units, with canonical
transaction ranges used before each incremental mutation. Split views and
canonical file aliases share one document. Documents admit at most 8 MiB each
and 32 MiB total; larger buffers stay editable and retain native fallback within
its own source limits.

`lsp-disconnect` excludes the current canonical buffer across all its views until
`lsp-connect` restores eligibility or the buffer closes. Other attachments retain
their connection. Restore never enables globally disabled LSP. Reopening or
restoring advances document generation; undo and equal bytes with a new revision
cannot reuse old synchronization evidence. Missing or failed
servers remain unavailable until `lsp-restart` or disable/enable. Restart replaces
the eligible current root while preserving other roots and exclusions.
`lsp-status` reports transport readiness. Admitted semantic results use the native
popup; empty, unsupported or unavailable results use native fallback. Selected
resolve and same-document imports publish with the primary as one undo
transaction. See [Language servers](language-servers.md) for the protocol profile,
edit limits and finite lifecycle.

### Editing, presentation and retention

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `editor.interaction_profile` | String: "non-modal" or "modal" | `"non-modal"` | Immutable session profile, selected at startup. Modal editing begins in Normal; no language or runtime override. |
| `editor.soft_wrap` | Boolean | `true` | Word-aware wrapping at each pane's available text width. M-x visual-line-mode toggles one live buffer for all its views; false retains horizontal scrolling. No source or history changes. |
| `editor.indent_style` | String: "spaces" or "tabs" | `"spaces"` | Buffer policy; Tab emits the selected indentation form. Language-overridable. |
| `editor.indent_width` | Integer 1..=32 | `4` | Buffer policy; indentation-stop display columns, independently of tab display width. Language-overridable. |
| `editor.tab_width` | Integer 1..=32 | `4` | Buffer policy; literal-tab display cells. Shared views use the same width. Language-overridable. |
| `scroll.page_overlap` | Integer 0..=1000 | `2` | Session; overlap in page rows, clamped so a usable pane advances at least one row. |
| `windows.split_ratio` | Finite float 0.1..=0.9 | `0.5` | Session; preferred first-child share after the separator, clamped by both recursive subtree minima. |
| `minibuffer.candidate_limit` | Integer 1..=32 | `8` | Session; maximum visible command/path candidate rows, including any list diagnostic, clipped to geometry. |
| `minibuffer.match` | String: "prefix-then-substring", "prefix" or "substring" | `"prefix-then-substring"` | Session; M-x/colon matching only. Exact names precede the four-recent/four-frequent ordering within each match rank. File candidates retain literal component-prefix matching. |
| `minibuffer.history_limit` | Integer 1..=4096 | `128` | Session; distinct accepted names for M-p/M-n, newest first. Navigation is not persisted; the separate fixed 256-invocation window persists candidate ranking. |
| `minibuffer.input_limit` | Integer 1..=65536 | `4096` | Session; Unicode scalars per prompt, including paste, completion and history recall. Excess rejects wholly. |
| `completion.enabled` | Boolean | `true` | Native manual and automatic completion; completion-mode toggles this session only. |
| `completion.automatic` | Boolean | `true` | Requests after eligible native identifier/separator edits; completion-auto-mode toggles this session only. |
| `completion.delay_ms` | Integer 0..=2000 | `150` | Milliseconds after the newest eligible edit; explicit invocation bypasses delay. |
| `completion.min_prefix` | Integer 1..=64 | `2` | Unicode identifier characters; manual requests and native language separators bypass the minimum. |
| `completion.in_comments` | Boolean | `false` | Allow automatic requests in admitted native comments. |
| `completion.in_strings` | Boolean | `false` | Allow automatic requests in admitted native string/character literals. |
| `completion.max_candidates` | Integer 1..=200 | `200` | Ranked, deduplicated current-request candidate limit. |
| `completion.popup_rows` | Integer 1..=32 | `10` | Candidate rows, clipped inside the selected pane with a single border. |
| `history.group_limit` | Integer 1..=65536 | `512` | Per buffer; maximum retained coherent undo/redo groups. Pruning never splits an operation. |
| `history.payload_target_bytes` | Integer 1..=1073741824 | `67108864` | Per buffer; soft logical fragment-payload target in bytes, not resident memory. The newest complete group survives excess. |
| `macros.ring_limit` | Integer 1..=32 | `8` | Session; bounded ring shared across source buffers; eviction retains independent snapshots. |
| `macros.execute_before_append` | Boolean | `true` | Session; pre-execute a prefixed append unless double plain C-u skips it. |
| `kill_ring.capacity` | Integer 1..=4096 | `32` | Session; linear entries, separate from retained rectangle rows. |
| `kill_ring.coalesce` | Boolean | `true` | Session; consecutive compatible directional kills join; unrelated commands still end the chain. |
| `tree.decorated` | Boolean | `true` | Startup; kind symbols and semantic entry colors, with the catalog chosen by gutter.nerd_fonts. Set false for the existing minimal labels. |
| `tree.follow_active_buffer` | Boolean | `true` | Startup; highlight the active editing file when its row is loaded. Reveal offscreen matches without expanding directories or replacing roots. False retains the manual selection across buffer changes. |
| `tree.visible` | Boolean | `false` | Initial show preference; showing never steals editor focus. Narrow geometry can temporarily hide the pane. |
| `tree.auto_width` | Boolean | `true` | Startup/reset policy; fit all admitted expanded rows with a stable high-water mark. Explicit width actions retain session ownership. |
| `tree.width` | Integer 1..=512 | `24` | Startup/reset; automatic minimum or fixed preferred columns when auto_width is false. Actual width is constrained by frame geometry. |
| `tree.max_fraction` | Finite float 0.1..=0.5 | `0.3333333333333333` | Session; maximum terminal-width fraction for the tree. The divider is separate. |
| `tree.root` | String: "auto", "cwd" or "file-parent" | `"auto"` | Explicit directory targets win. Otherwise auto/file-parent follow the visiting file parent, falling back to launch cwd; cwd retains launch cwd. |
| `tree.show_hidden` | Boolean | `true` | Session; admit dot-prefixed tree entries. This does not filter find-file candidates. |
| `tree.directories_first` | Boolean | `true` | Session; tree directories sort before leaves, then by filename bytes. This does not change find-file ordering. |
| `markdown.pretty` | Boolean | `true` | Startup default for each view; markdown-pretty-mode toggles symbols and markdown-reset-presentation restores this value. Splits inherit the current choice, then change independently. |
| `markdown.enabled` | Boolean | `true` | Startup; enable native Markdown recognition and explicit mode selection. Ordinary text and optional language servers remain available. |
| `syntax.enabled` | Boolean | `true` | Session; enable derived colors in admitted syntax and native document modes. Text remains editable on fallback. |
| `syntax.max_bytes` | Integer 0..=1073741824 | `0` | Session; UTF-8 byte cutoff. Zero means no configured cutoff; equality is admitted and larger buffers omit colors. Native modes retain their own finite observation bounds. |
| `files.newline` | String: "lf" or "crlf" | `"lf"` | Creation default for new buffers only; existing separators and final-newline state are preserved. |
| `files.bom` | Boolean | `false` | Creation default for one leading UTF-8 BOM; existing files retain their observed BOM state. |
| `files.external_changes` | String: "reload" or "ask" | `"reload"` | Reload clean visiting buffers automatically, or retain text and notify under "ask". Both preserve unsaved text and its overwrite guard. |
| `files.external_change_debounce_ms` | Integer 0..=2000 | `150` | Coalesce external observation bursts in milliseconds. |
| `files.external_change_max_bytes` | Integer 1..=134217728 | `16777216` | Maximum complete reload image, including its BOM. |
| `files.external_change_max_buffers` | Integer 1..=4096 | `256` | Maximum concurrently observed visiting buffers. |
| `bookmarks.file` | Absolute native path string, no NUL | Unset | User-wide collection; unset resolves to `$XDG_DATA_HOME/rev/bookmarks.toml` or `$HOME/.local/share/rev/bookmarks.toml`. No shell expansion. |
| `bookmarks.save` | String: "on-exit", "on-change" or "manual" | `"on-exit"` | Save after source quit decisions, after explicit bookmark mutations, or only on explicit save. Source edits dirty evidence without IO. |
| `bookmarks.markers` | Boolean | `true` | Initial per-view marker visibility, independent of numbering; splits inherit the current view. |
| `bookmarks.max_entries` | Integer 1..=4096 | `1024` | Admission cap for both memory and disk; excess rejects without eviction. |
| `bookmarks.context_chars` | Integer 16..=256 | `64` | Source scalars captured on each side at a checked checkpoint. Existing stored context remains bounded to 256 per side. |
| `bookmarks.search_chars` | Integer 0..=1048576 | `262144` | Total scalar search window on reopen; zero disables relocation. Each pass consumes at most 16384 scalars. |

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
| `language.<id>.grammar` | String: "rust", "c", "go", "html", "css", "javascript", "typescript", "tsx", "json", "toml" or "plain" | Matching bundled grammar; plain otherwise | Selected language's grammar; no external parser download or installation. |
| `language.<id>.external.parser` | Nonempty local path | Unset | Precompiled native library; relative to resolved XDG data root `rev/grammars`, or absolute. At most 32 complete registrations. |
| `language.<id>.external.symbol` | ASCII `tree_sitter_` identifier, at most 128 bytes | Unset | Explicit parser export; required with the three asset paths. |
| `language.<id>.external.highlights` | Nonempty local path | Unset | UTF-8 query, 1..=1048576 bytes; supported highlight captures only. |
| `language.<id>.external.completions` | Nonempty local path | Unset | UTF-8 query, 1..=1048576 bytes; finite declaration/scope/literal profile. |

Association keys are literal basenames or a leading `*.suffix`, including
compound suffixes. Other wildcards, separators and controls reject. Matching is
case-sensitive: any user match precedes all built-ins; within each class, exact
basename wins, then the longest suffix. Recognition uses the visiting basename,
retaining the first alias when canonical reuse finds an existing buffer; unique
buffer-label suffixes do not participate. No language table is required for an
identity. Unmatched or unnamed text is `plain`; Rust, C, Go, HTML, CSS,
JavaScript, TypeScript, TSX, JSON and TOML identities default to their bundled grammars.
A complete `external` table replaces its registration as a unit when layers
override it. Assets are optional and user supplied; see
[external grammar assets](syntax.md#external-grammar-assets) for admission,
query captures, trust and restart requirements.
`.tsx` retains the `typescript` preference identity while selecting its TSX parser. Other identities can use indentation
without a parser, or explicitly select an admitted grammar.

### Palette roles

Every value is a string of exactly `#RRGGBB` with six ASCII hexadecimal digits;
hexadecimal letter case is accepted. Each present leaf overrides only that role.
All palette roles are global, with no language-scoped color override.

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `palette.background` | RGB string | `"#1a1b26"` | Document and ordinary prompt background. |
| `palette.foreground` | RGB string | `"#c0caf5"` | Document foreground; minimal tree text inherits it. |
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
| `palette.tree_entries.root` | RGB string | `"#7aa2f7"` | Root icon and name foreground in decorated mode; focused selection overrides it. |
| `palette.tree_entries.directory` | RGB string | `"#e0af68"` | Directory icon and name foreground in decorated mode; focused selection overrides it. |
| `palette.tree_entries.source` | RGB string | `"#7aa2f7"` | Source-file icon and name foreground in decorated mode; focused selection overrides it. |
| `palette.tree_entries.document` | RGB string | `"#e0af68"` | Document icon and name foreground in decorated mode; focused selection overrides it. |
| `palette.tree_entries.configuration` | RGB string | `"#bb9af7"` | Configuration-file icon and name foreground in decorated mode; focused selection overrides it. |
| `palette.tree_entries.file` | RGB string | `"#c0caf5"` | Unclassified-file icon and name foreground in decorated mode; focused selection overrides it. |
| `palette.tree_entries.symlink` | RGB string | `"#7dcfff"` | Symlink icon and name foreground in decorated mode; focused selection overrides it. |
| `palette.tree_selected` | RGB string | `"#283457"` | Selected tree entry background. |
| `palette.tree_selected_text` | RGB string | `"#c0caf5"` | Selected tree entry foreground. |
| `palette.message` | RGB string | `"#c0caf5"` | Normal echo/message foreground. |
| `palette.warning` | RGB string | `"#e0af68"` | Warning foreground. |
| `palette.error` | RGB string | `"#db4b4b"` | Error foreground. |
| `palette.git.heading` | RGB string | `"#7aa2f7"` | Git view titles and patch file headers. |
| `palette.git.metadata` | RGB string | `"#a9b1d6"` | Repository and comparison metadata. |
| `palette.git.untracked` | RGB string | `"#2ac3de"` | Untracked section and paths. |
| `palette.git.unstaged` | RGB string | `"#e0af68"` | Unstaged section and paths. |
| `palette.git.staged` | RGB string | `"#9ece6a"` | Staged section and paths. |
| `palette.git.conflict` | RGB string | `"#f7768e"` | Unmerged section and paths. |
| `palette.git.added` | RGB string | `"#9ece6a"` | Added patch lines, including new files. |
| `palette.git.removed` | RGB string | `"#f7768e"` | Removed patch lines. |
| `palette.git.hunk` | RGB string | `"#9d7cd8"` | Unified patch hunk headers. |
| `palette.gutter.background` | RGB string | `"#1a1b26"` | Number-field background. |
| `palette.gutter.foreground` | RGB string | `"#565f89"` | Ordinary logical labels. |
| `palette.gutter.current_line` | RGB string | `"#7aa2f7"` | Current-line label; bold emphasis remains without color. |
| `palette.gutter.bookmark` | RGB string | `"#7aa2f7"` | Resolved bookmark marker foreground. |
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
  the highlighted command once, including with an empty query; C-g cancels.
- Editing panes, tree and completion rows share composition rules. The tree has
  no editing-window identity, so `C-x o` cycles editing panes only. Candidate rows
  borrow space without scrolling retained document views; tiny/zero geometry
  retains prompts, selection and view identities. No panes are created by completion.
- [Find-file](prompts.md) starts at the visiting file's parent, otherwise a chosen
  directory or launch cwd. Tab completes the highlighted full path, initially
  the first suggestion, and keeps the prompt open. Return accepts a directory
  as a fresh temporary directory context after native source/departure decisions.
  A file visit stays within the current context; an ordinary directory opening
  never registers a Project or replays its saved session.
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
| Display | Word-aware soft wrap at each pane's text width. `editor.soft_wrap = false` selects horizontal scrolling; `visual-line-mode` toggles all views of one buffer. Tabs use four-cell stops; control text uses visible notation. |
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
[native Org source coloring](syntax.md#native-org-source). Rust, C, Go, HTML, CSS,
JavaScript, TypeScript/TSX, JSON and TOML use ten bundled parsers and
[native completion](completion.md). Org uses its native structural contexts.
Explicit [external grammar assets](syntax.md#external-grammar-assets) can admit
another language or replace bundled syntax. Failed admission retains bundled
syntax when available; unsupported identities and syntax failures retain editable
plain presentation. TOML startup
preferences configure existing actions. [Keyboard macros](macro-editing.md) use
native session input and explicit TOML persistence. Trusted local
[experimental plugins](plugins.md) add qualified commands and checked native
operations through owned processes. The command catalog describes the
implemented editing subset and each optional facility's boundaries.

Use [Bindings](bindings.md), [Focused prompts](prompts.md),
[Failure recovery](recovery.md) and [File tree](file-tree.md) for exact behavior.

### Markdown semantic colors

[Native Markdown syntax](syntax.md#native-markdown-source) uses these independent
RGB roles. Inline objects override their enclosing heading, quote, list or table.
ATX heading symbols retain the source prefix width; Setext underlines stay literal.

| Key | Type | Default | Effect |
| --- | --- | --- | --- |
| `palette.markdown.heading` | RGB hex string | `"#7aa2f7"` | Heading level 1, including its ATX symbol and ordinary title. |
| `palette.markdown.heading_2` | RGB hex string | `"#9d7cd8"` | Heading level 2, including its ATX symbol and ordinary title. |
| `palette.markdown.heading_3` | RGB hex string | `"#c0caf5"` | Heading level 3, including its ATX symbol and ordinary title. |
| `palette.markdown.heading_4` | RGB hex string | `"#9bb9f9"` | Heading level 4, including its ATX symbol and ordinary title. |
| `palette.markdown.heading_5` | RGB hex string | `"#b59ce1"` | Heading level 5, including its ATX symbol and ordinary title. |
| `palette.markdown.heading_6` | RGB hex string | `"#bcd0fb"` | Heading level 6, including its ATX symbol and ordinary title. |
| `palette.markdown.emphasis` | RGB hex string | `"#9d7cd8"` | Emphasis and strong emphasis. |
| `palette.markdown.link` | RGB hex string | `"#7aa2f7"` | Links, reference links, autolinks and images. |
| `palette.markdown.code` | RGB hex string | `"#9ece6a"` | Inline, fenced and indented literal code. |
| `palette.markdown.quote` | RGB hex string | `"#565f89"` | Block quotes. |
| `palette.markdown.list` | RGB hex string | `"#89ddff"` | Ordered and unordered lists, including continuations. |
| `palette.markdown.table` | RGB hex string | `"#89ddff"` | GFM tables, including header and delimiter. |
