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
| `generate-new-buffer` | Add an unnamed buffer to current editing; from the startup screen, resolve departure and start fresh standalone work. |
| `open-directory` | Choose an existing directory and replace the context after source decisions. |
| `context-close` | Resolve current work and return to the startup screen over fresh standalone work. |
| `dashboard-open` | Show the native startup screen while retaining current buffers, views and context identity. |
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
| `project-rename` | Select a Project and change only its display name. |
| `project-forget` | Remove Project metadata; active live work becomes a temporary directory. |
| `project-set-root` | Select a Project and an existing replacement directory without moving files. |
| `project-promote-to-workspace` | Convert a Project atomically to a fresh Workspace; active dirty text stays live. |
| `project-duplicate-as-workspace` | Resolve source decisions, retain the Project and open an independent Workspace copy of its file session. |
| `workspace-rename` | Select a Workspace and change only its display name. |
| `workspace-forget` | Remove Workspace metadata; active work resolves source decisions and closes the context. |
| `workspace-set-root-path` | Select a Workspace/root identity and an existing replacement directory. |

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

## Startup screen

Executable startup without a target shows New Buffer, Open File, Open Dir, Open
Project and Open Workspace. An explicit file or directory opens directly. Library
constructors remain passive; embedded hosts can call `dashboard-open` explicitly.
Automatic restoration of the last application session is separate.

`dashboard-open` retains current work until a replacement succeeds. New Buffer
uses the normal checked departure to start standalone work. Open File uses
`find-file` within the retained context; Open Dir uses `open-directory`. Project
and Workspace actions use their typed native pickers and independent sessions.
Choosing the active artifact returns to its live work. Empty pickers name their
creation commands. Opening or canceling a picker creates no registration.

Arrows and Enter select actions, with native C-p/C-n in the non-modal profile and
j/k in the modal profile. M-x or modal colon opens the usual command selector,
including ranking, configured effective hints and name-only completion. Other
explicitly selected named commands return to retained editing before running
their existing handlers. C-g cancels a prompt to the startup screen; C-g from the
screen returns to retained work. Source typing and paste cannot edit the screen
or its retained document. Pointer press/release activates only a matching action;
artwork and spacing have no actions, and resized gestures are canceled.

The supplied Unicode/RGB banner appears above a compact centered action group on
a black content background. It is built-in artwork rather than a buffer, file or
saved entry. Source palettes remain unchanged. Narrow or short terminals show a
plain `rev` label and scroll the actions to keep selection visible. Zero-sized
presentation retains action identity. `NO_COLOR` also uses the plain label with
the ordinary monochrome renderer, visible marker and reverse-video selection.
Nerd Fonts and pointer input are optional.

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

## Lifecycle operations

Lifecycle commands first select a Project or Workspace by typed identity, even
when another context is active. Rename changes only the name and record revision;
roots, selected snapshots and live buffers remain intact. Forget asks for `yes`
and removes only registration and its owned session. Forgetting an active Project
retains dirty buffers, positions and views in a temporary directory context.
Forgetting an inactive artifact leaves current work unchanged. An active Workspace
uses native clock, source and owned-job departure before removal, then closes to
the startup screen over standalone work with no chosen replacement root. Source directories and copied
Workspace roots remain usable.

`project-set-root` and `workspace-set-root-path` validate an existing replacement
directory. Workspace relocation also selects the root by its ordinal, label and
path. Artifact and root IDs remain stable; ordered root membership and its
matching session revision advance together. Root-relative file and active-file
references follow the new path; absolute loose entries retain their paths.
Unavailable relative files remain saved entries with diagnostics. Source files
are never moved or copied, and other artifacts retain their own paths. Another
Project's directory and duplicate Workspace membership are rejected. Active
relocation resolves source decisions and stages the remapped disk-backed session
before publication; cancellation or storage failure retains live sources.

`project-promote-to-workspace` asks for the new Workspace name, allocates fresh
Workspace/root identities and remaps the source file and active references. Its
single catalog replacement adds the Workspace/session and consumes the Project.
Active conversion preserves live dirty text, buffer identities, histories and
views without reopening them; inactive conversion uses the saved Project session
and retains the current context. Unsaved text is never stored in the catalog.

`project-duplicate-as-workspace` retains the original Project and its selected
snapshot. It initializes a fresh independent Workspace from the active Project's
file positions or its inactive saved session, then activates it. Native
save/discard/cancel resolves modified and unnamed buffers in the departing
context first. Cancel creates nothing. Discard restores the destination from
disk; an approved save retains its ordinary filesystem effects. Later session
checkpoints belong only to their selected artifact. Both artifacts still refer
to the same source files, so external-change handling remains applicable.

C-g cancels any picker or prompt before publication. Busy, stale, invalid or
unsupported state never resets the catalog or removes source files. Root
observations and source stamps are checked again before the durable replacement.
A damaged selected snapshot blocks relocation/conversion/duplication; rename and
forget remain available to manage its registration without interpreting it.

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
Workspace AI defaults and LSP boundaries use the selected source's most specific
root; loose sources retain their parent. An explicit AI directory still takes
precedence. Default terminals use the selected source root while editing, or the
chosen tree root while the tree owns focus; terminal-here keeps its existing
per-file/process cwd behavior. Git and shell jobs retain their per-file directory
selection. No consumer receives a union of peer roots. Org source ownership,
global bookmarks, command usage and kill-ring state remain with their native
owners across context changes.

This is explicit named Project/Workspace restoration. Automatic restoration of the last
application session on executable startup is not provided by these commands.
