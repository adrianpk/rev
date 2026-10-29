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
| `plain` | `%?` | Literal body text. |

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
