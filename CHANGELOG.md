# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Choose how new Org views open with `org.startup_folded` or document STARTUP
  metadata. Keep ordinary text visible with drawers hidden using `showall`,
  choose heading-only states, or retain the fully visible `showeverything`
  default. Hide, show or toggle all drawers in the selected pane, and restore
  its startup policy explicitly without changing source or other panes.

- File-tree selection stays visible after opening a file. By default, the tree
  follows the active editing file across buffers and panes, scrolling loaded
  matches into view while preserving collapsed folders and chosen roots.
  Set `tree.follow_active_buffer = false` to retain manual selection instead.
- Every built-in command shows a concise description in M-x and modal colon
  lists, including commands without shortcuts and context-specific actions.
  Descriptions clip to the available width; Tab still completes only the name.
- [Work with files, Projects and Workspaces](docs/tutorials/user-guide/20-projects-and-workspaces.md).
  Start `rev` with four native opening actions, or pass a file/directory directly.
  Open… accepts either a file or a directory through the same path selector.
  Keep ordinary directory work temporary, promote it to a named Project while
  retaining live edits, or collect independent roots in a Workspace. Return by
  name to restore files, loose files and source positions after normal
  save/discard/cancel decisions. Missing files and roots retain their metadata.
  Rename or forget registrations without deleting sources, reconnect moved roots,
  and convert or duplicate Projects as independent Workspace sessions. Root
  removal resolves only files losing coverage. `dashboard-open` retains current
  work behind the startup screen; `context-close` resolves it and returns there.
  The screen uses a compact logo, shares the text-buffer background, blends the
  logo's dark edges into it and separates the logo from its menu with more space. It fits small
  and color-free terminals. Context commands use M-x or
  modal colon with existing bindings unchanged.

### Fixed

- Enter runs the highlighted command in M-x and modal colon even when the query
  is empty. Repeat the latest named command directly without completing its name
  with Tab first.

## 0.6.0 - 2026-11-06

### Added

- M-x and modal colon command lists show one effective shortcut beside each
  bound command, using the editing state and input context where the list opened.
  Configured bindings take effect; unbound commands remain available by name.
  Names and bracketed shortcuts use distinct colors in adjacent left-aligned
  columns, with space for command descriptions. Missing descriptions stay blank;
  complete hints disappear when they cannot fit beside the full name.
- [Org heading bodies and wrapped continuations](docs/tutorials/user-guide/06-org-notes.md#align-heading-bodies)
  align with their enclosing heading by default. Set `org.indent=false` for a
  flat outline with heading markers at the left margin and no added body spacing.
  Use `org-indent-mode`, `org-indent-enable` or `org-indent-disable` to change
  one buffer and all its views without writing configuration. The choice survives
  edits and reload, and resets when the buffer closes. Visual spacing leaves
  source whitespace, copied text and saved bytes intact.
- [Work with Git inside rev](docs/tutorials/user-guide/git/index.md): inspect
  saved changes with C-x g, expand staged/unstaged diffs, stage exact paths,
  hunks or compatible marked groups, and compose normal, amend or fixup commits.
  Colors distinguish status groups, added and removed lines, and hunk headers;
  selection preserves those colors. Each tone is configurable under `palette.git`.
  Explicit submission retains Git hooks and signing policy; cancelled drafts
  remain available without saving or staging source text automatically.
- [Explore history and refs](docs/reference/git.md): browse bounded ancestry,
  revision/range detail, reflog and blame; manage local branches and lightweight
  or annotated tags. Fetch updates tracking facts separately from integration,
  and push reviews one exact branch and its configured upstream without force.
- [Integrate changes deliberately](docs/how-to/integrate-git-changes.md): choose
  a merge policy, rebase or reviewed autosquash; apply ordered cherry-pick/revert
  sets or run manual bisect. Inspect actual conflict sides, edit saved results,
  review side replacement or deletion, stage and continue separately. Optional
  AI explanation leaves manual resolution available.
- [Manage repository work](docs/reference/commands.md#native-git-status): inspect
  and review stashes and linked worktrees while protecting unsaved sources and
  retained sessions. Return from Git to the same editing layout and terminals;
  inspect active/recent processes, cancel explicitly and confirm application
  exit while Git work remains owned. Failed refreshes retain confirmed facts
  with explicit freshness, truncation and unavailable states.
- [Prepare repository commands](docs/how-to/prepare-git-commands.md): clone/init,
  edit ignore files, manage submodules, subtree, notes and cone sparse checkout;
  save checked patches, mailboxes or request text to new local files and apply
  local patches/mailboxes with actual result checks. Explicit Git argv and a
  separately selected shell route review executable, arguments and directory;
  arbitrary effects stay unknown even after exit zero.

### Changed

- [M-x and modal colon](docs/reference/commands.md#matching-and-invocation)
  show up to four recent commands, then four other frequent commands, before
  the alphabetical remainder. Ranking remembers the last 256 named invocations
  across editor starts, preserves text-match priority and keeps the selected
  command stable during resize. Bindings and canceled drafts do not affect usage;
  unavailable history retains session ranking without blocking editing or exit.
- [The file tree](docs/reference/file-tree.md#presentation) uses ordinary symbols
  and semantic colors by default to distinguish roots, directories, source files,
  documents, configuration files and symlinks. Set `gutter.nerd_fonts = true` for
  the fixed Nerd Fonts icons, or `tree.decorated = false` to retain minimal labels.
  Both icon sets keep names aligned and clip Unicode labels safely in narrow panes.
  Customize entry colors under `palette.tree_entries`; tree visibility,
  navigation and focused selection keep their existing behavior.
- [Bookmark markers](docs/reference/gutter.md#bookmark-markers) appear before
  left-side line numbers and after right-side numbers, keeping the separator
  beside the text. The default is a blue square; set `gutter.nerd_fonts = true`
  for the built-in flat Nerd Fonts bookmark. Older custom marker strings are
  ignored, while explicit marker colors remain effective. Markers remain visible
  when numbers are hidden and yield first when the pane is too narrow.

### Fixed

- Org capture rejects malformed code blocks interrupted by unescaped headings.
  Comma-escaped headings remain literal code.

## 0.5.0 - 2026-11-04

### Added

- [Run shell commands and text filters](docs/tutorials/user-guide/15-shell-commands.md)
  from M-x or modal colon. Send selected text to tools such as sort or tr,
  inspect their output, or apply it in one undo step. Non-modal editing also
  provides M-!, M-\| and their C-u insert/filter forms. Keep editing while a
  command runs; cancellation, failures and stale results preserve source text.

- [Use integrated terminals](docs/tutorials/user-guide/16-integrated-terminal.md)
  for interactive shells and tools such as lazygit. Open them in the lower panel
  or an editing pane, move a running session between both, and keep several
  sessions available. C-g returns to source; C-c C-t opens output copy mode and
  C-y pastes. Shell exit closes its terminal buffer. Closing live sessions or
  quitting asks before terminating their jobs. See the
  [terminal reference](docs/reference/integrated-terminal.md) for startup settings,
  keyboard and mouse behavior, and retained-output navigation.

### Changed

- C-x b shows a filterable buffer list immediately, including terminals in the
  lower panel and editing panes. Use arrows and Enter to select, or Tab to
  complete a name; selecting a terminal preserves its session and placement.

### Fixed

- Keyboard input remains responsive when a terminal resize and queued keys
  arrive together.

## 0.4.0 - 2026-11-03

### Added

- [Display Org heading markers](docs/tutorials/user-guide/06-org-notes.md#inspect-heading-stars)
  by level while preserving the saved stars, cursor positions and wrapping.
  Configure markers and deeper-level repetition under `org.headings`. Use
  `org-pretty-headings-mode` to switch one pane between markers and source stars,
  or `org-reset-heading-presentation` to restore its startup choice. Both commands
  work through M-x or modal colon, including in read-only Org results.

- [Edit with optional Pi 1.3.0](docs/tutorials/user-guide/14-ai-editing.md) through
  `ai-explain`, `ai-prompt` and `ai-implement`. Continue editing while a request
  runs, then inspect its read-only Org result with the answer first and request
  details in a folded drawer. Apply a proposal to its original checked target
  and undo it normally. Choose a model, allow bounded directory discovery,
  retain private directory history or use input-only context. Save a reusable
  instruction with `ai-save-command` and invoke it by name or a configured key.

- [Reload a file](docs/tutorials/user-guide/13-file-reload.md) with
  `M-x revert-buffer` or `:revert-buffer`. Confirm before discarding unsaved text,
  retain pane positions and prior undo history, and undo the reload normally.

- [Set up optional language servers](docs/tutorials/user-guide/12-language-servers.md)
  through native prompts. Reuse an installed server, register another language,
  or install a supported server with optional tools on Linux x86-64. Select
  semantic or native-only completion per language, inspect server readiness,
  and manage verified updates and removal in either editing profile.

### Changed

- Clean visiting files automatically refresh after external writes or atomic
  replacement, including hidden and read-only buffers. Set
  `files.external_changes = "ask"` to keep the text and receive a reload notice.
  Unsaved edits remain protected when an external change conflicts with them.

## 0.3.0 - 2026-11-01

### Added

- [Select non-modal or modal editing at startup](docs/tutorials/user-guide/11-modal-editing.md).
  The default `non-modal` profile keeps Emacs-like direct text entry; `modal`
  provides Vim-like editing, starting in Normal with explicit Insert entry,
  logical line opening, state feedback and shared undo. Change
  `editor.interaction_profile` and restart to select a profile.

- Navigate in modal Normal with grapheme and logical-line motions, checked counts,
  word/WORD and character targets, page controls and viewport alignment. Use
  `/` or `?` for literal search, `n`/`N` to continue and `*`/`#` for a whole word.

- Compose modal delete/change/copy and indentation with counted motions or text
  objects. Share retained text and linewise paste with native yank, replace whole
  graphemes, join logical lines and undo each operation through ordinary history.

- Select text in modal editing with Character, Line and Block Visual states. Extend or
  exchange the active end, copy/delete/change the selection, and undo multi-line
  edits in one step. Visual blocks count whole graphemes and preserve tabs;
  Non-modal rectangles keep their display-cell edges.

- Revisit checked file and source locations with C-o/C-i in modal Normal. Each pane
  keeps its own bounded jump history, including locations rebased through edits
  and undo. In Insert, C-o runs one complete Normal command and resumes insertion
  when the editing context remains compatible.

- Overwrite whole graphemes with R in modal Replace, then return to Normal with
  Escape. Repeat the latest supported edit with dot at a new target, override its
  count, and undo each repetition in one step. Insert and Replace sessions share
  ordinary undo groups; oversized changes keep their full text and undo history
  while remaining unavailable for repetition.

- Open the shared command selector with colon in modal Normal or Visual, use
  canonical names and ordinary argument prompts, and manage panes with C-w.
  Native and language-server completion works in Insert and Replace, preserving
  popup controls, syntax colors, atomic imports and undo. Cancelling command
  entry retains a compatible Visual selection; stale replies cannot reopen it.
  Escape cancels the innermost prompt or menu before returning to Normal. It
  preserves unsaved text when canceling a save/quit decision; C-g remains available.

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
