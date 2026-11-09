# Saved macro library

`kmacro-save` selects a named session snapshot and asks for confirmation before
saving it. `kmacro-load` reloads saved definitions without executing input.
`kmacro-delete` confirms deletion of one saved record; ring and session snapshots
remain available. `insert-kbd-macro` inserts a standalone single-record TOML
document through ordinary source editing and undo, without writing the library.
All four commands are available through M-x and modal colon and are unbound.

Any supplied prefix includes the name's admitted source binding when saving or
exporting. Without a prefix the document omits it. Replacing a saved binding with
an unprefixed save explicitly asks to remove that binding from the saved record;
the current session binding remains available. Naming, binding, replay, ring
changes and exit perform no implicit save. Anonymous bindings must first receive
a command name before they can be saved.

## Location and ownership

Startup and reload read `STATE/macros/library.toml`, where STATE is the independently
resolved `$XDG_STATE_HOME/rev` or `$HOME/.local/state/rev` directory described in
[startup configuration](defaults.md#startup-configuration). A missing library is
empty and creates no directory or lock. The first explicit write creates private
directories (0700), the library and `library.toml.lock` (0600). Ordinary files and
directories are required; symlinks and nonregular library/lock entries are refused.

Back up `macros/library.toml`: it contains valuable explicitly saved definitions,
unlike session-only recordings or recomputable cache content. Keep it private;
recorded prompt values and pasted text can contain personal source content. Macro
operations never modify command history, contexts, bookmarks, AI, LSP or other
state files. The lock belongs only to this concern and is held only while writing.

## Schema 1

The aggregate has exactly `schema_version = 1`, a positive `revision`, and `macros`,
an ordered array of saved records. An empty library encodes `macros = []`.
Unknown fields, tags, duplicate IDs/names and inconsistent profiles are rejected
before any record is admitted. Revisions are checked positive integers through
9223372036854775807. Persistent IDs are opaque 32-character lowercase hexadecimal
values; they do not depend on names, paths or process-local ring identities.

| Record field | Meaning |
| --- | --- |
| `id`, `revision`, `name` | Stable saved identity, saved record revision, and 1..64 lowercase ASCII letters/digits/hyphens beginning with a letter. |
| `description` | Optional UTF-8 description, at most 512 bytes. Absence adds no placeholder. |
| `profile`, `entry_state` | `non-modal` with `editing`, or `modal` with `normal`, `insert`, `replace`, `visual-character`, `visual-line` or `visual-block`. |
| `binding` | Optional native sequence such as `C-x C-k 8`, included only by a prefixed save/export. |
| `counter` | Table with signed 64-bit `current`, `previous`, `iteration_start`, and bounded integer `format`. |
| `initial_format` | Optional captured replay-start format when it differs from the working counter format; omission uses `counter.format`. |
| `steps` | Ordered typed steps below. |

```toml
schema_version = 1
revision = 1

[[macros]]
id = "4f967f8112b34ea98a7a2e9900dfc601"
revision = 1
name = "prefix-task"
profile = "non-modal"
entry_state = "editing"
counter = { current = 0, previous = 0, iteration_start = 0, format = "%d" }
steps = [
  { kind = "key", key = "C-a" },
  { kind = "text", text = "- [ ] " },
  { kind = "key", key = "C-n" },
]
```

Steps are data, never an embedded program or shell expression. Each variant
admits only the fields shown; strings preserve exact UTF-8 content.

| `kind` | Other fields |
| --- | --- |
| `key` | `key`: one normalized native chord, including `F3`, `ESC`, `RET`, `SPC`, `C-a` or `M-x`. |
| `text` | `text`: exact pasted text, without NUL. |
| `command` | `name`: canonical command; `argument`: typed prefix table; `arguments`: ordered role/value tables; optional `keys`: recorded native trigger chords. |
| `search` | `direction`, `entry`, typed `argument`, optional trigger `keys`, accepted `pattern`, ordered `continuations`, and native `termination`. |
| `completion` | `edits`: ordered relative edit tables; `cursor`: checked relative scalar offset. No provider or popup row is retained. |
| `counter` | `action`: typed counter operation table. |
| `redisplay` | No additional fields. |
| `query` | `mode = "confirm"` or `"edit"`; no recorded answer. |

A prefix table uses exactly one form: `{ form = "absent" }`,
`{ form = "plain-prefix", depth = 1 }`, `{ form = "integer", value = 4 }`, or
`{ form = "negative-only" }`. Plain depth must produce a positive native count
within 1000000, and integer magnitude must not exceed 1000000. Plain C-u retains
its distinction from integer 4.

Each accepted command argument has `role` and `value`. Roles are `path`, `buffer`,
`line`, `write`, `search`, `search-forward`, `search-backward`, `replace-query` and
`replacement`. Search-repeat roles retain direction. A command owns its
arguments. Selector queries, completion navigation and consent answers are absent.
Optional `keys` preserves physical invocation semantics under the current map;
omission dispatches the canonical accepted M-x/colon name.

A `search` step uses `direction = "forward"` or `"backward"`, and
`entry = "incremental"` or `"modal"`. Each continuation has a literal `pattern`
and repeat `direction`; these run in order before accepting the final top-level
`pattern` with `termination = "accept"`. Its typed prefix and optional physical
trigger keys retain native invocation semantics. There are at most 31 repeat
continuations plus the final accepted value. Search failures stop before following
source edits, and no selector or popup state is retained.

Each completion edit has signed relative scalar `start` and `end`, exact
`expected` source text, and replacement `text`. Its source length must match the
range. Playback checks applicability and cursor arithmetic before editing.

Counter `action` uses `operation = "insert"` with `argument`, `"set"` or `"add"`
with signed `value`, `"reset"`, `"restore"`, or `"format"` with `format`. Formats
admit one d/o/x/X integer conversion, at most 256 UTF-8 bytes, width/precision at
most 128, and rendered output at most 512 bytes. The replay-start format remains
independent of later recorded format changes.

## Schema 2 plugin calls

Readers accept schema_version=1 and schema_version=2. Schema 2 retains every
schema-1 record field and native step and adds this distinct inert variant:

```toml
steps = [{ kind = "plugin-call", command = "field.notes.collect", arguments_json = '{}', prefix = -3 }]
```

command is a qualified publisher.package.command identity. arguments_json is a
JSON object encoded as a TOML string, preserving null, boolean, integer, string,
array and object literals without evaluating them. prefix is an optional signed
64-bit integer; omission means no prefix was supplied. Native prefix input retains
its own magnitude limit before capture. No source handle, process generation,
handler address or executable path is stored in the step.

A plugin-call in schema 1 is invalid. Unknown steps and malformed argument JSON
reject the complete aggregate. Current package availability, argument contracts,
prefix policy and semantic recording policy are checked again at replay.

An explicit save that adds a plugin step to a schema-1 library writes schema 2
while preserving other records' identities, revisions and content. Reload never
rewrites the library. A standalone export uses schema 2 only if its selected
record contains plugin steps. An already schema-2 library retains that version
on later explicit writes. Older editors do not support schema 2; keep a copy of
an older library before saving plugin calls if backward compatibility is needed.

## Reload, concurrency and limits

After complete structural validation, conflicting compiled, prompt or AI names
are quarantined individually. Other valid records can load. A conflicting saved
binding stays inactive with a diagnostic while its named definition remains
available. Reload preserves unsaved local bodies, descriptions, counters and
bindings and reports conflicts. It is not an implicit discard operation.

A write takes the concern's nonblocking lock, rereads the bounded fresh aggregate,
checks the selected record identity/revision, then changes only that record.
Unrelated concurrent writes survive; stale same-record updates and same-name
additions fail without overwriting either session or disk. Publication synchronizes
a same-directory temporary file and atomically replaces the library. A failure
before replacement preserves the previous file. A later durability/observation
warning retains the exact published identity/revision. A later save checks that
same committed record before making another revision; it cannot duplicate the
addition or treat an unrelated stale session as a retry.

The library admits at most 256 saved records and 8 MiB encoded text. Each definition
admits at most 4096 steps and 256 KiB encoded text. Session and loaded names share
the 256-name ceiling; retained macro data shares the 48 MiB limit. Native source
editing and playback retain their independent checks. Invalid, unsupported or
oversized files stay untouched; no migration, backup tree or automatic cleanup
replaces them with an empty default.
