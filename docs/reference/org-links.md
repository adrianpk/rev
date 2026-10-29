# Org links

## Commands and source syntax

`C-c l` (`org-store-link`) retains a checked location for insertion. Existing
ID and CUSTOM_ID properties identify headings without creating new metadata.
Dedicated targets and named elements retain their source names. Other Org
headings use a file link with an exact heading search; other associated files
use a line number. Unsaved, unassociated sources need an existing ID or an
associated file before a link can be stored.

`C-c C-l` inserts a link or edits the link at point. Enter its target and then
its optional description. The latest stored link is the default; TAB lists
recent targets. A selected linear region supplies the description and is
replaced only after both arguments are accepted. A successful insertion
removes that target from the stored list. C-g cancels without source edits.
Source edits use one ordinary undo group, map shared views and leave point after
the inserted link. Read-only sources support navigation and storing links;
insertion requires an editable canonical buffer.

`C-c C-o` follows a recognized bracket, angle or plain link at point. Literal
blocks never recognize active links. Descriptions, delimiter escapes and source
spelling remain source; following does not rewrite or save the document.

## Native destinations

| Form | Resolution |
| --- | --- |
| `[[name]]` | Unique dedicated target or named element, then exact heading. |
| `[[*Heading]]` | Exact heading title, ignoring its TODO keyword, priority and tags. |
| `[[#custom-id]]` | Unique local CUSTOM_ID property. |
| `[[id:identifier]]` | Unique ID in explicitly selected agenda sources and open Org buffers. |
| `[[file:path]]` | Existing local file, absolute or relative to the source file directory. |
| `[[file:path::search]]` | Exact native target/heading, `#custom-id` or checked one-based line number. |
| `[[attachment:path]]` | Existing file inside the heading's admitted attachment directory. |

Local paths and names decode percent-encoded UTF-8 exactly once. `file+emacs:`
requests native visiting. Remote/home expansion is diagnosed; no environment or
shell expansion occurs. Missing files, invalid UTF-8, stale source observations
and ambiguous targets preserve the current editing state. A dirty destination
buffer supplies its canonical text before disk text. A failed visit never
creates a replacement document or silently chooses the first duplicate.

ID lookup uses configured agenda roots, open Org buffers and retained selected
agenda sources. No directory or global ID database is scanned implicitly.
Attachment links use a local DIR property, or `org.attachments.directory`
(initially `data`) followed by the ID's first two characters and remainder.
Relative directories use the source document. Configured property inheritance
can admit inherited DIR values. Traversal and canonical symlink escape from the
selected attachment directory are rejected; following never creates folders.

## Navigation and bounds

`C-c C-x C-n` and `C-c C-x C-p` move to the next/previous recognized link.
At a boundary the first request reports it; repeat without moving to wrap.
Following a native destination records a source point. `C-c &` returns to it;
`C-c %` records the current point explicitly. Closed or changed source revisions
cannot turn an old return point into authority over newer text.

The session retains 32 stored links and 64 return points. A native observation
admits at most 4096 link/target/heading records and 1 MiB of labels, with targets
and descriptions limited to 4096 bytes each. A complete link occupies at most
8192 bytes. Structure retains the existing 8 MiB, 200000-node and 256-level
profile. Discovery uses the existing single cancellable Org worker and configured
source/file/time limits. No worker or polling deadline remains active when idle.

References: [Handling links](https://orgmode.org/manual/Handling-Links.html),
[Internal links](https://orgmode.org/manual/Internal-Links.html),
[External links](https://orgmode.org/manual/External-Links.html).
