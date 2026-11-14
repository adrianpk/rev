# Contextual Shortcut Review

Status: Semantic review complete; exact amended mapping accepted, runtime pending
Date: 2026-11-14
Mode audit candidate: `f9510fc851b8` (documentation closure after Slice 1)
Native source tree: `442765af9faac4351c208b2bfc5ca02c1b896331`
Spec: [Factory Shortcut Alignment](../../spec/editor-factory-shortcut-alignment.md)
Matrix: [All operation dispositions](semantic-matrix.md)
Proposal: [Mapping contract](mapping-contract.md)

## Evidence boundary

The mode audit classifies 238 Org, 119 Git and 17 Markdown public names. The
core audit's 333 rows and six prompt actions remain part of the same inventory.
Aliases, inherited entries and point-dependent dispatch remain distinct from
independent operations. The shipped source and defaults are unchanged.

[reference-modes.json](reference-modes.json) records 374 name/family queries,
1,080 contextual factory-route lookups, public function summaries and isolated
editing effects. Its reference process uses `emacs -Q --batch` and explicitly
loads installed package directories; it reads no personal init or keymap.
References are GNU Emacs 31.1, Org 9.8.7, Markdown mode 2.8-alpha at revision
`f8d1be7cd5dfd64c0e4f88c29a3f84408cde0b47`, and Magit 4.5.0 at revision
`c800f79c2061621fde847f6a53129eca0e8da728`. The artifact stores source hashes.

Static `nil`, `PREFIX`, `INCOMPLETE` and absent same-name functions are lookup
results, not capability findings. Agenda's reference `v` reads a following
choice inside its dispatcher; a two-key lookup is not an execution test.
Magit's family entries often open transients where the native command prepares
one checked action. Those names alone do not establish an interchangeable
shortcut. The family decisions below use the native handler and source target.

[mode-audit.py](mode-audit.py) and [mode-probe-results.json](mode-probe-results.json)
retain native effects, normalized proposals and explicit limits. All fixtures,
drafts and the local repository belong to the probe. No external transport,
personal catalog, publication or source runner is used. Input events are native
events, not physical terminal delivery. The ordinary exact-HEAD validation is separate from this evidence. Product
approval is recorded in the contract; runtime acceptance remains pending.

## Org families

### H01: Activation, visibility and heading structure

Keep native mode selection, view-local folding, startup visibility and heading
navigation. TAB is a dispatcher: heading/drawer visibility and table field
movement have separate branches. M-RET selects a sibling heading, list
continuation or the admitted table branch. Meta arrows preserve complete native
subtrees, list ownership or table structure at point. They do not promise all
reference body-indentation, region-conversion or syntax branches.

Heading decorations and visual indentation change presentation, not source
whitespace; they are native named controls. Do not assign a language indentation
key to these operations. Existing user overrides and selected-view ownership
stay authoritative in nonmodal and all six modal editing states.

Owners: [outline dispatch](../../../../src/session/outline.rs),
[heading operations](../../../../src/session/outline/headings.rs),
[subtrees](../../../../src/session/outline/subtree.rs),
[visibility](../../../../src/view/visibility.rs).
Validation case: admitted heading/drawer/table branch; coherent subtree source
and view state; counts and read-only rejection without a partial edit.

### H02: Lists, checkboxes and contextual dispatch

Keep native item continuation, list-tree indentation, movement, renumbering,
checkbox/radio operations and statistics repair. Helpers reached through M-RET,
Meta arrows or C-c C-c are indirectly reachable even without a literal helper
target in the raw factory map. C-c C-c is not a single global operation: its
list, checkbox, timestamp, property and table branches need their own source
admission. Plain text must not inherit an unrelated structural effect.

M-S-RET is only a list equivalent. The reference inserts a new checkbox list
item at a list and a TODO heading at a heading. Native checkbox insertion works
at the list and refuses the heading. Keep that partial equivalence explicit;
adding a heading branch is a separate behavior requirement.

Owners: [list dispatch](../../../../src/session/outline/list.rs),
[checkbox structure](../../../../src/org/list/checkbox/structural.rs),
[table context dispatch](../../../../src/session/tables.rs).
Evidence: paired reference list/heading effects and native checkbox/list probes.
Runtime cases must retain one checked edit, undo and rejected point contexts.

### H03: Properties, tags, tasks, priorities and effort

Keep existing task/property/tag keys, explicit local-versus-inherited property
ownership, workflow choices and inspection commands. Inspection does not edit
the source. Native effort cycling uses declared valid EFFORT_ALL choices and
wraps; missing choices remain a diagnostic rather than a fabricated estimate.
P05 proposes C-c C-x E through that existing owner.

Agenda wrappers have different reference names. For example, native `org-todo`,
`org-schedule`, `org-deadline`, `org-set-effort`, tags and clocks operate on the
selected agenda entry through the checked source-action owner. They must not
act on the editor's incidental point behind the projection. The native probe
selects the second of two entries and changes only its effort property.

Owners: [properties](../../../../src/session/outline/property.rs),
[task workflows](../../../../src/session/outline/tasks.rs),
[effort](../../../../src/session/outline/estimates/effort.rs),
[agenda target resolution](../../../../src/session/agenda_actions/resolve.rs),
[agenda action execution](../../../../src/session/agenda_actions/execute.rs).
Validation cases: declared choices/errors, selected source identity, stale
projection refusal, prompt cancellation and source transaction undo.

### H04: Timestamps and the calendar picker

Keep the compact native date/time picker, explicit active/inactive timestamp
forms, planning fields, captured timezone/instant and offset-choice prompt.
Calendar day/week/month/year controls select within that prompt; they are not
ordinary source motions. The reference full Calendar uses different unshifted
navigation keys and broader calendar facilities. Preserve the native picker
convention and named access; do not impose a second calendar UI.

The native probe selects the next civil day from a captured 2024-03-01 date and
accepts it through RET, then separately checks C-g cancellation without a source
edit. Local calendar maps shadow inherited minibuffer actions only for admitted
date controls. An offset prompt retains its own input owner.

Owners: [timestamp workflow](../../../../src/session/outline/timestamps.rs),
[calendar selection](../../../../src/session/outline/timestamps/calendar.rs),
[contextual shifts](../../../../src/session/outline/timestamps/context.rs).
Runtime cases: month/year boundaries, invalid drafts, timezone choices and
source/view revision changes; no claim of complete Calendar feature parity.

### H05: Clocks, timers and habits

Keep task-owned interval edits, recovery/resolve workflows, explicit clocktable
updates and session timers. Relative timers are not task clock intervals.
Clock inspection, view refresh and report insertion have different source
effects. P06 removes clock-report from C-c C-x C-c in Org only; it remains
available by name. Do not move it to C-c C-x C-r, the radio-button route.

Habits are agenda projections, not edits to their source dates. K and C-u K
retain the compatible habit display family under native view ownership. D01
proposes moving the conflicting A/G view controls into the native v namespace;
the old d graph-date action becomes named-only when daily view takes d.

Owners: [clock lifecycle](../../../../src/session/outline/clocks.rs),
[clock reports](../../../../src/session/outline/reports/work.rs),
[habit views](../../../../src/session/habits.rs),
[timer controls](../../../../src/session/timers.rs).
Validation cases: source interval identity, canceled repair decisions, report
versus view-only effects, retained timer state and unchanged habit source.

### H06: Agenda views, queries, filters, columns and bulk actions

P08 proposes d for the existing daily view and retains v d. Accepted D01 uses
v M for tag view, v / for search, v A for show-all-today habits, v G for graph
window and v S for sort. Existing v t/d/w/v/m/y/s remain native view/span routes.
This is an explicit native namespace, not a claim that all reference v suffixes
match. Old M/s/A/G/S become unbound; the unsupported reference moon, save-all,
append, time-grid and sunrise operations are not invented to occupy them.

Preserve source selection, day/week/calendar projection, custom views,
query/filter language, bulk marks, source-result reports and transaction undo.
Native agenda columns choose displayed fields; they are not editable inline
reference columns. Generic source commands route through the selected row's
qualified buffer/path and heading. View changes and cancellation cannot grant
source mutation authority. Source discovery remains explicit and bounded.

Owners: [calendar controls](../../../../src/session/agenda/control.rs),
[queries](../../../../src/session/agenda_queries.rs),
[custom controls](../../../../src/session/agenda_custom/control.rs),
[source actions](../../../../src/session/agenda_actions.rs),
[bulk actions](../../../../src/session/agenda_actions/bulk.rs).
Evidence: current reference single-key identities, prospective map admission,
native daily key behavior and selected-second-entry effort change. Runtime
acceptance must cover each relocation, cancellation and unchanged view-only
source bytes, plus retained explicit old overrides.

### H07: Capture, refile and archive

Keep capture's transient text buffer, finalize/abort routes and explicit target
publication. C-c C-c finalizes only under capture ownership; it is not document
Ctrl-C-C dispatch there. Ordinary Save cannot publish the capture draft.
The probe edits a draft, observes Save retaining it, aborts with C-c C-k and
verifies the origin and absent destination remain unchanged.

Refile and archive keep checked complete subtrees, participant identities and
independent save outcomes. Their result/inspection helpers stay named-only.
No key creates an implicit save, overwrites a participant or bypasses consent.

Owners: [capture](../../../../src/session/capture.rs),
[capture publication](../../../../src/session/capture/finalize.rs),
[refile](../../../../src/session/refile.rs),
[archive](../../../../src/session/refile/archive.rs).
Validation cases: retained draft after failure, exact target, cancellation,
source participants, undo and explicit per-file outcomes.

### H08: Links, attachments, document elements and previews

Keep stored links, the qualified mark ring, source navigation, native attachment
dispatch, footnotes/emphasis/entities and local previews. Attachment children
are contextual workflows through the C-c C-a dispatcher where its prompt admits
them; do not advertise the dispatcher chord as a direct helper binding. Link
following can involve an explicit viewer, while local preview is disposable
presentation. Same source spelling does not imply arbitrary external URL or
runner support. Native result inspectors and presentation resets stay named.

Owners: [links](../../../../src/session/links.rs),
[attachment dispatcher](../../../../src/session/attachment.rs),
[document operations](../../../../src/session/document.rs),
[previews](../../../../src/session/previews.rs).
Validation cases: actual point target, canonical source, checked destination,
cancelled prompts, stale views and source unchanged for inspection.

### H09: Tables and property columns

Keep Org's admitted native table family, including indirect TAB/Shift-TAB,
Return and Meta-arrow branches. C-c *, C-c - and C-c ^ cover table recalculate,
separator and sort here; the reference's broader contextual dispatchers also
cover non-table cases. Preserve the partial table equivalence without claiming
those missing heading/list/region branches. Named formula/import/export and
field/column helpers retain their bounded native admission.

P06 proposes the existing read-only property-column projection at C-c C-x C-c.
The probe verifies a query projection with source stamp/text/history unchanged.
An explicit dynamic columnview update is a different source edit and stays
named. Reference editable inline column behavior is outside the target.

Owners: [table owner](../../../../src/session/tables.rs),
[table model](../../../../src/org/table.rs),
[column projection](../../../../src/session/outline/reports/columns.rs).
Validation cases: point in a real cell, one checked edit/undo, read-only and
formula admission, import/export bounds and projection source identity.

### H10: Source editing, Babel, tangling and execution

P07 proposes C-c C-v j and C-c C-v C-j for the existing checked header prompt,
and removes its misleading h assignment. The reference uses h for Babel help;
no native help command is introduced. Header cancellation leaves source bytes
unchanged; accepting one argument edits the opening line through the same owner.

Source-edit exit and abort preserve their current chords. In the source draft,
C-x C-s still resolves to Save but the native retained-draft guard refuses it.
The reference's source-edit Save performs a different operation. Mapping Save
to native exit would change retention and is rejected as an approximation;
support needs a separate behavior requirement. The native probe confirms Save
retains the edited draft and abort restores unchanged Org source.

Tangling, named references and configured runner execution remain distinct
operations with explicit outputs, confirmation, cancellation and result owners.
There is no general reference evaluation engine or implicit result insertion.
Keep existing tangling/execution keys and named result/single-block helpers.

Owners: [source draft](../../../../src/session/source_edit.rs),
[draft lifecycle](../../../../src/session/source_edit/lifecycle.rs),
[tangling](../../../../src/session/tangling.rs),
[execution](../../../../src/session/execution.rs).

### H11: Export, publication and feeds

Keep native backend export, project publication and feed operations through
their current named or existing direct routes. The reference export dispatcher
has no equivalent native dispatcher command to bind at C-c C-e; do not pretend
one backend implements its menu. Native backend names differ from reference
backend functions, and supported output/source/region boundaries remain explicit.
Compiled outputs and external runners require the existing separate consent.
Feed fetch/import is explicit; inbox navigation and result inspection do not
fetch. This audit invokes none of those external or publication workflows.

Owners: [export](../../../../src/session/exports.rs),
[publication](../../../../src/session/publishing.rs),
[feeds](../../../../src/session/feeds.rs).
Validation cases: source scope and revision, checked output, canceled external
work, partial results and retained native failure/consent boundaries.

## Markdown families

### M01: Native mode and presentation

Keep mode selection and view-local heading/list decorations. Presentation reset
restores installed settings without rewriting configuration. These are native
named controls, not source-edit or package-mode aliases.

Owners: [Markdown session](../../../../src/session/markdown.rs),
[mode selection](../../../../src/session/mode.rs).

### M02: Plain text, lists and Return

Keep current RET/M-RET behavior. Native RET continues a list or exits an empty
level; stock Markdown mode's RET inserts a newline in the observed fixture.
Native M-RET inserts a native sibling; the reference can select a different
indentation/bullet. This is a deliberate product convention, not full package
Return parity. Outside a list/table, both native Return routes insert a newline.
Indent/outdent own complete native item trees and continuation text.

`markdown-continue-item` is reached indirectly by the Return dispatchers in an
admitted list. Do not add a key merely because the helper has no raw target.
Code/literal contexts, stale parsing and read-only buffers retain their own
refusal and plain-text admission.

Owners: [Return dispatch](../../../../src/session/markdown.rs),
[list model](../../../../src/markdown/lists.rs).
Evidence: paired Return/list effects and native plain/list/table branches.

### M03: Tables

Preserve the existing Org-like native table key family. Table contributions are
filtered outside tables; active state/language/point matters more than a static
major-mode lookup. Native RET reaches table-next-row indirectly and can align
source before moving; it is not a guaranteed read-only motion. M-RET owns body
row insertion. Field navigation, row/column edits and last-column protection
remain their own operations. No new Markdown keys are proposed.

Owners: [table dispatcher](../../../../src/session/markdown/tables.rs),
[table model](../../../../src/markdown/tables.rs),
[point-filtered maps](../../../../src/input/keymap/contributions.rs).
Validation cases: direct helper and Return effects agree, header/delimiter/body
contexts, literal code, row creation, one edit group and retained overrides.

## Git families

### J01: Status, index actions and comparisons

Keep status entry/return/refresh, semantic navigation, detail toggling, path/hunk
marks, staging, unstaging and checked whole-path discard. Native Status c opens
commit text directly; a following c is message text, not a transient suffix.
Status h and b/y remain native history/ref navigation. The action view shows
native available commands; it is not Magit's full transient system. Selected
resource/object identities and review/confirmation remain authoritative.

Owners: [Git dispatch](../../../../src/session/git/commands.rs),
[index targets](../../../../src/session/git/index.rs),
[comparisons](../../../../src/session/git/comparisons.rs),
[discard](../../../../src/session/git/discard.rs).

### J02: History, revision/range detail, reflog and blame

Accepted D02 uses History A for the existing selected/ordered cherry-pick owner,
removes its old P alias, and removes F fixup and f paging. The precise owner
amendment adds c a amend, c f fixup and + next page, retaining named access. The reference P/F/f families require push/pull/fetch transients
which History cannot supply through existing native operation owners. Leaving
them unbound avoids substituting an unrelated edit or a guaranteed refusal.
Explicit old user assignments remain valid under current override rules.

Keep Tab revision detail, marked-base/selected-endpoint Return ranges, ordered
sequence marks, parent review, reflog and blame targets. Range direction and
ordered cherry-pick/revert lists are independent identities. Historical source
is read-only; visiting current source keeps its native canonical buffer owner.

Owners: [history](../../../../src/session/git/history.rs),
[range/ordered marks](../../../../src/session/git/history/marks.rs),
[revision detail](../../../../src/session/git/history/inspection.rs).
Evidence: native History fetch/push refusals; no remote or mutation journey is
claimed. Runtime Slice 6 must update panel guidance and verify A's exact target,
all c a/c f/+ routes, removed defaults, names and retained explicit overrides.

### J03: Refs, branches, tags, fetch, push and integration

Accepted D02 adds f for fetch and P for push in Refs only, retaining &/M f and p/M p.
These are familiar family letters for native direct operations, not transient
menu parity. Fetch requires a selected configured remote; push requires an
exact local branch or qualified push mark and upstream review. Neither can be
moved to History by changing a key alone. No generic native pull command exists;
fetch and a separately selected integration mode are not a pull substitute.

Keep checked branch/tag operations, source marks, explicit fast-forward/merge/
rebase choices and finite autosquash preparation. Native refs keys remain an
explicit bounded product model, not blanket Magit equivalence. No force, tag,
multi-ref or deletion push is introduced by the proposed key aliases.

Owners: [Refs](../../../../src/session/git/refs.rs),
[push source](../../../../src/session/git/refs/transport.rs),
[integration](../../../../src/session/git/refs/integration.rs),
[tags](../../../../src/session/git/refs/tag.rs).
Evidence: owned no-remote Refs refusal and exact normalized proposals. Later
runtime checks use local fixtures and preserve operation review and consent.

### J04: Sequences, bisect and conflicts

Keep native operation-progress/sequence controls, exact ordered revisions,
mainline choices, continue/skip/abort and explicit manual bisect verdicts.
The native Sequence panel is not an editable rebase-todo buffer. Conflict
selection and base/current/applied candidate review are source/resource actions,
not ordinary buffer editing or a reference key-name match. Keep selected native
routes; missing reference merge/rebase UI is not supplied through an alias.

Owners: [sequences](../../../../src/session/git/refs/sequences.rs),
[manual bisect](../../../../src/session/git/refs/bisect.rs),
[conflicts](../../../../src/session/git/conflicts.rs).

### J05: Worktrees and stash

Keep the native worktree registry and exact stash object workflows with their
existing panel keys. Paths/branch labels, stash ordinal labels and subject text
are not mutation authority. Native lock/prune/remove and stash apply/pop/drop/
branch commands retain review, dirty-source protection and observed outcomes.
The direct native panel model is an explicit exception to reference transients;
no shared key changes are proposed for these independent contexts.

Owners: [worktrees](../../../../src/session/git/repository/worktrees.rs),
[stash/repository workflows](../../../../src/session/git/repository.rs).

### J06: Composers, processes and direct command review

Keep native commit/amend/fixup/tag/family drafts, complete review and explicit
submission/abort. GitCommit overlays RET/text editing independently from other
panels. Process Return is indirectly reachable through `git-visit-file` and
dispatches to process-show for that page. Process C-g is an input interception
that cancels the selected owned operation only after menus/prompts retain first
refusal; q leaves observation while work continues. Do not advertise C-g as a
global direct process-cancel binding.

Literal Git argv and explicit shell-command composition are distinct reviewed
workflows. Neither a familiar key nor a suffix can supply execution consent.

Owners: [composer](../../../../src/session/git/commit.rs),
[processes](../../../../src/session/git/processes.rs),
[input interception](../../../../src/session/git/input.rs),
[family drafts](../../../../src/session/git/families.rs).

### J07: Repository creation, submodules, subtrees, notes, sparse and patches

Keep the existing native families and named variants for checked clone/init,
ignore files, submodules, subtrees, notes, cone sparse checkout and patch/mailbox
input/output. Each command's literal operands, selected source, absent output,
captured revision, bounded observation and postconditions remain specific to
that operation. Reference package names/prefixes are a family comparison, not
permission to add unsupported options, arbitrary todo commands, external remotes
or an implicit shell. The audit runs none of these changing workflows.

Owners: [native family review](../../../../src/session/git/families.rs),
[public command contracts](../../../../src/command/git.rs).

## Accepted decision and follow-through

Every operation has a semantic disposition. The owner accepted the exact base
proposal with three additional History routes: c a/c f/+. The mapping contract
records accepted provenance, digest and all 49 base-context edits. D01 is settled;
D02 includes quick displaced access and the current direct-action scope.
D03 remains an inventory of current differences, not permission to keep arbitrary
shared-capability divergence permanently.

[Emacs Interaction Parity](../../spec/editor-emacs-interaction-parity.md)
records the requested core-first functional requirements, stock source/effect
evidence, related Org/Git work and subsequent slicing preparation. Current
shortcut slices keep their seven identities, frozen checks, consent and overrides.
No additional acceptance stop is needed for the precise accepted revision.
