# Files, Directories, Projects and Workspaces

A standalone context contains ordinary file and unnamed buffers. A temporary
directory context adds an explicit tree root. A Project registers one existing
directory under a name and owns an independent saved file session. Opening a
directory through `rev path`, `find-file` or `open-directory` always uses temporary
directory work, including when that directory is already registered.

A Workspace owns zero or more ordered directory roots and its own file session.
It can contain plain directories and directories selected from Projects. Adding a
Project directory retains its registration and does not import its saved files.

## Commands

Invoke these names through M-x or modal colon. They add no default bindings.

| Command | Behavior |
| --- | --- |
| `generate-new-buffer` | Add an unnamed buffer to the current context. |
| `open-directory` | Choose an existing directory and replace the context after source decisions. |
| `context-close` | Resolve current work and continue in an empty standalone buffer. |
| `project-create` | Choose an existing directory, edit its proposed basename and register a Project before opening it. |
| `project-promote` | Name and register the current temporary directory while retaining its buffers, dirty text, cursor positions and views. |
| `project-switch-project` | Select a registered Project by name/path and restore its saved file session. |
| `project-list` | Open the same native Project picker. |
| `workspace-create` | Name and open an empty Workspace with an add-directory hint. |
| `workspace-open` | Select a Workspace by name/path and restore its independent file session. |
| `workspace-list` | Open the same Workspace picker. |
| `workspace-add-directory` | Append an existing directory to the active Workspace. |
| `workspace-add-project` | Select a Project and append only its directory, retaining optional provenance. |
| `workspace-remove-root` | Select a root and resolve only files that lose coverage before committing removal. |

Names are trimmed, case-sensitive and preserve Unicode. Empty names, control
characters and duplicate names within the same artifact type are rejected while
name input remains open. A Project and Workspace may share a name. Names are
labels, never storage paths. A canonical root or filesystem alias
already registered opens a choice for the existing Project; it does not overwrite
that Project's saved session. C-g cancels before publication.

The picker filters names and paths. Use arrows to choose an entry, or type a
query and Tab to complete its label, then Enter. Selection retains the stored
artifact ID and checks its record again before opening. Choosing the active ID
keeps live work without replaying saved state or asking for discard.

## File sessions

Project and Workspace sessions contain file-backed buffers in native buffer
order, the active file, logical cursor positions and source-anchored viewports. In-root files use
root-relative references; explicitly visited outside files remain absolute loose
files. Unnamed text, terminal contents, process handles and transient results are
excluded. Promotion retains unsaved text in memory; it does not save that text
into context metadata or write source files.

Before leaving a persistent context or completing clean quit, rev resolves
modified and unnamed buffers through native save/discard/cancel decisions, settles active
clocks and owned terminal/Git work, and publishes the session before departure.
C-g retains the source context. Ordinary file saves already approved during the
interaction retain their filesystem effects. A storage failure retains live
sources and reports that departure did not complete.

Restoration uses the native file loader, reuses canonical file identities and
clamps changed source coordinates to valid Unicode cursor boundaries. Missing,
unreadable or unsupported files are skipped with an unavailable-file diagnostic;
no empty replacement file is created. Their saved entries remain available for
later restoration. A missing Project root blocks opening and preserves its
registration. Unavailable Workspace roots remain visible with diagnostics, and
available roots and loose files can still restore. All-unavailable membership
retains the Workspace identity without substituting another directory. A damaged
saved session blocks its artifact, while unrelated
registrations remain usable. The original state bytes are retained.

## Workspace roots

Roots appear as peer tree nodes in insertion order, initially collapsed. Expand
one to read its immediate children. Each route retains its own expansion,
selection and pointer identity, including overlapping roots. Root labels include
their path to disambiguate equal basenames. Decorated/minimal labels and the Nerd
Fonts preference use the existing [file-tree policies](file-tree.md).

Equivalent canonical/symlink directories are rejected within one Workspace;
distinct ancestor/descendant roots remain allowed, and another Workspace can
contain the same directory independently. A file opened through overlapping
routes reuses its canonical buffer. Session membership chooses the most specific
covering root, with stored order breaking ties.

Removing a root retains files covered by another root and remaps their saved
references. Files losing their only covering root use save/discard/cancel before
membership is published. Cancel or storage failure keeps both membership and
live work. Loose files and unrelated dirty buffers remain open. Only a clock
whose source closes needs its native departure decision; unrelated terminals and
jobs retain their existing owners. Removal leaves directories and source files
in place and does not unregister Projects. Removing the last root leaves a valid
empty Workspace; loose and unnamed live buffers can remain usable.

Root changes publish the membership and matching session together. Empty and
unavailable trees retain their actual membership through refresh and resizing;
showing the tree never creates a cwd root for a Workspace. Ordinary tree rename
cannot relocate a registered root or one of its ancestor directories.

M-x works while the tree owns input. Modal colon belongs to Normal/Visual source
editing; use C-g to return from tree focus before entering a colon command.

## Storage and ownership

Executable startup supplies `$XDG_STATE_HOME/rev/contexts.toml`, with the normal
`$HOME/.local/state/rev/contexts.toml` fallback. Opening ordinary files/directories
and showing the tree do not read or create this catalog. Artifact listing is
read-only; listing an empty catalog creates no directories or state files.

The versioned aggregate contains registrations and their selected snapshots. A
publication uses one nonblocking lock attempt and one atomic replacement.
Competing or stale writers report a conflict instead of overwriting newer
sessions. A warning after successful replacement means the catalog was published
but final directory durability/observation could not be confirmed. Do not treat
that warning as an unregistered context and create another copy.

The initial bounds are 256 Projects, 128 Workspaces, 64 roots per Workspace, 512 files per
session, 256 UTF-8 bytes per name, 4096 native bytes per path, 2 MiB per encoded
session and 8 MiB per catalog. Overflow rejects the operation without truncating
saved work. Restoration admits at most 16 files per cooperative pass. One owned
IO job performs state reads, file ingress or publication; no polling remains for
context work after it settles.

Embedded `Session::new`/`empty` callers supply their own state path through
`Session::set_context_storage`; they never discover user state implicitly.
Persistent context selection retains existing per-file service authority. It does not run
shell commands, enable analysis or load executable configuration from a root.

This is explicit named Project/Workspace restoration. Automatic restoration of the last
application session on executable startup is not provided by these commands.
