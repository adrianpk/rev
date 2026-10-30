# Org files

## Refile and copy

In an Org buffer, `C-c C-w` (`org-refile`) moves the current complete subtree.
An active linear region must contain complete sibling subtrees. `C-c M-w`
(`org-refile-copy`) keeps the source. Copies can retain existing ID properties;
choose destinations deliberately when those IDs need to remain unique.

Set `org.refile.targets` to explicit local file/directory roots. Relative paths
use the configuration file that assigned the array. Arrays replace as one leaf;
`[]` restores admitted agenda/source headings, including open Org buffers and
explicit agenda roots. Directories admit immediate `.org` children only.
Configuration loading never visits or creates targets.

The target prompt shows numbered file/line/heading labels. Enter a displayed
number or a complete unambiguous label. TAB filters labels using the current
prompt text and shows up to eight matches. Duplicate titles retain their exact
source locations; a partial or ambiguous label never selects the first match.
C-g cancels without editing either participant.

The selected entry becomes the last child of the destination heading. Relative
heading levels, planning/properties, nested source and literal bodies remain
coherent. New separators use the destination format, including BOM/CRLF for an
unopened file. Moving into the selected subtree or its descendants is rejected.
Missing, malformed, stale, read-only or externally replaced participants reject
before either canonical text changes.

## Archive

`C-c C-x a` toggles the current heading's local `ARCHIVE` tag without
removing its source. Setting it hides the subtree; normal visibility cycling
keeps archived trees closed. `org-show-subtree` and `org-show-all` reveal them
explicitly. `C-c C-x A` moves the complete entry under the same
parent's `Archive :ARCHIVE:` sibling, creating that sibling when absent.
Duplicate archive siblings reject rather than choosing an arbitrary owner.

`C-c C-x C-a` moves the current subtree to `org.archive.location`, initially
`%s_archive::`. `%s` expands to the associated source file's canonical path;
relative literal paths use the source directory. `file::` appends a level-one
entry; `file::* Heading` appends below an exact unique heading and creates it
when absent. The nearest inherited `ARCHIVE` property overrides the native
`#+ARCHIVE:` keyword, which overrides configuration. Unsupported escapes and
heading forms reject explicitly. Associate the origin with a file before
external archiving. The archive file can be missing, but its parent directory
must already exist and be writable. Archiving never creates directories.

External moves preserve the subtree's task, planning, properties, descendants
and literal body. The root receives `ARCHIVE_TIME`, `ARCHIVE_FILE`,
`ARCHIVE_OLPATH`, `ARCHIVE_CATEGORY`, `ARCHIVE_TODO` when it is a task, and
`ARCHIVE_ITAGS` for inherited tags. The origin file and outline path remain
inspectable source properties. Existing IDs and other metadata remain intact.
Archive commands target one current subtree; cancel an active region first.
`org.agenda.include_archived` controls tagged-tree inclusion; external archive
files remain available through explicit source selection, with native Org
recognition for `.org_archive`.

Archive and refile share coherent canonical publication and independently
acknowledged file saves. `org-archive-results` exposes the last move result.
A rejected target preserves both texts; a failed later Save retains the
canonical archive buffer and reports that file's actual outcome.

## Publication and saving

Refiling changes canonical buffers; it does not save files. Both sides pass
revision, point, viewport, read-only and history admission before either side
publishes. Shared views map through the same native edit path. Each changed
buffer gets one ordinary undo group; same-buffer refiling uses one group.
Undo in one buffer does not claim to undo another buffer or an earlier save.

Use ordinary `C-x C-s` to save each associated buffer. A destination opened by
refile remains in the normal buffer inventory, with its modified state and undo
history. `org-refile-results` reports the last operation's source and destination
as unchanged, changed in buffer, saved, save rejected or completion unresolved.
Later edits or buffer closure invalidate that participant's pinned result.
A save captured before the move cannot acknowledge the later refile state.

File saves remain independent. If source save succeeds and destination save
fails, the destination buffer retains the moved text; the result never claims
atomic disk success. A post-replacement failure records actual publication with
unresolved completion. Saving again uses that same canonical buffer, without
running the move a second time. Ordinary save conflict/permission checks and
dirty-buffer close/quit guards continue to apply.

## Bounds

Refile uses the existing single cancellable Org worker, source byte/file and job
time limits. Native structure is bounded to 8 MiB, 200000 nodes and 256 levels;
selection retains at most `min(org.limits.agenda_rows, 4096)` headings and 1 MiB
of labels. Archive metadata values are limited to 4096 bytes each and capture
placement admits at most 2 MiB of insertion text. No worker or polling deadline remains active after work completes.

Reference: [Org refile and copy](https://orgmode.org/manual/Refile-and-Copy.html).

Archive references: [Moving subtrees](https://orgmode.org/manual/Moving-subtrees.html), [Internal archiving](https://orgmode.org/manual/Internal-archiving.html).

## Link and image previews

`C-c C-x C-v` (`org-link-preview`) explicitly observes description-free external
links in the active region, at point, or in the current section. `org-link-preview-region`
uses the active region, or the buffer. `C-c C-x C-M-v` (`org-link-preview-refresh`)
refreshes an enabled view across the region/buffer; `org-link-preview-clear`
removes that view's previews and disables them. Splits copy the current preference
and observation; subsequent clear/refresh operations remain independent.

`org.preview.enabled` defaults to false and supplies the initial view preference.
A preview command enables it; refresh of a disabled view diagnoses the enabling
command. Even with the preference enabled, visiting, highlighting and redraws
never read image files, fetch remote content or launch viewers.
`org.preview.max_bytes` defaults to 4194304 (1..16777216) per local resource.
Selections keep at most 32 resources and 16 MiB of encoded image data. Static PNG
containers admit 4096 chunks, 8192 pixels per axis and 16,777,216 pixels per
selection. CRC, truncation, animation and unsupported critical-chunk failures
retain ordinary readable source. Pixel decompression belongs to the terminal.

Only a direct `TERM=xterm-kitty`, outside tmux/screen, admits native PNG output.
There is no capability query, implicit fetch or image conversion. Source remains
readable in other terminals, with `org-open-at-point` as the explicit configured
viewer path. For other local file kinds, an explicit `file+sys:` link selects
that viewer instead of a native text visit. `org.tools.viewer` uses the existing confirmation, checked resource,
process-group, output/time ceiling and child cleanup owner. Missing tools are a
usable diagnostic. A preview command never grants viewer authority.

Graphics are a disposable layer behind existing text. They add no source rows or
coordinates and change no text, mark, point, gutter, search, mouse mapping or
history. Placements clip to the owning text pane (at most 40 columns and 12 rows)
and exclude statuses, prompts, gutters and neighboring panes. Hidden, off-screen
and horizontally scrolled anchors have no placement. Each frame caps 32 placements,
16 MiB encoded bytes and 16,777,216 pixels. Edits invalidate the retained source
revision; scroll, resize, folds and view changes recompute geometry without IO.
Clear/removal, normal exit, unwind and output failure retain cleanup obligations
for only the terminal owner's image IDs. Geometry changes reuse transferred data.

References: [Org previews](https://orgmode.org/manual/Images-and-link-previews.html),
[Kitty graphics](https://sw.kovidgoyal.net/kitty/graphics-protocol/).
