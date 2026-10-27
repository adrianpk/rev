# Org Mode

Status: accepted
Approved: product owner, 2026-10-27; specification and model accepted, with the explicitly listed open decisions retained.
Date: 2026-10-27
Format: OpenSpec capability proposal
Source: product-owner discussion and confirmed interaction decisions
Model: [Org Mode Model](editor-org-mode-model.md)
Baseline: [rev Editor](rev-editor.md)
Related: [Editor Configuration](editor-configuration.md), [Editor Gutter](editor-gutter.md), [Native Autocompletion](editor-autocompletion.md)
Delivery boundary: future work; the active configuration/UI delivery is unchanged

## Purpose

Provide Org as a native editing and work-organization mode in rev. The mode
includes structured editing, drawers, tags, properties, timestamps, tasks,
agenda, capture and useful table calculations. Syntax coloring is part of the
editing experience, not a substitute for these workflows.

Rev remains a lightweight Emacs-like terminal editor. Org behavior is compiled
into the Rust binary and developed within rev. It requires no editor addon,
shared cross-project library or programmable command layer. Optional LSP is
not required for Org editing, agenda, capture or table calculations.

This specification describes observable behavior and controls. It does not
prescribe another editor's architecture, runtime composition or module layout.

## Decisions already recorded

- Existing plain-text documents remain the canonical source of truth.
- Editing, navigation and M-x follow familiar Emacs-like interaction.
- Org commands, keybindings and preferences have native implementations with
  useful factory behavior and appropriate TOML/runtime controls.
- Folds and other presentation choices belong to each view, not document text.
- Agenda and capture operate on real document locations and preserve unsaved work.
- Tables support arithmetic, parentheses, cell/range references, sum, mean,
  minimum, maximum, rounding, decimal formatting and explicit recalculation.
- Full Calc and Emacs Lisp evaluation are outside the agreed table scope.
- Unknown or unsupported source forms are preserved without executing them.
- CPU/memory costs must be bounded and measured; compatibility complexity is
  not itself evidence of high runtime resource consumption.

The numbered requirements define the accepted behavioral direction. Exact
factory defaults and remaining scope choices are listed explicitly at the end.
Those decisions remain open; specification approval is not implementation
authorization.

## Familiar interaction

These default bindings apply in Org editing context. They invoke the same
native command identities that M-x discovers. An active prompt, agenda,
capture or candidate list uses its own appropriate transient context; C-g
remains cancellation. Outside Org, ordinary editor bindings retain their meaning.

| Input | Org behavior |
| --- | --- |
| `TAB` | Cycle the current heading's visibility; use the admitted contextual table action in a table. |
| `S-TAB` | Cycle whole-view overview, contents and show-all. |
| `M-RET` | Insert a sibling heading after the current heading. |
| `M-Left`, `M-Right` | Promote/demote the heading and its descendants coherently. |
| `M-Up`, `M-Down` | Move the current subtree between siblings in structural context. |
| `C-c C-n`, `C-c C-p` | Visit the next/previous visible heading. |
| `C-c C-f`, `C-c C-b` | Visit the next/previous same-level heading. |
| `C-c C-u` | Visit the parent heading. |
| `C-c C-c` | Perform the admitted action for the precise current context. |
| `C-c C-t` | Change task state using the configured workflow. |
| `C-c C-s` | Set or change scheduling. |
| `C-c C-d` | Set or change a deadline. |
| `C-c C-q` | Edit the current heading's locally stored tags. |
| `C-c C-w` | Refile the current complete subtree. |
| `C-c C-x p` | Set a locally owned property. |
| `C-c C-x C-a` | Archive the current subtree to an admitted destination. |
| `C-c C-x a` | Toggle the compatible internal archive state. |
| `C-c` then `\|` | Create a table through the native table workflow. |
| `C-c l` | Store a link to the checked current location. |
| `C-c a` | Open the agenda dispatcher. |
| `C-c c` | Start capture. |

Agenda and capture are also available by their M-x command identities from
ordinary non-Org editing buffers. Their compact bindings above remain Org-local
unless the user explicitly supplies global aliases.

The command catalog must also expose direct visibility choices, drawer folding,
list editing, checkbox operations, property/tag inspection, timestamp entry,
link following, table navigation/alignment/row-column editing, recalculation,
agenda refresh/source navigation and capture finalize/abort. Exact spellings
and additional compact bindings require the catalog review below. No invented
prefix family or unrelated reassignment is implied.

## ADDED Requirements

### Requirement: ORG01 Native activation and complete working mode

Opening an admitted `.org` document or explicitly selecting Org SHALL enable
its native editing behavior. Structured editing, planning, agenda, capture and
basic table calculations SHALL be available without an editor package or LSP.
Absent user configuration SHALL select useful factory behavior; an operation
needing a destination or source set SHALL ask for that input explicitly.

#### Scenario: Open and edit without setup

- **GIVEN** no user configuration or language-server process
- **WHEN** the user opens an Org document and invokes a supported native operation
- **THEN** the operation is available through the built-in interaction
- **AND** the operation does not require installing an editor extension.

### Requirement: ORG02 Source fidelity and recovery

Ordinary opening, analysis, coloring, folding and saving SHALL preserve
untouched source bytes, line endings, BOM/final-newline state, spacing and
unknown forms under the existing file contract. Malformed structures SHALL
remain editable and identifiable. A semantic command SHALL reject an invalid
or stale target rather than guess and rewrite unrelated text. Opening or
inspecting a document SHALL NOT evaluate formulas or executable source.

#### Scenario: Preserve an unfamiliar form

- **GIVEN** a document containing an unknown directive and a malformed block
- **WHEN** the user edits an unrelated heading and saves
- **THEN** the unaffected original source remains intact
- **AND** analysis neither executes the block nor normalizes the document.

### Requirement: ORG03 Independent outline and drawer visibility

Local heading visibility SHALL cycle through heading-only, direct children
and complete subtree. Global visibility SHALL cycle through overview, contents
and show-all, with direct commands for each state. Drawer folding SHALL hide
only its body while retaining its delimiters. Visibility SHALL be view-local.
Entering hidden source for editing/navigation SHALL reveal the relevant range
or reject safely; hidden source SHALL NOT alias another visible position.

#### Scenario: Fold one shared view

- **GIVEN** two views display the same document
- **WHEN** the user folds a heading or drawer in one view
- **THEN** only that view changes visibility
- **AND** canonical text, undo state and the other view remain unchanged.

### Requirement: ORG04 Coherent structural editing

Heading and list operations SHALL insert, select, move, promote, demote,
copy or remove complete admitted structures while preserving descendants,
continuation paragraphs, drawers, blocks and table metadata. A hierarchy
change SHALL update related levels coherently. Structural mutations SHALL be
single checked undo operations and respect read-only state. Motion SHALL use
real logical targets and report a missing target without approximate movement.

#### Scenario: Move a complete subtree

- **GIVEN** a heading owns children, a drawer and a table
- **WHEN** the user moves its subtree after a sibling and then undoes
- **THEN** the complete owned source moves and returns as one operation
- **AND** no descendant or attached metadata is left behind.

### Requirement: ORG05 Drawers, properties and tags

Org SHALL distinguish ordinary drawers from property drawers in their valid
contexts. Tag/property operations SHALL distinguish locally stored values from
inherited or derived values. Tag matching SHALL preserve case sensitivity.
Configured inheritance SHALL be computed without inserting inherited text into
children. Editing inherited data SHALL identify its actual owner. Stored
property values and additive forms SHALL preserve their compatible source.

#### Scenario: Inspect inherited metadata

- **GIVEN** a child receives tags or properties through an admitted parent policy
- **WHEN** the user inspects the child and edits a local tag
- **THEN** inspection distinguishes local and inherited membership
- **AND** editing the local value does not silently rewrite the parent.

### Requirement: ORG06 Tasks, priorities and checkboxes

Task keywords SHALL follow an ordered configured workflow, including
unfinished and finished groups and admitted document-local definitions.
Unknown leading words SHALL remain title text. Task transitions, selected
logging, closure timestamps and progress-cookie updates SHALL form one coherent
edit. Priorities and unchecked/checked/intermediate checkbox states SHALL keep
their distinct meaning. Any selected completion-blocking policy SHALL explain
unsatisfied conditions before mutation.

#### Scenario: Complete a task coherently

- **GIVEN** a task has an admitted finished state and configured closure logging
- **WHEN** the user selects that finished state and then undoes
- **THEN** the task and its related closure/log changes are one undo operation
- **AND** unrelated heading metadata is preserved.

### Requirement: ORG07 Calendar timestamps and repeaters

Timestamp entry SHALL support active/inactive forms, dates, optional time,
ranges, scheduling, deadlines, repeaters and warning delays with compatible
stored syntax. Calendar validation SHALL be explicit. Zone-less values SHALL
use one captured effective time zone for the operation; invalid or ambiguous
local times SHALL follow a documented policy or request a choice.

Repeater behavior SHALL remain distinct: `+N` advances one interval from the
stored base, `++N` advances whole intervals until future, and `.+N` advances
one interval from completion time. Related task/timestamp/log changes SHALL
publish coherently. Merely querying an agenda SHALL NOT advance source dates.

#### Scenario: Complete an overdue repeating task

- **GIVEN** a task uses an admitted repeating timestamp and captured completion time
- **WHEN** the user completes it
- **THEN** the next date follows the selected repeater semantics
- **AND** the transition and related log updates are reversible together.

### Requirement: ORG08 Bounded agenda sources and truthful results

Agenda sources SHALL be explicitly selected files or bounded declared
directories. Agenda queries SHALL use current unsaved buffers when present
and checked file observations otherwise. Sources SHALL be deduplicated by
real document identity. Daily/weekly, unfinished-task and tag/property views
SHALL combine source facts without becoming a second editable task store.
Unavailable sources and truncated results SHALL remain visible; neither SHALL
be reported as a successful complete empty agenda.

#### Scenario: Keep unsaved work visible in the agenda

- **GIVEN** an agenda source has an open buffer with unsaved task/date changes
- **WHEN** the user refreshes its agenda
- **THEN** the current admitted buffer state is represented
- **AND** the agenda does not substitute older disk content.

#### Scenario: Report a partial source set

- **GIVEN** one selected source is unreadable
- **WHEN** the user queries the agenda
- **THEN** available entries remain usable with the source failure identified
- **AND** the view does not claim completeness.

### Requirement: ORG09 Checked agenda navigation and actions

Agenda entries SHALL retain their real source location. Navigation SHALL
re-resolve stale evidence before moving. Supported state, priority, tag,
schedule, deadline, refile and archive actions SHALL act on the source through
ordinary checked edits, never by editing cached agenda rows. Refresh, date/span
movement, sorting/filtering and close/return SHALL preserve valid document and
view state. Any bulk behavior SHALL have explicit selection and truthful
per-target outcomes before it is offered.

#### Scenario: Act on an entry whose source changed

- **GIVEN** an agenda row refers to an older source revision
- **WHEN** the user requests a task transition from that row
- **THEN** the target is re-resolved or the action is rejected visibly
- **AND** no unrelated source entry is edited.

### Requirement: ORG10 Capture with safe finalize and abort

Capture SHALL be available without first visiting its destination. Native
factory/configured templates SHALL declare entry kind, target and bounded
text/prompt/time/selection/link substitutions. Starting capture SHALL collect
the required input and retain an editable draft. Finalize SHALL validate the
current destination and publish one coherent source edit. Abort SHALL discard
the draft and leave the target unchanged. Template text SHALL NOT become an
executable configuration language. Successful insertion and durable save SHALL
remain distinguishable under the ordinary file lifecycle.

#### Scenario: Abort capture

- **GIVEN** an edited capture draft targets an existing Org entry
- **WHEN** the user aborts or cancels capture
- **THEN** the original destination remains unchanged
- **AND** the draft is discarded without a partial insertion.

#### Scenario: Reject a stale capture destination

- **GIVEN** a capture draft was prepared against a destination that changed
- **WHEN** the user finalizes
- **THEN** the destination is revalidated or the draft is retained with a clear failure
- **AND** source is not overwritten under stale assumptions.

### Requirement: ORG11 Refile and archive without silent loss

Refile/copy SHALL target complete subtrees and validate source/destination
identity before changing either. Cross-document operations SHALL provide an
atomic visible result or an explicitly recoverable outcome that identifies
what changed. They SHALL NOT silently lose or duplicate the subtree.
Internal archive state SHALL preserve source in place and influence agenda
inclusion explicitly. External archiving SHALL retain the admitted origin
information needed for navigation. No operation SHALL claim a successful save
when only an in-memory edit succeeded.

#### Scenario: Refile to an unavailable destination

- **GIVEN** a valid source subtree and an unreadable or conflicting destination
- **WHEN** the user attempts refile
- **THEN** the source is preserved or a precisely recoverable partial outcome is reported
- **AND** the operation does not report silent success or discard content.

### Requirement: ORG12 Source-preserving table editing

Native table editing SHALL navigate cells/rows, align presentation/source,
insert/remove/move rows and columns, handle separator rows and edit cell text.
Alignment SHALL preserve cell values. Structural edits SHALL retain associated
names, captions and formula lines; admitted reference rewrites SHALL keep their
intended targets or report unsupported transformation before publication.
Copying table content SHALL exclude display-only decorations. Each structural
operation SHALL be one checked undo unit.

#### Scenario: Move a table column

- **GIVEN** a table has cell values and associated formulas
- **WHEN** the user moves a column and undoes
- **THEN** values and admitted references move coherently and return together
- **AND** unrelated affiliated source is preserved.

### Requirement: ORG13 Basic formulas and explicit recalculation

The native formula subset SHALL include arithmetic, parentheses, cell/range
references, sum, mean, minimum, maximum, rounding and decimal formatting.
Formulas SHALL be representable in compatible `#+TBLFM` source for a field or
column. Recalculation SHALL be an explicit native command, including access
through M-x; opening, saving or coloring SHALL NOT evaluate formulas.

Before changing result cells, recalculation SHALL capture the source, resolve
admitted references, evaluate within finite bounds, format proposed results
and validate all targets. The selected operation SHALL publish as one undo
unit. It SHALL preserve previous results on failed all-or-nothing evaluation.

#### Scenario: Recalculate a column

- **GIVEN** a table stores the admitted formula `$4=$2*$3`
- **WHEN** the user explicitly recalculates the table
- **THEN** column four receives the admitted product results
- **AND** undo restores the prior cell text as one operation.

### Requirement: ORG14 Truthful calculation errors and compatibility

Unsupported formulas SHALL remain intact and report their unavailable
operation on recalculation. Full Calc and Emacs Lisp evaluation SHALL NOT be
claimed or substituted. Numeric precision, rounding, empty/non-numeric cells,
invalid references, division by zero, dependency ordering/cycles and evaluation
limits SHALL be defined before the subset is finalized. A malformed or
unsupported operation SHALL NOT silently produce a plausible numeric value.
Formatting SHALL NOT change unrelated cells or rewrite the user's formula.

#### Scenario: Preserve a formula outside the native subset

- **GIVEN** a table contains an unsupported Lisp-form or advanced expression
- **WHEN** the user opens/saves the file and later requests recalculation
- **THEN** the original formula is preserved and unsupported evaluation is identified
- **AND** the editor does not run code or silently replace existing results.

### Requirement: ORG15 Links and checked destinations

Org SHALL preserve link syntax, descriptions and targets and offer native
store/follow/navigation commands for the admitted link kinds. Internal and file
locations SHALL resolve through real documents and checked targets. Following
is an explicit action; recognition or highlighting SHALL NOT open files,
applications or network resources. Invalid or ambiguous destinations SHALL be
reported without discarding the current editing state.

#### Scenario: Follow an internal target

- **GIVEN** a document contains a supported internal link and an identifiable target
- **WHEN** the user invokes its native follow action
- **THEN** the checked target is revealed and visited
- **AND** an unresolved target would leave the current text intact.

### Requirement: ORG16 Semantic coloring and source presentation

Headings, task states, tags, properties, drawers, timestamps, links, blocks
and tables SHALL receive meaningful semantic presentation using rev's palette.
Org coloring SHALL work without an LSP. Presentation and folding SHALL preserve
canonical source and logical coordinates; syntax inside literal blocks SHALL
not be mistaken for surrounding Org structure. Unsupported presentation SHALL
retain readable editable source.

#### Scenario: Display a literal example

- **GIVEN** an example/source block contains text resembling an Org heading
- **WHEN** the document is colored and its surrounding outline is navigated
- **THEN** literal block content retains its correct context
- **AND** the apparent heading is not treated as an outer task or subtree.

### Requirement: ORG17 Native command and configuration consistency

Every supported operation SHALL have a concrete compiled M-x command.
Familiar bindings SHALL invoke those same handlers in Org context, with
transient input and cancellation taking precedence. Contextual action SHALL
select the exact admitted form; an unavailable deeper form SHALL NOT fall
through to an unrelated destructive ancestor action.

Appropriate TOML preferences and runtime commands SHALL control workflows and
presentation. Startup defaults SHALL not require a config file. Admitted
source-local Org directives SHALL affect only their documented data semantics,
never execute configuration or change unrelated editor authority. Keymaps,
command identities, types/defaults/bounds and the exhaustive sample SHALL be
reviewed together before implementation planning.

#### Scenario: Use the same operation by key and name

- **GIVEN** equivalent Org sessions with the same source and view state
- **WHEN** one uses a familiar binding and the other its M-x command
- **THEN** both perform the same admitted semantic operation
- **AND** unrelated global key meanings and transient cancellation are preserved.

### Requirement: ORG18 Bounded work and ordinary editor guarantees

Org SHALL preserve ordinary text editing, safe-save, read-only, undo/redo and
terminal cleanup guarantees. Agenda discovery, indexing, capture expansion,
formula evaluation and retained result sets SHALL have finite documented bounds
and cancellation where work can be long. Limits SHALL produce truthful
incomplete/failure states without losing source.

Org SHALL add no permanent analysis process or idle agenda/recalculation loop.
Work SHALL occur in response to the selected editing/query/recalculation action.
Large-document, multi-file and idle behavior SHALL be measured separately;
implementation complexity SHALL NOT be reported as measured CPU or memory cost.

#### Scenario: Cancel a bounded multi-file query

- **GIVEN** an agenda operation is still reading its admitted source set
- **WHEN** the user cancels or a configured limit is reached
- **THEN** the operation stops with a truthful cancelled/incomplete result
- **AND** documents, unsaved buffers and normal editor interaction remain usable.

## Control coverage to refine

| Concern | Configuration and/or native runtime control |
| --- | --- |
| Mode activation, default visibility, coloring and display | Startup defaults; explicit mode/visibility commands. |
| Task sequences, finished states, priorities, logging and dependency policy | Typed defaults and admitted document directives; transition/inspection commands. |
| Tags and properties | Typed inheritance/default policies; set/remove/search/inspect commands. |
| Time zone, timestamp entry and repeaters | Typed policy defaults; explicit timestamp/schedule/deadline commands. |
| Agenda sources, span, filters, sort and limits | Configurable sources/defaults; native dispatcher, refresh and runtime view controls. |
| Capture | Native factory/configured templates; choose, edit, finalize and abort commands. |
| Refile and archive | Configurable target policy; explicit selection and action commands. |
| Tables and formulas | Native editing/recalculation commands; documented numeric/format/limit preferences. |
| Bindings and named operations | Validated context keymaps and one compiled M-x catalog. |

This matrix records the agreed control principle. Exact keys and defaults are
not frozen by this specification. The paired model records the source and preference
shapes that must stay consistent.

## Choices still owned by the product owner

### Scope to confirm

- How far to take clocking/time reports, effort estimates, habits, timers and
  reminders. Their source should remain readable regardless of selected actions.
- Additional agenda views, property columns, Boolean/query syntax and bulk actions.
- Attachments, feeds, export, publishing, source-block execution and rich previews.
  These remain discussion topics, not automatically included or removed.
- Detailed list/table import/export, footnotes and other document conveniences.

### Behavior to finalize

- Factory task sequence, logging/dependency defaults and admitted source-local
  settings. Preserve existing user workflow declarations without guessing them.
- Default agenda view/source selection, archive location and capture templates/
  target selection when no user config exists.
- Calendar/zone selection precedence, ambiguous/invalid dates and formatting.
- The exact basic formula grammar, function names, mixed multiplication/division
  precedence, rounding mode/precision, empty-cell treatment and cycle handling.
- Contextual `TAB`/`M-RET` behavior in lists, plain text and tables; capture,
  agenda and recalculation shortcuts; link-follow/source-result precedence.
- Full command names and context bindings, without inventing another prefix
  family or exposing functional-looking unsupported commands.
- Finite source/result/time/memory bounds and representative performance budgets.

No item in these lists is decided solely because it was recognized in source
syntax or appeared in another editor. Resolve choices through discussion before
finalizing the corresponding implementation scope.

## Validation obligations

Cover plain-text fidelity, malformed recovery, Unicode/tab/CRLF boundaries,
independent folding, exact structural edits and their undo/redo, inheritance,
task/log transitions, repeaters and time-zone edge cases. Verify real-file
agenda/capture/refile failures, dirty buffers, stale destinations, cancellation
and truthful incomplete/partial outcomes. Verify table structural edits,
formula success/failure, unsupported-expression preservation, numeric policy
and one-operation undo. Exercise the familiar keys and matching M-x commands
through real terminal journeys, plus configuration/default/sample consistency.

Measure large-document, many-source, bounded calculation and idle workloads.
This specification contains acceptance obligations, not runtime test results.

## Delivery state

The specification and model are accepted and remain isolated from the active
delivery. The listed open decisions must be resolved before implementation
planning depends on them.
No implementation, dependencies, slice plan, tracker, controller, commit,
publication or remote copy is created by writing these documents.
