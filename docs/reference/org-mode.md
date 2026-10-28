# Native Org outline

An admitted `.org` file or `M-x org-mode` uses native Org mode. Source starts
fully visible. Visibility controls change only the selected editing view; they also
work on read-only text. Folding never changes saved bytes, modification state,
regions or undo history.

## Keys and named commands

| Key | Command | Behavior |
| --- | --- | --- |
| TAB | `org-cycle` | On a heading, cycle through heading-only, direct child headings and complete subtree. In a drawer, toggle its body. |
| S-TAB | `org-shifttab` | Cycle the whole view through overview, contents and show-all. |

Overview shows top-level headings. Contents shows all heading lines and hides
their bodies. Show-all restores all source, including drawer bodies. Drawer
folding keeps both boundary lines visible. A complete subtree also opens drawers
inside it.

M-x provides the same handlers and these direct choices:

| Commands | Target |
| --- | --- |
| `org-overview`, `org-content`, `org-show-all` | The selected view. |
| `org-hide-subtree`, `org-show-children`, `org-show-subtree` | The heading at point or containing point. |
| `org-hide-drawer`, `org-show-drawer`, `org-toggle-drawer` | The drawer containing point. |

TAB requires a heading line or admitted drawer. Ordinary body text, literal block
text and unavailable structural targets report that no target is available.
They do not insert text or fold another heading. Plain-text, capture and source
editing retain their own TAB binding; a focused prompt retains completion.
The [binding reference](bindings.md) and [configuration sample](config.toml)
list the context maps.

## Navigation and independent views

Splitting a pane copies its current visibility. Later choices affect only the
selected pane. Resize preserves those choices. When explicit folding would hide
point, point moves to the visible heading or drawer opener that owns the hidden
body.

Vertical, page and wheel motion skip hidden rows. A targeted jump or search
reveals the target entry and its ancestor path while retaining unrelated folded
entries. C-g during search restores the starting point, scroll and visibility.
Logical line numbers and source regions continue to refer to canonical text.
Edits, undo and redo release obsolete visibility choices in affected views.

## Pending observations

A command can briefly report a pending structural observation; editing remains
available. C-g cancels its owned request. Moving to another view, changing source
or opening another interaction prevents that request from folding a later state.
Visibility works with syntax coloring disabled and with NO_COLOR.

An incomplete observation rejects folding and keeps source readable. At most
4096 heading/drawer records participate, within the ordinary native Org source
observation limits. `org-show-all` remains available without an observation.

## Heading insertion and motion

| Key | Command | Behavior |
| --- | --- | --- |
| M-RET | `org-meta-return` | Insert a heading in ordinary heading/paragraph context. |
| C-c C-n | `org-next-visible-heading` | Visit the next visible heading. |
| C-c C-p | `org-previous-visible-heading` | Visit the previous visible heading. |
| C-c C-f | `org-forward-heading-same-level` | Visit the next sibling, skipping descendants. |
| C-c C-b | `org-backward-heading-same-level` | Visit the previous sibling, skipping descendants. |
| C-c C-u | `org-up-heading-safe` | Visit the parent heading. |

M-x exposes every name above and `org-insert-heading`, which uses the same
insertion handler. A new heading has the current heading's level, or level one
before the first heading. At the start of a heading, insertion creates a heading
above it. In a title or paragraph, insertion splits at point; at a paragraph
start it prefixes the existing line. Following section text remains after the
new heading. Drawers, blocks, descendants and blank separators retain their
original bytes and order. BOM, LF/CRLF and final-newline presence are preserved.
Point stays on the new title. Insertion is one ordinary undo/redo operation.

`org-insert-heading` reports unavailability in lists, tables, drawers and
literal bodies. M-RET selects list continuation in an admitted item; other
unsupported deeper contexts retain their source ownership. C-g can cancel an observation request
before insertion, just as for visibility commands.

Heading motion also works on read-only source and never creates history.
Next/previous motion skips folded headings even outside the viewport. Same-level
motion stops at a parent boundary; parent motion visits the nearest structural
parent, including skipped numerical levels. These targeted moves use ordinary
hierarchy reveal. A missing target reports unavailability and keeps the view.

## Subtree editing

| Key | Command | Behavior |
| --- | --- | --- |
| M-Left in heading context | `org-metaleft` | Remove one star from the heading and every descendant. |
| M-Right in heading context | `org-metaright` | Add one star to the heading and every descendant. |
| M-Up in heading context | `org-metaup` | Move the complete subtree before its previous sibling. |
| M-Down in heading context | `org-metadown` | Move the complete subtree after its next sibling. |

The explicit M-x names `org-promote-subtree`, `org-demote-subtree`,
`org-move-subtree-up` and `org-move-subtree-down` retain heading-only ownership.
Meta-arrow names select actual list context before these heading handlers.
Point can be on the heading or in its ordinary
section text. A list, table, drawer, literal block or other deeper context keeps
its own ownership and reports that the subtree edit is unavailable there.

Demotion requires a preceding sibling to become the new parent. Promotion rejects
level one. Movement stops at parent boundaries. Missing parent levels, unclosed
owned/parent structures, incomplete observations and levels beyond 256 reject the
whole operation. Malformed source outside the affected ownership remains intact.
Read-only buffers reject every subtree mutation.

A linear region may cover one or more complete adjacent sibling subtrees with
the same parent and level. Both endpoints must match those ownership boundaries.
Partial, empty or rectangular regions reject; descendants are never detached.

Only heading stars change during promotion/demotion. Movement retains drawers,
blocks, table metadata, unknown directives and blank separators with their
subtree. At an unterminated EOF, the existing separator run remains between the
reordered subtrees; no newline bytes are invented. BOM, LF/CRLF and final-newline
presence remain intact.

Stored marks, peer points and viewport anchors follow canonical source fragments.
A boundary at a heading start follows that heading; an unterminated line end
follows that line. One undo/redo reverses the complete operation and restores the
initiating point, while peers map from their current positions. As with ordinary
edits, publication releases active selection and obsolete folds in affected views.

## Lists and checkbox progress

| Key | Command | Behavior in an admitted list |
| --- | --- | --- |
| M-RET | `org-meta-return` | Split or continue an item, retaining its marker/description form. |
| M-Left / M-Right | `org-metaleft` / `org-metaright` | Outdent/indent the item and continuation text, leaving nested items in place. |
| M-S-Left / M-S-Right | `org-outdent-item-tree` / `org-indent-item-tree` | Change indentation with complete nested source. |
| M-Up / M-Down | `org-metaup` / `org-metadown` | Exchange complete adjacent items and renumber ordered siblings. |
| M-S-RET | `org-insert-item-checkbox` | Insert a new unchecked checkbox item. |
| C-c C-c | `org-ctrl-c-ctrl-c` | Edit heading tags, or toggle an existing checkbox and verify current list numbering in list context. |
| C-c C-x C-b | `org-toggle-checkbox` | Toggle the current checkbox, selected checkboxes, or the current heading section. |
| C-c C-x C-r | `org-toggle-radio-button` | Select a checkbox and clear same-level radio peers. |
| C-c # | `org-update-statistics-cookies` | Refresh existing item/heading fraction and percentage cookies. |

M-x also provides `org-insert-item`, `org-indent-item`, `org-outdent-item`,
`org-move-item-up`, `org-move-item-down`, `org-renumber-list` and
`org-toggle-checkbox-presence`. Presence adds/removes checkboxes explicitly;
factory description items reject adding them. Existing checkbox text in a
description item remains actionable. Ordinary drawers may contain real lists;
literal example/source blocks and table cells never become list targets.

Ordered markers retain period/parenthesis spelling and explicit `[@N]` resets.
Decimal counters do not overflow a machine integer; generated counters share the
finite native source byte budget. Description continuation creates an empty
term, including existing relative-timer description spelling, without arming a
timer or inventing elapsed time. New checkbox items start unchecked unless their
existing nested children determine a derived state.

Ordinary outdent rejects detaching children. Use the tree command for that
operation. The first outermost tree command shifts the whole list by one column;
ordinary first-item indentation and nested first-item demotion require a real
preceding item. Promoting a nested item adopts the following source siblings.
Structural list commands require an unselected item. Checkbox operations accept
linear regions in either direction and use the first selected checkbox's new
state; rectangles reject. Read-only, incomplete and unclosed current-entry
observations reject before publication.

Checkbox parents reflect direct children as `[ ]`, `[-]` or `[X]`. Existing `[/]`
and `[%]` cookies update as fractions or integer percentages. A heading normally
counts its direct checkboxes, or direct TODO/DONE child headings when no checkbox
count applies. Local `COOKIE_DATA: todo` or `COOKIE_DATA: checkbox` selects that
source explicitly. Local `ORDERED: t` blocks newly completing a checkbox before
an unfinished predecessor. `#+ATTR_ORG: :radio t` applies radio choice to its
list; hierarchy still derives parent states from child checkboxes.

A list edit and its parent/cookie changes undo as one operation. Untouched source,
BOM, line endings and EOF spelling remain intact. Moves retain source anchors in
shared panes, including outside a list whose counter widths changed. At most
8192 item/list/heading records and 4096 statistics cookies participate in this
bounded derived model. Other entries' cookie spelling stays untouched.

The [Org list reference](https://orgmode.org/manual/Plain-Lists.html) and
[checkbox reference](https://orgmode.org/manual/Checkboxes.html) describe the
corresponding source forms and familiar interactions.

## Properties and source owners

`C-c C-x p` and `M-x org-set-property` prompt for a name and local value.
New drawers follow the heading and its planning line. Before the first heading,
they precede non-comment content. Existing key spelling, indentation, unknown
properties and surrounding source remain intact. Names are case-insensitive;
new names use uppercase. Append `+` to add a value without replacing its base.

`org-delete-property` removes only the named local base; a name ending in `+`
removes its local additions. It preserves ancestors, file declarations and the
other form. Removing the final field retains the empty drawer. `org-inspect-property`
shows the effective value and source line for each local, inherited or file
contribution. Derived fields such as ITEM, TODO, PRIORITY, TAGS and FILE are
identified as derived and cannot be stored through these commands.

`org-edit-property-owner` explicitly edits an effective source contribution.
It identifies the source line before asking for a value. When there are several
contributions, select their 1-based number first. This action can edit an ancestor
or a file directive; ordinary local set/delete never silently changes that owner.
One accepted edit forms one undo group and maps shared views through canonical
source. C-g cancels either prompt without changing the document.

Factory property inheritance is off. Set `org.properties.inherit = ["PROJECT",
"CUSTOM"]` to inherit only those stored names, including file defaults. A local
base wins over the nearest ancestor, then the file drawer and `#+PROPERTY:`
declarations. Local `NAME+` lines extend the selected inherited base. A `nil`
value stops the search and represents an explicitly undefined value. File
`#+PROPERTY: NAME value` declarations apply in source order; a later base resets
its earlier file additions. A file drawer base overrides file declarations.
Recognized keywords change data semantics only. They cannot alter inheritance
policy, editor configuration or process authority, and keywords inside literal
or nested forms do not become file property defaults.

Names are 1..128 UTF-8 bytes without whitespace, colons or interior `+`.
Inheritance accepts at most 128 unique stored names; an empty array replaces an
included array and disables inheritance. Property prompts use the existing
minibuffer input limit. The observation retains at most 4096 source entries and
8192 stored/declaration records. Incomplete observations, ambiguous duplicate
local bases, malformed metadata, literal/deeper contexts and read-only mutation
reject before publication. Inspection does not insert inherited text. Existing
ordinary and property drawers fold independently through the drawer commands.

## Tags and membership

`C-c C-q`, `M-x org-set-tags-command` and `C-c C-c` on an actual heading
replace that heading's local tags. Enter colon-separated, comma-separated or
space-separated names. Empty input clears local tags. Names are case-sensitive;
`Work` and `work` remain distinct. Each name permits letters, numbers, `_`, `@`,
`#` and `%`, up to 128 UTF-8 bytes. Each heading admits at most 128 local names.
Duplicate entered names are collapsed without changing their order. The edit
replaces only the suffix: existing title spacing, unknown source, line endings
and EOF spelling remain intact. A new suffix uses two spaces before its colons.

`org-remove-tag` removes one exact local name and leaves inherited owners alone.
`org-inspect-tags` identifies local, ancestor and FILETAGS source lines. Factory
`org.tags.inherit = true` includes ancestors and file membership; false selects
local tags only. Inheritance never inserts text. Tab completes case-sensitive
names from observed tags and direct `#+TAGS:` declarations, extending their common
prefix and showing up to 16 candidates. Completion changes only the prompt.
Return publishes one checked undo group; C-g cancels without publication.

The observation retains at most 4096 heading records, 8192 tag/candidate
occurrences and 8192 metadata contexts. Incomplete observations and invalid or
excessive suffixes reject. Literal/deeper contexts do not fall through to an
ancestor tag action. Read-only source admits inspection and matching but rejects
tag mutation. Prompt acceptance rechecks its originating view and source revision.

## Tag and property matches

`M-x org-match-tags` prompts for an expression and reports whether the current
heading matches. It uses that heading's actual effective tags, configured inherited
properties and source task declarations. Matching changes neither text nor history.

The native data grammar uses `&` for AND, `|` for OR and leading `-` or `!` for
NOT. Leading `+` selects positively; subsequent `+` or `-` implies AND. AND binds
more strongly than OR; parentheses group expressions. For example,
`(Work|home)+urgent-boss` selects an urgent heading with Work or home membership
and excludes boss. Tag and TODO state spelling remains case-sensitive.

Property comparisons use `=`, `==`, `<>`, `!=`, `/=`, `<`, `<=`, `>` and `>=`.
Property names are case-insensitive. Bare finite numbers use numerical comparison;
double-quoted literals use case-sensitive string ordering. Signs, decimal points
and exponents are admitted in numbers. Missing numeric/string values compare as
zero/empty text; append `*` to the operator to require a present value instead,
as in `EFFORT<*2`. Invalid or non-finite stored numbers do not match numerical
comparisons. Strings admit `\"`, `\\`, `\n`, `\r` and `\t` escapes; unknown
escapes reject. Backslashes escape punctuation in property names, for example
`OWNER\-ROLE="lead"`. Unknown stored property names remain data.

Derived TODO, LEVEL, PRIORITY, ITEM, TAGS and ALLTAGS values come from the actual
heading and effective membership. A suffix `/WAITING|NEXT` applies a separate
Boolean expression to TODO states; `/!` additionally requires an unfinished task.
`Work/!-WAITING` selects unfinished Work tasks outside WAITING. Factory states
are TODO/DONE; direct TODO/SEQ_TODO/TYP_TODO data declarations identify configured
states for matching. Task keywords retain their own punctuation rules; escape
punctuation in a TODO suffix, as in `/NEXT\-ACTION`. Their fast-key/log
annotations do not execute anything.

Compilation admits at most 4096 UTF-8 bytes, 32 nested forms and 512 nodes.
Evaluation is iterative, with at most 8192 tag and 512 property records and a
1048576-byte temporary metadata projection. Regexp/function forms are not scalar
comparison operands; they reject rather than being evaluated as code. Malformed
syntax, unsupported operators and excess input reject with a source-byte
location. There is no Lisp, predicate callback or external process evaluation.

## Task workflows and logs

`C-c C-t` and `M-x org-todo` use the same native task transition. With no fast
keys they cycle the current sequence through its states and then unmarked.
With declared selection keys they open a state prompt; enter the key or full
keyword and Return. `org-todo-set` always opens that prompt; empty or `-` clears
the state. `org-todo-next` and `org-todo-previous` cycle without selection.
`org-inspect-todo` reports the current recognized state and available choices,
including on read-only source. Unknown leading words remain ordinary title text.

Typed `org.todo.sequence` and `org.todo.finished` default to TODO/DONE with DONE
finished. File-local TODO, SEQ_TODO and TYP_TODO declarations replace that workflow.
Each declaration has a separate sequence; `|` begins its finished states, or the
last state is finished when no separator appears. There are at most 64 unique
keywords across the file, each 1..128 UTF-8 bytes without whitespace, controls,
`|`, `(` or `)`. Duplicate keys/keywords and malformed annotations reject editing
and matching before mutation.

For example, `#+TODO: TODO(t) WAIT(w@/!) | DONE(d!)` selects keys and state
logging. `!` requests a timestamp and `@` a one-line note, with `/` selecting
logging on departure. Entry logging takes precedence over departure logging.
`org.todo.log_done = true` additionally records CLOSED on completion; factory
ordinary completion logging is off. A state note can be empty to record only
time. C-g cancels every pending part of the transition.

The inherited LOGGING property resets the default logging policy and annotations
when nonempty. Supported data tokens are `nil`, `logdone`, `lognotedone` and
annotations on declared keywords, such as `WAIT(!)` or `DONE(@)`. Unsupported
tokens reject before mutation. State notes use normal source placement after
planning/properties by default. `org.todo.log_drawer` selects an explicit drawer;
inherited LOG_INTO_DRAWER overrides it with a drawer name, `nil` for normal
placement or `t` for LOGBOOK. Existing selected drawers receive newest entries
first; unrelated drawers retain their contents.

`org.todo.block_children` optionally blocks completion while a descendant task
is unfinished. An ORDERED parent additionally requires earlier sibling tasks
to finish. Local NOBLOCKING=t exempts an entry. The refusal explains the condition
before any state, log or history changes.

A checked transaction publishes the keyword, selected CLOSED/state log/note and
applicable immediate-parent TODO progress cookies together. COOKIE_DATA selects
TODO or checkbox ownership as before. Priorities, tags, unrelated planning and
properties stay intact. Reopening removes CLOSED and preserves an adjacent
property drawer's ownership. Undo/redo restore the entire transaction and shared
source anchors. Deeper literal/list contexts never fall through to an ancestor
task action; drafts recheck their originating view, source and read-only state.

Logging captures local time once before opening its note prompt. The session
reads at most 1 MiB from the local TZif file; TZ can select UTC or a TZif zone
name/absolute path. Leap-aware, malformed, unsupported or uncovered local time
rejects before publication. No source directive can execute code or select a
process, and task logging invokes no external date command.

## Priorities and effort estimates

`C-c ,` (`org-priority`) prompts for a priority character; empty input or a space
removes the explicit cookie. `S-Up` and `S-Down` cycle through the admitted
range and unmarked. Up from an unmarked heading inserts the lowest priority;
down inserts the highest. Passing an endpoint removes the cookie.
`org-inspect-priority` identifies an explicit value or the implicit default
without inserting text. These keys act on the current heading, including plain
headings without task keywords; deeper literal/list contexts reject.

The single-character native range uses uppercase A-Z. Startup
`org.priority.highest`, `lowest` and `default` are A, C and B, with
highest <= default <= lowest. Direct `#+PRIORITIES: D F E` data overrides that
buffer's range. Malformed declarations or unsupported existing cookies reject
before mutation. Priority insertion follows an actual recognized task keyword;
unknown words remain title text. Task transitions preserve cookies. Derived
PRIORITY matching sees the effective default while ITEM excludes only actual
state/cookie prefixes.

`C-c C-x e` (`org-set-effort`) edits the current heading's local EFFORT property.
Enter a nonnegative decimal number of minutes, integer `H:MM`/`H:MM:SS`, or unit
components such as `1d3h5min`, `1.5h` or `1 h 2min`. Values remain in their entered
spelling, including fractional work units; they are not normalized on storage.
Empty input removes the local base estimate. `org-inspect-effort` shows source
spelling and elapsed minutes, honoring only selected EFFORT inheritance.
Ordinary set/delete never silently rewrites an inherited owner.

EFFORT_ALL properties/declarations supply up to 64 validated choices through
explicit source inheritance. Tab completes a unique choice; free duration entry
remains available. `org-inc-effort` cycles those choices, retaining their source
spelling. The factory has no implicit allowed-value list; cycling without a list
reports that condition without changing source.

Duration entry is bounded to 128 UTF-8 bytes and 32 components. Signed values,
scientific notation, unknown units, fractional colon components and non-finite
results reject the whole edit. Typed positive finite scales under
`org.duration.units` default to min=1, h=60, d=1440, w=10080, m=43200 and y=525960
elapsed minutes. Colon forms retain canonical hour/minute/second arithmetic.
Work-unit scales do not perform calendar scheduling and source directives cannot
alter them or execute programs.

All edits use the existing checked transaction and history owner. Priority edits
replace only their cookie; effort edits replace only the owned property value or
record, preserving key case, indentation, tags, planning, unknown metadata,
BOM/line endings and unrelated EOF spelling. Undo restores the whole operation
and independently mapped peer points. C-g cancels drafts; changed source/view
qualifiers and read-only mutations reject. Read-only inspection stays usable.

The agenda role admits e/comma for these same operational source handlers.
Agenda row construction and checked source-row routing are defined by the
agenda owner; adding a keymap does not create a report or a task copy.
