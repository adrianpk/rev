# Changelog

All notable changes to this project are documented here.

This project uses Semantic Versioning.

## Unreleased

### Added

- Export Org buffers, subtrees or selected regions to ASCII, UTF-8 text, Markdown, Org, HTML, LaTeX, Beamer or OpenDocument text files through native commands. Export includes unsaved text, retains tables, links and footnotes, and never saves source or runs code. Select options explicitly, confirm existing destination replacements, and receive a diagnostic when ASCII cannot represent a character. HTML retains document structure, captions and admitted attributes, with escaped code and safe local image references. TeX output retains native structure and bounded math with escaped source; explicit PDF commands can run a configured compiler after confirmation, with bounded time/output and owned cleanup. OpenDocument text retains headings, lists, table cells and notes in a native package; unsupported rich content stays readable with diagnostics.

- Inspect source hierarchies with `org-columns` and capture property, effort and clock totals in native `columnview` reports. Summaries remain projections; explicit updates replace only recognized report bodies and undo as one change.


- Create and align native Org tables, navigate their cells with TAB and S-TAB, insert or remove rows and columns, move them with Meta-arrows, and sort selected data rows. Convert flat lists and one-column tables through native commands. Explicitly recalculate a table, field or column with arithmetic, references, aggregates, rounding and decimal formatting. Use C-c * for the table or C-c C-c on a chosen formula line; configure numeric precision and work limits. Results change together in one undo step; unsupported formulas and failed calculations preserve existing source. Supported absolute formula references follow structural changes; unsupported or ambiguous rewrites leave the source intact. Import and export quoted CSV/TSV table data through explicit local files, with checked replacement and one-operation import undo. Alignment retains cell text and uses the editor’s tab and Unicode display policy; each structural change supports ordinary undo.

- Create and follow Org footnotes with `C-c C-x f`, renumber numeric notes, edit emphasis with `C-c C-x C-f`, insert named entities and affiliated data with ordinary source undo, and follow checked radio targets in ordinary text.

- Manage heading attachments with `C-c C-a`: copy, move, hard-link, list and open selected files using ID or DIR locations. Confirm replacements, retain partial file outcomes, and keep heading metadata edits in ordinary undo.

- Store, insert, edit and follow native Org links with familiar commands. Visit checked internal targets, headings, IDs, local files and attachments, preserve descriptions and source bytes, and return through recorded source positions. Link insertion uses ordinary undo; missing or ambiguous destinations retain the editing state. Explicitly confirmed external links use configured viewer arguments, bounded output and time, and owned process cleanup.

- Archive Org subtrees in place with the ARCHIVE tag, below an Archive sibling, or in a checked archive file. External archives retain the origin file, outline path and task context. Select archive files explicitly for agenda views, and use `org.agenda.include_archived` to include tagged trees. Archive moves preserve ordinary undo history and report each file's save separately through `org-archive-results`.

- Move complete Org subtrees with `C-c C-w` or copy them with `C-c M-w` into an explicitly selected heading. Configure `org.refile.targets` or use admitted agenda/source headings. Refile preserves shared views and undo history, rejects stale or unavailable destinations, and reports each ordinary file save separately through `org-refile-results`.
- Start a native Org capture draft with `C-c c`, select a configured template or the Task fallback, and enter missing destination or text values explicitly. Edit the isolated draft with ordinary undo/redo, then save it with `C-c C-c` or abort with `C-c C-k` to return to the originating pane. Finalization checks the current destination, preserves its Org structure and file format, and keeps the draft after failure. If replacement succeeded but save completion is unresolved, retry saves the same insertion without duplicating it. Template expansion preserves inserted source literally and rejects executable forms.

- Browse native Org day, week, month, year and custom-span agendas with `C-c a`, including appointments, time grids, scheduled tasks, overdue reminders and deadline warnings. Select configured local sources or open buffers; unsaved text takes precedence. Move between dates, refresh explicitly with `g`, visit checked source with Enter, and close with `q` without editing the agenda. Select unfinished or explicit TODO states, search text with phrases or Boolean terms, and match tags/properties with inherited metadata and effort. Configure named block views, property columns and stable sort keys; filter visible entries and restore preset filters without rereading source. Change task state, dates, tags, priority, effort and clocks through checked source actions. Mark entries for bulk operations, inspect each target outcome, and undo unchanged source transactions. Source failures and resource limits remain visible.

- Complete native Org habits with repeating schedules and minimum/maximum calendar intervals. Existing task controls preserve compatible history, logging choices and single-step undo. Set `org.time.extend_today_until` for date-only habit recurrence before your day starts; completion logs keep their actual time. Inspect bounded consistency graphs with `C-c C-x h`, refresh explicitly, and toggle visibility, today-only or all-today display. Graphs retain actual completion markers and late status, clip safely in narrow panes and remain readable without color.

- Take timed Org notes with native relative timers, pause/continue/stop controls and H:MM:SS insertion. Timed description lists continue with M-RET and undo through normal source history. Explicit countdowns show one in-editor expiry notice, stop on session exit and add no wakeups while paused or inactive.

- Compare recorded Org task, subtree or file time with effort estimates using `org-clock-display`. Date windows clip actual intervals, while open, unresolved and overlapping records remain visible without changing source. Explicitly insert or refresh compatible clock tables; updates preserve surrounding source and undo in one step.

- Record task time with native Org clock-in, clock-out, cancel and goto commands. Compatible CLOCK intervals retain elapsed minutes across midnight and zone transitions, use task-owned drawers, and undo through normal history without periodic source changes. Configure drawer placement and optional zero-minute removal with `org.clock.drawer` and `org.clock.remove_zero`. Switching tasks closes the previous interval; completing a clocked task closes its interval with the task changes, including cancellable notes. Quit and buffer close offer closing or preserving an open interval; `org-resolve-clocks` explicitly resumes, closes or cancels incomplete time with checked local-time choices. Opt into checked restart recovery with `org.clock.persist`, and re-clock recent tasks within the configured history limit.

- Complete repeating Org tasks with distinct `+`, `++` and `.+` schedules. Reset state, planning dates, selected history and LAST_REPEAT change together and undo in one step. Configure repeat time/note logging with `org.todo.log_repeat`; notes and local-time choices remain cancellable.

- Enter active or inactive Org timestamps, date/time ranges, schedules and deadlines through a native calendar prompt. Use shifted arrows to adjust timestamp fields or dates, normalize weekdays with `C-c C-c`, and enter a range with successive timestamp commands. Relative dates, English month/weekday forms and explicit DST offset choices retain checked source edits and single-step undo.

- Set and cycle Org priorities, inspect implicit defaults, and store effort estimates in native duration forms. Effort choices, original value spelling and unrelated task metadata survive checked edits and single-step undo.

- Change Org task states with `C-c C-t` or M-x using configured or file-local workflows. Selected completion timestamps, state notes and progress cookies change together, respect task dependencies, and undo in one step. Set `org.time.zone` to use local, system, UTC or a named zone for task logging; each operation retains one captured time and diagnoses invalid zone data before editing.

- Edit and complete Org tags with `C-c C-q`, inspect local and inherited tag owners, and test Boolean tag/property expressions against the current heading without changing source.

- Edit Org properties with `C-c C-x p`, inspect local and inherited values with source owners, and explicitly edit an inherited owner. Selective property inheritance preserves additive values and keeps inherited text out of child entries.

- Continue, indent, move and renumber Org lists with their nested source. Toggle ordinary or radio checkboxes, respect ordered completion, and update parent states and progress cookies together in one undoable change. Familiar Meta-arrow and C-c keys select the actual list context.

- Promote, demote and reorder complete Org subtrees with Meta-arrow keys or M-x, including descendants, drawers, blocks and table metadata. Whole sibling regions change together, shared panes and marks follow the source, and each operation undoes as one change.

- Insert Org headings with M-RET and navigate visible headings, siblings and parents with the familiar C-c chords or M-x. Insertion preserves source separators and attached text, undoes as one change, and keeps shared panes aligned. Heading motion also works on read-only files.

- Fold Org headings with TAB and cycle overview, contents and show-all with S-TAB. M-x provides direct subtree and drawer choices. Each pane keeps independent visibility, searches reveal the target path, and canceling search restores the starting view without changing source text.

- First startup creates the complete editable default configuration at the resolved user XDG path when absent. Existing files, links and other entries remain untouched; creation failures show a diagnostic and keep editing usable with factory defaults.

- Click to focus panes and place point, scroll the pointed pane or file tree without changing keyboard focus, and drag to select text with bounded autoscroll. Mouse support starts enabled; configure `mouse.enabled` and `mouse.scroll_lines`, or toggle the current session with `M-x mouse-mode` without writing settings. Copy and kill use the ordinary native region.

- Show absolute or relative logical line numbers with independent settings in each editing view. Use M-x to switch modes, hide numbers, change the current-line label or side, inspect overrides and restore startup defaults. Stable widths preserve document columns, and narrow views retain their text area.

- Convert marked tabs and spaces with `M-x tabify` and `M-x untabify`, preserving text columns, Unicode and file line endings. Each conversion undoes or redoes as one change; an unselected document stays untouched.

- Go to a logical line with `M-g g`, `M-g M-g` or `M-x goto-line`. Cancel without moving the current view, correct invalid numbers in place, and navigate read-only text without changing its history.

- Configure existing editing, display, palette, history, file and context-keybinding preferences through startup TOML under the XDG configuration directory. Explicit includes and language overrides compose before startup; invalid sources show a diagnostic and retain complete defaults. Configuration changes take effect after restarting, and startup creates no settings files or empty XDG directories. The [complete reference sample](docs/reference/config.toml) lists every supported value and default context binding.
- Edit Unicode text directly in the terminal, with whole-grapheme movement and deletion, literal tabs, visible controls and scrolling that retains the intended display column. The status shows position, modification, read-only state and language mode.
- Open UTF-8 files at startup or with `C-x C-f`, preserving BOM, LF/CRLF and final-newline presence. Canonical paths reuse live buffers; absent paths start empty without creating files. Invalid UTF-8 and NUL-containing binary input reject without replacing live text.
- Complete find-file paths in a vertical list from the visiting file's directory or cwd. Tab fills selected or unique names and extends shared prefixes; Up/Down or C-p/C-n selects candidates. Return opens the explicit selection or freely typed filename, and directories descend within the prompt. Canonical aliases reuse unsaved buffers, new files remain in memory until save, and failed opens retain the editable draft and original buffer. Relative, absolute, home, space and Unicode paths retain their identities; incomplete lists show a warning, and resize or cancellation preserves the editing workspace.
- Save with `C-x C-s` or write another path with `C-x C-w`. Occupied destinations and external changes require fresh consent; symlink saves preserve the link and write its resolved target. Saves preserve access permissions and distinguish rejected writes from published files whose completion remains unresolved.
- Switch, list and close buffers with `C-x b`, `C-x C-b` and `C-x k`. Modified close asks for save/discard/cancel; a failed required save keeps the buffer open. Closing the final buffer leaves a fresh editing buffer.
- Create nested shared-text windows with `C-x 2` and `C-x 3`; cycle with `C-x o`, close a view with `C-x 0` or keep only it with `C-x 1`. Each view retains its own cursor and scrolling. Tiny/resized terminals retain text, layout and history while presentation adapts or suspends.
- Undo with `C-x u` or reported `C-/`/`C-_`, and redo with `M-x undo-redo`. Adjacent typing shares one undo step until another command or save; paste and compound edits undo whole. History remains per buffer across saves, restores retained clean states and maps shared views; a new edit discards redo.
- Select a linear region with `C-SPC`, extend it with motion, exchange point and mark with `C-x C-x`, or clear selection with `C-g`. Copy with `M-w`, kill with `C-w` and yank with `C-y`. Immediately after yank, `M-y` rotates entries within that exact insertion. The session retains 32 linear entries; directional line/word kills coalesce in document order.
- Select display-column rectangles with `C-x SPC`, copy with `C-x r M-w`, kill with `C-x r k` and yank with `C-x r y`. Whole graphemes and partial-tab spaces preserve adjacent text. Short source rows gain no absent trailing text; insertion adds only required padding and lines. Each block mutation undoes as one group, separate from the linear ring.
- Find commands with `M-x`, a vertical candidate list, configurable prefix/substring matching and row limits, Up/Down or C-p/C-n candidate selection, Tab completion and separate M-p/M-n session history. Return invokes the selected or exact typed command once. The highlighted candidate stays visible through navigation and resize without scrolling the document underneath. Named commands use the same handlers and focused arguments as bindings; `C-g` cancels without inserting prompt text.
- Search literally with `C-s`/`C-r`, using Unicode smart case and visible matches across lines. Repeat either chord to continue without implicit wrap; Return accepts and `C-g` restores the starting view. `M-%` confirms each query-replace occurrence with y/n/q, retaining one undo step per accepted change.
- Paste bracketed multiline Unicode literally as one undo step, preserving LF/CRLF and keeping command-looking text out of key decoding. Single-line argument paste stays in its prompt; decision prompts require explicit keys.
- Show/focus the left file tree with `C-x t t`/`C-x t f`. Navigate with arrows, expand lazily and refresh without restarting. Directory symlinks remain leaves; narrow terminals hide the pane and release its focus. Return opens files through normal buffer reuse.
- Create tree files/directories with `C-x t n`/`C-x t d` and rename with `C-x t R`, retaining visited text and reconciling descendant paths. Delete with `C-x t D`, confirming again for a nonempty directory. Open text survives deletion and requires explicit saving to recreate files; symlink removal retains referents.
- Highlight `.rs` files with bundled Rust syntax and Tokyo Night RGB syntax colors, without installing a grammar. Unicode edits, paste, rectangles and undo/redo keep colors aligned across shared views. Unsupported files and syntax failures stay editable as plain text.
- Exit with `C-x C-c` or `M-x save-buffers-kill-terminal`, deciding save/discard/cancel for each modified buffer, including hidden and unnamed text. Cancel retains the live session and earlier completed saves. Failed or unresolved required saves stop exit.
- Use reported Alt/Meta or a compatible Escape prefix with a 100-ms deadline. Standalone Escape clears pending key prefixes while preserving focused prompts; `C-g` cancels the active interaction. The [terminal input reference](docs/reference/terminal-input.md) documents supported reports and physical-terminal limits.

### Changed

- Each pane groups its buffer name, clean/modified state, access and position on the left and aligns its language mode to the right. Narrow panes omit or compact whole fields and restore the complete status when space returns.

- The terminal UI uses Tokyo Night RGB colors for document text, status, tree, separators, selections, search and messages. Supported terminals apply cursor color and restore their default colors on exit or failure.

- Editing panes and the file tree share continuous Unicode separators and connected junctions. Stacked panes reserve a separator row in addition to their status rows.

- Tab inserts spaces to the next four-column indentation stop by default. Pasted tab characters remain literal.

### Fixed

- Repeated vertical navigation stops promptly after input ends, including wide terminal surfaces; queued edits and commands retain their original order.

- Rectangle copy expands selected tabs into retained spaces without changing source text; rectangle kill preserves complete tabs. Yanking aligns row starts at the destination column, retaining whole destination graphemes and using the destination tab stops.
- Visiting a file through an alias retains its visible basename for the buffer label and language selection, while saves and repeated visits share the canonical destination.
- Terminal cleanup attempts every restoration step after an output error, reports failed steps together and still releases raw input.
- Saving rechecks the destination after staging temporary output, preserving changed, removed or replaced external data observed before replacement and requiring a new decision.
- Tree creation rejects a directory replaced while its name prompt was open, keeping entries out of the replacement directory.
- Canceling `M-x` retains the intended vertical column and recenter cycle. Argument edits clear outdated feedback; Tab in write/save prompts retains the draft and displays path-entry guidance.
