# Experimental plugin declarations

The experimental catalog admits explicitly configured local packages for
rev.plugins/0.1. It reads passive metadata and adds qualified command names and
descriptions to M-x and the modal colon selector. It starts no programs.
Activation, contributed keybindings and executable editor operations are
unavailable in this catalog. The host advertises no executable API capability.

## Configuration

These startup-only values participate in the ordinary configuration include and
leaf precedence rules. An explicit empty array replaces an inherited array.

```toml
[plugins]
enabled = false
manifests = []
startup = []
keybindings = []
```

With enabled=false, manifests are not opened and no package commands are
admitted. With enabled=true, every listed declaration must validate before
any package is admitted. Failure retains the editor's complete built-in
configuration and reports one diagnostic; no partial catalog becomes visible.

Manifest paths are explicit local paths. Resolve them relative to the source
that last assigned plugins.manifests, including an included source. Resolve a
relative program path against its manifest directory. Both paths are frozen
as absolute paths at preparation. Do not expand variables, search PATH, probe
executables, scan directories or run a shell. Program arguments remain literal.

startup contains at most eight distinct declared package IDs. It records an
explicit activation request; this passive catalog cannot activate them yet.

## Local manifest

```toml
[plugin]
id = "field.notes"
version = "0.1.0"
api = "rev.plugins/0.1"

[process]
program = "./notes-plugin"
args = []

[[commands]]
name = "collect"
description = "Collect task lines from open text buffers."
parameters = []
prefix = "none"
recording = "reject"

[activation]
filetypes = []
events = []
```

Every object rejects unknown fields. The plugin and process tables are required.
The commands, bindings and activation collections default to empty. Package
IDs contain two lowercase publisher.package segments of 1..=32 ASCII characters;
each starts with a letter and then permits letters, digits and hyphens. The rev
publisher is reserved. Version is a nonempty opaque label of at most 128 visible
ASCII bytes, distinct from the exact required API revision.

Command names contain 1..=64 lowercase ASCII letters, digits or hyphens. The
canonical ID is derived as publisher.package.command, at most 130 bytes. Names
and package identities must be unique. Descriptions are nonempty, single-line
UTF-8 text of at most 4096 bytes. There are no implicit short-name aliases.

Each optional parameters entry contains name, kind and required. Names are
distinct lexical lowercase ASCII letters/digits/hyphens of 1..=64 bytes. Kinds
are null, boolean, integer, string, array or object. Arguments are inert JSON
values. prefix is none or integer; recording is reject or semantic. These
declarations do not enable invocation or recording before execution is available.

Activation filetypes use existing native language IDs. Events are opened,
changed or closed. Both arrays reject duplicates. They describe future lazy
activation conditions, without executing the package during discovery.

## Binding declarations

Each manifest [[bindings]] entry and each user [[plugins.keybindings]] entry
has these fields:

| Field | Contract |
| --- | --- |
| command | Required qualified command ID; manifest defaults must name a command declared by that same package. User targets must exist in the admitted catalog. |
| keys | Required native key sequence, at most eight normalized chords and 4096 UTF-8 bytes. Uses the existing native terminal aliases and parser. |
| context | Required editor, vim_normal, vim_insert, vim_replace, vim_visual_character, vim_visual_line or vim_visual_block. |
| scope | Required global or buffer. Global rejects selectors; buffer requires at least one selector. |
| filetypes | Optional distinct native filetype IDs, default empty. |
| buffer_owner | Optional package ID matching native provenance of package-created buffers. |

When both selectors are present they intersect. Declarations remain inspectable
and ineffective; they do not replace a native keymap or transient/recovery owner.
Native prefix reachability, recovery and conflict resolution apply before any
future binding becomes executable. No activation order selects a winner.

## Native command metadata

| Command | Description |
| --- | --- |
| `plugin-list` | Inspect configured experimental plugin packages and their availability. |
| `plugin-activate` | Activate a configured experimental plugin package. |
| `plugin-deactivate` | Deactivate a configured experimental plugin package. |
| `plugin-restart` | Restart a configured experimental plugin package. |

plugin-list opens a read-only inventory with package IDs, versions, installed /
inactive state, declared commands and ineffective binding declarations. Display
output is bounded to 64 KiB with an explicit truncation notice. It performs no
program or PATH probe. Selecting an unavailable contributed command reports its
qualified identity and state without editing or saving source text. Qualified
plugin selections participate in session usage ranking. They are not written to
the native schema-1 command history, whose name grammar and limits remain
unchanged; subsequent native command usage continues to persist.

The management commands select an explicitly admitted package ID. Activation and
restart report that activation is unavailable. Deactivation reports that the
package is already inactive. C-g cancels selection without changing source text.

## Admission limits

| Resource | Ceiling |
| --- | --- |
| Explicit manifests / source bytes | 64 / 256 KiB per regular UTF-8 TOML file |
| Commands | 64 per package; 512 total |
| Binding defaults / user overrides | 128 per package; 512 default contributions total; 512 user overrides |
| Command parameters / activation filetypes / selector filetypes | 64 per collection |
| Literal argv | 64 strings, 4096 bytes each, 64 KiB total; NUL rejected |
| Paths | 4096 UTF-8 bytes; nonempty local paths without NUL or URL schemes |

Admission validates all declarations atomically before native discovery. It
does not promise that a program exists, is executable or can complete a future
handshake. Package distribution, downloads and installation are separate work.
