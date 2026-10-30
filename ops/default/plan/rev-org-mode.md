# Native Org Mode Delivery

Status: Approved
Date: 2026-10-28
Delivery set: rev-org-mode
Scope authority: product owner; native Org, then clocking/habits/effort, followed by Org-in-Emacs behavior for remaining choices, retaining the advanced-spreadsheet exception.
Approved: product owner, 2026-10-28; explicitly requested starting this single 64-slice Looper cycle after delegating remaining behavior choices to Org in Emacs.
Slice strategy: layered
Active slice: Slice 59 previews
Full gate: `scripts/check-all.sh`
Planning baseline: `887feeb6c8dfec2a616d7943ac6f2a298d133bbf`
Spec: [Org Mode](../spec/editor-org-mode.md)
Model: [Org Mode Model](../spec/editor-org-mode-model.md)
Tracker: [Native Org Tracker](../tracker/rev-org-mode.md)

## Purpose and complete scope

Deliver the selected native Org capability in one Looper cycle: source-safe
editing, independent folding, metadata/tasks, dates, clocking, effort, habits,
timers, agenda/queries/columns, capture, refile/archive, links/attachments,
footnotes and markup, tables/basic formulas, data interchange, native export,
project publishing, source editing/tangling, explicit optional program jobs,
feeds and terminal/resource previews. All ORG01-ORG31 contracts belong to this
set. The existing bookmark/LSP/autocompletion proposals do not become Org work.

The core is in-house Rust in the existing executable. No shared Org crate,
embedded Emacs Lisp, plugin registry, extra service or new Cargo dependency is
part of the plan. External programs are optional explicit operation adapters,
like a PDF compiler, language runner, network transport or viewer. Their absence
must be a usable diagnosed outcome; core Org editing/agenda/capture/formulas
work offline without them. Never report optional conversion/execution as tested
merely because native output was rendered.

Tables retain the accepted basic subset. Full Calc, arbitrary-precision
spreadsheet compatibility and Lisp table formulas remain outside the selected
formula evaluator. Unsupported source remains preserved and truthful.

Layered slicing is selected because canonical source indexing, folded-view
coordinates, multi-document publication, typed contexts and owned jobs have
shared invariants. Each slice compiles and passes its focused checks. Preparatory
contracts remain distinguishable from fully operational commands; no skipped
core assertion, inert sample setting or placeholder M-x command substitutes for
a delivered behavior. All 64 slices must integrate before set acceptance.

## Existing owners and native boundaries

- Extend the canonical rope/buffer edit and history owners; retain source bytes,
  BOM/line endings, read-only state, undo/redo and independent shared views.
- Introduce native `src/org.rs` and private source/edit/time/agenda/capture/table/
  IO/export/job modules as their slices need them. Keep entrypoints thin and
  avoid copying existing file/input/view responsibilities into an Org framework.
- Folded and preview projections must map point, gutter, mouse, selection,
  searches and overlays to actual source. A hidden row cannot identify a
  different logical source position. Native job/clock locators are revision-
  qualified; cached agenda rows never authorize unchecked edits.
- Keep structural observations, agenda/report/preview results and capture
  drafts distinct from canonical editable Org documents. Multi-buffer edits and
  per-file saves have explicit outcomes; no failed durable step claims atomic
  disk success. Preserve recoverable drafts and source when revalidation fails.
- Data-local directives cannot grant process authority. External jobs have
  explicit confirmation, bounded children/IO/time and owned cleanup; table
  recalculation, visits, highlighting and export discovery never execute code.
- No global repeat/input events may be dropped to make a slow path look fast.
  Extend native readiness/deadlines only for active work; stopped timers and
  inactive Org add no idle polling. Keep the existing terminal decoder burst
  limitation distinct from this delivery rather than changing unrelated scope.

## Reference behavior and concrete decisions

The spec/model freeze the selected source behavior, defaults, commands, table
numeric policy, preference fields, query/template data and finite limits. The
reference defaults/keys were checked against a clean Emacs 31.1 / Org 9.8.7
session and primary Org documentation. No source code is ported from it.
Use the specified key meanings, including context precedence, rather than
inventing another prefix family. Do not bind clock reports to C-c C-x C-r:
that is the reference radio-checkbox action. C-g remains reserved cancellation.

All product choices formerly listed as open are resolved through the delegated
Org reference and the paired native profile. A genuinely incompatible source
form or failed resource contract discovered later is an explicit finding; it
cannot silently shrink scope, invent a result or relax acceptance.

## Validation strategy and expected cost

Required ordered command prefixes are recorded for each slice below. Six new
public targets are introduced with real behavior: org_source at Slice 1,
org_editing at 5, org_time at 16, org_io at 21, org_agenda at 22 and org_tables at
39. Existing targets remain real regressions. An affected display/input path
includes native display/command checks; each representative user workflow also
gets real terminal evidence. Use named table-driven cases and independent
fixture expectations, not tests that mirror implementation internals.

Extend the existing `scripts/configuration_ui/` collectors and admission schema.
The named Org cases below are new additions implemented in their assigned
slices, not commands claimed to work at planning time. Do not create a parallel
acceptance framework. Preserve every earlier native case, historical integration
identity, terminal/resource workload and same-head provenance guard.

The single full gate remains `scripts/check-all.sh` with its eleven command
identities/order. Add Org evidence through existing stages. It runs once on the
clean integrated candidate; catalog listings and prerequisites do not count as
runtime proof. Native tests, terminal journeys, actual file operations, supported
format validators and resource observations must cover every required contract
with zero skips. The final mapping also checks all 64 canonical integrations,
reports and completed tasks, preserving prior delivery closures.

The latest warm gate took 70.5 seconds. This larger set adds representative Org
journeys and bounded workloads; target a warm complete gate of approximately
three to six minutes. Record the measured cost and investigate expansion. Do not
multiply full terminal journeys per unit variant, run the whole gate for every
slice or silently raise limits. No test waits out a production-length timer;
use injected clocks at native boundaries and a short real expiry journey.

Every process test owns HOME, all XDG roots, cwd, temporary source/output files
and child processes. Gate admission must not read operator documents or contact
live feed endpoints. External-job acceptance uses actual owned local programs
and an intentionally missing executable; it must pass without TeX, Emacs, an
image converter, browser or network availability. ODT is checked independently
with Python ZIP/XML tooling; HTML/text/iCalendar use semantic fixtures. Do not
claim PDF generation from LaTeX text or use absent optional tools to skip tests.

## Resource acceptance

Use the explicit model profile as the source of truth. Measure a 1MiB /
10000-heading source, a 256-source agenda, habit windows, 10000-cell formulas,
representative exports and active/inactive timers. Retain the existing input
150ms responsiveness expectation on owned fixtures. Target additional retained
caches at most 32MiB and fixture RSS at most 96MiB; record actual release/debug
and OS/fixture identities separately. These are selected budgets, not measured
claims. Existing baseline ceilings cannot be weakened.

Background work is bounded and operation-owned, with at most one outstanding
analysis/IO job and finite observations. Cancellation is checked between bounded
compute/read steps; do not claim that Rust can impose a hard deadline on a
blocked kernel filesystem operation. A delayed old result cannot overwrite a
new revision. Repeated cancels cannot leak children, threads or retained drafts.

## Controller authority and recovery proposal

The approved startup instruction permits copying these reviewed documents into canonical dev,
commit the approved planning set and use one new configuration/conversation for
rev-org-mode. Do not resume, reset or edit completed runs. The proposed scope
permits exact slice PR implementation and controller-owned Rebase + Fast-forward
integration into dev, one final full gate and exact-path documentation closure.
It does not authorize main promotion, releases, mirror publication, desktop
configuration, new dependencies or arbitrary command execution by the agent.

Controller budget: 400 worker turns, five stalled turns. Pin the
native bounded-v1 policy before starting: two diagnostics per family, eight per
run, three failed repairs per family and three environmental retries. Reserve
at most four final-gate repair PRs under fix/rev-org-mode-validation, with title
fix(gate): correct native Org acceptance and canonical Slice 64 report identity.
No change to the frozen full-gate command or acceptance conditions is a repair.

Repair paths are runtime/tests/examples/reference docs and existing acceptance
collector/checker files. Exclude Cargo.toml/Cargo.lock, AGENTS/.agents, specs,
plans/trackers and scripts/check-all.sh. Enumerate exact closure docs and all 64
report paths in the generated config; do not grant arbitrary ops directory
writes. Diagnostics/repairs keep the same run, conversation, counters and actual
failed-head evidence. Never kill another process or reset state to evade a stop.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | define native Org source contracts | `feat/rev-org-01-source-contract` | `feat(slice-1): define native Org source contracts` | `ops/default/report/slices/rev-org-mode/slice-1-source-contract.md` |
| Slice 2 | index Org structure with bounded work | `feat/rev-org-02-incremental-index` | `feat(slice-2): index Org structure with bounded work` | `ops/default/report/slices/rev-org-mode/slice-2-incremental-index.md` |
| Slice 3 | recognize inline Org source forms | `feat/rev-org-03-inline-syntax` | `feat(slice-3): recognize inline Org source forms` | `ops/default/report/slices/rev-org-mode/slice-3-inline-syntax.md` |
| Slice 4 | recognize Org headings metadata and blocks | `feat/rev-org-04-structural-syntax` | `feat(slice-4): recognize Org headings metadata and blocks` | `ops/default/report/slices/rev-org-mode/slice-4-structural-syntax.md` |
| Slice 5 | activate native Org mode and contexts | `feat/rev-org-05-mode-contexts` | `feat(slice-5): activate native Org mode and contexts` | `ops/default/report/slices/rev-org-mode/slice-5-mode-contexts.md` |
| Slice 6 | color native Org source semantically | `feat/rev-org-06-semantic-colors` | `feat(slice-6): color native Org source semantically` | `ops/default/report/slices/rev-org-mode/slice-6-semantic-colors.md` |
| Slice 7 | map folded Org views to canonical source | `feat/rev-org-07-fold-projection` | `feat(slice-7): map folded Org views to canonical source` | `ops/default/report/slices/rev-org-mode/slice-7-fold-projection.md` |
| Slice 8 | deliver native Org visibility controls | `feat/rev-org-08-fold-controls` | `feat(slice-8): deliver native Org visibility controls` | `ops/default/report/slices/rev-org-mode/slice-8-fold-controls.md` |
| Slice 9 | insert and navigate Org headings | `feat/rev-org-09-outline-navigation` | `feat(slice-9): insert and navigate Org headings` | `ops/default/report/slices/rev-org-mode/slice-9-outline-navigation.md` |
| Slice 10 | move and promote coherent Org subtrees | `feat/rev-org-10-subtree-edits` | `feat(slice-10): move and promote coherent Org subtrees` | `ops/default/report/slices/rev-org-mode/slice-10-subtree-edits.md` |
| Slice 11 | edit Org lists and progress cookies | `feat/rev-org-11-lists-checkboxes` | `feat(slice-11): edit Org lists and progress cookies` | `ops/default/report/slices/rev-org-mode/slice-11-lists-checkboxes.md` |
| Slice 12 | edit Org drawers and inherited properties | `feat/rev-org-12-drawers-properties` | `feat(slice-12): edit Org drawers and inherited properties` | `ops/default/report/slices/rev-org-mode/slice-12-drawers-properties.md` |
| Slice 13 | edit and match Org task tags | `feat/rev-org-13-tags-matching` | `feat(slice-13): edit and match Org task tags` | `ops/default/report/slices/rev-org-mode/slice-13-tags-matching.md` |
| Slice 14 | deliver Org task workflows and logging | `feat/rev-org-14-todo-workflows` | `feat(slice-14): deliver Org task workflows and logging` | `ops/default/report/slices/rev-org-mode/slice-14-todo-workflows.md` |
| Slice 15 | record Org effort estimates and priorities | `feat/rev-org-15-effort-priorities` | `feat(slice-15): record Org effort estimates and priorities` | `ops/default/report/slices/rev-org-mode/slice-15-effort-priorities.md` |
| Slice 16 | model Org calendar dates and durations | `feat/rev-org-16-civil-time` | `feat(slice-16): model Org calendar dates and durations` | `ops/default/report/slices/rev-org-mode/slice-16-civil-time.md` |
| Slice 17 | resolve Org local times and zone changes | `feat/rev-org-17-time-zones` | `feat(slice-17): resolve Org local times and zone changes` | `ops/default/report/slices/rev-org-mode/slice-17-time-zones.md` |
| Slice 18 | enter and edit Org timestamps | `feat/rev-org-18-timestamp-entry` | `feat(slice-18): enter and edit Org timestamps` | `ops/default/report/slices/rev-org-mode/slice-18-timestamp-entry.md` |
| Slice 19 | reschedule repeated Org tasks correctly | `feat/rev-org-19-repeaters` | `feat(slice-19): reschedule repeated Org tasks correctly` | `ops/default/report/slices/rev-org-mode/slice-19-repeaters.md` |
| Slice 20 | record native Org task clock intervals | `feat/rev-org-20-clock-records` | `feat(slice-20): record native Org task clock intervals` | `ops/default/report/slices/rev-org-mode/slice-20-clock-records.md` |
| Slice 21 | switch and recover Org task clocks | `feat/rev-org-21-clock-lifecycle` | `feat(slice-21): switch and recover Org task clocks` | `ops/default/report/slices/rev-org-mode/slice-21-clock-lifecycle.md` |
| Slice 22 | compare Org clock totals and effort | `feat/rev-org-22-clock-reports` | `feat(slice-22): compare Org clock totals and effort` | `ops/default/report/slices/rev-org-mode/slice-22-clock-reports.md` |
| Slice 23 | deliver native Org relative and countdown timers | `feat/rev-org-23-timers` | `feat(slice-23): deliver native Org relative and countdown timers` | `ops/default/report/slices/rev-org-mode/slice-23-timers.md` |
| Slice 24 | maintain Org habit schedules and history | `feat/rev-org-24-habit-records` | `feat(slice-24): maintain Org habit schedules and history` | `ops/default/report/slices/rev-org-mode/slice-24-habit-records.md` |
| Slice 25 | render native Org habit consistency graphs | `feat/rev-org-25-habit-graphs` | `feat(slice-25): render native Org habit consistency graphs` | `ops/default/report/slices/rev-org-mode/slice-25-habit-graphs.md` |
| Slice 26 | discover bounded canonical Org agenda sources | `feat/rev-org-26-agenda-sources` | `feat(slice-26): discover bounded canonical Org agenda sources` | `ops/default/report/slices/rev-org-mode/slice-26-agenda-sources.md` |
| Slice 27 | display native day and week Org agendas | `feat/rev-org-27-agenda-calendar` | `feat(slice-27): display native day and week Org agendas` | `ops/default/report/slices/rev-org-mode/slice-27-agenda-calendar.md` |
| Slice 28 | query Org tasks tags and properties | `feat/rev-org-28-agenda-queries` | `feat(slice-28): query Org tasks tags and properties` | `ops/default/report/slices/rev-org-mode/slice-28-agenda-queries.md` |
| Slice 29 | configure native Org agenda views | `feat/rev-org-29-agenda-customization` | `feat(slice-29): configure native Org agenda views` | `ops/default/report/slices/rev-org-mode/slice-29-agenda-customization.md` |
| Slice 30 | edit canonical Org source from the agenda | `feat/rev-org-30-agenda-actions` | `feat(slice-30): edit canonical Org source from the agenda` | `ops/default/report/slices/rev-org-mode/slice-30-agenda-actions.md` |
| Slice 31 | define native Org capture targets and templates | `feat/rev-org-31-capture-templates` | `feat(slice-31): define native Org capture targets and templates` | `ops/default/report/slices/rev-org-mode/slice-31-capture-templates.md` |
| Slice 32 | expand and edit native Org capture drafts | `feat/rev-org-32-capture-drafts` | `feat(slice-32): expand and edit native Org capture drafts` | `ops/default/report/slices/rev-org-mode/slice-32-capture-drafts.md` |
| Slice 33 | finalize or abort Org capture safely | `feat/rev-org-33-capture-finalize` | `feat(slice-33): finalize or abort Org capture safely` | `ops/default/report/slices/rev-org-mode/slice-33-capture-finalize.md` |
| Slice 34 | refile Org subtrees across canonical documents | `feat/rev-org-34-refile` | `feat(slice-34): refile Org subtrees across canonical documents` | `ops/default/report/slices/rev-org-mode/slice-34-refile.md` |
| Slice 35 | archive Org subtrees without silent loss | `feat/rev-org-35-archive` | `feat(slice-35): archive Org subtrees without silent loss` | `ops/default/report/slices/rev-org-mode/slice-35-archive.md` |
| Slice 36 | store and follow checked Org links | `feat/rev-org-36-links` | `feat(slice-36): store and follow checked Org links` | `ops/default/report/slices/rev-org-mode/slice-36-links.md` |
| Slice 37 | manage native Org heading attachments | `feat/rev-org-37-attachments` | `feat(slice-37): manage native Org heading attachments` | `ops/default/report/slices/rev-org-mode/slice-37-attachments.md` |
| Slice 38 | edit native Org footnotes and inline content | `feat/rev-org-38-document-conveniences` | `feat(slice-38): edit native Org footnotes and inline content` | `ops/default/report/slices/rev-org-mode/slice-38-document-conveniences.md` |
| Slice 39 | navigate and align native Org tables | `feat/rev-org-39-table-basics` | `feat(slice-39): navigate and align native Org tables` | `ops/default/report/slices/rev-org-mode/slice-39-table-basics.md` |
| Slice 40 | edit Org table rows columns and references | `feat/rev-org-40-table-structure` | `feat(slice-40): edit Org table rows columns and references` | `ops/default/report/slices/rev-org-mode/slice-40-table-structure.md` |
| Slice 41 | parse the bounded Org formula subset | `feat/rev-org-41-formula-syntax` | `feat(slice-41): parse the bounded Org formula subset` | `ops/default/report/slices/rev-org-mode/slice-41-formula-syntax.md` |
| Slice 42 | evaluate basic Org table formulas safely | `feat/rev-org-42-formula-evaluation` | `feat(slice-42): evaluate basic Org table formulas safely` | `ops/default/report/slices/rev-org-mode/slice-42-formula-evaluation.md` |
| Slice 43 | publish Org recalculation as one transaction | `feat/rev-org-43-formula-publication` | `feat(slice-43): publish Org recalculation as one transaction` | `ops/default/report/slices/rev-org-mode/slice-43-formula-publication.md` |
| Slice 44 | import and export Org table data | `feat/rev-org-44-table-interchange` | `feat(slice-44): import and export Org table data` | `ops/default/report/slices/rev-org-mode/slice-44-table-interchange.md` |
| Slice 45 | display Org property columns and native reports | `feat/rev-org-45-columns-dynamic-blocks` | `feat(slice-45): display Org property columns and native reports` | `ops/default/report/slices/rev-org-mode/slice-45-columns-dynamic-blocks.md` |
| Slice 46 | prepare checked native Org export snapshots | `feat/rev-org-46-export-contract` | `feat(slice-46): prepare checked native Org export snapshots` | `ops/default/report/slices/rev-org-mode/slice-46-export-contract.md` |
| Slice 47 | export Org to text Markdown and Org | `feat/rev-org-47-text-exports` | `feat(slice-47): export Org to text Markdown and Org` | `ops/default/report/slices/rev-org-mode/slice-47-text-exports.md` |
| Slice 48 | export native Org documents to HTML | `feat/rev-org-48-html-export` | `feat(slice-48): export native Org documents to HTML` | `ops/default/report/slices/rev-org-mode/slice-48-html-export.md` |
| Slice 49 | export Org to LaTeX and Beamer | `feat/rev-org-49-tex-exports` | `feat(slice-49): export Org to LaTeX and Beamer` | `ops/default/report/slices/rev-org-mode/slice-49-tex-exports.md` |
| Slice 50 | export native Org to OpenDocument text | `feat/rev-org-50-odt-export` | `feat(slice-50): export native Org to OpenDocument text` | `ops/default/report/slices/rev-org-mode/slice-50-odt-export.md` |
| Slice 51 | export Org to Texinfo and man formats | `feat/rev-org-51-manual-exports` | `feat(slice-51): export Org to Texinfo and man formats` | `ops/default/report/slices/rev-org-mode/slice-51-manual-exports.md` |
| Slice 52 | export Org planning data as iCalendar | `feat/rev-org-52-calendar-export` | `feat(slice-52): export Org planning data as iCalendar` | `ops/default/report/slices/rev-org-mode/slice-52-calendar-export.md` |
| Slice 53 | publish native Org document projects | `feat/rev-org-53-publishing` | `feat(slice-53): publish native Org document projects` | `ops/default/report/slices/rev-org-mode/slice-53-publishing.md` |
| Slice 54 | edit native Org source blocks and headers | `feat/rev-org-54-source-block-editing` | `feat(slice-54): edit native Org source blocks and headers` | `ops/default/report/slices/rev-org-mode/slice-54-source-block-editing.md` |
| Slice 55 | tangle Org source with bounded noweb references | `feat/rev-org-55-tangling` | `feat(slice-55): tangle Org source with bounded noweb references` | `ops/default/report/slices/rev-org-mode/slice-55-tangling.md` |
| Slice 56 | execute selected Org blocks through bounded tools | `feat/rev-org-56-block-execution` | `feat(slice-56): execute selected Org blocks through bounded tools` | `ops/default/report/slices/rev-org-mode/slice-56-block-execution.md` |
| Slice 57 | insert checked Org block and call results | `feat/rev-org-57-block-results` | `feat(slice-57): insert checked Org block and call results` | `ops/default/report/slices/rev-org-mode/slice-57-block-results.md` |
| Slice 58 | import native Org feed entries on demand | `feat/rev-org-58-feeds` | `feat(slice-58): import native Org feed entries on demand` | `ops/default/report/slices/rev-org-mode/slice-58-feeds.md` |
| Slice 59 | preview native Org links and images | `feat/rev-org-59-previews` | `feat(slice-59): preview native Org links and images` | `ops/default/report/slices/rev-org-mode/slice-59-previews.md` |
| Slice 60 | complete the native Org control catalog | `feat/rev-org-60-complete-controls` | `feat(slice-60): complete the native Org control catalog` | `ops/default/report/slices/rev-org-mode/slice-60-complete-controls.md` |
| Slice 61 | verify complete native Org terminal workflows | `test/rev-org-61-combined-terminal` | `test(slice-61): verify complete native Org terminal workflows` | `ops/default/report/slices/rev-org-mode/slice-61-combined-terminal.md` |
| Slice 62 | verify bounded Org work and idle resources | `perf/rev-org-62-bounded-work` | `perf(slice-62): verify bounded Org work and idle resources` | `ops/default/report/slices/rev-org-mode/slice-62-bounded-work.md` |
| Slice 63 | reconcile native Org contracts and references | `docs/rev-org-63-baseline-reconciliation` | `docs(slice-63): reconcile native Org contracts and references` | `ops/default/report/slices/rev-org-mode/slice-63-baseline-reconciliation.md` |
| Slice 64 | prepare complete native Org delivery acceptance | `test/rev-org-64-delivery-acceptance` | `test(slice-64): prepare complete native Org delivery acceptance` | `ops/default/report/slices/rev-org-mode/slice-64-delivery-acceptance.md` |

## Requirement coverage

| Requirement | Assigned slices |
| --- | --- |
| ORG01 | 1, 5, 61, 63, 64 |
| ORG02 | 1, 2, 3, 4, 10, 32, 34, 36, 38, 46, 54, 61 |
| ORG03 | 7, 8 |
| ORG04 | 4, 9, 10, 11 |
| ORG05 | 4, 12, 13 |
| ORG06 | 11, 14, 15 |
| ORG07 | 16, 17, 18, 19, 24, 52 |
| ORG08 | 26, 27, 28 |
| ORG09 | 27, 30, 35 |
| ORG10 | 31, 32, 33 |
| ORG11 | 34, 35 |
| ORG12 | 4, 39, 40, 44 |
| ORG13 | 41, 42, 43 |
| ORG14 | 40, 41, 42, 43, 57 |
| ORG15 | 3, 36, 37, 59 |
| ORG16 | 3, 6, 25 |
| ORG17 | 5, 6, 8, 9, 12, 14, 18, 29, 31, 39, 43, 60, 61, 63 |
| ORG18 | 1, 2, 7, 16, 17, 21, 23, 26, 33, 50, 55, 56, 58, 61, 62, 64 |
| ORG19 | 20, 21, 22 |
| ORG20 | 19, 24, 25 |
| ORG21 | 15, 16, 20, 22, 45 |
| ORG22 | 23 |
| ORG23 | 13, 22, 28, 29, 30, 45 |
| ORG24 | 37 |
| ORG25 | 46, 47, 48, 49, 50, 51, 52, 53 |
| ORG26 | 53 |
| ORG27 | 38, 44, 45, 47, 51 |
| ORG28 | 54, 55, 56, 57 |
| ORG29 | 58 |
| ORG30 | 3, 11, 38, 48, 49, 59 |
| ORG31 | 60, 62, 63, 64 |

## Slice details

## Slice 1: define native Org source contracts

Contracts: ORG01 ORG02 ORG18.
Prerequisite: approved committed plan/tracker and delivered baseline.
Branch: `feat/rev-org-01-source-contract`
Expected PR title: `feat(slice-1): define native Org source contracts`
Report: `ops/default/report/slices/rev-org-mode/slice-1-source-contract.md`

- T1.1: Introduce the in-house Org owner and revision-qualified source ranges; keep canonical rope bytes authoritative.
- T1.2: Represent recognized and unknown forms without executing directives or retaining a second editable document.
- T1.3: Introduce the org_source target and verify UTF-8, CRLF, malformed input and checked-range failures.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test coordinates`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 2: index Org structure with bounded work

Contracts: ORG02 ORG18.
Prerequisite: Slice 1 integrated in this same delivery set.
Branch: `feat/rev-org-02-incremental-index`
Expected PR title: `feat(slice-2): index Org structure with bounded work`
Report: `ops/default/report/slices/rev-org-mode/slice-2-incremental-index.md`

- T2.1: Build bounded line/ancestor indexes with explicit parse observations and truthful incomplete states.
- T2.2: Invalidate affected scopes after canonical edits and introduce the shared bounded cancellable observation-job owner; never copy the full rope per key.
- T2.3: Verify large edits, stale observations and bounded worker cancellation while preserving ordinary editing.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test editing`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 3: recognize inline Org source forms

Contracts: ORG02 ORG15 ORG16 ORG30.
Prerequisite: Slice 2 integrated in this same delivery set.
Branch: `feat/rev-org-03-inline-syntax`
Expected PR title: `feat(slice-3): recognize inline Org source forms`
Report: `ops/default/report/slices/rev-org-mode/slice-3-inline-syntax.md`

- T3.1: Recognize emphasis, code, entities, links, targets and footnotes with exact source spans.
- T3.2: Keep escaping, malformed delimiters and literal/code contexts distinct from structural markup.
- T3.3: Verify Unicode boundaries, nested forms and recovery without normalizing unknown source.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test coordinates`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 4: recognize Org headings metadata and blocks

Contracts: ORG02 ORG04 ORG05 ORG12.
Prerequisite: Slice 3 integrated in this same delivery set.
Branch: `feat/rev-org-04-structural-syntax`
Expected PR title: `feat(slice-4): recognize Org headings metadata and blocks`
Report: `ops/default/report/slices/rev-org-mode/slice-4-structural-syntax.md`

- T4.1: Recognize headings, sections, lists, drawers, planning lines, affiliated keywords and tables.
- T4.2: Treat source/example/export blocks as owned literal regions with valid delimiter recovery.
- T4.3: Verify subtree ownership, malformed drawers and apparent headings inside blocks.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test coordinates`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 5: activate native Org mode and contexts

Contracts: ORG01 ORG17.
Prerequisite: Slice 4 integrated in this same delivery set.
Branch: `feat/rev-org-05-mode-contexts`
Expected PR title: `feat(slice-5): activate native Org mode and contexts`
Report: `ops/default/report/slices/rev-org-mode/slice-5-mode-contexts.md`

- T5.1: Activate the native Org mode by visiting suffix or explicit compiled mode command.
- T5.2: Add Org editing, agenda, capture, calendar and special-edit context routing with prompt precedence.
- T5.3: Introduce org_editing; expose only operational commands and verify context cancellation and plain-text fallback.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test configuration --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 6: color native Org source semantically

Contracts: ORG16 ORG17.
Prerequisite: Slice 5 integrated in this same delivery set.
Branch: `feat/rev-org-06-semantic-colors`
Expected PR title: `feat(slice-6): color native Org source semantically`
Report: `ops/default/report/slices/rev-org-mode/slice-6-semantic-colors.md`

- T6.1: Project native Org spans through existing palette and display owners, including literal-block boundaries.
- T6.2: Define typed semantic Org roles with complete factory/sample coverage as each role activates.
- T6.3: Verify incremental coloring, shared views, malformed source and usable no-color rendering.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test syntax --test display`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 7: map folded Org views to canonical source

Contracts: ORG03 ORG18.
Prerequisite: Slice 6 integrated in this same delivery set.
Branch: `feat/rev-org-07-fold-projection`
Expected PR title: `feat(slice-7): map folded Org views to canonical source`
Report: `ops/default/report/slices/rev-org-mode/slice-7-fold-projection.md`

- T7.1: Add view-local visibility projection over source ranges, retaining independent shared-buffer views.
- T7.2: Map cursor, gutter, search, mouse and selections through hidden ranges without aliasing source positions.
- T7.3: Verify tiny panes, Unicode, hit testing, stale folds and edit-driven reveal.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test display --test gutter --test pointer`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 8: deliver native Org visibility controls

Contracts: ORG03 ORG17.
Prerequisite: Slice 7 integrated in this same delivery set.
Branch: `feat/rev-org-08-fold-controls`
Expected PR title: `feat(slice-8): deliver native Org visibility controls`
Report: `ops/default/report/slices/rev-org-mode/slice-8-fold-controls.md`

- T8.1: Implement heading TAB cycle, whole-view S-TAB cycle and direct visibility/drawer commands.
- T8.2: Integrate hierarchy-aware reveal with ordinary motion, searches and saved independent view state.
- T8.3: Exercise native keys, M-x, cancellation and source immutability in an owned outline PTY journey.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test commands --test display`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-outline`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 9: insert and navigate Org headings

Contracts: ORG04 ORG17.
Prerequisite: Slice 8 integrated in this same delivery set.
Branch: `feat/rev-org-09-outline-navigation`
Expected PR title: `feat(slice-9): insert and navigate Org headings`
Report: `ops/default/report/slices/rev-org-mode/slice-9-outline-navigation.md`

- T9.1: Deliver sibling insertion and next/previous/same-level/parent heading movement.
- T9.2: Handle contextual M-RET while preserving attached sections, blank lines and final newline state.
- T9.3: Verify read-only navigation, inactive context behavior, undo and logical source bounds.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test commands --test editing`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 10: move and promote coherent Org subtrees

Contracts: ORG04 ORG02.
Prerequisite: Slice 9 integrated in this same delivery set.
Branch: `feat/rev-org-10-subtree-edits`
Expected PR title: `feat(slice-10): move and promote coherent Org subtrees`
Report: `ops/default/report/slices/rev-org-mode/slice-10-subtree-edits.md`

- T10.1: Implement checked promote/demote and sibling subtree moves over complete owned source.
- T10.2: Retain descendants, affiliated metadata, marks and independent views through one atomic history change.
- T10.3: Verify boundary rejections, malformed parents, regional edits and full undo/redo fidelity.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test editing --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 11: edit Org lists and progress cookies

Contracts: ORG04 ORG06 ORG30.
Prerequisite: Slice 10 integrated in this same delivery set.
Branch: `feat/rev-org-11-lists-checkboxes`
Expected PR title: `feat(slice-11): edit Org lists and progress cookies`
Report: `ops/default/report/slices/rev-org-mode/slice-11-lists-checkboxes.md`

- T11.1: Continue, indent, outdent, reorder and renumber ordered/unordered/description lists contextually.
- T11.2: Implement ordinary/radio checkboxes and coherent list/task progress cookies through checked edits.
- T11.3: Verify nested lists, timer-item contexts, literal blocks and one-operation undo.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test editing --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 12: edit Org drawers and inherited properties

Contracts: ORG05 ORG17.
Prerequisite: Slice 11 integrated in this same delivery set.
Branch: `feat/rev-org-12-drawers-properties`
Expected PR title: `feat(slice-12): edit Org drawers and inherited properties`
Report: `ops/default/report/slices/rev-org-mode/slice-12-drawers-properties.md`

- T12.1: Implement property set/remove/inspect and drawer folding without confusing ordinary and property drawers.
- T12.2: Resolve local/additive/inherited values with source-owner provenance and safe directive precedence.
- T12.3: Verify selective inheritance, unknown properties and edits to the actual owner.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test org_source --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 13: edit and match Org task tags

Contracts: ORG05 ORG23.
Prerequisite: Slice 12 integrated in this same delivery set.
Branch: `feat/rev-org-13-tags-matching`
Expected PR title: `feat(slice-13): edit and match Org task tags`
Report: `ops/default/report/slices/rev-org-mode/slice-13-tags-matching.md`

- T13.1: Deliver tag entry, removal, completion and local/inherited membership inspection.
- T13.2: Implement the declared Org tag/property match grammar with typed comparisons and bounded parsing.
- T13.3: Verify case sensitivity, escaping, malformed queries and unchanged unrelated source.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test org_source --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 14: deliver Org task workflows and logging

Contracts: ORG06 ORG17.
Prerequisite: Slice 13 integrated in this same delivery set.
Branch: `feat/rev-org-14-todo-workflows`
Expected PR title: `feat(slice-14): deliver Org task workflows and logging`
Report: `ops/default/report/slices/rev-org-mode/slice-14-todo-workflows.md`

- T14.1: Admit TODO/SEQ_TODO/TYP_TODO declarations and complete native state transitions with selected logging.
- T14.2: Integrate priorities, blocking rules, CLOSED/state notes and cookies into coherent transactions.
- T14.3: Verify key/M-x equivalence, read-only behavior and task/metadata history in an owned PTY journey.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test configuration --test commands`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tasks`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 15: record Org effort estimates and priorities

Contracts: ORG06 ORG21.
Prerequisite: Slice 14 integrated in this same delivery set.
Branch: `feat/rev-org-15-effort-priorities`
Expected PR title: `feat(slice-15): record Org effort estimates and priorities`
Report: `ops/default/report/slices/rev-org-mode/slice-15-effort-priorities.md`

- T15.1: Deliver native priority and EFFORT editing with Org duration forms and factory choices.
- T15.2: Keep workflow metadata compatible and expose operational commands in Org and agenda contexts.
- T15.3: Verify numeric/duration rejection, source spelling preservation and atomic undo.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test configuration --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 16: model Org calendar dates and durations

Contracts: ORG07 ORG18 ORG21.
Prerequisite: Slice 15 integrated in this same delivery set.
Branch: `feat/rev-org-16-civil-time`
Expected PR title: `feat(slice-16): model Org calendar dates and durations`
Report: `ops/default/report/slices/rev-org-mode/slice-16-civil-time.md`

- T16.1: Introduce org_time with checked Gregorian civil dates, intervals and Org duration units.
- T16.2: Separate scheduling calendar arithmetic from elapsed work durations and formatting.
- T16.3: Verify leap years, month ends, overflow, duration forms and an injected operation clock.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_source`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 17: resolve Org local times and zone changes

Contracts: ORG07 ORG18.
Prerequisite: Slice 16 integrated in this same delivery set.
Branch: `feat/rev-org-17-time-zones`
Expected PR title: `feat(slice-17): resolve Org local times and zone changes`
Report: `ops/default/report/slices/rev-org-mode/slice-17-time-zones.md`

- T17.1: Implement bounded native TZif/POSIX-zone observations and system/local/UTC/IANA selection.
- T17.2: Capture effective date/zone once per operation; resolve folds/gaps explicitly without guessing.
- T17.3: Verify DST transitions, invalid zones and external zone changes with owned fixtures.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 18: enter and edit Org timestamps

Contracts: ORG07 ORG17.
Prerequisite: Slice 17 integrated in this same delivery set.
Branch: `feat/rev-org-18-timestamp-entry`
Expected PR title: `feat(slice-18): enter and edit Org timestamps`
Report: `ops/default/report/slices/rev-org-mode/slice-18-timestamp-entry.md`

- T18.1: Deliver native date/time/calendar prompts for active/inactive timestamps, ranges, schedule and deadline.
- T18.2: Use familiar context actions and cancellation, retaining weekday/source formatting where untouched.
- T18.3: Verify invalid dates, ambiguous local times, Unicode prompts and one-command undo.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_editing --test display --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 19: reschedule repeated Org tasks correctly

Contracts: ORG07 ORG20.
Prerequisite: Slice 18 integrated in this same delivery set.
Branch: `feat/rev-org-19-repeaters`
Expected PR title: `feat(slice-19): reschedule repeated Org tasks correctly`
Report: `ops/default/report/slices/rev-org-mode/slice-19-repeaters.md`

- T19.1: Implement distinct +, ++ and .+ recurrence, warning delays and habit interval ranges.
- T19.2: Apply repeat completion, state history, LAST_REPEAT and planning changes coherently.
- T19.3: Verify overdue/backlog, calendar boundaries, duplicate completion dates and full undo.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_editing`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 20: record native Org task clock intervals

Contracts: ORG19 ORG21.
Prerequisite: Slice 19 integrated in this same delivery set.
Branch: `feat/rev-org-20-clock-records`
Expected PR title: `feat(slice-20): record native Org task clock intervals`
Report: `ops/default/report/slices/rev-org-mode/slice-20-clock-records.md`

- T20.1: Deliver clock-in/out/cancel/goto with compatible CLOCK source and task-owned interval durations.
- T20.2: Preserve document transactions and remove or retain zero intervals according to the frozen defaults.
- T20.3: Verify normal save/read-only/history boundaries and no per-second idle source updates.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_editing --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 21: switch and recover Org task clocks

Contracts: ORG19 ORG18.
Prerequisite: Slice 20 integrated in this same delivery set.
Branch: `feat/rev-org-21-clock-lifecycle`
Expected PR title: `feat(slice-21): switch and recover Org task clocks`
Report: `ops/default/report/slices/rev-org-mode/slice-21-clock-lifecycle.md`

- T21.1: Enforce one active session clock, checked task switching and completion-triggered clock-out.
- T21.2: Handle quit/close, stale edits, undo and explicit incomplete-clock resolution without losing source.
- T21.3: Introduce org_io and verify optional state persistence, failed recovery and no unsolicited startup scans.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_io --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 22: compare Org clock totals and effort

Contracts: ORG19 ORG21 ORG23.
Prerequisite: Slice 21 integrated in this same delivery set.
Branch: `feat/rev-org-22-clock-reports`
Expected PR title: `feat(slice-22): compare Org clock totals and effort`
Report: `ops/default/report/slices/rev-org-mode/slice-22-clock-reports.md`

- T22.1: Deliver bounded task/subtree/file/date-range clock totals and effort comparisons.
- T22.2: Generate compatible clock-table reports only on explicit refresh and preserve unrelated dynamic-block source.
- T22.3: Introduce org_agenda contracts and verify overlaps, open intervals and transactional report replacement.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_agenda`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 23: deliver native Org relative and countdown timers

Contracts: ORG22 ORG18.
Prerequisite: Slice 22 integrated in this same delivery set.
Branch: `feat/rev-org-23-timers`
Expected PR title: `feat(slice-23): deliver native Org relative and countdown timers`
Report: `ops/default/report/slices/rev-org-mode/slice-23-timers.md`

- T23.1: Implement relative time insertion/list items, pause/resume/stop and countdown entry with familiar bindings.
- T23.2: Arm only selected deadlines in the existing native event owner; cancel before due work and retain terminal cleanup.
- T23.3: Verify one-shot expiry, restart, exit and no inactive wakeups in the owned Org time journey.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test commands --test display --test recovery`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-time`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 24: maintain Org habit schedules and history

Contracts: ORG20 ORG07.
Prerequisite: Slice 23 integrated in this same delivery set.
Branch: `feat/rev-org-24-habit-records`
Expected PR title: `feat(slice-24): maintain Org habit schedules and history`
Report: `ops/default/report/slices/rev-org-mode/slice-24-habit-records.md`

- T24.1: Recognize STYLE habit and integrate logged completion, interval ranges and Org day semantics.
- T24.2: Preserve compatible histories and distinguish missing observations from actual failed/completed days.
- T24.3: Verify recurrence types, source-local declarations and coherent completion undo.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_editing`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 25: render native Org habit consistency graphs

Contracts: ORG20 ORG16.
Prerequisite: Slice 24 integrated in this same delivery set.
Branch: `feat/rev-org-25-habit-graphs`
Expected PR title: `feat(slice-25): render native Org habit consistency graphs`
Report: `ops/default/report/slices/rev-org-mode/slice-25-habit-graphs.md`

- T25.1: Render bounded habit graphs with verified defaults, completion/today markers and semantic palette states.
- T25.2: Deliver agenda visibility, all-today/today-only and graph controls through native commands.
- T25.3: Verify narrow panes, late completions, dirty source observations and idle immutability.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_agenda --test display --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 26: discover bounded canonical Org agenda sources

Contracts: ORG08 ORG18.
Prerequisite: Slice 25 integrated in this same delivery set.
Branch: `feat/rev-org-26-agenda-sources`
Expected PR title: `feat(slice-26): discover bounded canonical Org agenda sources`
Report: `ops/default/report/slices/rev-org-mode/slice-26-agenda-sources.md`

- T26.1: Resolve explicitly selected files/directories without scanning the home or project implicitly.
- T26.2: Prefer current buffers, deduplicate real file identities and observe disk sources through cancellable owned work.
- T26.3: Verify unreadable sources, cycles, limits and truthful partial results while keeping editing responsive.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_agenda --test org_io --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 27: display native day and week Org agendas

Contracts: ORG08 ORG09.
Prerequisite: Slice 26 integrated in this same delivery set.
Branch: `feat/rev-org-27-agenda-calendar`
Expected PR title: `feat(slice-27): display native day and week Org agendas`
Report: `ops/default/report/slices/rev-org-mode/slice-27-agenda-calendar.md`

- T27.1: Deliver read-only agenda rows, time grid, overdue/scheduled/deadline sections and day/week navigation.
- T27.2: Use revision-qualified source locators and the captured effective date/zone for every row.
- T27.3: Verify real refresh, source return, shared buffers and no agenda-as-document mutation.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_agenda --test display --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 28: query Org tasks tags and properties

Contracts: ORG08 ORG23.
Prerequisite: Slice 27 integrated in this same delivery set.
Branch: `feat/rev-org-28-agenda-queries`
Expected PR title: `feat(slice-28): query Org tasks tags and properties`
Report: `ops/default/report/slices/rev-org-mode/slice-28-agenda-queries.md`

- T28.1: Deliver TODO/search/tag/property views with declared Boolean/comparison grammar and source scopes.
- T28.2: Evaluate bounded parsed queries against canonical metadata, including inheritance and effort.
- T28.3: Verify escaping, invalid operators, completeness and stable result identity.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_agenda --test org_source`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 29: configure native Org agenda views

Contracts: ORG23 ORG17.
Prerequisite: Slice 28 integrated in this same delivery set.
Branch: `feat/rev-org-29-agenda-customization`
Expected PR title: `feat(slice-29): configure native Org agenda views`
Report: `ops/default/report/slices/rev-org-mode/slice-29-agenda-customization.md`

- T29.1: Deliver typed custom/block views, columns, filtering, sorting and declared date spans.
- T29.2: Expose compiled view/refresh controls and native agenda context bindings without a scripting registry.
- T29.3: Verify combination limits, visibility/reset behavior and consistent exhaustive sample defaults.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_agenda --test configuration --test display`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 30: edit canonical Org source from the agenda

Contracts: ORG09 ORG23.
Prerequisite: Slice 29 integrated in this same delivery set.
Branch: `feat/rev-org-30-agenda-actions`
Expected PR title: `feat(slice-30): edit canonical Org source from the agenda`
Report: `ops/default/report/slices/rev-org-mode/slice-30-agenda-actions.md`

- T30.1: Route agenda task/date/tag/priority/effort/clock actions through checked source handlers.
- T30.2: Deliver explicit selection, bulk operations and per-target outcomes with revision re-resolution.
- T30.3: Exercise dirty buffers, cancellation, failure and undo in an owned multi-source agenda journey.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_agenda --test org_editing --test org_io`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-agenda`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 31: define native Org capture targets and templates

Contracts: ORG10 ORG17.
Prerequisite: Slice 30 integrated in this same delivery set.
Branch: `feat/rev-org-31-capture-templates`
Expected PR title: `feat(slice-31): define native Org capture targets and templates`
Report: `ops/default/report/slices/rev-org-mode/slice-31-capture-templates.md`

- T31.1: Admit typed native capture templates/targets and the source-compatible factory fallback.
- T31.2: Resolve file/headline/outline/date-tree/table/list targets without writing them during discovery.
- T31.3: Verify explicit destination entry, missing targets and template/default/sample coherence.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_agenda --test configuration --test org_io`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 32: expand and edit native Org capture drafts

Contracts: ORG10 ORG02.
Prerequisite: Slice 31 integrated in this same delivery set.
Branch: `feat/rev-org-32-capture-drafts`
Expected PR title: `feat(slice-32): expand and edit native Org capture drafts`
Report: `ops/default/report/slices/rev-org-mode/slice-32-capture-drafts.md`

- T32.1: Expand the declared bounded text substitutions, prompts and checked origin links without evaluating Lisp.
- T32.2: Retain the editable capture draft in its own transient owner with familiar context bindings.
- T32.3: Verify malformed expansion, literal preservation, cancellation and return to the originating view.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_agenda --test org_editing --test commands --test display`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 33: finalize or abort Org capture safely

Contracts: ORG10 ORG18.
Prerequisite: Slice 32 integrated in this same delivery set.
Branch: `feat/rev-org-33-capture-finalize`
Expected PR title: `feat(slice-33): finalize or abort Org capture safely`
Report: `ops/default/report/slices/rev-org-mode/slice-33-capture-finalize.md`

- T33.1: Revalidate destinations and publish capture once through ordinary source/file safeguards.
- T33.2: Preserve a recoverable draft after stale, read-only, save or filesystem failure; abort leaves targets unchanged.
- T33.3: Verify file/heading/list/table/date-tree capture and failed publication with actual owned files.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_agenda --test org_io --test files --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 34: refile Org subtrees across canonical documents

Contracts: ORG11 ORG02.
Prerequisite: Slice 33 integrated in this same delivery set.
Branch: `feat/rev-org-34-refile`
Expected PR title: `feat(slice-34): refile Org subtrees across canonical documents`
Report: `ops/default/report/slices/rev-org-mode/slice-34-refile.md`

- T34.1: Prepare cross-buffer subtree moves and destination insertion as one checked semantic operation.
- T34.2: Keep source/destination revisions, history groups and durable-save outcomes explicit and recoverable.
- T34.3: Verify duplicate/stale targets, read-only buffers, external file replacement and partial publication truth.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_editing --test files --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 35: archive Org subtrees without silent loss

Contracts: ORG11 ORG09.
Prerequisite: Slice 34 integrated in this same delivery set.
Branch: `feat/rev-org-35-archive`
Expected PR title: `feat(slice-35): archive Org subtrees without silent loss`
Report: `ops/default/report/slices/rev-org-mode/slice-35-archive.md`

- T35.1: Deliver archive tag, archive sibling and %s_archive destination behavior with Org metadata.
- T35.2: Reuse checked multi-document moves and retain archived source discoverability under selected agenda policy.
- T35.3: Exercise capture/refile/archive, undo and failed destinations in one owned terminal journey.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_editing --test org_agenda`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-capture`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 36: store and follow checked Org links

Contracts: ORG15 ORG02.
Prerequisite: Slice 35 integrated in this same delivery set.
Branch: `feat/rev-org-36-links`
Expected PR title: `feat(slice-36): store and follow checked Org links`
Report: `ops/default/report/slices/rev-org-mode/slice-36-links.md`

- T36.1: Deliver internal targets, file/ID/attachment links, insertion, storage and navigation.
- T36.2: Re-resolve source and dispatch external resources through the bounded owned job path only after explicit follow; introduce the shared child-lifetime primitive here.
- T36.3: Verify ambiguity, encoded paths, read-only documents and source-preserving failure.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test org_io --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 37: manage native Org heading attachments

Contracts: ORG24 ORG15.
Prerequisite: Slice 36 integrated in this same delivery set.
Branch: `feat/rev-org-37-attachments`
Expected PR title: `feat(slice-37): manage native Org heading attachments`
Report: `ops/default/report/slices/rev-org-mode/slice-37-attachments.md`

- T37.1: Deliver ID/DIR attachment locations, linking, copy/move/list/open and reviewed dispatcher controls.
- T37.2: Recheck real paths and collisions and retain operation evidence across partial file/source failure.
- T37.3: Verify directory traversal, symlinks, unreadable files and that visiting a heading performs no attachment IO.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test files --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 38: edit native Org footnotes and inline content

Contracts: ORG27 ORG30 ORG02.
Prerequisite: Slice 37 integrated in this same delivery set.
Branch: `feat/rev-org-38-document-conveniences`
Expected PR title: `feat(slice-38): edit native Org footnotes and inline content`
Report: `ops/default/report/slices/rev-org-mode/slice-38-document-conveniences.md`

- T38.1: Deliver footnote create/follow/renumber, entities, text markup and source-preserving affiliated forms.
- T38.2: Keep literal blocks and unknown forms distinct during structured editing and context actions.
- T38.3: Verify Unicode, malformed references, radio targets and coherent edit/history behavior.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test display`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 39: navigate and align native Org tables

Contracts: ORG12 ORG17.
Prerequisite: Slice 38 integrated in this same delivery set.
Branch: `feat/rev-org-39-table-basics`
Expected PR title: `feat(slice-39): navigate and align native Org tables`
Report: `ops/default/report/slices/rev-org-mode/slice-39-table-basics.md`

- T39.1: Introduce org_tables and native cell/row navigation, insertion and alignment over original cell source.
- T39.2: Retain separators, names, captions, formulas and non-table text while preserving displayed columns.
- T39.3: Verify tabs, wide graphemes, CRLF, empty cells and one-operation undo.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_tables --test org_editing --test coordinates`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 40: edit Org table rows columns and references

Contracts: ORG12 ORG14.
Prerequisite: Slice 39 integrated in this same delivery set.
Branch: `feat/rev-org-40-table-structure`
Expected PR title: `feat(slice-40): edit Org table rows columns and references`
Report: `ops/default/report/slices/rev-org-mode/slice-40-table-structure.md`

- T40.1: Deliver row/column insert/delete/move, sorting and list/table conversion with checked ownership.
- T40.2: Rewrite only supported formula references and reject ambiguous transformations before source mutation.
- T40.3: Verify cell value fidelity, separator boundaries and full structural undo/redo.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_tables --test org_editing`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 41: parse the bounded Org formula subset

Contracts: ORG13 ORG14.
Prerequisite: Slice 40 integrated in this same delivery set.
Branch: `feat/rev-org-41-formula-syntax`
Expected PR title: `feat(slice-41): parse the bounded Org formula subset`
Report: `ops/default/report/slices/rev-org-mode/slice-41-formula-syntax.md`

- T41.1: Parse declared arithmetic, field/column/range references, functions and compatible TBLFM assignments.
- T41.2: Enforce precedence, right-associative exponentiation and bounded syntax/depth before evaluation.
- T41.3: Verify unsupported Calc/Lisp preservation, malformed references and precise diagnostics.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_tables --test org_source`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 42: evaluate basic Org table formulas safely

Contracts: ORG13 ORG14.
Prerequisite: Slice 41 integrated in this same delivery set.
Branch: `feat/rev-org-42-formula-evaluation`
Expected PR title: `feat(slice-42): evaluate basic Org table formulas safely`
Report: `ops/default/report/slices/rev-org-mode/slice-42-formula-evaluation.md`

- T42.1: Evaluate dependency graphs, aggregates, rounding and formatting under the frozen numeric policy.
- T42.2: Detect cycles, empty/text misuse, division by zero, non-finite values and work-budget exhaustion.
- T42.3: Verify expected arithmetic independently, including negative half rounding and reference direction.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_tables --test org_time`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 43: publish Org recalculation as one transaction

Contracts: ORG13 ORG14 ORG17.
Prerequisite: Slice 42 integrated in this same delivery set.
Branch: `feat/rev-org-43-formula-publication`
Expected PR title: `feat(slice-43): publish Org recalculation as one transaction`
Report: `ops/default/report/slices/rev-org-mode/slice-43-formula-publication.md`

- T43.1: Prepare all admitted results before applying the selected recalculation as one checked history unit.
- T43.2: Expose table/cell/column recalculation through contextual commands and M-x without implicit evaluation.
- T43.3: Exercise success, unsupported formulas, failure preservation and undo in the owned table journey.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_tables --test org_editing --test commands`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tables`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 44: import and export Org table data

Contracts: ORG12 ORG27.
Prerequisite: Slice 43 integrated in this same delivery set.
Branch: `feat/rev-org-44-table-interchange`
Expected PR title: `feat(slice-44): import and export Org table data`
Report: `ops/default/report/slices/rev-org-mode/slice-44-table-interchange.md`

- T44.1: Deliver bounded CSV/TSV import/export and list/table conversion through native commands.
- T44.2: Preserve quoting and representable cell values; reject embedded line breaks or unsupported transformations before mutation rather than inventing lossy escapes.
- T44.3: Verify actual file round trips and rejected malformed input without lossy recovery.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_tables --test org_io --test files`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 45: display Org property columns and native reports

Contracts: ORG21 ORG23 ORG27.
Prerequisite: Slice 44 integrated in this same delivery set.
Branch: `feat/rev-org-45-columns-dynamic-blocks`
Expected PR title: `feat(slice-45): display Org property columns and native reports`
Report: `ops/default/report/slices/rev-org-mode/slice-45-columns-dynamic-blocks.md`

- T45.1: Deliver property column views and declared native dynamic report blocks, including effort and clock totals.
- T45.2: Keep projections separate from source; replace only recognized explicit report bodies on update.
- T45.3: Verify aggregates, source provenance, unknown dynamic blocks and atomic update undo.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_agenda --test org_tables --test org_editing`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 46: prepare checked native Org export snapshots

Contracts: ORG25 ORG02.
Prerequisite: Slice 45 integrated in this same delivery set.
Branch: `feat/rev-org-46-export-contract`
Expected PR title: `feat(slice-46): prepare checked native Org export snapshots`
Report: `ops/default/report/slices/rev-org-mode/slice-46-export-contract.md`

- T46.1: Freeze an export snapshot and backend/options/include/macro contracts over native source forms.
- T46.2: Resolve bounded includes and text-only macros without executing blocks or file-local Lisp.
- T46.3: Verify cycles, absent includes, unsaved source, export scope and destination preservation.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_source --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 47: export Org to text Markdown and Org

Contracts: ORG25 ORG27.
Prerequisite: Slice 46 integrated in this same delivery set.
Branch: `feat/rev-org-47-text-exports`
Expected PR title: `feat(slice-47): export Org to text Markdown and Org`
Report: `ops/default/report/slices/rev-org-mode/slice-47-text-exports.md`

- T47.1: Deliver ASCII/UTF-8, Markdown and Org outputs retaining structure, tables, links and footnotes.
- T47.2: Apply selected subtree/region/options and explicit destination writes from checked snapshots.
- T47.3: Verify independent semantic fixtures, Unicode conversion diagnostics and no source mutations.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_source --test files`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 48: export native Org documents to HTML

Contracts: ORG25 ORG30.
Prerequisite: Slice 47 integrated in this same delivery set.
Branch: `feat/rev-org-48-html-export`
Expected PR title: `feat(slice-48): export native Org documents to HTML`
Report: `ops/default/report/slices/rev-org-mode/slice-48-html-export.md`

- T48.1: Deliver native HTML structure, escaping, metadata, links, images, tables, notes and source blocks.
- T48.2: Implement declared attributes/options and safe relative asset references without executing contents.
- T48.3: Verify generated markup using independent HTML parsing and representative full-document fixtures.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_source`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 49: export Org to LaTeX and Beamer

Contracts: ORG25 ORG30.
Prerequisite: Slice 48 integrated in this same delivery set.
Branch: `feat/rev-org-49-tex-exports`
Expected PR title: `feat(slice-49): export Org to LaTeX and Beamer`
Report: `ops/default/report/slices/rev-org-mode/slice-49-tex-exports.md`

- T49.1: Deliver native LaTeX/Beamer rendering with declared headings, lists, tables, notes and math/source forms.
- T49.2: Support explicit optional compiler jobs for PDF through the bounded external owner, never on export discovery.
- T49.3: Verify escaped output and truthful absent-tool behavior without requiring a TeX installation.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_source`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 50: export native Org to OpenDocument text

Contracts: ORG25 ORG18.
Prerequisite: Slice 49 integrated in this same delivery set.
Branch: `feat/rev-org-50-odt-export`
Expected PR title: `feat(slice-50): export native Org to OpenDocument text`
Report: `ops/default/report/slices/rev-org-mode/slice-50-odt-export.md`

- T50.1: Deliver bounded native ODT XML and ZIP packaging without adding a runtime service or crate.
- T50.2: Preserve document structure, Unicode, links, lists and tables with validated package relationships.
- T50.3: Verify generated archives with Python zipfile/XML tooling and truthful unsupported-rich-content handling.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_source`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 51: export Org to Texinfo and man formats

Contracts: ORG25 ORG27.
Prerequisite: Slice 50 integrated in this same delivery set.
Branch: `feat/rev-org-51-manual-exports`
Expected PR title: `feat(slice-51): export Org to Texinfo and man formats`
Report: `ops/default/report/slices/rev-org-mode/slice-51-manual-exports.md`

- T51.1: Deliver native Texinfo and man renderers over the shared export snapshot.
- T51.2: Retain declared hierarchy, references, preformatted source and escaping without invoking tools implicitly.
- T51.3: Verify structural golden fixtures and ordinary save/cancellation safeguards.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_source`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 52: export Org planning data as iCalendar

Contracts: ORG25 ORG07.
Prerequisite: Slice 51 integrated in this same delivery set.
Branch: `feat/rev-org-52-calendar-export`
Expected PR title: `feat(slice-52): export Org planning data as iCalendar`
Report: `ops/default/report/slices/rev-org-mode/slice-52-calendar-export.md`

- T52.1: Deliver native iCalendar events/tasks with checked IDs, timestamps, time zones and recurrence conversion.
- T52.2: Preserve unsupported recurrence source and report conversion limits rather than inventing dates.
- T52.3: Verify independent calendar fixtures, line folding, escaping and stable refresh output.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_time`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 53: publish native Org document projects

Contracts: ORG26 ORG25.
Prerequisite: Slice 52 integrated in this same delivery set.
Branch: `feat/rev-org-53-publishing`
Expected PR title: `feat(slice-53): publish native Org document projects`
Report: `ops/default/report/slices/rev-org-mode/slice-53-publishing.md`

- T53.1: Deliver typed project selection, bounded source discovery, export and attachment-copy publication.
- T53.2: Honor explicit output ownership, collision consent and per-file outcomes; optional upload uses declared external tools.
- T53.3: Exercise actual exports/publishing, failure and source immutability through owned terminal/filesystem fixtures.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test files --test configuration`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-export`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 54: edit native Org source blocks and headers

Contracts: ORG28 ORG02.
Prerequisite: Slice 53 integrated in this same delivery set.
Branch: `feat/rev-org-54-source-block-editing`
Expected PR title: `feat(slice-54): edit native Org source blocks and headers`
Report: `ops/default/report/slices/rev-org-mode/slice-54-source-block-editing.md`

- T54.1: Admit declared source block/name/header structure and native special-edit entry/return.
- T54.2: Reuse available language presentation while keeping embedded Org-looking text literal and source authoritative.
- T54.3: Verify header precedence, malformed blocks, edit cancellation and checked reinsertion.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test syntax`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 55: tangle Org source with bounded noweb references

Contracts: ORG28 ORG18.
Prerequisite: Slice 54 integrated in this same delivery set.
Branch: `feat/rev-org-55-tangling`
Expected PR title: `feat(slice-55): tangle Org source with bounded noweb references`
Report: `ops/default/report/slices/rev-org-mode/slice-55-tangling.md`

- T55.1: Expand declared named references and tangle selected blocks into explicit checked destinations.
- T55.2: Bound expansion/cycles and reject unsafe or unresolved targets before overwriting any file.
- T55.3: Verify separate-file outputs, indentation/source fidelity and truthful multi-file failures.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_source --test files`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 56: execute selected Org blocks through bounded tools

Contracts: ORG28 ORG18.
Prerequisite: Slice 55 integrated in this same delivery set.
Branch: `feat/rev-org-56-block-execution`
Expected PR title: `feat(slice-56): execute selected Org blocks through bounded tools`
Report: `ops/default/report/slices/rev-org-mode/slice-56-block-execution.md`

- T56.1: Deliver explicit execution confirmation and typed external program adapters without an embedded interpreter.
- T56.2: Own child groups, stdin/out/err, temporary files, result ceilings, cancellation and finite deadlines.
- T56.3: Verify real owned shell jobs, missing tools, timeout and reaping; visiting/exporting source never executes it.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test commands --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 57: insert checked Org block and call results

Contracts: ORG28 ORG14.
Prerequisite: Slice 56 integrated in this same delivery set.
Branch: `feat/rev-org-57-block-results`
Expected PR title: `feat(slice-57): insert checked Org block and call results`
Report: `ops/default/report/slices/rev-org-mode/slice-57-block-results.md`

- T57.1: Deliver named calls, declared Library of Babel observations and results insertion/replacement as checked edits.
- T57.2: Retain header/argument semantics, source stamps and confirmation across related block resolution.
- T57.3: Verify output/value/table results and reject stale/failed publication without disturbing prior results.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_editing --test org_source`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 58: import native Org feed entries on demand

Contracts: ORG29 ORG18.
Prerequisite: Slice 57 integrated in this same delivery set.
Branch: `feat/rev-org-58-feeds`
Expected PR title: `feat(slice-58): import native Org feed entries on demand`
Report: `ops/default/report/slices/rev-org-mode/slice-58-feeds.md`

- T58.1: Deliver bounded native RSS/Atom parsing, declared feed targets and compatible FEEDSTATUS deduplication.
- T58.2: Fetch only explicit updates using configured optional transport, with no background poller or new service.
- T58.3: Verify local XML fixtures, repeat imports, hostile entity rejection and failed fetch/destination preservation.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_source --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 59: preview native Org links and images

Contracts: ORG30 ORG15.
Prerequisite: Slice 58 integrated in this same delivery set.
Branch: `feat/rev-org-59-previews`
Expected PR title: `feat(slice-59): preview native Org links and images`
Report: `ops/default/report/slices/rev-org-mode/slice-59-previews.md`

- T59.1: Deliver bounded link/image preview controls over explicit source/resource observations.
- T59.2: Use admitted terminal PNG presentation when supported and explicit external viewers otherwise; retain readable fallback.
- T59.3: Verify clipping, terminal restoration, source immutability and child cleanup in the owned external-feature journey.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_editing --test display --test pointer`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-external`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 60: complete the native Org control catalog

Contracts: ORG17 ORG31.
Prerequisite: Slice 59 integrated in this same delivery set.
Branch: `feat/rev-org-60-complete-controls`
Expected PR title: `feat(slice-60): complete the native Org control catalog`
Report: `ops/default/report/slices/rev-org-mode/slice-60-complete-controls.md`

- T60.1: Reconcile every shipped command, default context binding, typed preference and palette role with the catalog.
- T60.2: Extend the complete sample and embedded template with actual supported Org controls and optional tool examples.
- T60.3: Verify no inert controls, no unimplemented M-x entries and exact parser/default/template/reference agreement.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration --test commands --test org_editing --test org_agenda`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 61: verify complete native Org terminal workflows

Contracts: ORG01 ORG02 ORG17 ORG18.
Prerequisite: Slice 60 integrated in this same delivery set.
Branch: `test/rev-org-61-combined-terminal`
Expected PR title: `test(slice-61): verify complete native Org terminal workflows`
Report: `ops/default/report/slices/rev-org-mode/slice-61-combined-terminal.md`

- T61.1: Extend existing terminal admission with the declared representative Org journeys and same-head receipts.
- T61.2: Exercise canonical editing, capture/refile, clocks/habits, agenda, tables and optional-tool failures together.
- T61.3: Retain every earlier journey and verify zero skips, isolated HOME/XDG roots and owned process cleanup.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test org_time --test org_agenda --test org_tables --test org_io --test commands --test display`
- `python3 scripts/check-configuration-ui.py --suite terminal`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 62: verify bounded Org work and idle resources

Contracts: ORG18 ORG31.
Prerequisite: Slice 61 integrated in this same delivery set.
Branch: `perf/rev-org-62-bounded-work`
Expected PR title: `perf(slice-62): verify bounded Org work and idle resources`
Report: `ops/default/report/slices/rev-org-mode/slice-62-bounded-work.md`

- T62.1: Extend existing performance collection for large Org source, many files, habit graphs, formulas and exports.
- T62.2: Measure responsive cancellation and inactive/armed timer behavior under the frozen resource ceilings.
- T62.3: Retain baseline ceilings, same-candidate provenance and explicit warm gate cost; investigate regressions.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_time --test org_agenda --test org_tables --test org_io`
- `python3 scripts/check-configuration-ui.py --suite performance`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 63: reconcile native Org contracts and references

Contracts: ORG01 ORG17 ORG31.
Prerequisite: Slice 62 integrated in this same delivery set.
Branch: `docs/rev-org-63-baseline-reconciliation`
Expected PR title: `docs(slice-63): reconcile native Org contracts and references`
Report: `ops/default/report/slices/rev-org-mode/slice-63-baseline-reconciliation.md`

- T63.1: Reconcile delivered Org mode with editor/configuration/view baselines and complete user references.
- T63.2: Preserve all previous tested candidates and make the new capability matrix and optional-tool boundaries explicit.
- T63.3: Verify every requirement/scenario links to native, real-terminal and resource evidence; keep historical closures intact.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test org_time --test org_agenda --test org_tables --test org_io --test configuration --test commands`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite prerequisites`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 64: prepare complete native Org delivery acceptance

Contracts: ORG01 ORG18 ORG31.
Prerequisite: Slice 63 integrated in this same delivery set.
Branch: `test/rev-org-64-delivery-acceptance`
Expected PR title: `test(slice-64): prepare complete native Org delivery acceptance`
Report: `ops/default/report/slices/rev-org-mode/slice-64-delivery-acceptance.md`

- T64.1: Admit all sixty-four canonical integrations, reports and agreed requirement mappings without declaring prerequisites runtime proof.
- T64.2: Prepare clean integrated dev for the unchanged controller full gate and exact-path documentation closure.
- T64.3: Verify coherent plan/model/tracker state, zero skips and the distinction between tested head and later closure head.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test org_time --test org_agenda --test org_tables --test org_io --test configuration --test commands --test display`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-acceptance.py --prerequisites`
- `python3 -m unittest discover -s scripts -p test_acceptance.py`
- `python3 scripts/check-source.py`
- `git diff --check`

## Final gate and closure

After Slice 64 integration, the controller validates exact clean pushed dev
with scripts/check-all.sh. A failure requires the admitted same-run bounded
repair workflow, not documentation-only claims of success. Record the actual
tested SHA, validation identity, full command set, logs and zero-skip runtime
coverage. Only then may all reports/tasks be delivered and the plan/tracker
become Completed.

If later approved documentation closure changes the SHA, preserve the tested
candidate separately and validate the exact path boundary. Do not relabel its
runtime evidence as belonging to another head. Record canonical pushes and
completed significant outcomes in Acta. Final reporting must give implemented
scope, exact checks, resource observations and any explicit compatibility limits.
