# Native Org capability and evidence map

Status: reconciled references; final delivery acceptance pending
Specification: [Org Mode](editor-org-mode.md)
Model: [Org Mode Model](editor-org-mode-model.md)
Plan: [Native Org delivery](../plan/rev-org-mode.md)
Tracker: [Native Org tracker](../tracker/rev-org-mode.md)

## Preserved tested candidates

| Delivery | Original tested candidate | Owning evidence |
| --- | --- | --- |
| Editor | `b56d705c7a84b823bf069fc874bc762938d6d897` | [Editor baseline](rev-editor.md), A01-A20 and R01-R37. |
| Configuration/interaction | `95713efb7f79fbc72417615a921841f8d49730ed` | [Configuration](editor-configuration.md), gate `FWDUCVRJCYBKFXEWEFLKPKA35E`. |
| Input/navigation | `dedd85064cb5565a5382a9d8e2e2902daae24d88` | [Editor baseline](rev-editor.md), gate `PZ4OIPXA6GUBSWDH5ZS7MJUOGX`. |
| Org combined terminal | `14cbf761a6492994da8d1f6674c8013046cb4878` | [Slice 61](../report/slices/rev-org-mode/slice-61-combined-terminal.md), required `B6DJ6COVBO6RDOADHXBI24YS5G`: 948 native tests and 69 owned PTYs. |
| Org resources | `18f55af4af0ecef5a88feeab388edbb3a53d66b7` | [Slice 62](../report/slices/rev-org-mode/slice-62-bounded-work.md), required `SLV234DTE5VQ7XX7PJVCU22J5S`: 522 native tests, historical and Org release measurements. |

These identities keep their original meaning. Terminal and resource records from
different candidates are not combined into a new tested head. Slice 63 runs its
own required native/reference/prerequisite checks. The remaining integration and
unchanged full delivery gate still belong to the controller.

## Native capability boundaries

[Controls](../../../docs/reference/org-controls.md) maps every shipped capability
to actual compiled commands, typed preferences and fixed native policies.
[Org mode](../../../docs/reference/org-mode.md) owns canonical editing and view
behavior; the feature references linked there retain their detailed context,
source preservation, refusal and publication contracts.

Core Org editing, calendar/agenda, capture, formulas and native export work without
an interpreter, converter, network connection or external service. Optional
programs run only for an explicitly selected documented action and remain outside
the editor's configuration/runtime language. Missing tools produce usable failure
results; they are not reported as skipped native acceptance.

## Capability matrix

Each row links the shipped control family and its source authority. The scenario
map below supplies native and terminal identities; this matrix alone is not test
execution evidence.

| Native capability | Reference | Canonical authority / effect |
| --- | --- | --- |
| Activation, outline and drawers | [Org mode](../../../docs/reference/org-mode.md) | Buffer mode; view-local folds; source/history unchanged by visibility. |
| Lists, tasks, properties, tags and effort | [Org mode](../../../docs/reference/org-mode.md#lists-and-checkbox-progress) | Revision-qualified checked source edits; one ordinary undo unit. |
| Civil timestamps and recurrence | [Dates](../../../docs/reference/org-mode.md#operation-time-zones) | Captured operation zone; explicit fold/gap choices; exact source spelling retained. |
| Clocks, reports and timers | [Clock controls](../../../docs/reference/bindings.md#native-task-clocks) | Task CLOCK source and optional recovery locator; independent session timer. |
| Habits | [Consistency graphs](../../../docs/reference/org-mode.md#habit-consistency-graphs) | Actual source completions and bounded disposable graph; no inferred history. |
| Agenda, queries and actions | [Agenda](../../../docs/reference/org-agenda.md) | Explicit sources; unsaved canonical buffers win; stale/incomplete results cannot grant edits. |
| Capture | [Capture](../../../docs/reference/org-capture.md) | Editable draft; checked destination; abort, stale refusal and retry preserve source. |
| Refile/archive | [Files](../../../docs/reference/org-files.md) | Coherent participating buffer edits; each save has its own actual outcome. |
| Links and attachments | [Links](../../../docs/reference/org-links.md), [attachments](../../../docs/reference/org-attachments.md) | Checked target/file identities; explicit effect and conflict decisions. |
| Footnotes and inline content | [Document editing](../../../docs/reference/org-document.md) | Literal canonical source and ordinary history; unknown forms stay inert. |
| Tables, formulas and interchange | [Tables](../../../docs/reference/org-tables.md) | Bounded native arithmetic/graph; checked one-transaction results; unsupported formulas preserved. |
| Columns and dynamic reports | [Columns](../../../docs/reference/org-columns.md) | Source-derived retained view; explicit body refresh; no automatic source rewrite. |
| Eleven native export formats | [Export](../../../docs/reference/org-export.md) | Unsaved checked snapshot; text, UTF-8, Markdown, Org, HTML, LaTeX, Beamer, ODT, Texinfo, man, iCalendar. |
| Project publishing | [Projects](../../../docs/reference/org-export.md#project-publishing) | Explicit local files/assets; truthful partial outcomes; optional separate upload. |
| Source blocks, noweb and results | [Source blocks](../../../docs/reference/org-source-blocks.md) | Native source editing/tangling; explicit selected program; checked result publication. |
| Feed import | [Feeds](../../../docs/reference/org-feeds.md) | Explicit XML observation and canonical FEEDSTATUS/entry transaction; no update loop. |
| Link and image previews | [Previews](../../../docs/reference/org-files.md#link-and-image-previews) | Explicit view-local bounded resources; capability fallback preserves readable source. |
| Complete controls | [Catalog](../../../docs/reference/org-controls.md) | 178 schema/catalog entries, 54 RGB roles, 263 local bindings across eight contexts; nine optional examples. |

## Optional program boundaries

| Selected action | Configuration / capability | Native behavior when unavailable |
| --- | --- | --- |
| Produce PDF from LaTeX/Beamer | `org.tools.pdf`, literal argv and explicit compilation consent. | Native TeX export remains usable; failed compiler keeps prior output and source. |
| Fetch a remote feed | `org.tools.fetch` or the selected feed transport. | Local XML/native feed import remains usable; missing transport records a failure. |
| Upload a published project | `org.tools.upload` on an explicit project action. | Local publication remains inspectable; an upload failure cannot undo or hide saved files. |
| Open an external link/resource | `org.tools.viewer`, checked target and consent. | Native source navigation/preview fallback remains usable; no implicit browser launch. |
| Execute a source block/call | `org.source.programs`, selected language/profile, bounded streams and owned children. | Recognition, editing, noweb tangling and native exports remain usable; absent runner is diagnosed. |
| Show native PNG graphics | Explicit preview plus admitted terminal image capability. | Readable text/status fallback; no graphics or viewer launch without authority. |

Tool presence and provenance do not prove program semantics. Execution evidence
requires real owned executable/failure fixtures, captured output/source assertions,
consent and cleanup. Ordinary source directives cannot select a startup interpreter,
load a plugin or grant executable authority. Advanced spreadsheet evaluation,
arbitrary precision and executable Lisp table formulas remain outside the selected
native formula subset.

## Resource and acceptance provenance

[The model profile](editor-org-mode-model.md#resource-profile) retains 150 ms
response/progress, 10 ms compute boundaries, 32 MiB additional retained caches and
96 MiB fixture RSS targets. Earlier ceilings remain mandatory. Actual release
records in Slice 62 cover 1 MiB/10000 headings, 256 agenda files, 733 habit days,
10000 dependent formula cells, eleven exports and sixteen cancellation cycles.
Ten owned idle states observe CPU/output/reads/RSS and resource ownership, including
armed/paused/expired timers and open/closed/inactive views. Resident growth is
observed while results are held and includes allocator retention; it is not an
exact logical heap count or a hard kernel-IO latency promise.

Feature scenarios without a dedicated large-work fixture link to this shared
source/job/idle profile. That link supplies the common ownership/resource boundary,
not a measured performance claim for every feature algorithm. Native assertions
and real-terminal source/outcome checks remain separate obligations.

## Scenario evidence

The following named native assertions and owned terminal selectors protect the
scenario's observable behavior. Native targets are compiled in the prerequisite
check; listing them does not execute them. A scenario may need multiple assertions
within its linked test. Terminal names resolve through the existing collector's
Org groups; [Slice 61](../report/slices/rev-org-mode/slice-61-combined-terminal.md)
records actual complete journey execution. Resource names resolve through the
existing release collector; `idle:` names refer to measured terminal states.
Shared source/idle records have the limited meaning stated above.

| Requirement / scenario | Native assertion | Owned terminal | Resource observation |
| --- | --- | --- | --- |
| ORG01: Open and edit without setup | [`mode::visiting_mode`](../../../tests/org_source/mode.rs) | [`org-outline`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`idle:org_idle`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG02: Preserve an unfamiliar form | [`inline_bounds::unknown_fidelity`](../../../tests/org_source/inline_bounds.rs) | [`org-outline`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG03: Fold one shared view | [`folds::independent_map`](../../../tests/org_editing/folds.rs) | [`org-outline`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`idle:closed_view`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG04: Move a complete subtree | [`subtrees::subtree_moves`](../../../tests/org_editing/subtrees.rs) | [`org-outline`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG05: Inspect inherited metadata | [`properties::selective_properties`](../../../tests/org_editing/properties.rs) | [`org-tasks`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG06: Complete a task coherently | [`tasks::task_transaction`](../../../tests/org_editing/tasks.rs) | [`org-tasks`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG07: Complete an overdue repeating task | [`repeaters::repeater_completion`](../../../tests/org_editing/repeaters.rs) | [`org-time`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG08: Keep unsaved work visible in the agenda | [`sources::agenda_source_selection`](../../../tests/org_agenda/sources.rs) | [`org-agenda`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_agenda`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG08: Report a partial source set | [`agenda_sources::agenda_partial_sources`](../../../tests/org_io/agenda_sources.rs) | [`org-agenda`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_agenda`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG09: Act on an entry whose source changed | [`source_actions::source_action_stale`](../../../tests/org_agenda/source_actions.rs) | [`org-agenda`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_agenda`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG10: Abort capture | [`capture_drafts::native_draft`](../../../tests/org_editing/capture_drafts.rs) | [`org-capture-drafts`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_cancel`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG10: Reject a stale capture destination | [`capture_publication::finalize_stale_cancel`](../../../tests/org_agenda/capture_publication.rs) | [`org-capture-finalize`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_cancel`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG11: Refile to an unavailable destination | [`refile_failures::refile_file_rejections`](../../../tests/org_io/refile_failures.rs) | [`org-refile`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`org_cancel`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG12: Move a table column | [`structure::column_structure`](../../../tests/org_tables/structure.rs) | [`org-tables`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG13: Recalculate a column | [`recalculation::recalculation_selection`](../../../tests/org_tables/recalculation.rs) | [`org-tables`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_formulas`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG14: Preserve a formula outside the native subset | [`recalculation::recalculation_preservation`](../../../tests/org_tables/recalculation.rs) | [`org-tables`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_formulas`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG15: Follow an internal target | [`links::links_internal`](../../../tests/org_io/links.rs) | [`org-links`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG16: Display a literal example | [`color::literal_colors`](../../../tests/org_source/color.rs) | [`org-outline`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG17: Use the same operation by key and name | [`org_catalog::default_routes`](../../../tests/commands/org_catalog.rs) | [`org-combined`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`idle:inactive`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG18: Cancel a bounded multi-file query | [`sources::agenda_source_cancel`](../../../tests/org_agenda/sources.rs) | [`org-agenda`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_agenda`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`org_cancel`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG19: Record time against a task | [`clocks::owned_interval`](../../../tests/org_editing/clocks.rs) | [`org-time`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`idle:org_idle`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG20: Complete a habit with an interval range | [`habits::habit_completion`](../../../tests/org_editing/habits.rs) | [`org-time`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_habits`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG20: Inspect consistency without changing the document | [`habits::habit_agenda_controls`](../../../tests/org_agenda/habits.rs) | [`org-time`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_habits`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`idle:open_view`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG21: Compare estimated and recorded effort | [`contracts::source_provenance`](../../../tests/org_agenda/contracts.rs) | [`org-time`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_agenda`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG22: Finish and stop a countdown | [`timers::countdown_lifecycle`](../../../tests/org_time/timers.rs) | [`org-time`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`idle:armed`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`idle:expired`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`idle:stopped`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG23: Apply an action to selected agenda entries | [`bulk_actions::bulk_action_sources`](../../../tests/org_agenda/bulk_actions.rs) | [`org-agenda`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_agenda`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`org_cancel`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG24: Reject a conflicting attachment destination | [`attachments::attachments_collision`](../../../tests/org_io/attachments.rs) | [`org-attachments`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_cancel`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG25: Export unsaved source without executing it | [`export::export_snapshot_contract`](../../../tests/org_io/export.rs) | [`org-exports`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_exports`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG26: Publish a partially failing project | [`publishing::project_partial_sources`](../../../tests/org_io/publishing.rs) | [`org-export`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_exports`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`org_cancel`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG27: Import quoted table data | [`interchange::interchange_roundtrip`](../../../tests/org_tables/interchange.rs) | [`org-tables`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG28: Cancel a selected block execution | [`execution_edges::execute_lifetime`](../../../tests/org_io/execution_edges.rs) | [`org-execution`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_cancel`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG29: Refresh the same feed twice | [`feed_import::feed_transaction`](../../../tests/org_io/feed_import.rs) | [`org-feeds`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_cancel`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG30: Preview without terminal image capability | [`previews::preview_fallback`](../../../tests/org_io/previews.rs) | [`org-external`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`idle:closed_view`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
| ORG31: Verify the complete selected delivery | [`org_catalog::control_reference`](../../../tests/commands/org_catalog.rs) | [`org-combined`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation) | [`org_source`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`org_agenda`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`org_habits`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`org_formulas`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`org_exports`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation), [`org_cancel`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation) |
