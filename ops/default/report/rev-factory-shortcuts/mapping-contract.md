# Factory Shortcut Mapping Contract

Status: Proposed; owner decision required before Slice 2 PR
Date: 2026-11-14
Delivery set: rev-factory-shortcuts
Native source tree: `442765af9faac4351c208b2bfc5ca02c1b896331`
Spec: [Factory Shortcut Alignment](../../spec/editor-factory-shortcut-alignment.md)
Plan: [Approved delivery and decision checkpoint](../../plan/rev-factory-shortcuts.md)
Tracker: [Factory Shortcut Alignment Tracker](../../tracker/rev-factory-shortcuts.md)
Evidence: [Complete semantic matrix](semantic-matrix.md), [contextual review](mode-review.md)

## Proposal identity and acceptance boundary

This is the complete proposed shortcut increment, not an accepted runtime
manifest. [mapping-proposal.json](mapping-proposal.json) contains 26 change
records covering 46 base-context edits. Each record states the exact current
leaf, replacement or removal and inherited context footprint. The source tree
and frozen 707-command catalog are unchanged. No command, configuration schema,
unbinding sentinel, runtime adapter or personal setting is added by this audit.

The proposal file's SHA-256 is `f532aaa2b7c7362174205d62626b3f440ac9527a8153963ce699edccfb327b3e`.
Owner acceptance must identify this proposal and D01-D03, record the accepted
choices in this contract and the specification, and reconcile the plan/tracker
before adding the mapping approval stamp. Neither a successful probe nor audit
execution authority supplies that acceptance. The approved plan requires a
clean committed proposal branch and a controller decision before PR submission.

## Exact before/after map

`absent` is no existing base-context leaf. `remove` means remove only the
factory leaf; old explicit user assignments remain valid. Editing owners E/N/I/
R/VC/VL/VB mean editor and the six named modal contexts in the JSON. Org, Agenda,
History and Refs refer to their distinct configurable contexts, in both profile
selections; projected roles retain input ownership over editing maps.

| Record | Base contexts | Normalized sequence | Current factory target | Proposed target |
| --- | --- | --- | --- | --- |
| P01 | E/N/I/R/VC/VL/VB | `M-m` | absent | `modal-line-first-text` |
| P02 | E/N/I/R/VC/VL/VB | `C-x +` | absent | `balance-windows` |
| P03 | E/I/R | `C-M-_` | absent | `undo-redo` |
| P04 | E/N/I/R/VC/VL/VB | `C-x p p` | absent | `project-switch-project` |
| P05 | Org | `C-c C-x E` | absent | `org-inc-effort` |
| P06 | Org | `C-c C-x C-c` | `org-clock-report` | `org-columns` |
| P07.j | Org | `C-c C-v j` | absent | `org-babel-insert-header-arg` |
| P07.C-j | Org | `C-c C-v RET` | absent | `org-babel-insert-header-arg` |
| P07.h | Org | `C-c C-v h` | `org-babel-insert-header-arg` | remove |
| P08 | Agenda | `d` | `org-habit-graph-date` | `org-agenda-day-view` |
| P09.old | Agenda | `M` | `org-tags-view` | remove |
| P09.new | Agenda | `v M` | absent | `org-tags-view` |
| P10.old | Agenda | `s` | `org-search-view` | remove |
| P10.new | Agenda | `v /` | absent | `org-search-view` |
| P11.old | Agenda | `A` | `org-habit-toggle-show-all-today` | remove |
| P11.new | Agenda | `v A` | absent | `org-habit-toggle-show-all-today` |
| P12.old | Agenda | `G` | `org-habit-set-graph-window` | remove |
| P12.new | Agenda | `v G` | absent | `org-habit-set-graph-window` |
| P13.old | Agenda | `S` | `org-agenda-sort` | remove |
| P13.new | Agenda | `v S` | absent | `org-agenda-sort` |
| P14 | History | `A` | `git-commit-amend` | `git-cherry-pick` |
| P15 | History | `P` | `git-cherry-pick` | remove |
| P16 | History | `F` | `git-commit-fixup` | remove |
| P17 | History | `f` | `git-log-next-page` | remove |
| P18 | Refs | `f` | absent | `git-fetch` |
| P19 | Refs | `P` | absent | `git-push` |

The JSON keeps the reference key spelling and its native normalized leaf.
In particular, native C-j normalizes to RET: P07.C-j is the exact
`C-c C-v RET` leaf, distinct from plain j. The existing factory has neither
leaf; later tests must preserve these aliases and prefix cancellation.

P01/P02/P04 have seven explicit base owners; P03 has only E/I/R. Factory
inheritance also reaches `macro_definition`, `git_commit`, `markdown`, `org`,
`org_capture` and `org_source_edit` when their language/profile overlay admits
it. Modal maps are independent owners: a user `keys.editor` override does not
silently become a modal override. Normal/Visual redo controls, C-r backward
search in editing, C-o state-specific controls, protected C-g and modal M-x
remain intact. Prompt/completion/projection owners retain first refusal.

The native probe compiles additions/replacements for both profiles, checks
all 28 compiled maps, and removes exact leaves only in private map snapshots
for the conflict check. It records 92 normalized traces and 21 Text/Org/Markdown
first-text sessions across nonmodal and all modal states. Snapshot removal is
not a runtime unbinding implementation or an assertion that every changed key
has executed. Later slices must change compiled factories and test actual
key-to-effect routes, hints and explicit overrides.

## D01: Agenda family decision

Proposed choice: accept P08-P13. Daily view takes d while retaining v d. Move
native tags/search/show-all-habits/graph-window/sort into v M, v /, v A, v G,
v S. Preserve existing v t/d/w/v/m/y/s and K/C-u K. The namespace is explicitly
native: reference v dispatch is not a universal suffix-equivalence contract.

The reference's M/s/A/G/S invoke moon phases, save-all, append agenda, time grid
and sunrise/sunset. Those operations are absent from the native catalog and
are not introduced here. Retiring the conflicting single keys avoids claiming
those effects. The moved native actions remain available by name; graph-date
becomes named-only when d changes. The projection's selected qualified source
continues to own task edits; view changes do not mutate its source.

Alternative: retain current native M/s/A/G/S as explicit exceptions while
accepting only daily d. This reduces relocations but preserves the documented
reference conflicts. It needs a different accepted manifest; the worker has
not selected that alternative as a runtime decision.

## D02: Git family decision

Proposed choice: accept P14-P19. History A uses the existing selected/ordered
cherry-pick owner. Remove History P/F/f rather than bind push/pull/fetch there:
fetch/push require current Refs selection and review, and no native `git-pull`
exists. In Refs, add f/P for direct fetch/push, retaining &/M f and p/M p.
These are familiar family letters for bounded native direct actions, not Magit
transient menus. No second suffix may provide review or transport consent.

Keep the remaining Status, Refs, Sequence, Worktree, Stash, Conflict and Inspect
panel model as explicit native conventions. Status c opens commit text directly;
a second c is text. The native action view is not a Magit transient. Existing
range order, ordered revision lists, stale-source refusal, merge/mainline
review, transport scope, cancellation and postconditions remain authoritative.

Alternatives requiring an explicit decision:

- Retain all current Git defaults as intentional exceptions. This removes the
  meaningful shortcut increment from provisional Slice 6; the owner/controller
  must resolve the slice map/configuration identity before proceeding.
- Specify reference-style transient menus and History transport selection.
  This adds behavior beyond a shortcut reassignment and needs separate approved
  scope. No such menus or adapters are included in this proposal.

The proposed choice keeps all seven recorded slice identities, exact PR titles
and selected gate commands. Slice 6 has four History edits and two Refs aliases,
with regressions and the corresponding panel hint/reference updates. Its scope
is concrete without changing the native Git operation owners.

## Displaced access and retained exceptions

| Displaced route | Remaining access under the proposed map |
| --- | --- |
| Org C-c C-x C-c clock report | `org-clock-report` by name; C-c C-x C-r remains radio-button behavior. |
| Org C-c C-v h header insertion | j/C-j alternatives under the same prefix and named command; h stays unbound, without invented Babel help. |
| Agenda d graph-date | `org-habit-graph-date` by name. |
| Agenda M/s/A/G/S | `v M`, `v /`, `v A`, `v G`, `v S` respectively and the same named native commands. |
| History A amend | `git-commit-amend` by name. |
| History P cherry-pick | History A and `git-cherry-pick` by name. |
| History F fixup | `git-commit-fixup` by name. |
| History f paging | `git-log-next-page` by name. |

All explicit old user assignments remain valid in their original owning maps.
Defaults are inherited when omitted; the upgrade never writes personal maps.
No unbinding sentinel is added to the configuration contract.

D03 proposed classification is complete, with these boundaries retained:

- M-S-RET inserts a native checkbox list item but refuses a heading; reference
  heading TODO insertion requires separate behavior before full equivalence.
- Source-edit Save retains the draft and refuses publication. Source-edit exit
  publishes back to the unsaved Org buffer and closes the draft; it is not a
  replacement for reference source-edit Save. A matching save owner needs a
  separate behavior contract. Capture Save also retains its transient draft.
- Markdown RET continues native lists and reaches table-row movement (which can
  align source), whereas the observed package list RET inserts a plain newline.
  Preserve the approved Org-like table convention and indirect helpers.
- The file tree retains C-x t and current panning/selection conventions. Native
  Calendar remains a compact date/time picker, not the full reference Calendar.
- Nine core approximations remain rejected as recorded in the matrix. Native
  Git direct action, bounded export/Babel execution, read-only property columns
  and registered Project restoration retain their explicit feature limits.
- Named operations without an appropriate key remain named-only. Attachment,
  Org structural/table and process dispatchers are recorded as indirect access
  only in their admitted context; no arbitrary key fills a raw catalog gap.

Accepting D03 accepts these classifications and exclusions. It does not
commission the missing behavior or silently expand the runtime slices.

## Selected legacy supersessions

Only after owner acceptance and runtime delivery, reconcile these current
contract rows. Keep historical reports and validation evidence unchanged.

| Legacy contract | Exact selected change |
| --- | --- |
| [Org Familiar interaction](../../spec/editor-org-mode.md) | Preserve partial M-S-RET; add effort; replace the document clock-report key with read-only columns; move header insertion from h to j/C-j. Reconcile the affected Agenda d/M/s/A/G/S rows through the accepted native v family. |
| [Git Binding review matrix](../../spec/editor-git-integration.md) | History f paging, A amend, F fixup and P cherry-pick are superseded only by P14-P17; record named displaced access and A cherry-pick. Add Refs f/P as direct aliases with explicit transient difference. |
| [Markdown](../../spec/editor-markdown-mode.md) and tree contracts | No supersession: retain native table and tree conventions. |
| [Bindings](../../../../docs/reference/bindings.md), [configuration reference](../../../../docs/reference/config.toml), current guide | Update changed keys, exhaustive defaults and History panel guidance in their runtime slice; selector hints continue to derive from effective maps. |

## Remaining delivery and validation cost

Slice 3 implements P01-P04; Slice 4 implements P05-P07; Slice 5 implements
P08-P13; Slice 6 implements P14-P19. Slice 7 verifies their accepted effective
maps, command effects and representative physical input. Every identity and
exact check in the approved slice map remains unchanged by this proposal.

Use one Cargo job, incremental output disabled and dev/test debuginfo disabled.
Native variants share the owned build/fixture batch; three selected PTYs remain
representative. No historical full feature gate is added. The exact final
selection stays make lint, make test-hooks, check-factory-shortcuts,
check-documentation, build-user-guide, then git diff --check as written in the
plan. The new factory script remains planned output of T7.1.

The [previous bounded acceptance report](../slices/rev-presentation-continuity/slice-5-acceptance.md)
records warm Cargo checks below one second, a cold private check of 1m31s,
a release relink of 38.95s and an observed release acceptance build of 5m42s.
It records 12 PTYs totaling 20.155s, native Org 13.504s and Markdown 6.335s.
These are separate measurements, not a reconstructed total or this candidate's
time. For the smaller selected gate, budget approximately 3-8 minutes with
compatible warm build outputs and 10-20 minutes when compilation is cold.
This is an estimate dominated by build/lint, not a timeout change or guarantee.
T7.3 must record actual phase durations and cache conditions; no assertion,
resource limit or frozen command is weakened to meet the estimate.

The Slice 2 owner checkpoint precedes the required approval-dependent clean-HEAD
validation. Before that checkpoint, focused native/reference and structural
proposal checks are evidence only. No PR, runtime acceptance, delivery-set gate
or mapping approval is claimed by this document.
