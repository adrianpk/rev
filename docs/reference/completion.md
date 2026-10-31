# Native completion

Rust, C and Go completion opens automatically after an identifier edit of at least two
Unicode characters or a native `.` / `::` separator. The default debounce waits
150 ms after the newest eligible native edit. `M-TAB` or
`M-x completion-at-point` requests immediately without a minimum prefix.
Terminal or desktop shortcuts may intercept M-TAB; the named command uses the
same handler. Candidates come from declarations in the current unsaved buffer
and bounded same-language open buffers, using their retained syntax owners. No server or file scan runs.

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

Candidates are observed declaration names, with kinds displayed separately.
Labels never become insertion text. Lowercase input matches without case
sensitivity; uppercase input requires matching case. Prefix matches precede
ordered subsequences, followed by current-buffer locality, declaration proximity,
source ordering and stable lexical ties.
Equal insertions are deduplicated. A single candidate still requires acceptance.
No usable candidates produce no popup. Automatic requests stay quiet in admitted
comments, strings and character literals by default; manual invocation bypasses
that suppression while retaining the same declaration coverage.

Acceptance replaces the full identifier around point with literal text through
the ordinary edit owner, as one undo operation. Read-only buffers and obsolete
view, point, revision, syntax or invocation identities cannot publish that edit.
Selection, cancellation and failed admission create no history entry. Native
completion does not expand snippets, execute commands or save a file.

Rust observes functions/methods, types/traits, modules, fields, constants and
local/parameter declarations. C observes functions, typedef/type/enum names,
fields, enumerators and local/parameter declarations. Go observes functions/
methods, types, fields, constants and local/parameter declarations. Observed
names do not imply imports, macro expansion, type checking or member resolution. Missing or unsupported syntax degrades
assistance while preserving ordinary editing.

## Preferences and work bounds

The complete [startup sample](config.toml) and [configuration catalog](defaults.md)
list all eight `completion` leaves and their finite ranges. Defaults are
`enabled = true`, `automatic = true`, `delay_ms = 150`, `min_prefix = 2`,
`in_comments = false`, `in_strings = false`, `max_candidates = 200` and
`popup_rows = 10`. `completion-mode` toggles completion for this session;
`completion-auto-mode` toggles automatic requests while retaining manual access.
Neither command writes settings or changes the startup-only loading contract.

Extraction admits at most 2 MiB and 4096 symbols per buffer, 32 observed buffers,
8 MiB total shared source snapshots, 16384 retained symbols, 8 MiB observation
payload and 512-byte labels. Current context precedes other sources; older
other-buffer observations are omitted first under pressure. One temporary worker
borrows each admitted buffer's sole retained parser/tree and shared rope snapshot. At most one worker and one newest pending request exist;
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
