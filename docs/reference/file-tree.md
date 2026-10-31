# File tree

`M-x toggle-file-tree` or `C-x t t` shows or hides the left pane. It starts
hidden. First display expands the root's immediate entries and focuses the
retained selection. Child directories stay collapsed. Later hide/show preserves
deliberate collapse and selection.
The focused selection uses its palette colors, or reverse video with `NO_COLOR`. The footer shows `Tree *` while the tree owns workspace input.
The editor's selected window, text, point, mark and undo history remain live.
The tree and editing panes share Unicode separators and connected junctions;
content and footer stay inside reserved cells alongside nested splits.

Startup and `C-x C-f` accept a file or directory. An explicit directory opens
its tree even when startup visibility is false, without allocating file text or
changing launch cwd. Its root survives files opened inside or outside it until
another explicit directory replaces it. Without that choice, automatic roots
follow the visiting file's resolved parent; explicit `tree.root = "cwd"` retains
launch cwd. A session without a file uses its chosen directory or launch cwd.
Drawing and visible movement perform no filesystem reads or Git discovery.

## Keyboard focus

- `C-x t f` or `M-x focus-file-tree` shows and focuses the pane.
- `C-x C-f` opens the file/directory prompt while the tree retains workspace
  focus. Canceling the prompt returns to that tree selection.
- Tab or `C-g` in the tree returns to the retained editing window, leaving the
  pane visible. Editing keys and bracketed paste cannot change document text
  while the tree owns workspace focus.
- M-x uses its normal focused command prompt. Canceling or entering an unknown
  name returns to the tree. Named tree commands use the same handlers as keys.
- `other-window` cycles only editing leaves. The tree never enters that order;
  return to the editor before cycling. Guarded session exit remains available.

## Traversal and refresh

Up/Down (or C-p/C-n) move through visible entries without wrapping. Right
(or C-f) expands a directory; Left (or C-b) collapses an expanded directory
or selects the visible parent. Enter or `M-x tree-open` toggles a directory or opens a file through the same
canonical pipeline as `find-file`. Existing visits reuse their live buffers,
including unsaved text. Successful open returns to the retained editing window;
failed open preserves tree focus and existing text. Symlinks remain leaves;
opening a file link resolves its target through the normal file pipeline,
while opening a directory link reports the regular-file requirement.
These actions preserve editor text, point and history.

The corresponding names are `tree-previous-line`, `tree-next-line`,
`tree-expand`, `tree-collapse` and `tree-toggle-directory`. Tree navigation
requires tree focus. M-x names and bound keys use the same handlers.

Directory markers are `>` for collapsed and `v` for expanded. Symlinks use
`@`. Children sort directories first, then by the native filename order;
comparison is case-sensitive and independent of locale. Dot entries are shown.
Each expansion reads only that directory's direct children. Collapsing removes
its loaded descendants; expanding again observes fresh children. There is no
recursive scan or implicit symlink traversal.

Each directory observation processes at most 4096 entries and 512 KiB of raw
name bytes, including names filtered by hidden-file preferences. Reaching an
exact bound marks the retained subset incomplete without another read. The
entry label shows `[incomplete]` and the action reports
`Incomplete tree listing: directory observation limit reached`. An incomplete
listing is not an empty or complete observation. Failed initialization retains
the previous root, source and usable tree state.

Executable startup and supplied library snapshots apply `tree.visible`,
`width`, `max_fraction`, `root`, `show_hidden` and `directories_first`. Initial
visibility preserves editing focus. Width is the lesser of the preferred
columns and maximum frame share; frames narrower than 32 columns still hide
the tree temporarily. Widening restores its focus intent unless editing or
explicit cancellation has taken over. Explicit directory targets take precedence
over `root` policy. Initial visible presentation and a new visible root observe
immediate children once. Filtering and sorting apply during initialization,
expansion or refresh, with native case-sensitive filename ordering and symlinks
as leaves.
Startup reads these preferences once; source edits take effect after restarting.

`C-x t r` or `M-x tree-refresh` reloads the root and previously expanded
branches. New directories stay collapsed. Selection follows its retained path;
if that entry disappears, the nearest visible ancestor becomes selected.
Read/inspection errors identify their operation and path in the echo area.
An expansion failure keeps the existing list. Refresh collapses a failing
branch while retaining successful observations elsewhere; a missing root
remains selected and collapsed so refresh can recover when it returns.
The pane scrolls to keep the selected entry above its footer, including after
resize. Refresh and expansion run only on explicit commands and require work
proportional to the direct entries of the requested open branches.

## Creating entries

Select a directory, then use `C-x t n` or `M-x tree-create-file` for an empty
file, or `C-x t d` or `M-x tree-create-directory` for a directory. `C-x t o`
also invokes `tree-open`. The name prompt retains its selected directory until
acceptance; C-g cancels without writing. Tab keeps the draft and shows a hint.
Creation requires tree focus and a directory selection, including a collapsed
directory. File and symlink selections cannot redirect the target implicitly.

Names must be one nonempty filename component, with no controls, dot/parent
components, absolute path or path separators. Unicode and spaces are allowed;
a name containing only whitespace is refused. An existing file, directory or
symlink, including a dangling link, is never replaced. The parent is revalidated
at publication; exclusive file creation and directory creation also reject a
conflict that appears after validation.

Successful creation retains workspace focus, expands the parent and selects the new
entry. It does not open a buffer; Enter opens a new file explicitly. An error
reports its operation/path without discarding editor text. If creation succeeds
but observation fails, the echo says `Created` and identifies the refresh error
so a published entry is not reported as an unperformed write.

Opens and published saves refresh already loaded parent directories. Collapsed
branches remain lazy. Refresh failures retain the successful open/save outcome
and report the separate tree error. New entries appear without restarting, and
tree selection continues to follow paths rather than buffer display names.

A pending creation request retains the selected directory's device and inode.
Before creating a file or child directory, it rechecks that identity as well as
the entry type. Replacing the directory at the same path cancels creation;
retry from a refreshed selection to authorize the new directory.

## Renaming entries

`C-x t R` (uppercase R) or `M-x tree-rename` prompts for one new name at the
selected entry's current parent. The echo identifies the source. The same
single-component name policy as creation applies. C-g cancels without changing
disk or visiting associations. Existing destination files, directories and
symlinks are refused, including dangling links. A destination already reserved
by a different live buffer is also refused; directory targets check live
reservations under the entire target prefix. The source identity and target
conflict are checked again immediately before the filesystem rename.

Only a successful rename changes visiting paths and the canonical path index.
Renaming a directory maps visited descendants, including intended missing
files, by path components. Text, dirty/saved states, revision, mark, history
and editing windows remain live. Renaming a file updates its display name,
keeping labels distinct from other live buffers. Directory descendants retain
their existing distinct labels. Later visits reuse these owners, and saves
write their new destinations.

Renaming a symlink moves the link entry; the canonical visiting buffer of its
referent keeps its path. No recursive traversal through a directory link is
introduced. Loaded tree paths, expanded branches and selection follow the new
entry, including when the tree root itself is renamed, before ordinary refresh.

Rename preserves existing external-change protection: an observation is
updated only when the prior and current identity/content/permissions agree,
allowing rename's expected ctime change. A pre-existing or concurrently observed
content change still requires the normal later-save decision. If post-rename
observation or tree refresh fails, the echo reports `Renamed` and the separate
warning; successful disk publication and new associations remain recorded.

## Deleting entries

`C-x t D` (uppercase D) or `M-x tree-delete` asks whether to delete the selected
entry, identifying its path. Answer y or n; C-g cancels. A nonempty directory
requires a second, separate question stating that it contains children and
asking for recursive deletion. Canceling either question leaves disk unchanged.
Repeated keys and bracketed paste cannot approve a decision. Source identity
is checked again before mutation. An empty directory never gains recursive
permission merely because children appear after its initial decision.

Deletion removes symlink entries without following file or directory referents,
including dangling links. Open visiting buffers keep their exact text, point,
mark, revision and history. Associations whose paths are actually absent are
marked missing, with an explicit-save warning and a modified status so closing
or quitting cannot silently discard retained text. Their canonical reservations
remain: revisiting an absent path reuses the live buffer without recreating it.
Only an explicit save recreates a file; missing parents are not created. Old
save captures cannot recreate a path from before its association changed.

Recursive removal can fail after deleting some children. The echo reports the
actual path/error and says deletion may be partial. Each affected visiting path
is inspected; only confirmed absence changes its association. Permission or
observation errors do not imply absence. The tree refreshes after successful or
failed deletion, retaining usable branches and ancestor selection. A refresh
error is reported separately. No filesystem rollback or all-or-nothing deletion
is claimed; all in-memory ropes survive either outcome.

## Geometry

The pane uses at most 24 columns and at most one third of the frame, plus a
single divider column. Editing panes divide the remaining width; status,
selection, search and cursor coordinates use that editing area. The session
keeps a full-width echo/prompt row.

Below 32 columns, or without usable text/status geometry, the pane hides and
releases tree focus. The show preference and root remain. Widening restores
presentation with editor focus. Explicitly hiding the tree restores full
editing width. Unicode names and control text follow the normal safe drawing
and clipping policy.
