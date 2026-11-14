# Org and Markdown Presentation Continuity

Status: Approved
Date: 2026-11-14
Approved: product owner, 2026-11-14; specification, defined correction scope and established autonomous plan/slicing/Looper workflow
Format: OpenSpec change specification; workspace specification contract
Inspection baseline: `5e21c0bb73f21afddf6fcd73be9c5a30a139673e`
Readiness: approved five-slice delivery; controller startup follows preparation publication and preflight
Plan: [Presentation Continuity Delivery](../plan/rev-presentation-continuity.md)
Tracker: [Presentation Continuity Tracker](../tracker/rev-presentation-continuity.md)
Baselines: [Org Heading Presentation](editor-org-heading-presentation.md), [Org Visual Indentation](editor-org-indentation.md), [Markdown Editing and Presentation](editor-markdown-mode.md)
Related models: [Org Heading Presentation Model](editor-org-heading-presentation-model.md), [Org Indentation Model](editor-org-indentation-model.md), [Markdown Policy and Actions](editor-markdown-mode-model.md)

## Purpose

Keep heading symbols, structural margins and heading-level colors continuous
while ordinary text is edited in Org and Markdown. A new source revision SHALL
not force a verified, structurally unchanged heading through a temporary literal
presentation merely because background analysis has not finished.

The correction preserves immediate source editing and exact-revision authority
for structural commands. It introduces a bounded display-only continuation of
the last complete observation for conservatively admitted edits. It does not
require a new dependency, preference, command or persistent data model.

## Findings and current behavior

Canonical publication advances the buffer revision. The previous Org outline
then fails exact-stamp admission. The next frame skips heading substitution and
uses default margins. Background analysis later publishes the current outline
and requests another frame. Org can therefore emit decorated headings, literal
stars with different geometry, then decorated headings again for one body edit.
Markdown explicitly clears its observation on publication; its analogous frame
shows source hashes before restoring symbols. Its sampled title columns remain
constant because substitution preserves source-prefix width.

Controlled native PTY captures established the following:

| Case | Observed transition |
| --- | --- |
| Org, indentation and pretty headings enabled | Both title columns and markers change temporarily. |
| Org, indentation enabled and pretty headings disabled | Title columns change temporarily. |
| Org, indentation disabled and pretty headings enabled | Markers and concealed-prefix geometry change temporarily. |
| Org, both disabled | Sampled heading text and columns remain stable; complete color stability was not established. |
| Markdown, pretty enabled | Hashes temporarily replace symbols; sampled title columns remain stable. |
| Org and Markdown with colors enabled | Heading foreground temporarily becomes the default foreground. |

Sixty separated body/title key samples across twelve timing cases measured
fallback-to-restored output intervals. Minimal Org had a 10.823 ms median for
body edits. At 50,059 bytes it was 21.228 ms; at 500,059 bytes it was 182.634 ms,
with an observed maximum of 193.731 ms. Org with 3,003 headings and 127,949 bytes
had a 51.632 ms median. Markdown medians in the sampled inputs were about
10.6–11.2 ms. These are timestamps after PTY reads, not internal parser timings,
graphical frame durations or worst-case performance guarantees.

These findings establish a presentation-state transition before terminal font
shaping or compositing. A fixed redraw delay, faster parsing or synchronized
terminal output alone does not remove the invalidation state. The selected
correction retains current source output without waiting for analysis.

## Scope and terminology

An **exact observation** is complete owner data for the current buffer revision.
A **base observation** is the last complete owner data retained for display.
A **carried presentation** is a checked facade over that base whose mapping and
current stamp have advanced through every admitted publication. The original
base does not acquire structural authority for those newer revisions.

An **admitted edit** preserves the format's relevant structural interpretation,
line count and heading/list identities under the rules below. A **dirty lexical
region** has no retained inline-style claim until fresh analysis establishes it.
The **geometry token** identifies presentation geometry independently of whether
its evidence is exact or carried.

Initial coverage includes single-change, same-line body and heading-title
edits. Multiline changes, multi-change transactions, structural changes and
unknown contexts retain literal fallback. Local lexical styling can disappear
in dirty regions; eliminating every inline-style transition is not claimed.
Fold persistence and terminal row-diff/synchronized-output changes are separate
concerns. Current publication clears folds; this correction preserves that policy.

## ADDED Requirements

### Requirement: PC01 Preserve admitted presentation through pending analysis

For an admitted edit, Rev SHALL immediately display current source using the
carried presentation while exact analysis is pending. Unchanged heading markers,
prefix concealment, structural margins and heading-level foreground SHALL not
switch to literal/default values between publication and exact replacement.
Actual text changes and their normal wrapping/reflow SHALL remain visible.

This SHALL apply independently to Org indentation, Org pretty headings,
Markdown pretty presentation and enabled heading colors. Disabling pretty
presentation SHALL retain source markers without disabling separately enabled
colors or Org indentation. Existing per-view and per-buffer policy ownership
SHALL remain unchanged.

#### Scenario: Type below nested headings

- **GIVEN** complete analysis of an Org document with three heading levels
- **WHEN** one ordinary character is inserted in a body line without changing structure
- **THEN** every emitted update uses current text and the same unaffected heading presentation
- **AND** exact analysis completion introduces no temporary marker, margin or heading-color change.

#### Scenario: Edit a title without changing its prefix

- **GIVEN** an admitted Org or Markdown heading-title edit
- **WHEN** its text changes while analysis is pending
- **THEN** the unchanged heading prefix and ordinary title text keep their structural presentation
- **AND** current point and any normal title wrapping follow the edited source.

### Requirement: PC02 Preserve exact structural authority

Structural commands SHALL continue to require their current revision, complete
coverage and existing command-specific checks. Display carry SHALL not authorize
folding commands, promote/demote, navigation destinations, refiling, properties,
table/list mutations or any source edit. Display APIs SHALL distinguish carried
presentation from an authoritative Org outline or Markdown action observation.

Source publication, history, points and marks SHALL remain owned by the existing
buffer/coordinate path. Presentation SHALL introduce no source bytes, dirty
state, undo entries, persisted overrides or alternate editable document.

#### Scenario: Invoke an action during carried presentation

- **GIVEN** a visible carried presentation and no current structural action observation
- **WHEN** a structural command is invoked
- **THEN** it follows the existing current-analysis or unavailable-action path
- **AND** it obtains no authority from the display facade and publishes no stale edit or destination.

### Requirement: PC03 Admit a bounded same-line mutation

Carry admission SHALL require exactly one change in the canonical transaction,
with neither CR nor LF in removed or inserted text. The base SHALL have complete
coverage. Current mode, settings, buffer identity and source limits SHALL admit
presentation, and the existing carry chain SHALL cover the immediate pre-edit
revision without a rejected gap.

Admission SHALL inspect both before and after source and reject uncertain
classification, invalid mapping, exhausted work budgets or exceeded bounds.
Typing, deletion and single-line replacement SHALL share the same predicate;
history grouping SHALL not replace inspection of the actual mutation.
Undo/redo, reload, multi-change and multiline publications SHALL retire carry
in this initial scope. Read-only source permissions SHALL remain independent.

#### Scenario: Reject a structural boundary deletion

- **GIVEN** a complete observation containing a block or drawer boundary
- **WHEN** that boundary is deleted or corrupted into ordinary-looking text
- **THEN** checking the old line rejects carry even if the new line appears harmless
- **AND** later headings receive no carried classification across that mutation.

#### Scenario: Reject a transaction outside the initial shape

- **GIVEN** a valid carried presentation
- **WHEN** a publication contains a newline, CR, multiple changes, undo/redo or reload
- **THEN** that publication retires carry
- **AND** source editing proceeds through its normal checked path.

### Requirement: PC04 Use Org recognition for Org admission

Org body carry SHALL require that the row is not a recognized heading start and
that its before/after lines are safely non-structural under the owned Org line
classifier and applicable context. Opening/closing blocks, drawers, keywords,
definitions and other recognized structural constructs SHALL reject carry.
The classifier SHALL be reused or factored within the Org owner; no parallel
Org parser or independent star-scanning interpretation SHALL be introduced.

Org title carry SHALL require an already recognized heading, unchanged level
and byte-identical recognized prefix/separator, with the edit after that prefix.
The current classifier's ASCII-space separator SHALL remain authoritative;
a hand-written broader whitespace rule SHALL not redefine Org syntax. Changes
to TODO text, priority or tags after the prefix MAY carry geometry while their
lexical styles become dirty. Heading-like text inside a literal block SHALL
not gain heading identity.

#### Scenario: Preserve literal ownership

- **GIVEN** a source/example block with a star-prefixed line
- **WHEN** text on that line is edited
- **THEN** it receives no invented heading marker or heading margin
- **AND** uncertain or structural-looking lines reject carry.

#### Scenario: Delete the heading separator

- **GIVEN** a recognized heading with its source prefix intact
- **WHEN** backspace removes a prefix star or the required separator
- **THEN** title carry is rejected and current analysis determines the new structure
- **AND** a title edit after an unchanged recognized prefix remains eligible.

### Requirement: PC05 Keep Markdown admission subordinate to its parser

Pulldown-cmark through the existing private adapter SHALL remain authoritative
for Markdown headings, items, literal contexts and tables. The carry guard
SHALL be a conservative admission filter, validated against full parser output,
not a replacement Markdown parser or an assumed existing block classifier.

Body admission SHALL preserve non-blankness and the leading-whitespace prefix
in both source versions. It SHALL exclude structural openers, fences, setext
underlines, thematic breaks, list/quote markers, HTML boundaries, relevant table
syntax and definitions with effects elsewhere. Indented code, list continuation,
setext and surrounding block context SHALL be considered; unknown cases reject.

Already recognized ATX title edits after an unchanged recognized prefix SHALL
be supported. Non-blank edits to an already recognized setext title MAY carry
only when its underline, level and surrounding context remain verified; otherwise
they SHALL reject. Recognized list markers on unaffected rows SHALL retain their
existing depth/symbol treatment through an admitted carry. Editing list syntax
or invoking list commands SHALL still require current structural authority.

#### Scenario: Retain Markdown heading and list decoration

- **GIVEN** complete analysis with ATX headings and nested unordered items
- **WHEN** an admitted plain body edit changes no heading, marker or nesting
- **THEN** unaffected hashes/list markers do not temporarily replace their symbols
- **AND** ordered numbering, source indentation and literal code remain unchanged.

#### Scenario: Reject a definition with distant effects

- **GIVEN** Markdown containing a link reference definition
- **WHEN** its definition line is edited without changing line count
- **THEN** the admission filter rejects carry for that publication
- **AND** it does not assume that all effects are confined to the edited row.

### Requirement: PC06 Retain bounded owner data and checked row mapping

Rev SHALL retain at most one complete base per buffer for carried display and
a bounded record of changed rows. The record SHALL contain base/current stamps,
logical row identities, checked cumulative byte deltas and lexical-dirty state.
Repeated changes on one row SHALL merge without growing an edit log. A ninth
distinct carried row SHALL retire carry under the eight-row ceiling.

Because admitted edits preserve line breaks, source-row identity SHALL remain
stable. For row R, base line-start byte equals the current line-start byte minus
the sum of byte deltas in carried rows strictly before R. The existing owner
SHALL resolve structure at that mapped base position. A current byte offset
SHALL never be passed unadjusted to an old observation.

The facade SHALL borrow retained data for synchronous queries or use the
existing ownership mechanism. It SHALL not clone the whole outline/highlight
store, retain a second document copy, or create a second whole-document index.
Mapping SHALL use checked byte arithmetic and existing scalar/cell conversions.

#### Scenario: Compose edits across two rows

- **GIVEN** a complete base and several accepted edits on two logical rows
- **WHEN** an unchanged deeper heading is queried before exact analysis finishes
- **THEN** its mapped base position yields the same level and identity as fresh analysis
- **AND** multibyte insertions and deletions do not confuse byte, scalar or cell offsets.

#### Scenario: Reach the carry bound

- **GIVEN** carried changes on eight distinct rows with analysis still pending
- **WHEN** a change reaches a ninth row
- **THEN** carry is retired before displaying the new revision
- **AND** no unbounded row record or edit history is accumulated.

### Requirement: PC07 Prepare continuation before canonical installation

All fallible admission, bounded source inspection, allocation and mapping work
SHALL occur during existing edit preparation from the before/after ropes.
Canonical reserved installation SHALL only install precomputed state alongside
the source; it SHALL remain infallible and perform no new unbounded line copy,
analysis scan or wait.

The initial transaction scope SHALL not reinterpret descending base-coordinate
`InputEdit`s as a sequential final-text edit stream. Admission SHALL use the
canonical prepared change and its real before/after source.

#### Scenario: Edit an oversized logical line

- **GIVEN** a body line exceeding the admission scan budget
- **WHEN** one character is edited in that line
- **THEN** preparation rejects carry within the finite budget
- **AND** canonical installation still publishes ordinary source editing without a full-line scan.

### Requirement: PC08 Preserve heading faces and invalidate lexical uncertainty

Display SHALL obtain structural heading roles and levels from the exact or
carried presentation. Color admission SHALL explicitly support those roles;
returning a carried level while an exact-only color gate discards it is
insufficient. Colors SHALL come from the active palette and retain existing
syntax-enabled, byte-limit, NO_COLOR, inline-style, search and selection rules.
Enabled glyph/indentation presentation SHALL remain available without colors.

Each carried row SHALL be entirely lexically dirty in the initial design.
Base lexical captures intersecting such a row SHALL be dropped, including their
parts on other rows. Unchanged lexical runs MAY be retained only with checked
source mapping and established context. Delimiter or delimiter-adjacent edits
SHALL invalidate a conservatively established owning context. A fixed row±1
rule or a viewport edge SHALL not serve as proof of a syntax boundary.

If that context cannot be established within budget, uncertain visible lexical
runs SHALL be suppressed. Dirty ordinary title text SHALL retain the structural
heading face; dirty body text MAY use the default face until exact publication.
This is not a guarantee of continuous lexical colors inside edited constructs.

#### Scenario: Keep ordinary title foreground

- **GIVEN** colors enabled and an admitted edit with no inline style at the title sample
- **WHEN** every intermediate frame is decoded
- **THEN** the heading marker and ordinary title sample retain the active level color
- **AND** neither falls through to the default foreground while exact analysis is pending.

#### Scenario: Render an uncertain inline context

- **GIVEN** an edit that changes or abuts an inline delimiter
- **WHEN** its full affected context cannot be verified within budget
- **THEN** uncertain visible lexical runs are omitted until exact publication
- **AND** retained heading identity does not authorize a claim about those inline styles.

### Requirement: PC09 Keep current geometry and stable cache identity

Margins, concealment, symbols, wrapping, cursor placement, selection and pointer
hits SHALL consume one coherent current-revision projection. Fragments SHALL
retain source-stamp, policy and width checks; stale fragments SHALL remain
unusable for hits. Carried evidence SHALL not revive old screen coordinates.

Geometry identity SHALL remain stable across an admitted carry and an exact
replacement with equivalent geometry. Exact/carried provenance alone SHALL
not trigger whole-cache retirement. Current source stamps and affected-range
invalidation SHALL remain enforced, preserving existing unchanged-prefix reuse
where valid. Real structural, policy or geometry changes SHALL retire affected
fragments or reset the appropriate cache.

#### Scenario: Replace carry with equivalent exact geometry

- **GIVEN** a current carried layout produced by an admitted body edit
- **WHEN** exact analysis confirms the same heading geometry
- **THEN** provenance replacement does not cause a second whole-cache reset
- **AND** source stamps, current hits and normal source-driven reflow remain correct.

### Requirement: PC10 Distinguish pending work from retired presentation

Carry SHALL be retired on explicit cancellation, incomplete/limited analysis,
rejected mutations, mapping failure/overflow, reload, buffer close and relevant
mode or policy changes. Exact publication SHALL replace carry atomically through
the existing owner. Late or canceled results SHALL retain existing revision and
generation rejection. A newly opened buffer without a complete base SHALL use
literal fallback until exact evidence exists.

Pretty choices SHALL remain view-local; Org indentation SHALL remain buffer-local.
A view toggle SHALL immediately use the selected policy and retire its affected
projection without changing another view's choice. Shared evidence SHALL not
override those choices or revive invalid presentation. Re-enabling after a
retired state SHALL follow existing analysis-request behavior.

#### Scenario: Cancel pending analysis

- **GIVEN** current text displayed through carried evidence
- **WHEN** KeyboardQuit cancels the pending analysis
- **THEN** carry is removed and the documented literal fallback remains
- **AND** no stale symbol, hit target or extra continuation timer survives.

#### Scenario: Use independent pane policies

- **GIVEN** two differently sized views of one Org buffer with different pretty choices
- **WHEN** an admitted edit occurs and one view subsequently changes presentation policy
- **THEN** both views keep current source mappings and their own effective pretty choice
- **AND** the shared indentation policy and canonical point identities remain correct.

### Requirement: PC11 Bound work without delaying input

The correction SHALL use a maximum of eight carried rows and at most 32 KiB of
source-byte inspection per publication for admission, across before/after text
and any inspected context. Every pass SHALL charge that budget. Exhaustion SHALL
reject carry. These are internal limits, not new user settings.

Retained mapping state SHALL be O(8), independent of the number of keys since
the base. Retained exact owner data SHALL remain within its existing bounds.
Display mapping SHALL use O(8) additional work per query, plus the existing
bounded owner/coordinate lookup. Existing 8,192-fragment and 1 MiB text-cache
ceilings per view SHALL not increase. New per-view projection data SHALL be
charged to the owning cache's existing budget; bounded buffer-level carry
storage SHALL be accounted for separately rather than hidden from resource evidence.

The correction SHALL add no redraw hold, debounce, idle deadline, worker,
parser dependency or project scan. Typing and cursor output SHALL not wait for
background analysis. Existing analysis, cancellation, scheduling, limits and
owned cleanup SHALL remain responsible for exact completion.

#### Scenario: Type faster than analysis completes

- **GIVEN** a larger document and repeated admitted edits on one or two rows
- **WHEN** more keys arrive before the next exact publication
- **THEN** current typed text and stable heading presentation remain available
- **AND** state/work remain bounded without a delay intended to hide analysis latency.

### Requirement: PC12 Verify intermediate frames and preserve source fidelity

Acceptance SHALL inspect all observed updates between source publication and
exact completion, rather than checking only settled frames. For admitted cases,
it SHALL verify stable unaffected heading markers, prefix/title columns,
structural margins and foregrounds while allowing actual text-driven reflow.
Multiple equal emitted frames SHALL be permitted; terminal diffing is not part
of this correction.

The admission/mapping predicate SHALL be checked against fresh full analysis
on deterministic cases and generated accepted mutations. Results SHALL report
accepted and rejected counts per format/category to expose vacuous rejection.
Finite generated samples SHALL not be presented as a formal proof. Required
positive examples below SHALL actually carry; blanket rejection SHALL fail.

Acceptance SHALL preserve source bytes, newline/BOM identity, copy/save behavior,
history, points, marks, read-only refusal and unaffected modes. Native lifecycle
and representative PTY checks SHALL reuse existing fixtures and owners. Timing
evidence SHALL identify input, exact runtime candidate and measurement boundary.

#### Scenario: Detect a regression hidden by settling

- **GIVEN** the minimal fixture and a worker that has not published the edited revision
- **WHEN** one required positive edit is rendered before and after exact completion
- **THEN** the intermediate state satisfies the same unaffected-heading assertions as the final state
- **AND** a literal/default intermediate frame fails acceptance even if the final frame is correct.

## MODIFIED Requirements

Each block below replaces the same named requirement in the linked baseline.
Existing scenarios are retained in full. Unmentioned requirements, source
syntax, palette choices and historical delivery records retain their meaning.

### Requirement: OGP08 Bounded projection and delivery isolation

Baseline: [Org Heading Presentation, OGP08](editor-org-heading-presentation.md#requirement-ogp08-bounded-projection-and-delivery-isolation).

Decoration SHALL reuse native Org recognition and visible projection, with
revision/settings invalidation and bounded retained state. It SHALL not add
an idle scan, whole-project redraw loop or another Org parser. Acceptance SHALL
cover reference markers, configuration, literal contexts, source fidelity,
Unicode/cell geometry, source/pretty switching and real terminal interaction.

An exact observation or a current, proven display carry under PC01–PC12 SHALL
admit heading presentation. Missing, incomplete, canceled or unmapped evidence
SHALL leave undecided prefixes as readable source. The old observation alone
SHALL not authorize structural actions. Disabling syntax coloring SHALL not
disable independent heading recognition or admit unverified star-like text.
Respect `org.enabled` and existing analysis, cancellation and work limits.

Retain the bounded startup marker list and viewport-derived projection, with
only the bounded carry facade over existing owner data. No second per-line or
whole-document decoration store SHALL be added. Acceptance SHALL include both
editing profiles, splits, reload, soft wrap, disabled coloring, literal contexts
and read-only Org result buffers. Completed Org delivery records SHALL remain
historical evidence and SHALL not be relabeled as continuity acceptance.

#### Scenario: Refresh after a local heading change

- **GIVEN** a large Org document with an existing recognized outline
- **WHEN** one heading changes level
- **THEN** current decoration updates through the admitted recognition/projection owners
- **AND** source is not reparsed or scanned as an unrelated project-wide operation.

#### Scenario: Retain a proven heading presentation while pending

- **GIVEN** a complete outline and an admitted same-line body edit
- **WHEN** exact analysis of the new revision is pending
- **THEN** the bounded current facade retains unchanged heading presentation
- **AND** structural commands still require current exact evidence.

### Requirement: OIN05 Keep layout bounded and document the delivered behavior

Baseline: [Org Visual Indentation, OIN05](editor-org-indentation.md#requirement-oin05-keep-layout-bounded-and-document-the-delivered-behavior).

Projection SHALL reuse native Org observations and bounded layout work. A frame
SHALL not scan an entire document, allocate padding proportional to unchecked
heading depth or schedule a new idle poller. A proven current carry under
PC01–PC12 SHALL preserve admitted margins while exact evidence is pending.
Otherwise missing/stale evidence SHALL use coherent literal presentation;
unmapped old margins and current text SHALL never be combined. Existing cache
and work ceilings SHALL remain enforced under editing, toggles and resize.

Delivery SHALL verify native geometry, one representative real-terminal journey
in each interaction profile and resource bounds. The Markdown Org guide SHALL
show the two presentations and commands; its Spore build SHALL derive from that
content. References and Unreleased SHALL describe only delivered behavior.

#### Scenario: Deep outline in a small pane

- **GIVEN** a deeply nested document and a pane narrower than its ideal margin
- **WHEN** indentation, resize and navigation are exercised repeatedly
- **THEN** layout remains finite, preserves source and permits forward progress
- **AND** cache retention and per-pass work remain within existing bounds.

#### Scenario: Preserve margins during admitted body typing

- **GIVEN** indentation enabled and current carried presentation
- **WHEN** body text changes without changing heading ownership
- **THEN** unaffected structural margins do not collapse before exact analysis finishes
- **AND** every displayed margin has a checked mapping to the current source.

### Requirement: MD06 Bound analysis and preserve existing editor behavior

Baseline: [Markdown Editing and Presentation, MD06](editor-markdown-mode.md#requirement-md06-bound-analysis-and-preserve-existing-editor-behavior).

Observation SHALL respect configured syntax byte limits, finite work budgets,
cancellation and revision checks. Pending, canceled or stale observations SHALL
never authorize structural edits; ordinary source editing SHALL remain available.
A proven display-only carry under PC01–PC12 SHALL preserve admitted heading/list
presentation and heading faces during ordinary pending work. Cancellation,
incomplete results, rejected carry and unavailable evidence SHALL retain literal
fallback. Pulldown-cmark SHALL remain the format authority behind its adapter.

Retain existing table bounds: 8 MiB source/output, 10,000 rows, 256 columns,
100,000 cells and 4,096 displayed columns per cell. Native Markdown SHALL work
offline without an LSP. Existing Org behavior, other language highlighting and
configured LSP behavior SHALL remain intact outside the specified continuity
change. No command or persistent preference contract is added.

#### Scenario: Retire stale structural work

- **GIVEN** a table action pending on a particular buffer revision
- **WHEN** source changes, cancellation occurs, or a table bound is exceeded
- **THEN** the action publishes no partial edit or stale cursor destination
- **AND** ordinary text editing and other buffers remain usable.

#### Scenario: Keep a Markdown heading decorated during body typing

- **GIVEN** complete Markdown analysis and enabled pretty presentation/colors
- **WHEN** an admitted body edit advances the source revision
- **THEN** unaffected heading symbols and level colors remain continuous
- **AND** a carried list/table observation supplies no authority for a source action.

## Design and ownership

| Owner | Required change |
| --- | --- |
| `src/buffer/edit/` | Prepare one-change admission/mapping with before/after source; install reserved display state atomically with canonical publication. |
| `src/buffer/org.rs`, `src/org/` | Keep exact structural APIs; expose private bounded display queries; share the existing Org classifier and retain complete base data under existing ownership. |
| `src/buffer/markdown.rs`, `src/markdown/` | Retain an admitted base instead of unconditional display loss; guard carry against parser-backed context; keep action observations exact. |
| `src/view/indentation.rs`, `src/display/headings.rs`, `src/display/markdown.rs` | Read the same current display facade for margins, prefix geometry and symbols. |
| `src/display/syntax.rs` | Admit carried structural heading roles under existing color policy; map safe runs and suppress uncertain lexical context. |
| `src/view/wrap/` and current coordinate owners | Separate geometry identity from provenance; keep current fragment/hit checks and affected-prefix reuse. |
| `src/session/org.rs`, `src/session/markdown.rs` | Own retirement/publication/cancellation transitions and stale-result rejection through existing jobs. |

Names in this table identify current owners, not mandated new files or public
types. The facade is transient runtime state. No model companion or persisted
schema is introduced by this change specification.

### Presentation state transitions

| Current state | Event | Result |
| --- | --- | --- |
| No usable base | Open, mode activation or edit | Literal presentation; existing analysis request. |
| Exact complete | Admitted edit | Current carried presentation; existing analysis request. |
| Carried | Another admitted edit within bounds | Updated current carry over the same base. |
| Exact or carried | Rejected edit, cancellation, reload, invalidation or bound exhaustion | Retire carry; coherent literal presentation. |
| Carried or literal | Current complete result | Install exact observation; retire base/carry as appropriate. |
| Any state | Late or foreign result | Existing rejection; no state or source change from that result. |
| Any state | Buffer close | Drop retained state and preserve existing owned worker cleanup. |

For an equal exact replacement, the geometry token remains stable. A different
exact result retires affected layout before cursor/hit use. Source revision and
effective per-view policy remain part of projection validity in either case.

## Baseline model reconciliation

Before implementation begins, reconcile the MODIFIED blocks into their actual
baseline requirements and update only the following related model clauses.
Preserve native document formats, existing scenarios, completed statuses and
historical receipt/candidate text. Add this spec as the owner of the approved
continuity contract; do not describe historical gates as having tested it.

| Model and existing section | Required replacement/addition |
| --- | --- |
| [Org heading model, Derived view records](editor-org-heading-presentation-model.md#derived-view-records) | Permit exact recognized spans or a current proven carry; record base/current provenance separately from geometry. Keep literal/incomplete fallback, one-cell marker admission, source mappings and the ban on a second whole-document index. |
| [Org indentation model, State ownership](editor-org-indentation-model.md#state-ownership) | Add bounded transient carry to the existing observation owner; keep indentation buffer-local and pretty choices view-local. |
| [Org indentation model, Lifetime and bounded work](editor-org-indentation-model.md#lifetime-and-bounded-work) | Replace unconditional literal-pending rendering with the PC admission exception; keep bounds, source stamps and fallback whenever carry retires. |
| [Markdown model, Owners and analysis](editor-markdown-mode-model.md#owners-and-analysis) | Distinguish exact action evidence from carried display data, define same-line mappings and preserve the private parser adapter and existing worker limits. |
| [Markdown model, Preference contract](editor-markdown-mode-model.md#preference-contract) | Clarify unchanged palette/view policy and structural heading faces while carried; add no new preferences or marker sequences. |
| [Markdown model, Transactions and validation](editor-markdown-mode-model.md#transactions-and-validation) | Keep action plans exact; require intermediate-frame, admission/mapping and cancellation checks for display carry. |

## Reproducible acceptance

Use these minimal UTF-8/LF fixtures without BOM first. A body edit inserts `z`
after `Child body.`; a title edit inserts it after `Child`. Match headings by
source identity rather than by their current title string.

~~~org
* Root
Root body.
** Child
Child body.
*** Deep
Deep body.
~~~

~~~markdown
# Root
Root body.
## Child
Child body.
### Deep
Deep body.
~~~

With gutter disabled and 120x30 cells, record each decoded frame/update from
the key through exact completion. Expected marker/color values come from the
resolved configuration and active palette, not hardcoded theme values. Initial
title columns establish the reference; admitted body edits SHALL not change
those columns through a temporary source fallback.

### Mandatory positive admission cases

- Org plain body insertion/deletion and repeated replacement on one row.
- Org title insertion/deletion after the unchanged recognized prefix, including
  Unicode text and TODO/priority/tag edits that preserve heading identity.
- Markdown plain paragraph insertion/deletion with stable indentation/context.
- Markdown recognized ATX title insertion/deletion after the unchanged prefix.
- Admitted edits on two rows before one exact result; repeated edits earlier
  and later on the same carried row; shifts caused by multibyte source.
- Unchanged neighboring Markdown unordered-list symbols during admitted body
  typing, without granting those items authority for list actions.

### Coverage matrix

| Boundary | Required evidence |
| --- | --- |
| Org policy | All four indent/pretty combinations; colors independently enabled/disabled; NO_COLOR. |
| Markdown policy | Pretty/source views, colors independently enabled/disabled, NO_COLOR; setext carry only when proven. |
| Geometry | 120x30 normal and 32x12 wrapped/clipped frames; width changes, horizontal scroll, wide/combining text, source-prefix hits and selections. |
| Shared views | Different pane widths/pretty choices, shared Org indentation, current points and independent policy changes. |
| Structural refusal | Heading-star/hash/separator edits, block/drawer corruption and deletion, literal stars, fences, definitions, indentation/blankness changes, multiline and multi-change transactions. |
| Source fidelity | Copy/save, dirty state, exact single-action history, read-only refusal and representative BOM+CRLF input with same-line edits. |
| Retirement | Explicit cancellation, incomplete/over-limit results, reload, undo/redo, mode/policy changes, close and late result rejection. |
| Resources | Eight-row carry limit, ninth-row refusal, 32 KiB admission budget exhaustion, many repeated keys without growth, existing fragment/cache ceilings and no new timers/workers. |
| Isolation | Focused Org/Markdown structural actions, folding-policy preservation and one unaffected syntax mode. |

### Larger inputs and sustained input

Repeat required body/title positives with approximately 50 KB and 500 KB of
plain body text, and with 1,003 and 3,003 headings under the existing bounds.
Use a controlled pending worker for deterministic native transition assertions;
also retain a representative real-worker PTY journey. The PTY journey SHALL
decode marker, title-column and foreground state, not merely search for a glyph
somewhere in the final ANSI output.

Exercise at least five keys within 180 ms on one row and then on two rows, so
the carry is validated across revisions before exact completion. Record whether
the real worker actually remained pending; do not infer it from key timing.
Check the bounded mapping against fresh complete analysis after accepted edits
in source/example blocks, drawers, CRLF, Unicode, bare prefixes, setext, fences,
code, lists and definitions. Rejected cases SHALL retain readable literal output
and exact source/action safety rather than false continuity.

Extend existing native/PTY owners and parameterized tests. Preserve the intent
of `cancelled_observation` in `tests/markdown/presentation.rs` and
`evidence_invalidation` in `src/view/indentation.rs`. Test positive carries and
retirement separately so no assertion is weakened to hide stale evidence.
Record accepted/rejected mutation counts, actual source/capture sizes, mapping
retention, per-pass work and measured costs. A full repository gate and the
proportional final selection belong to the later approved delivery plan.

## Completion criteria

Completion requires reconciled baseline/model wording, implemented current
display carry, passing admission/mapping and intermediate-frame regressions,
representative native/PTY geometry and lifecycle evidence, unchanged source and
structural authority, and enforced resource ceilings. Acceptance must identify
the exact tested source candidate and distinguish native, emitted-terminal and
graphical evidence.

Specification approval is not a delivery receipt. Findings from the
existing runtime establish the defect; they do not validate the correction.
References, user documentation and a concise Unreleased entry are updated only
when the corrected behavior has been delivered under its approved workflow.
