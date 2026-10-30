# Org Mode

Status: Completed
Completed: 2026-10-30
Validated candidate: `72160e46648f3f28a9c8bb7f3eecc49173fca09d`
Validated tree: `8249daaf355af4f7be00173b5b107fb4d9c80fea`
Gate validation: `EXPY4POPOBG2CKQH6C4JTM5AVI`
Approved: product owner, 2026-10-27; specification and model accepted, with the explicitly listed open decisions retained.
Scope refinement: product owner, 2026-10-28; clocking, habits and effort approved; remaining choices follow Org in Emacs without further feature-by-feature questions. Advanced spreadsheet evaluation remains excluded.
Date: 2026-10-27
Format: OpenSpec additive capability requirements
Source: product-owner discussion and confirmed interaction decisions
Model: [Org Mode Model](editor-org-mode-model.md)
Baseline: [rev Editor](rev-editor.md)
Related: [Editor Configuration](editor-configuration.md), [Editor Gutter](editor-gutter.md), [Native Autocompletion](editor-autocompletion.md)
Delivery boundary: approved rev-org-mode delivery; one complete 64-slice Looper cycle, requested by the product owner on 2026-10-28.

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
- Task time tracking includes native clock-in and clock-out operations.
- Habits follow Org's source conventions, completion history and agenda consistency display.
- Tables support arithmetic, parentheses, cell/range references, sum, mean,
  minimum, maximum, rounding, decimal formatting and explicit recalculation.
- Full Calc and Emacs Lisp evaluation are outside the agreed table scope.
- Unknown or unsupported source forms are preserved without executing them.
- CPU/memory costs must be bounded and measured; compatibility complexity is
  not itself evidence of high runtime resource consumption.

The numbered requirements define the selected scope. The product owner has
authorized Org-in-Emacs behavior for the remaining choices; the profile below
and paired model resolve those choices. Implementation still requires the
approved, committed delivery plan and tracker.

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
Work SHALL occur in response to selected editing/query/recalculation actions,
explicit jobs or an armed timer deadline.
Large-document, multi-file and idle behavior SHALL be measured separately;
implementation complexity SHALL NOT be reported as measured CPU or memory cost.

#### Scenario: Cancel a bounded multi-file query

- **GIVEN** an agenda operation is still reading its admitted source set
- **WHEN** the user cancels or a configured limit is reached
- **THEN** the operation stops with a truthful cancelled/incomplete result
- **AND** documents, unsaved buffers and normal editor interaction remain usable.

### Requirement: ORG19 Native task time tracking

Task time tracking SHALL include compiled clock-in and clock-out operations
available through M-x and the reviewed Org bindings. Starting and closing a
clock SHALL record its task-owned interval in canonical Org source, retaining
start/end timestamps and duration according to the finalized storage policy.
Each operation SHALL preserve ordinary read-only, source-validation and
undo/redo guarantees. An active clock SHALL NOT require periodic source writes
or an idle repaint loop.

#### Scenario: Record time against a task

- **GIVEN** an editable Org task and a valid selected clock policy
- **WHEN** the user clocks into that task and later clocks out
- **THEN** the recorded interval belongs to that task and retains its elapsed time
- **AND** the source edits remain independently undoable without changing unrelated text.

### Requirement: ORG20 Habits compatible with Org behavior

Habits SHALL use unfinished tasks with `STYLE: habit`, repeating `SCHEDULED`
timestamps and completion history. Repeaters and minimum/maximum intervals,
including `.+2d/3d`, SHALL retain their Org meaning. Completion SHALL preserve
compatible history, `LAST_REPEAT` and coherent rescheduling under ORG06/ORG07.
Missing history SHALL NOT be replaced with invented completions.

The agenda SHALL derive a bounded daily consistency graph from current source,
distinguishing not-yet-due, available, imminent and overdue days, with completion
and current-day markers. Factory defaults SHALL use 21 preceding days, 7
following days, preferred graph column 40, today's agenda only, and no forced
green status for late completions. Native agenda controls SHALL toggle habit
visibility and display options; TOML SHALL expose appropriate graph preferences.
Graph inspection SHALL NOT rewrite source or introduce idle analysis.

#### Scenario: Complete a habit with an interval range

- **GIVEN** a habit scheduled with a minimum/maximum interval
- **WHEN** the user completes it
- **THEN** history and its next schedule follow the admitted Org recurrence
- **AND** undo restores the complete source transition.

#### Scenario: Inspect consistency without changing the document

- **GIVEN** recorded habit completions, including unsaved changes
- **WHEN** the user refreshes today's agenda
- **THEN** the graph reflects those observations and its configured date window
- **AND** absent history remains absent and the source is unchanged.

Behavior reference: [Org habit tracking](https://orgmode.org/manual/Tracking-your-habits.html).
Factory-default reference: [Org habit implementation](https://raw.githubusercontent.com/emacs-mirror/emacs/master/lisp/org/org-habit.el).

### Requirement: ORG21 Effort estimates and clock reports

Native commands SHALL edit EFFORT duration properties, compare estimates with
recorded task/subtree time, and produce bounded clock tables for selected
sources/date ranges. Reports SHALL derive from actual intervals, distinguish
open/incomplete records and change dynamic report bodies only on explicit
update. Source and agenda actions SHALL use the same checked handlers.

#### Scenario: Compare estimated and recorded effort

- **GIVEN** a task has an effort estimate and recorded clock intervals
- **WHEN** the user requests its time report
- **THEN** the report distinguishes planned and actual durations
- **AND** opening the report does not rewrite task source.

### Requirement: ORG22 Relative timers and countdowns

Native Org timers SHALL support start/reset, pause/continue, stop, relative-time
insertion/list items and explicit countdowns. A countdown SHALL signal once in
the editor at its deadline. Only an armed deadline or active user operation may
schedule wakeups; stopped timers SHALL add no idle polling or source writes.

#### Scenario: Finish and stop a countdown

- **GIVEN** the user has explicitly armed a countdown
- **WHEN** it expires or the user stops it first
- **THEN** expiry produces at most one notification and stop cancels it
- **AND** inactive timers retain no repaint loop.

### Requirement: ORG23 Agenda queries columns and bulk operations

Native agenda SHALL include date-span, TODO, text, tag/property and configured
block views, filtering, sorting, property columns and selected bulk actions.
The declared match grammar and typed view definitions SHALL not evaluate Lisp.
Each action SHALL preserve source provenance and disclose per-target failures;
projected columns SHALL not become another task database.

#### Scenario: Apply an action to selected agenda entries

- **GIVEN** selected entries include a stale or read-only source
- **WHEN** the user invokes a bulk action
- **THEN** the editor identifies the outcome for every selected target
- **AND** rejected targets and unrelated source remain unchanged.

### Requirement: ORG24 Native heading attachments

Native attachment commands SHALL resolve ID/DIR-based attachment locations,
copy/move/link/list/open selected files and retain compatible attachment links.
Destination collisions, symlinks and partial source/file publication SHALL use
checked native file safeguards. Recognition SHALL not create or open resources.

#### Scenario: Reject a conflicting attachment destination

- **GIVEN** the selected attachment would replace an existing file
- **WHEN** the user declines replacement
- **THEN** both file and heading source remain unchanged
- **AND** the dispatcher remains usable.

### Requirement: ORG25 Native document export

Explicit export SHALL support text, Org, Markdown, HTML, LaTeX, Beamer, ODT,
Texinfo, man and iCalendar from checked source snapshots, including declared
metadata, structure, links, tables, footnotes, blocks and bounded includes.
Native rendering SHALL not evaluate code. Optional PDF/tool conversions SHALL
be separate explicit jobs with truthful availability and destination checks.

#### Scenario: Export unsaved source without executing it

- **GIVEN** a dirty Org buffer containing a source block
- **WHEN** the user exports it to HTML
- **THEN** output reflects the checked current buffer snapshot
- **AND** neither block execution nor an implicit save occurs.

### Requirement: ORG26 Checked project publishing

Typed project definitions SHALL select bounded Org sources, export formats,
assets and publication destinations. Publication SHALL be explicit and retain
per-file outcomes, overwrite decisions and source fidelity. Optional upload
uses an explicitly configured tool; publishing SHALL not start a server or
remove unowned destination files.

#### Scenario: Publish a partially failing project

- **GIVEN** an explicit project with one unavailable source
- **WHEN** the user publishes
- **THEN** the result identifies successful and failed files truthfully
- **AND** it does not claim complete publication or discard source.

### Requirement: ORG27 Document conveniences and data interchange

Native Org SHALL provide footnote actions, entities, markup, affiliated data,
property columns, explicit dynamic reports and table/list/CSV/TSV interchange.
Unknown macros and dynamic forms SHALL remain source, with unsupported actions
identified. Supported transformations SHALL preserve unrelated text and provide
coherent undo or checked file publication.

#### Scenario: Import quoted table data

- **GIVEN** valid CSV with quoted delimiters and Unicode values
- **WHEN** the user explicitly imports it into an Org table
- **THEN** cell values retain their meaning and source boundaries
- **AND** undo restores the previous document in one operation.

### Requirement: ORG28 Source editing tangling and explicit execution

Native source-block support SHALL provide special editing, checked named/header
resolution, bounded noweb expansion and explicit tangling. Selected block/call
execution SHALL require confirmation by default and use a declared optional
external program, not an embedded editor language. Results SHALL publish only
against current source and retain prior content on failure. Child lifetime,
output, time and expansion SHALL be bounded and cancellable.

#### Scenario: Cancel a selected block execution

- **GIVEN** a confirmed block job is running in an owned temporary workspace
- **WHEN** the user cancels it
- **THEN** owned child work is stopped and reaped and the draft remains available
- **AND** existing results and unrelated documents remain unchanged.

### Requirement: ORG29 Explicit feed import

Native feed support SHALL parse bounded RSS/Atom observations and import items
into explicitly selected Org targets using compatible FEEDSTATUS identity.
Updates SHALL be on demand, use optional declared transport when needed, and
neither poll in the background nor treat fetched text as executable settings.

#### Scenario: Refresh the same feed twice

- **GIVEN** an imported feed item has a recorded stable identity
- **WHEN** the user explicitly refreshes an unchanged feed
- **THEN** the item is not duplicated
- **AND** unavailable transport or invalid data produces a truthful failure.

### Requirement: ORG30 Native previews and rich source presentation

Explicit link/image previews SHALL preserve canonical source and normal view
coordinates. Supported terminal PNG presentation SHALL stay within admitted
geometry and restore its output state; other resources SHALL retain readable
links and an explicit external-viewer path. Merely visiting source SHALL not
fetch remote content or launch viewers.

#### Scenario: Preview without terminal image capability

- **GIVEN** an Org image link in a terminal without admitted image support
- **WHEN** the user requests its preview
- **THEN** source remains readable and the editor identifies the available viewer path
- **AND** it does not emit unsupported graphics or lose the current view.

### Requirement: ORG31 Complete controls and measured delivery

Every delivered capability SHALL have working native commands, compatible
context bindings where applicable, typed bounded preferences, reference/default
coverage and meaningful acceptance evidence. The model's resource profile SHALL
bound source analysis, jobs and retained projections. The single delivery SHALL
finish with all assigned slices integrated and its exact clean candidate passing
the controller gate; catalog entries or prerequisite listings are not runtime proof.

#### Scenario: Verify the complete selected delivery

- **GIVEN** all assigned native Org slices are integrated
- **WHEN** the controller runs the full selected gate
- **THEN** required native, terminal and resource evidence covers the current candidate
- **AND** closure preserves earlier delivery provenance and reports every failure.

## Control coverage

| Concern | Configuration and/or native runtime control |
| --- | --- |
| Mode activation, default visibility, coloring and display | Startup defaults; explicit mode/visibility commands. |
| Task sequences, finished states, priorities, logging and dependency policy | Typed defaults and admitted document directives; transition/inspection commands. |
| Task time tracking | Native clock-in/clock-out commands; frozen clock defaults and explicit recovery controls. |
| Habits | Native agenda visibility/display controls; typed graph preferences and palette roles. |
| Tags and properties | Typed inheritance/default policies; set/remove/search/inspect commands. |
| Time zone, timestamp entry and repeaters | Typed policy defaults; explicit timestamp/schedule/deadline commands. |
| Agenda sources, span, filters, sort and limits | Configurable sources/defaults; native dispatcher, refresh and runtime view controls. |
| Capture | Native factory/configured templates; choose, edit, finalize and abort commands. |
| Refile and archive | Configurable target policy; explicit selection and action commands. |
| Tables and formulas | Native editing/recalculation commands; documented numeric/format/limit preferences. |
| Effort and reports | Native EFFORT/clock-table/property-column commands and typed report controls. |
| Timers | Native relative/countdown actions, inactive by default. |
| Attachments and feeds | Typed destinations/sources; explicit native dispatch/update. |
| Export and publishing | Native formats, typed project/options data and explicit checked publication. |
| Source blocks and optional tools | Native edit/tangle/actions, confirmation and bounded executable/argv profiles. |
| Previews | View-local native controls and admitted optional viewer paths. |
| Bindings and named operations | Validated context keymaps and one compiled M-x catalog. |

This matrix records the agreed control principle. The selected profile and
paired model define the defaults, typed fields and bounded policies; shipped
commands, parser/catalog and the complete sample must agree.

## Selected Org behavior

The remaining product choices follow Org in Emacs as authorized on 2026-10-28.
The reproducible local reference is Emacs 31.1 with Org 9.8.7, loaded with no user
configuration. Manual/source references explain behavior; no Org implementation
code or cross-project Org crate is reused.

The selected scope includes the numbered requirements, including time reports,
effort, timers, advanced agenda views, attachments, feeds, export/publishing,
source editing/tangling and explicit optional tool execution. The earlier
spreadsheet decision remains: native arithmetic/aggregates/rounding only, with
no complete Calc or table-formula Lisp evaluator. A reference to Org does not
turn its Emacs runtime, arbitrary Lisp hooks or third-party extensions into a
new editor programming layer.

### Factory behavior

- TODO workflow: TODO then DONE; A/B/C priorities, default B. Normal completion
  logging is off; repeating-task completion records time. Tag inheritance is
  on and general property inheritance off. Recognized local declarations win
  for their data semantics; they cannot grant executable authority.
- Initial outline visibility is show-all. TAB and S-TAB use contextual/local
  and whole-view cycling. M-RET, heading moves and the previously listed keys
  follow their actual structural context, including lists, tables and timers.
- Agenda starts with a Monday-based week. No directory is scanned implicitly.
  With no agenda sources configured, the dispatcher asks for source selection;
  current eligible buffers can be selected without first saving them.
- Capture uses the native Task fallback when no custom template exists. A
  missing destination is requested explicitly; no sample path is manufactured.
  Data-only tokens include %?, %i, %a, %l, %t/%T, %u/%U, %n, %c, %x,
  %^{prompt} and bounded time formatting. Function/Lisp template escapes are
  retained and rejected explicitly, not executed as editor configuration.
- Archive destination is %s_archive::. Refile targets use admitted agenda/source
  headings and explicit selection; ambiguity never picks an arbitrary heading.
- One active task clock belongs to the editor session. Clocking another task
  closes the previous interval; reaching DONE closes its clock. LOGBOOK is the
  default clock drawer, zero-minute intervals are retained, persistence is off.
  Quit offers closing the clock or preserving its open source interval. Optional
  persistence and explicit incomplete/idle/overlap resolution retain checked
  source identity and never silently count time the user rejected.
- Durations accept clock notation and Org min/h/d/w/m/y units: 1, 60, 1440,
  10080, 43200 and 525960 minutes. Scheduling month/year arithmetic remains
  calendar arithmetic; habit graph duration/day semantics follow Org.
- Timers provide relative and countdown modes, insertion and description-list
  continuation, pause/continue and stop. No countdown is armed by default;
  expiry is an in-editor notice, not a new desktop-notification dependency.
- Date entry captures the system zone by default; explicit configured IANA/UTC
  policy wins. Gregorian invalid dates reject. Ambiguous/nonexistent local
  times request an offset/valid time instead of inventing one. Untouched date
  spelling remains source, and completion/agenda operations capture time once.
- Habit graph defaults remain those in ORG20. K toggles agenda habits; named
  native controls expose today-only and all-today options without adding a
  general scripting or numeric-prefix language.
- Effort, clock tables, property columns, custom/block agendas and bulk actions
  follow their native Org meanings. The supported query language admits tag
  Boolean expressions, parentheses, TODO suffixes and typed property comparisons;
  regexp search is bounded and cannot invoke Lisp predicates.
- Attachments use ID/DIR and the ordinary attachment dispatcher. Feed updates
  use RSS/Atom plus FEEDSTATUS. Both are explicit operations; configured sources
  are never discovered by a background service.
- Native export formats are text/ASCII, UTF-8, Org, Markdown, HTML, LaTeX,
  Beamer, ODT, Texinfo, man and iCalendar. Region/subtree options, safe includes,
  text-only macros, names/captions and declared attributes are supported. PDF,
  upload, network fetch and external preview use optional configured tools.
- Source blocks support native editing, names/headers, noweb and tangling.
  Optional confirmed execution can use installed language programs, including
  Emacs for a selected emacs-lisp block; none becomes rev's editor runtime.
  No file visit, highlighting, export, include or table recalculation executes a
  block. Explicit :eval no/never restrictions always apply.

### Basic formula profile

TBLFM admits field/column assignments, absolute/relative cell references,
rectangular ranges, + - * / ^, parentheses, unary signs, vsum/vmean/vmin/vmax,
round and abs. Exponentiation associates right; multiplication/division precede
addition/subtraction. round halves away from zero, matching the reference.
Finite binary64 arithmetic is the bounded native numeric representation;
default result text uses at most 12 significant decimal digits. Supported
fixed decimal formatting is explicit and bounded; this is not arbitrary-
precision Calc compatibility. Empty cells are zero for aggregates and direct
arithmetic; nonnumeric cells are omitted by aggregates and reject direct
numeric use. Division by zero, non-finite results, invalid references, cycles
and exhausted budgets reject the complete selected update. Existing results
and unsupported formula source remain intact. Structural table changes rewrite
only understood references or reject before mutation.

### Native command families and bindings

Compiled command families cover org-mode/visibility; heading/list/drawer/tag/
property/task operations; timestamp/schedule/deadline; clock/effort/timer/habit;
agenda/source/query/columns/bulk; capture/refile/archive; links/attachments/
footnotes; table/formula/interchange; export/publish; source-edit/tangle/execute/
results; feeds and previews. Use source-compatible org-* spellings and native
context dispatchers, not commands whose bodies are arbitrary configured code.
Only operational commands may enter M-x, defaults or the shipped reference.

In addition to the established bindings, add:

| Context | Keys | Native action |
| --- | --- | --- |
| Org | C-c C-x C-i / C-c C-x C-o / C-c C-x C-q | Clock in / out / cancel. |
| Org | C-c C-x e / C-c C-x C-e | Set effort / modify the active task estimate. |
| Org | C-c C-x 0 / C-c C-x ; | Relative timer / countdown. |
| Org | C-c C-x . / C-c C-x - | Insert relative time / timed list item. |
| Org | C-c C-x , / C-c C-x _ | Pause/continue / stop timer. |
| Org | C-c C-e / C-c C-a | Export / attachment dispatcher. |
| Org | C-c C-l / C-c C-o / C-c C-x f | Insert link / follow / footnote action. |
| Org | C-c C-v t / C-c C-v e / C-c ' | Tangle / execute selected block / special edit. |
| Org | C-c C-x g / C-c C-x G | Update feeds / visit selected feed target. |
| Agenda | r / g / q / RET / TAB | Refresh current/all, return, switch source, visit source. |
| Agenda | d / w / f / b / . | Day / week / next span / previous span / today. |
| Agenda | t / e / K / ; | State / effort / habits / countdown. |
| Agenda | C-c C-x C-i / C-c C-x C-o | Clock into/out of the checked source task. |
| Capture or source edit | C-c C-c / C-c C-k | Finalize / abort the current checked draft. |

C-c C-c is precise-context dispatch, not an ancestor fallback. C-g always wins
in transient interaction. C-c C-x C-r is Org's radio-checkbox action in the
reference; do not repurpose it as a guessed clock-report binding. Optional
prefix variants are finite reviewed context bindings; do not introduce an
unbounded universal-argument subsystem as an incidental requirement.

### Behavior references

- [Effort estimates](https://orgmode.org/manual/Effort-Estimates.html)
- [Clocking](https://orgmode.org/manual/Clocking-Work-Time.html)
- [Timers](https://orgmode.org/manual/Timers.html)
- [Agenda commands](https://orgmode.org/manual/Agenda-Commands.html)
- [Capture templates](https://orgmode.org/manual/Capture-templates.html)
- [Attachments](https://orgmode.org/manual/Attachments.html)
- [Feeds](https://orgmode.org/manual/RSS-Feeds.html)
- [Export](https://orgmode.org/manual/Exporting.html)
- [Publishing](https://orgmode.org/manual/Publishing.html)
- [Code blocks](https://orgmode.org/manual/Evaluating-Code-Blocks.html)
- [Images](https://orgmode.org/manual/Images.html)

## Validation obligations

Cover plain-text fidelity, malformed recovery, Unicode/tab/CRLF boundaries,
independent folding, exact structural edits and their undo/redo, inheritance,
task/log transitions, habit history/graphs, repeaters and time-zone edge cases. Verify real-file
agenda/capture/refile failures, dirty buffers, stale destinations, cancellation
and truthful incomplete/partial outcomes. Verify table structural edits,
formula success/failure, unsupported-expression preservation, numeric policy
and one-operation undo. Exercise the familiar keys and matching M-x commands
through real terminal journeys, plus configuration/default/sample consistency.

Measure large-document, many-source, bounded calculation and idle workloads.
This specification contains acceptance obligations, not runtime test results.

## Baseline and evidence reconciliation

The [editor](rev-editor.md), [text/view model](rev-editor-model.md),
[configuration](editor-configuration.md), [configuration model](editor-configuration-model.md)
and [interaction](editor-interaction.md) baselines now reference the native Org
amendments without changing their earlier completed status or tested candidates.
The [capability and evidence map](editor-org-mode-evidence.md) connects every
requirement/scenario to native tests, owned terminal journeys and the applicable
shared or dedicated resource profile. Its prerequisite resolution is distinct
from actually executing those assertions on the final integrated candidate.
The [user control catalog](../../../docs/reference/org-controls.md) links all
shipped controls and their fixed/native versus typed startup policies.

## Delivery state

The original specification/model and later behavior refinements record the
product decisions. The approved [delivery plan](../plan/rev-org-mode.md) and
[tracker](../tracker/rev-org-mode.md) organize one 64-slice cycle. The product owner explicitly requested starting this single cycle on
2026-10-28. Planning checks are not implementation evidence. Technical defaults and bounds are explicit in the
paired model, and the advanced-spreadsheet exclusion is unchanged.

## Delivery acceptance

The controller completed `scripts/check-all.sh` on immutable clean candidate
`72160e46648f3f28a9c8bb7f3eecc49173fca09d` (tree `8249daaf355af4f7be00173b5b107fb4d9c80fea`),
receipt `EXPY4POPOBG2CKQH6C4JTM5AVI`. All eleven ordered commands passed: 1716
native tests, all 34 ORG01-ORG31 scenarios, four baseline terminal journeys,
10 configuration and 59 owned Org terminal sessions, baseline/configuration/Org
release workloads, complete control/reference/template agreement and zero skips.
All 64 canonical slice PRs, 192 task outcomes and repair PR #166 are integrated.

Later documentation closure retains that tested candidate and receipt. Its own
Git head identifies documentation only; runtime evidence is not relabeled as
belonging to the later head. Earlier editor/configuration/input candidates and
separate Slice 61/62 observations retain their original identities.

The [Slice 64 report](../report/slices/rev-org-mode/slice-64-delivery-acceptance.md)
records the exact commands, retained failure and correction, actual resource
observations and compatible optional-tool boundaries.
