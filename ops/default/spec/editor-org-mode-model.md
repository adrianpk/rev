# Org Mode Model

Status: Completed
Completed: 2026-10-30
Validated candidate: `72160e46648f3f28a9c8bb7f3eecc49173fca09d`
Validated tree: `8249daaf355af4f7be00173b5b107fb4d9c80fea`
Gate validation: `EXPY4POPOBG2CKQH6C4JTM5AVI`
Approved: product owner, 2026-10-27; specification and model accepted, with the explicitly listed open decisions retained.
Scope refinement: product owner, 2026-10-28; remaining product choices use Org in Emacs, preserving the advanced-spreadsheet exclusion.
Date: 2026-10-27
Specification: [Org Mode](editor-org-mode.md)
Existing text/view model: [rev Editor Model](rev-editor-model.md)
Configuration owner: [Editor Configuration Model](editor-configuration-model.md)

## Ownership and storage

Org documents remain ordinary canonical UTF-8 buffer/file text. Commands edit
that source through existing editor transactions. Recognition, indexes, folds,
agenda rows and calculated previews are derived observations, not another
editable document database. Normal save preserves unaffected source bytes.

User preferences use the existing TOML store. Allowed file-local Org directives
are document data and affect only their documented semantics. Runtime view and
query choices do not rewrite TOML implicitly. Derived indexes, folds, agenda results and previews have no persistent cache
format. The optional active-clock recovery record below is the one selected
additional state format; document CLOCK records remain canonical text.

## Source records in document order

| Concept | Semantic fields | Source/meaning |
| --- | --- | --- |
| Document | Identity, canonical revision, visiting file identity and source text. | Existing buffer/file lifecycle. |
| Heading | Level, optional task keyword, optional priority/comment marker, title, local tags, owned section and descendants. | One outline hierarchy from document text. |
| Drawer | Kind/name, opening/body/closing source, containing context. | Ordinary drawers and property drawers remain distinct. |
| Property | Name, stored value, additive form and local owner. | Effective inheritance is derived; special derived properties are not arbitrary stored fields. |
| List item | Bullet/counter, indentation, optional checkbox, body and nested content. | Structural source; continuation and nested ownership must be preserved. |
| Task state | Workflow identity, keyword, unfinished/finished classification and optional selected logging facts. | Keywords derive from admitted defaults/source directives. |
| Planning fact | SCHEDULED/DEADLINE/CLOSED kind and owned timestamp source. | Only in its valid heading context. |
| Timestamp | Active/inactive delimiters, date/day spelling, optional time/range, repeater, warning delay and exact text. | Invalid values remain preserved and diagnosable; query time/zone is separate. |
| Clock interval | Task/source owner, start timestamp, optional end timestamp, duration and original source. | Native clock-in/clock-out edit canonical Org text; the shipped clock placement, switching and recovery policies are documented in the [clock reference](../../../docs/reference/bindings.md#native-task-clocks). |
| Effort | Task/source owner, duration property, parsed minutes and original text. | Estimated work remains distinct from measured intervals. |
| Habit | Task owner, STYLE, repeating schedule/range, LAST_REPEAT and completion history. | Ordinary Org task source; no independent habit store. |
| Link | Form, target, optional description/search suffix and owned source. | Recognition has no external effect. |
| Table | Affiliated names/captions, rows, separator rows, cells and formula source. | Stored values and alignment/display are separate. |
| Formula | Original text, admitted target field/column and expression/format. | Unsupported syntax is preserved; evaluation is explicit. |

A source locator must identify the document, observed revision and exact target.
An existing stable source ID may help resolve a later observation; a heading
title or cached row number alone is not sufficient authority to mutate text.
Indexing does not insert IDs or normalize source merely to simplify navigation.

## Derived state and lifetime

| Record | Required evidence | Invalidation / outcome |
| --- | --- | --- |
| Effective metadata | Local owner, ancestor/file values and admitted policy. | Recompute after relevant source or setting changes; retain local/inherited distinction. |
| Outline/drawer visibility | View, document revision and requested visibility. | Independent across shared views; released on view close. |
| Agenda source observation | Document/file identity, current buffer revision or checked disk version, completeness/failure. | Stale state cannot authorize source edits; missing source is visible. |
| Agenda result | Query/source set, captured time/zone, filters/sort, checked source locators and completeness. | Derived rows refresh/re-resolve; actions target canonical source. |
| Habit graph | Source observations, effective date/zone, bounded date window, daily state and completion markers. | Derived agenda presentation; no inferred completions or source writes. |
| Capture session | Template choice, bounded substitutions/input, editable draft, destination evidence and lifecycle. | Abort leaves target unchanged; finalize revalidates and publishes once. |
| Table calculation | Source revision, admitted references/dependencies, numeric policy, proposed results and diagnostics. | Publish only after validation; failure preserves prior results. |
| Timer | Session identity, relative/countdown mode, start/remaining duration, paused state and optional deadline. | Only explicitly active deadlines wake the event owner; no implicit persistence. |
| Stored link | Checked source/target information and optional descriptive text. | Resolution must not rely on an obsolete display row alone. |

## Capture and move states

Capture has explicit choosing/collecting/editing, validating, finalized, aborted
and failed-with-draft outcomes. [Capture reference](../../../docs/reference/org-capture.md)
records the delivered transition, retry and abort behavior. A failed destination must not cause a
valid draft to disappear or claim a committed insertion.

Cross-document refile/archive identifies source and destination and records
which edits and durable saves actually succeeded. The [native file operations](../../../docs/reference/org-files.md)
provide coherent in-memory publication and explicitly recoverable file outcomes; they must not collapse
partial publication into a single false success state.

## Cross-document publication and job states

Refiling/capture/archive prepare checked source and destination edits before
publication. In-memory publication is coherent across participating buffers;
file saves retain the ordinary per-file identity checks. A failed durable step
retains recoverable source/destination buffers and an explicit operation result,
never a fabricated all-or-nothing disk success. Each target records prepared,
changed-in-buffer, saved, rejected or completion-unresolved as appropriate.

An optional persisted clock has a checked source locator/open-entry observation
and last acknowledged operation time under XDG state. Persistence defaults off;
state is created only when the user selects it. Recovery re-resolves source and
asks about an incomplete interval; it cannot substitute a stale display row.

| Record | Fields and authority | Lifetime |
| --- | --- | --- |
| Attachment | Heading ID/DIR, selected source/destination, action and file observations. | Explicit action; preserve partial outcome evidence. |
| Export snapshot | Checked buffer/file revisions, format/options, includes and captured date/zone. | Immutable for one export; no implicit save/evaluation. |
| Publish project | Named source/output selection, formats, assets and optional upload argv. | Data-only config; bounded explicit publication. |
| Block definition | Source/name/language/body, inherited headers and checked referenced blocks. | Revision-bound source; never executable by recognition. |
| External job | Explicit action/confirmation, executable argv, cwd, bounded IO, owned children and deadline. | Starting/running/completed/failed/cancelled/timed-out/stale-result; reap owned children. |
| Feed observation | Declared URL/local file, feed/item identity, target locator and FEEDSTATUS source. | Explicit import; no independent polling database. |
| Preview | Checked link/resource, admitted terminal geometry/capability and optional viewer job. | View-local and disposable; ordinary text remains authoritative. |

## Optional clock recovery representation

The selected XDG state namespace contains `org-clock.toml` only after explicit
opt-in checkpointing. Its version-1 TOML record has optional `[clock]` data:
canonical `path`, checked `file` observation, `heading`, `heading_source`,
`line_start`, `line_end`, original CLOCK `source`, `start_seconds` and
`acknowledged_seconds`. Absence of a clock records version 1 without an active
locator. This bounded locator is recovery evidence, not a substitute for saved
Org text or a source edit permission. Actual path, heading/CLOCK bytes, context,
file identity and chosen instant are revalidated independently.

Opening a missing store does not create directories. Explicit checkpoints use
checked temporary-file replacement and retain the ordinary not-published,
published or published-with-completion-unresolved outcome. A failed checkpoint
cannot roll back a real source clock transition. The [clock reference](../../../docs/reference/bindings.md#native-task-clocks)
owns user-facing recovery and source-visit bounds; no timer or agenda persistence
is added.

## Typed configuration profile

Extend the existing startup TOML owner; do not build a second loader. Global
org policy composes through the existing includes/root semantics. Data-local Org
keywords affect only admitted document data. Source paths are resolved relative
to their declaring config/document; no source text can grant tool or process
authority. Relative tool argv/path data receives the normal checked treatment.

The following are the semantic fields for the native [org] family and typed
subtables. Implementation uses these names and units consistently in native
settings, schema/catalog, sample and embedded template where they are startup
controls. Semantic profile labels also describe fixed native policy: startup
visibility is show-all; empty numeric cells use zero and rounding is half away
from zero. `startup_fold`, `tables.empty_numeric` and `tables.rounding` are not
additional accepted TOML keys. Overview/contents visibility is selected through
native view commands. The [actual catalog](../../../docs/reference/defaults.md#configuration-value-catalog)
is the parser allowlist; unsupported keys remain rejected. Arrays replace on merge;
absent values inherit. No unimplemented leaf may appear operational.

| Field under org | Factory value | Bounds and meaning |
| --- | --- | --- |
| enabled | true | Native mode available; per-buffer mode choice remains explicit. |
| startup_fold | "show-all" | show-all, overview or contents. |
| todo.sequence | ["TODO", "DONE"] | 1..64 unique keywords; finished list below identifies completed states. |
| todo.finished | ["DONE"] | Subset of declared workflow; native validation rejects ambiguity. |
| todo.log_done | false | Closure logging, separate from repeat logging. |
| todo.log_repeat | "time" | none, time or note. |
| todo.log_drawer | "" | Empty means normal source placement; explicit drawer name is bounded. |
| todo.block_children | false | Explicit completion-blocking policy. |
| priority.highest / lowest / default | "A" / "C" / "B" | Ordered single admitted characters. |
| tags.inherit | true | Case-sensitive membership with provenance. |
| properties.inherit | [] | Explicit property names; no general inheritance by default. |
| time.zone | "local" | System zone, UTC or validated IANA zone; finite TZif observation. |
| time.ambiguous / nonexistent | "prompt" / "prompt" | No guessed civil-time conversion. |
| clock.drawer | "LOGBOOK" | Compatible clock source location. |
| clock.persist | false | Optional checked XDG clock state; no directory until selected. |
| clock.out_when_done | true | Closing task also closes its active clock coherently. |
| clock.remove_zero | false | Retain zero-minute intervals by default. |
| clock.history_limit | 32 | 0..256 checked recent task locators. |
| duration.units | Org min/h/d/w/m/y | 1/60/1440/10080/43200/525960 minutes; positive finite typed overrides. |
| timer.default | "0" | No armed timer until explicit valid input. |
| habit.preceding_days / following_days | 21 / 7 | 0..366 each; bounded visible window. |
| habit.graph_column | 40 | 0..4096 preferred cell column; clip safely. |
| habit.today_only / show_all_today | true / false | Native agenda display options. |
| habit.done_always_green | false | Retain late completion status by default. |
| agenda.sources | [] | Explicit file/directory observations; no implicit home/project scan. |
| agenda.span / week_start | "week" / 1 | day/week/month/year or bounded custom span; Monday=1. |
| agenda.views | [] | Data-only typed block/query/filter/sort/column definitions. |
| capture.templates | [] | Empty selects native Task fallback and explicit missing destination prompt. |
| capture.default_file | "" | Empty prompts; resolving a path never creates it. |
| archive.location | "%s_archive::" | Native checked source-file expansion. |
| refile.targets | [] | Empty uses selected agenda/source headings; explicit target selection. |
| attachments.directory | "data" | ID-based child directory; explicit DIR property wins for data location. |
| tables.significant_digits | 12 | 1..15 finite binary64 output digits; not arbitrary-precision Calc. |
| tables.empty_numeric | "zero" | Empty numeric cells become zero; nonnumeric direct use rejects. |
| tables.rounding | "half-away-from-zero" | The admitted native round rule. |
| export.default_format | "html" | Native backend selection; never an automatic action. |
| export.backends | Native format list in ORG25 | Typed native choices; no loadable backend registry. |
| publish.projects | [] | Named bounded project definitions; no implicit upload. |
| source.confirm_execution | true | Explicit block/call confirmation by default. |
| source.programs | [] | Optional typed executable/argv profiles; zero implicit language programs. |
| source.library_files | [] | Bounded named-block observations, not code loaded on startup. |
| feeds.sources | [] | URL/local file plus selected Org target; explicit update only. |
| tools.fetch / upload / viewer / pdf | [] | Optional argv; missing tools produce usable diagnostics, never skips. |
| preview.enabled | false | Explicit per-view command can enable; no implicit network fetch. |
| preview.max_bytes | 4194304 | 1..16777216; bounded local preview observation. |

Org/agenda/capture/calendar/source-edit context keymaps extend the existing
keys owner. Every operation uses one compiled command identity. New semantic
palette roles cover Org syntax, agenda, clock status and the habit states;
Tokyo Night values come from the existing palette roles or their documented
native defaults. All supported roles appear in the complete sample.

### Query and template data

Match expressions admit Org-style tag AND/OR/NOT, parentheses and typed property
comparisons, including TODO selection; text/regexp queries are separate typed
views. Lexing, escaping, length and depth are bounded. No predicate/function
string becomes a Rust callback or an interpreted Lisp expression.

Capture templates are named typed records: key, description, entry/item/
checkitem/table-line/plain kind, bounded text and an explicit file/headline/
outline/date-tree/table/list target. Native data-only substitutions follow the
specification. Unknown/Lisp escapes reject before destination mutation and
retain the draft. Template expansion is not editor extensibility.

External program profiles contain language/action, an executable, argv tokens,
source delivery by stdin or an owned temporary file, optional compile/run steps
and bounded result type. Placeholders are fixed data substitutions, not shell
interpolation. An explicitly selected shell language still executes the user's
confirmed source as that program's input. Arbitrary Emacs Lisp is not rev's
configuration language; a confirmed emacs-lisp block may use optional Emacs.

### Typed array records and defaults

- Agenda view: key, title and 1..32 blocks; block kind (calendar/todo/search/tags/
  properties/columns/clock), bounded span/query, source selection, column names,
  sort keys and row limit. Unknown kinds/keys reject; no function expressions.
- Capture template: key/title/kind/body and discriminated target; file, headline,
  outline path and date-tree/table/list fields are validated for that target.
  The native fallback is Task, with explicit destination entry when absent.
- Publish project: name, source roots/include/exclude patterns, output directory,
  native formats, asset selections and optional upload profile. Discovery and
  output ownership are checked before any overwrite; no implicit cleanup.
- Program profile: name/language/action, executable, argv, stdin/file delivery,
  cwd policy, at most four compile/run steps, result type and limits. Native
  adapter choices may offer installed programs only after explicit invocation;
  arbitrary startup/config evaluation is never allowed.
- Feed source: name, local path or URL, target file/heading and optional transport
  profile. Item identities and update status remain compatible FEEDSTATUS source.
- Backend options: closed typed format-specific fields plus admitted Org metadata
  and attributes; unknown options reject or remain preserved source, not hooks.

All array counts, strings, nested depth and expanded values are capped by the
resource profile. CSV/TSV preserves representable cell values and quoting;
embedded cell line breaks that cannot be represented losslessly reject before
publication rather than inventing an undocumented escape.

## Resource profile

The rows below preserve the selected finite semantic bounds. The source-analysis
byte/node/depth profile and one active job are native limits, not extra startup
leaves named `analysis_bytes`, `source_nodes`, `source_depth` or `active_job_count`.
The [shipped catalog](../../../docs/reference/defaults.md#configuration-value-catalog)
identifies the configurable limits. Those controls vary only within their stated
finite bounds and must remain
truthful on truncation. Cancellation uses the ordinary C-g/native event path.
OS reads are checked at bounded chunk/source boundaries; no hard kernel-I/O
latency is falsely promised. At most one outstanding owned analysis/IO job
prevents unbounded accumulation if an operating-system read is delayed.

| Limit under org.limits | Default | Maximum |
| --- | --- | --- |
| analysis_bytes | 8388608 | 33554432 |
| source_nodes | 200000 | 1000000 |
| source_depth | 256 | 1024 |
| agenda_files | 512 | 4096 |
| agenda_file_bytes | 8388608 | 33554432 |
| agenda_total_bytes | 67108864 | 268435456 |
| agenda_rows | 20000 | 100000 |
| query_bytes / query_depth | 4096 / 32 | 65536 / 128 |
| include_depth / include_files | 16 / 128 | 64 / 512 |
| expansion_bytes | 1048576 | 8388608 |
| formula_cells / formula_steps | 100000 / 1000000 | 1000000 / 10000000 |
| formula_depth | 64 | 256 |
| formula_decimal_places | 12 | 15 |
| export_bytes | 16777216 | 134217728 |
| feed_bytes / feed_items | 4194304 / 1000 | 16777216 / 10000 |
| job_output_bytes | 1048576 | 16777216 |
| job_seconds | 30 | 600 |
| active_job_count | 1 | 1 |

Ordinary text remains editable if structured analysis reaches its limit. UI
jobs yield/observe cancellation at most every 10ms of native compute work,
subject to an already running native/OS operation completing. Inactive mode,
closed views and unarmed timers add no polling. Countdown expiry is one-shot;
relative elapsed time is calculated when requested, not written every second.

Representative release expectations: unchanged startup and non-Org idle;
zero extra idle output and no continuing CPU work; bounded retained Org caches;
visible progress/cancellation within 150ms on the owned fixtures. Use a 1MiB /
10000-heading document, 256-source agenda, bounded habit window, 10000-cell
formula graph and supported export fixtures. Target additional retained caches
at or below 32MiB and fixture RSS at or below 96MiB. Record raw observations;
these are selected acceptance budgets, not claims of measured performance.
Earlier delivery ceilings remain mandatory and cannot be inflated to pass.

## Measured delivery profile

The [capability/evidence map](editor-org-mode-evidence.md) retains the separate
historical editor/configuration/input candidates and the actual Slice 61 terminal
and Slice 62 resource candidates. The original resource ceilings above stay
acceptance obligations. The recorded release measurements are evidence only for
their exact fixture/build/commit/tree, not certification of arbitrary OS latency
or of another candidate. The final controller-gate observations below have their
own tested identity.


Final controller gate `EXPY4POPOBG2CKQH6C4JTM5AVI` passed on `72160e46648f3f28a9c8bb7f3eecc49173fca09d`
with tree `8249daaf355af4f7be00173b5b107fb4d9c80fea`, using Linux x86-64 release resource fixtures.
The native profile covered 1MiB/10000 headings, 256 agenda sources, 733 habit days,
10000 formula cells, 11 export backends and 16 cancellation cycles. Owned fixture
identity: 260 files / 264 entries, SHA-256
`be5fc5e3a9f49e0ac2adc971121f66c92460a4d16315f86a1f0d2cc0fefcb41f`.

| Observation | Actual maximum | Existing limit |
| --- | --- | --- |
| Native process peak RSS | 10928 KiB | 98304 KiB |
| Native retained growth | 3828 KiB | 32768 KiB |
| Source analysis chunk | 626.183 us | 10000 us |
| Cancellation observation | 34.984 us | 150000 us |
| Owned terminal peak RSS | 9716 KiB | 98304 KiB |
| Owned terminal response | 12.313 ms | 150 ms |

All ten idle/timer/view states recorded zero output, zero observed CPU work and
zero open/access events, with one thread, six descriptors and no child processes.
Countdown expiry occurred once; terminal restoration, direct-child reaping and
descriptor closure passed. Org resource collection took 5.290 seconds within the
14.613-second configuration/resource stage. These observations cover the owned
fixtures and exact release build, retaining the stated OS-operation limits.

## Model readiness

Product choices follow the delegated Org behavior profile. Concrete native
fields, defaults, limits, authority and lifetimes are recorded above. The controller gate verifies the selected native contracts and resource profile;
remaining compatibility boundaries stay as specified. The completed plan/tracker
retain execution and tested-candidate provenance.
