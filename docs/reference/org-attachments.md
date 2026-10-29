# Org heading attachments

## Dispatcher and commands

`C-c C-a` (`org-attach`) opens the attachment dispatcher for the current heading.
It does not create or scan a directory until an operation is selected.

| Dispatcher | Command | Behavior |
| --- | --- | --- |
| a / c | `org-attach-attach` / `org-attach-copy` | Copy one selected existing file; a uses the native copy default. |
| m | `org-attach-move` | Copy durably to the attachment directory, then remove the checked input. |
| l | `org-attach-link` | Create a hard link on the same filesystem. |
| o | `org-attach-open` | Select an existing attachment and follow native file/viewer rules. |
| O | `org-attach-open-in-emacs` | Force native text visiting, including files with media extensions. |
| f | `org-attach-reveal-in-emacs` | List checked attachment names in deterministic order. |
| s | `org-attach-set-directory` | Set the heading's DIR property without creating folders. |

`org-attach-results` shows the latest retained file and heading result. Named
commands are also available through M-x. Lists retain at most 4096 files and
1 MiB of names; open prompts accept an exact name or one-based number. The echo
shows the first eight names. C-g cancels the interaction and active work.
The selected file and directory observations remain pinned through native visiting
or viewer confirmation; substitution requires a fresh listing.

## Locations and links

DIR takes precedence over ID, including configured property inheritance.
Relative DIR values use the Org document's directory. Without DIR, an ID uses
`org.attachments.directory` (default `data`), its first two characters and its
remainder as child directories. ID directory components require 3..128 ASCII
letters, digits, hyphens or underscores. Copy/move/link generates a UUID when
neither property supplies a location. The generated ID remains proposed until
the file is published and the actual heading is still editable and current.

Successful attachment adds the local ATTACH tag and any generated ID through
one ordinary source transaction. It stores a compatible `attachment:` link for
`C-c C-l`; filename percent escapes remain literal link data. Metadata changes
remain unsaved until ordinary Save. Undo changes source metadata only; file
operations have independent effects. Visiting, highlighting and moving point
through a heading never creates, lists or opens its resources.

## File decisions and retained outcomes

Copy/move/link first observes the selected file and destination. A separate
`yes/no` confirmation identifies the actual method and target, including an
existing-file replacement. `no` or C-g before publication leaves both files and
heading unchanged. Names, paths, file identity/content/permission observations
and directory ownership are checked again before effects. Symlink components
and non-regular input/destination entries are rejected. Native attachment file
reads use `org.limits.agenda_file_bytes`, initially 8 MiB. Directory paths admit
at most 64 components; filenames occupy one component and at most 255 UTF-8 bytes.

Publication uses an exclusively created staging file. A new destination is
published without overwriting a concurrent entry; replacing an observed existing
file requires affirmative confirmation. Move removes its original only after
destination publication and directory sync. Moving a currently visited input
file is rejected. Hard links retain normal filesystem identity and cannot cross
filesystems. External media requires the [configured viewer](org-links.md) and
its separate explicit confirmation; opening does not grant execution authority
to document text.

File publication, source removal and heading edits have separate results. A late
cancellation, sync failure or stale heading cannot erase an already completed
file effect. Up to 16 results retain paths, generated ID, created directories,
any retained staging name and errors. Inspect the result before retrying a
partial operation. Empty directories created by an explicit failed operation
remain recorded; cleanup never recursively removes resources.

Linux directory descriptors anchor publication to the admitted parent. These
checks provide conservative native file observations, not a filesystem-wide
transaction or a promise against every concurrent namespace change. The shared
single Org worker checks cancellation between bounded compute/read steps and
releases its polling deadline after reaping; blocked kernel IO has no hard
interruption guarantee.

References: [Attachments](https://orgmode.org/manual/Attachments.html),
[Attachment dispatcher](https://orgmode.org/manual/Attachment-defaults-and-dispatcher.html),
[Attachment options](https://orgmode.org/manual/Attachment-options.html),
[Attachment links](https://orgmode.org/manual/Attachment-links.html).
