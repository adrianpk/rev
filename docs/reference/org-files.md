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
of labels. No worker or polling deadline remains active after work completes.

Reference: [Org refile and copy](https://orgmode.org/manual/Refile-and-Copy.html).
