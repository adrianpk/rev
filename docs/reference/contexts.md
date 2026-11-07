# Files, Directories and Projects

A standalone context contains ordinary file and unnamed buffers. A temporary
directory context adds an explicit tree root. A Project registers one existing
directory under a name and owns an independent saved file session. Opening a
directory through `rev path`, `find-file` or `open-directory` always uses temporary
directory work, including when that directory is already registered.

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

Names are trimmed, case-sensitive and preserve Unicode. Empty names, control
characters and duplicate Project names are rejected while name input remains
open. Names are labels, never storage paths. A canonical root or filesystem alias
already registered opens a choice for the existing Project; it does not overwrite
that Project's saved session. C-g cancels before publication.

The picker filters names and paths. Use arrows to choose an entry, or type a
query and Tab to complete its label, then Enter. Selection retains the stored
Project ID and checks its record again before opening. Choosing the active ID
keeps live work without replaying saved state or asking for discard.

## File sessions

Project sessions contain file-backed buffers in native buffer order, the active
file, logical cursor positions and source-anchored viewports. In-root files use
root-relative references; explicitly visited outside files remain absolute loose
files. Unnamed text, terminal contents, process handles and transient results are
excluded. Promotion retains unsaved text in memory; it does not save that text
into Project metadata or write source files.

Before leaving a Project or completing clean quit, rev resolves modified and
unnamed buffers through native save/discard/cancel decisions, settles active
clocks and owned terminal/Git work, and publishes the session before departure.
C-g retains the source context. Ordinary file saves already approved during the
interaction retain their filesystem effects. A storage failure retains live
sources and reports that departure did not complete.

Restoration uses the native file loader, reuses canonical file identities and
clamps changed source coordinates to valid Unicode cursor boundaries. Missing,
unreadable or unsupported files are skipped with an unavailable-file diagnostic;
no empty replacement file is created. Their saved entries remain available for
later restoration. A missing Project root blocks opening and preserves its
registration. A damaged saved session blocks that Project, while unrelated
registrations remain usable. The original state bytes are retained.

## Storage and ownership

Executable startup supplies `$XDG_STATE_HOME/rev/contexts.toml`, with the normal
`$HOME/.local/state/rev/contexts.toml` fallback. Opening ordinary files/directories
and showing the tree do not read or create this catalog. Project listing is
read-only; listing an empty catalog creates no directories or state files.

The versioned aggregate contains registrations and their selected snapshots. A
publication uses one nonblocking lock attempt and one atomic replacement.
Competing or stale writers report a conflict instead of overwriting newer
sessions. A warning after successful replacement means the catalog was published
but final directory durability/observation could not be confirmed. Do not treat
that warning as an unregistered Project and create another copy.

The initial bounds are 256 Projects, 128 Workspaces in stored state, 512 files per
session, 256 UTF-8 bytes per name, 4096 native bytes per path, 2 MiB per encoded
session and 8 MiB per catalog. Overflow rejects the operation without truncating
saved work. Restoration admits at most 16 files per cooperative pass. One owned
IO job performs state reads, file ingress or publication; no polling remains for
Project work after it settles.

Embedded `Session::new`/`empty` callers supply their own state path through
`Session::set_context_storage`; they never discover user state implicitly.
Project selection retains existing per-file service authority. It does not run
shell commands, enable analysis or load executable configuration from a root.

This is explicit named Project restoration. Automatic restoration of the last
application session on executable startup is not provided by these commands.
