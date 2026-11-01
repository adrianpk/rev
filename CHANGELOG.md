# Changelog

All notable changes to this project are documented here.

This project uses Semantic Versioning.

## Unreleased

### Added

- Select the Emacs or Vim editing profile at startup. Emacs keeps direct text
  entry; Vim starts in Normal with explicit Insert entry, logical line opening,
  state feedback and shared undo. Change `editor.interaction_profile` and restart
  to select a profile.

- Navigate in Vim Normal with grapheme and logical-line motions, checked counts,
  word/WORD and character targets, page controls and viewport alignment. Use
  `/` or `?` for literal search, `n`/`N` to continue and `*`/`#` for a whole word.

- Compose Vim delete/change/copy and indentation with counted motions or text
  objects. Share retained text and linewise paste with native yank, replace whole
  graphemes, join logical lines and undo each operation through ordinary history.

- Select text in Vim with Character, Line and Block Visual states. Extend or
  exchange the active end, copy/delete/change the selection, and undo multi-line
  edits in one step. Visual blocks count whole graphemes and preserve tabs;
  Emacs rectangles keep their display-cell edges.

## 0.2.0 - 2026-11-01

### Added

- [Native completion](docs/tutorials/user-guide/10-complete-names.md) and syntax
  coloring for Rust, C, Go, JavaScript, TypeScript/TSX, HTML, CSS, JSON and TOML.
  Suggestions use unsaved declarations in the current and same-language open
  buffers, plus HTML/CSS vocabulary. Org suggests task states, tags, properties,
  directives and link targets in their structural contexts. Type to request a
  list, or use `M-TAB` / `M-x completion-at-point`. Choose with arrows or C-p/C-n,
  accept with Tab/Return and undo in one step. Colored indicators identify
  candidate kinds without an additional font package. Automatic requests can be
  configured or toggled with `completion-auto-mode`; comments, strings and pasted
  text stay quiet by default.
- Optional [language-server completion](docs/how-to/use-language-server.md)
  shares the native popup. Enable `M-x lsp-mode` to use an installed server;
  Rust defaults to `rust-analyzer`. Suggestions follow unsaved text and can apply
  checked same-document imports with the name as one undo operation. Empty or
  unavailable results use native fallback. Exclude or restore a buffer with
  `lsp-disconnect` / `lsp-connect`, inspect `lsp-status`, or recover a failed
  connection with `lsp-restart`. LSP remains disabled by default.
- [Register precompiled grammars](docs/how-to/register-grammar.md) and matching
  highlight/completion queries for other languages or replacement syntax.
  Failed assets fall back to bundled or plain presentation.
- [Name file positions](docs/how-to/use-bookmarks.md) with `C-x r m`, return
  with `C-x r b`, list with `C-x r l` and remove with `M-x bookmark-delete`.
  Bookmarks follow edits, undo, rename and save-as, and persist on ordinary quit.
  Reopened locations use checked source context; ambiguous or unavailable
  destinations leave the current view intact. The list supports staged deletion,
  and gutter markers show resolved positions. Use `bookmark-next` /
  `bookmark-previous` within a file or toggle markers per pane with
  `bookmark-markers-mode`.

### Changed

- Long source lines wrap at word boundaries by default without changing file
  bytes. Movement and `C-k` follow visible rows; line numbers and goto-line keep
  source coordinates. Each split reflows independently. Toggle a buffer with
  `M-x visual-line-mode`, or set `editor.soft_wrap = false` at startup.
- Startup and `C-x C-f` accept files or directories. A chosen directory opens
  its tree with immediate entries shown and remains the root across file visits.
  Later hide/show preserves deliberate collapse and selection.
- Clicking a tree entry opens its file in the last selected editing pane or
  expands/collapses its directory. Set `mouse.tree_activate_on_click = false`
  for selection-only clicks.

### Fixed

- Syntax colors stay visible while completion waits or analyzes source, including
  when typing retires an older request.
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
