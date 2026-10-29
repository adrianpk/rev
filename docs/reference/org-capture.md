# Native capture templates and targets

Capture configuration is typed data. Template selection and target discovery
are non-writing operations. Editable drafts and finalize/abort are separate
stages; this layer does not add a placeholder capture command.

`org.capture.templates = []` selects the native `t` / `Task` entry fallback,
with target `{type="file+headline", file="", headline="Tasks"}` and text
`* TODO %?\n  %u\n  %a`. `org.capture.default_file = ""` requires explicit
file entry. An empty file is never replaced with a fabricated example path.

Each configured template has required `key`, `description` and `target` fields.
`kind` defaults to `entry`, `template` to the native kind's text and `prepend`
to false. Keys select exactly; at most 64 unique keys are admitted. Keys and
labels admit at most 16 and 256 UTF-8 bytes, respectively. No control characters
are accepted in keys or labels. Text admits at most 65536 bytes, permits newlines
and tabs, and rejects NUL. Text stays literal during selection/discovery.

| Kind | Empty/omitted template text | Target content |
| --- | --- | --- |
| `entry` | `* %?\n  %a` | Child heading, or top-level entry in a file. |
| `item` | `- %?` | First plain list at the selected location. |
| `checkitem` | `- [ ] %?` | First plain list at the selected location. |
| `table-line` | `\| %? \|` | First table at the selected location. |
| `plain` | Explicit nonblank text required. | Literal body text. |

| Target type | Fields in addition to `type` |
| --- | --- |
| `file` | Optional `file`, default empty. |
| `file+headline` | Optional `file`; required exact `headline`. |
| `file+olp` | Optional `file`; required `outline` array of 1..64 headings. |
| `file+olp+datetree` | Optional `file`; optional `outline`, default empty. |

Outline components and headlines admit 1..1024 UTF-8 bytes without control
characters. Destination paths admit at most 4096 bytes and reject control
characters and URL schemes. An empty target file uses `org.capture.default_file`.
Relative paths resolve against the configuration source that assigned their own
leaf (`templates` or `default_file`). Explicitly entered relative files use the
origin directory supplied by the caller. No environment expansion is performed.

For example, this data declares a capture destination without creating it:

```toml
[org.capture]
default_file = "notes.org"
templates = [
  {key="t", description="Task", kind="entry", target={type="file+headline", headline="Tasks"}, template="* TODO %?\n  %u\n  %a"},
  {key="j", description="Journal", kind="plain", target={type="file+olp+datetree", file="journal.org"}, template="%?"},
]
```

Unknown fields/kinds/targets reject. Function/Lisp template forms are never
configuration execution. Expansion admission belongs to the draft stage.

Reference: [capture template elements](https://orgmode.org/manual/Template-elements.html).

## Non-writing target discovery

`Choice::discover` prepares immutable work for the existing owned job. It
snapshots live canonical ropes without filesystem IO; `Discovery::run` performs
one explicit file observation and bounded native structural analysis. Dirty
canonical text takes precedence over an alias's disk text. Multiple canonical
buffers with one file identity reject instead of selecting an arbitrary owner.
Read-only status, the real buffer stamp or disk metadata, BOM and original
UTF-8/CRLF text remain in the observation. A missing path has explicit `Missing`
evidence; neither its file nor its parent directories are created or visited.

The result describes a parent, body/subtree byte boundaries and missing heading
plans. Global headline lookup requires uniqueness. Outline lookup requires an
exact unique ancestry path. Task keywords, priority and local tags retain their
native heading meaning. Literal blocks do not supply list/table/heading targets;
malformed or incomplete structure cannot certify a destination.

Date trees use the operation's supplied civil date. Existing year/month/day
headings are reused by their date prefix; display month/day names do not grant
identity. Missing headings are plans with explicit levels and an insertion
anchor before a later date sibling when present. No new date is read from the
clock halfway through discovery. At most 64 outline components and the three
date levels can be planned, within the native 256-level analysis profile.

Item/checkitem targets observe the first parsed list; table-line targets observe
the first parsed table. A heading target limits the search to its own body,
excluding child headings. A file target searches the file. Missing lists/tables
remain explicit container plans. `prepend` is retained for later publication.

Capture uses the existing per-file/total source byte and job-second budgets,
with defaults of 8 MiB / 64 MiB / 30 seconds and at most 4096 retained open-file
snapshots. Native structural analysis retains its finite node/depth limits.
Cancellation or incomplete analysis rejects discovery. Returned offsets belong
to the retained observation; they do not authorize an edit against newer text.
