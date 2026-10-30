# Native capture templates and targets

For the default interactive flow, see [Capture a task](../how-to/capture-task.md).

Capture configuration is typed data. Template selection and target discovery
are non-writing operations. `C-c c` / `M-x org-capture` selects a template,
collects required input and opens an editable transient draft. `C-c C-c` /
`org-capture-finalize` revalidates its destination, inserts once and saves.

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

## Expansion and editable drafts

Capture starts from normal text or Org editing without first visiting a target.
Enter a configured key exactly; empty input selects `t`. Tab shows the admitted
keys/descriptions. With no target/default file, enter one literal path relative
to the originating file's directory (or the working directory for unnamed text).
No file or parent is created by this input or destination discovery.

| Token | Expansion |
| --- | --- |
| `%?` | First final cursor marker, removed from generated text. |
| `%i` | Active origin region; the current line prefix repeats on later inserted lines. |
| `%a`, `%l` | Checked origin file link, empty for an unnamed buffer. |
| `%t`, `%T` | Captured civil date, or date/time, as an active Org timestamp. |
| `%u`, `%U` | The same captured date/date-time as an inactive timestamp. |
| `%n` | Explicitly supplied capture user name. |
| `%c` | Most recent native kill text, empty when unavailable. |
| `%x` | Explicitly supplied clipboard text; no external clipboard program runs. |
| `%^{label\|default\|choice...}` | String prompt with optional initial value and choices shown by Tab. |
| `%<FORMAT>` | Bounded civil formatting. |
| `%%`, `\%` | Literal percent. |

Formatting supports `Y m d e H M S F R T a A b h B u w j % n t` directives.
Weekday/month names use English; timestamps use whole minutes. Unavailable zone
or other directives fail explicitly. All time tokens use one snapshot captured
at start, including after a prompt crosses midnight. String prompts use the
ordinary single-line minibuffer limit. User/clipboard prompts ask once and
reuse that supplied value; host data is never guessed or obtained by executing
configuration. Inserted selection, kill text and answers remain opaque text.

Admission preserves the configured template and rejects unknown, unterminated,
function/Lisp and file-inclusion escapes before prompting or destination IO.
Limits are 4096 tokens, 64 string/host prompts, 256 bytes per time format,
256 KiB per supplied value and 1 MiB generated text. Work advances in 64-token
quanta; waiting for human input or an idle draft adds no polling. Destination
reads share the existing cancellable job slot and retain the bounded source
observation separately from the editable draft.

The draft's `Capture` status and `*Org Capture*` name identify a real transient
text owner. Ordinary editing, selection, undo/redo, search and kill/yank work;
`keys.org_capture` inherits editor bindings and can add local editing aliases.
Save, write-file, mode changes and replacing/splitting its view reject while
retaining the draft. Quit and buffer/window removal require abort first.
`C-g` cancels template/file/expansion/discovery preparation; in an open draft it
cancels the current ordinary interaction. During finalization validation it
returns to the editable draft. `C-c C-k` / `org-capture-kill` discards
the draft and returns to the originating pane. Shared canonical edits map its
retained point and viewport before return. Abort leaves destination bytes,
BOM, line endings, file association and source history unchanged.

Reference: [template expansion](https://orgmode.org/manual/Template-expansion.html).

## Finalize, retry and publication outcomes

`C-c C-c` requires the selected capture draft and completed prompt input. It
checks the retained destination path, version, BOM and actual source bytes in
the existing bounded, cancellable job. It requalifies every insertion/marker
range against the current canonical owner immediately before publication.
Stale or ambiguous source, read-only owners, malformed drafts and resource
limits retain the draft with a failure. A changed target requires restarting
capture against current source; it never authorizes an unchecked overwrite.

Entries become children at the target level, retaining their relative child
levels. Prepending a file entry preserves its preamble. Missing outline/date
headings and capture text form one coherent edit; date headings use the date
captured at start. List capture uses the first native list's marker family and
indentation. Prepending ordered items renumbers affected markers with native
counter/reset rules. Table rows precede trailing blank/formula metadata. Plain
text remains literal. New structural separators follow the target's prevailing
LF/CRLF style; existing BOM and source fragments remain exact.

Finalize saves through ordinary destination identity, permission, temporary-file
and replacement safeguards, including earlier unsaved canonical edits. History
and shared-point updates are reserved before IO. A rejected replacement leaves
canonical source/history/points unchanged and retains the draft. An unopened
new owner is installed only after actual replacement, and missing parents are
never created implicitly. Correcting a missing parent permits a safe retry.
Successful publication creates one undoable source edit and returns to the
mapped origin. Undo changes the canonical buffer; it does not undo saved files.

A failure after replacement reports that capture was inserted once and save
completion is unresolved. The source remains canonical with its ordinary save
warning, and the draft is retained. `C-c C-c` retries saving that exact source,
without inserting again. New draft edits or a changed canonical source reject
that retry and keep both recoverable owners. A save whose observation remains
unresolved requires ordinary inspection/save decisions. Aborting in this state
discards only the draft and reports that the already published source remains;
it does not claim to restore old disk bytes.

Draft text is nonempty and at most 1 MiB. Structural placement admits the native
256-heading-level profile and bounded destination parser. Ordered lists admit
at most 8192 root items. Finalization reads and structural work share the
configured Org job deadline; explicit local save uses the ordinary file owner.
