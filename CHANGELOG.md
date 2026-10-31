# Changelog

All notable changes to this project are documented here.

This project uses Semantic Versioning.

## Unreleased

### Added

- Name file positions with `C-x r m`, return with `C-x r b`, and remove names
  with `M-x bookmark-delete`. `C-x r M` prevents accidental replacement.
  Vertical name completion supports selection, Tab and cancellation. Bookmarks
  persist across sessions and follow edits, structural moves, undo, rename
  and save-as. Closed files reopen through checked context; ambiguous, removed
  or unavailable locations preserve the original view and source text. The
  collection saves on exit by default, with explicit save/reload and configurable
  timing and limits. Checked atomic writes preserve malformed or conflicting
  stores and offer retry, discard or cancellation when quit cannot save.
  `C-x r l` opens a read-only list with checked visits and staged deletion;
  configurable local keys navigate rows without editing source text. Use
  `M-x bookmark-next` or `bookmark-previous` to step through resolved locations
  in the current file, including distinct names at the same point. Gutter markers
  show resolved locations independently of number labels; wrapped continuations
  and hidden folded bodies stay blank. Configure the glyph and color, or toggle
  markers per view with `M-x bookmark-markers-mode`.
- Click a file-tree entry to select it and open its file or expand/collapse its
  directory. Set `mouse.tree_activate_on_click = false` for selection-only clicks.
  Blank tree space focuses the tree without activating an entry, and clicked
  files open in the editing pane you last selected.
- Open files or directories from startup and `C-x C-f`. Accepting a directory
  opens its expanded tree while preserving live documents. An explicit directory
  root stays selected when visiting files inside or outside it. The first tree
  display shows immediate entries; later hide/show preserves deliberate collapse
  and selection.
- Long source lines wrap at word boundaries by default, preserving file bytes,
  tabs and Unicode graphemes at each split pane's width. Use
  `editor.soft_wrap = false` for horizontal scrolling or `M-x visual-line-mode`
  to toggle wrapping for one buffer and all its views. While wrapped, Up/Down,
  `C-n`/`C-p`, `C-a`/`C-e` and `C-k` follow Emacs visual-line editing; paging and
  recentering follow display rows while goto-line keeps source line numbers.
  Pointer selection, wheel/drag scrolling and search highlights follow the same
  wrapped rows; rectangles and line numbers retain logical source coordinates.
  Org folds, table fields, source drafts and image anchors keep their source
  targets as visible text reflows.

### Fixed

- In Find File, Tab fills the highlighted path, including the first suggestion
  when names share a prefix. Completing a directory keeps the prompt open for
  its children; completing a file waits for Return before opening it.

## 0.1.0 - 2026-10-30

The initial terminal editor includes everyday editing and native Org support.
Start with the [User Guide](docs/tutorials/user-guide/index.md), or use the
independent [Org guide](docs/tutorials/user-guide/org/index.md) for notes and planning.

### Added

- Edit Unicode text with whole-grapheme movement, literal tabs, visible controls
  and scrolling that retains the intended display column. Open UTF-8 files,
  preserving BOM, LF/CRLF and final-newline details; new files reach disk on save.
  Saves ask before replacing occupied or externally changed destinations and
  distinguish rejected writes from files whose save completion is unresolved.
- Keep multiple buffers and split panes, with independent cursors, scrolling and
  Org folds over shared text. Buffer close and quit offer save/discard/cancel;
  failed required saves keep the text available.
- Undo and redo typing, paste and compound edits across saves. Select linear
  regions or display-column rectangles, copy or kill text, and yank retained
  entries. Linear kill-ring rotation and rectangular text have separate storage.
- Find commands through M-x completion and session history. Search literally
  with Unicode smart case, reveal matches and confirm replacements individually.
  Bracketed paste inserts text as one edit without interpreting it as commands.
- Complete file paths in a vertical candidate list and browse the optional file
  tree. Create, rename and delete files or directories with explicit prompts;
  canonical aliases reuse live buffers. The tree starts hidden and opens at the
  startup file's parent, or the working directory without a file argument.
- Set startup TOML preferences for editing, display, language rules, history,
  keymaps and Org. First startup creates the complete default file when absent;
  existing entries remain untouched. Includes and overrides compose before use,
  and invalid configuration retains built-in defaults with a diagnostic.
  Changes take effect after restart. See [core settings](docs/reference/defaults.md#startup-configuration)
  and [Org settings](docs/reference/org/configuration.md).
- Highlight Rust and native Org source, show absolute or relative line numbers
  per pane, and navigate files with the mouse. Click to focus or place point,
  scroll a pointed pane or tree, and drag a region with bounded autoscroll.
  Mouse support can be disabled or toggled for the session.
- Follow a six-chapter [editor learning path](docs/tutorials/user-guide/index.md)
  without entering Org. The separate Org guide, task guides and references cover
  the built-in planning and document features.
- Write Org outlines, move or fold complete subtrees, manage nested lists and
  checkboxes, and edit task states, tags, properties, priorities and effort.
  Timestamps, schedules, deadlines and repeating tasks retain their source data
  and ordinary undo. Footnotes, emphasis, entities and affiliated metadata remain
  editable plain text. See the [Org reference](docs/reference/org/index.md).
- Browse day, week, month, year and custom-span agendas from selected local files
  or open buffers, with unsaved text taking precedence. Query TODO states or
  tags/properties, configure named views, filter and sort entries, inspect habit
  graphs, and apply checked individual or bulk source actions.
- Capture tasks in an editable draft, then finalize or abort explicitly. Refile,
  copy and archive subtrees into chosen destinations, with per-file save results
  and retained drafts after failure. Store, insert and follow Org links, return
  to recorded source positions, and manage heading attachments.
- Record task clocks, compare time with effort and update clock tables or
  property-column reports. Optional clock recovery preserves checked restart
  state. Relative timers support timed notes; explicit countdowns show an
  in-editor expiry notice and stop when the session ends.
- Create and align Org tables, edit rows and columns, sort data, calculate native
  numeric formulas and import/export CSV or TSV. Table, field and column
  recalculation use ordinary undo; failed or unsupported formulas retain source.
  See [tables and formulas](docs/reference/org-tables.md) for the supported language.
- Edit source blocks in separate drafts and tangle them into explicit code files.
  Optional execution uses configured language programs, confirmation and bounded
  jobs; inspect output or update checked results without losing prior content on
  failure. Editing and tangling do not execute code. See [source blocks](docs/reference/org-source-blocks.md).
- Export documents, subtrees or regions to ASCII/Unicode text, Markdown, Org,
  HTML, LaTeX, Beamer, OpenDocument text, Texinfo, man or iCalendar. Optional PDF
  compilation uses a configured tool. Publish named projects with native exports
  and asset copies, inspect per-file outcomes and confirm optional upload
  separately. See [exports and publishing](docs/reference/org-export.md).
- Import declared RSS or Atom feeds on demand into Org inboxes, retaining stable
  identities to avoid duplicate entries. Local feeds work offline; optional fetch
  programs require confirmation. Failed imports preserve prior destination text.
- Preview local Org image links without changing source or history. Direct Kitty
  terminals can display PNGs; other terminals and resources retain readable
  links and the separately confirmed configured viewer path.

### Changed

- The terminal UI uses Tokyo Night RGB colors for text, syntax, status, tree,
  selections, search and messages. NO_COLOR retains readable selection and search
  feedback. Supported terminals apply cursor color and restore defaults on exit.
- Panes use connected Unicode separators and compact status fields: buffer name,
  modification, access and position on the left, language on the right. Narrow
  panes omit whole fields and restore them when space returns.
- Tab inserts spaces to the next four-column indentation stop by default.
  Pasted tab characters remain literal; explicit tabify/untabify commands convert
  a selected region without changing its display columns or file line endings.

### Fixed

- Repeated vertical navigation stops promptly after input ends, including wide
  terminal surfaces, while queued edits and commands retain their order.
- Rectangle copy expands selected tabs into retained spaces without editing
  source. Kill and yank retain their distinct tab and destination-column rules.
- Visiting a file through an alias retains its visible basename and language
  identity while repeated visits and saves share the canonical destination.
- Saving rechecks the destination after staging output, preserving newly changed,
  removed or replaced external data and requiring a new decision.
- Tree creation rejects a directory replaced while its name prompt was open.
- Terminal cleanup continues after an output error, reports failed restoration
  steps together and still releases raw input.
- Canceling M-x retains the intended vertical column and recenter cycle.
  Argument edits clear stale feedback; Tab in write/save prompts retains the draft.
- Org agenda bulk commands invoked from source report that an agenda is required
  and preserve subtree visibility.
