# Factory Shortcut Mapping Contract

Status: Accepted; runtime delivery pending
Approved: product owner, 2026-11-14; exact retained P01-P19 proposal plus History c a/c f/+ and core-first functional follow-through
Date: 2026-11-14
Delivery set: rev-factory-shortcuts
Native source tree: `442765af9faac4351c208b2bfc5ca02c1b896331`
Spec: [Factory Shortcut Alignment](../../spec/editor-factory-shortcut-alignment.md)
Plan: [Approved delivery](../../plan/rev-factory-shortcuts.md)
Tracker: [Factory Shortcut Alignment Tracker](../../tracker/rev-factory-shortcuts.md)
Evidence: [Complete semantic matrix](semantic-matrix.md), [contextual review](mode-review.md)
Follow-through: [Core-first functional parity](../../spec/editor-emacs-interaction-parity.md)

## Accepted identity and decision evidence

The owner accepted the exact retained proposal at
`a87929948c81ee33b5b7083719c277c21a35793a`, base JSON SHA-256
`f532aaa2b7c7362174205d62626b3f440ac9527a8153963ce699edccfb327b3e`,
with precisely three additional History routes: c a amend, c f fixup and + paging.
Accepted decision evidence SHA-256:
`cc915b2d4af14a6ad0da34a517724e9664f59436b40d1004877e8a539af21939`.

[mapping-proposal.json](mapping-proposal.json) retains its established path and
now holds the accepted manifest: 29 records and 49 base-context edits.
Its SHA-256 is `e578368195b2e46acb0c7e7e1a8f3b289ddf0595667f64f890a8cb418d8dd6cf`.
The original 707-command catalog and native source tree are unchanged. The
owner authorizes the remaining current slices 3-7 and controller-owned dev
integration with all seven identities, frozen checks, budgets, explicit user
overrides and consent boundaries preserved. This precise revision requires no
second acceptance stop. New behavior outside it still needs its own defined
delivery scope; the worker does not expand an active slice.

D01 placements are accepted. D02 is accepted with the quick-access patch below.
D03 is accepted as an inventory of current differences, not their permanent
retention. Missing functional behavior is requested work under the linked
core-first preparation specification and subsequent delivery slicing. The
current shortcut-only slices add no functional adapter or new menu behavior.

## Exact before/after map

`absent` means no existing base-context leaf. `remove` removes only the factory
leaf; explicit old user assignments remain valid. E/N/I/R/VC/VL/VB name the
editor and six modal base owners. Org, Agenda, History and Refs are distinct
configurable contexts. Projection and prompt owners retain input precedence.

| Record | Base contexts | Normalized sequence | Current factory target | Accepted target |
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
| P14.amend | History | `c a` | absent | `git-commit-amend` |
| P16.fixup | History | `c f` | absent | `git-commit-fixup` |
| P17.page | History | `+` | absent | `git-log-next-page` |

The JSON keeps reference spelling and native normalization. C-j normalizes to
RET, so P07.C-j owns C-c C-v RET, distinct from plain j. Neither leaf exists in
the current factory. Preserve those aliases and prefix cancellation in tests.

P01/P02/P04 have seven explicit editing owners; P03 has only E/I/R. Factory
inheritance reaches macro_definition, git_commit, markdown, org, org_capture
and org_source_edit when the active language/profile admits it. Modal maps are
independent owners: a keys.editor user override does not become a modal one.
Keep Normal/Visual redo, nonmodal C-r search, state-specific C-o, protected C-g
and modal M-x. Prompts/completion/projections retain first refusal.

The native audit compiles additions/replacements for both profiles and checks
all 28 maps. It verifies 98 normalized traces and 21 Text/Org/Markdown first-text
sessions. Exact private snapshot removal checks retired leaves and prefix
neighbors; it is not a configuration unbinding sentinel or runtime factory
implementation. Runtime slices must test actual key-to-effect routes, selection,
review, hints and explicit overrides, including all three added History routes.

## Accepted D01 Agenda placements

P08-P13 use daily d, retaining v d; tags v M; search v /; show-all-today habits
v A; graph window v G; sort v S. Existing v t/d/w/v/m/y/s and K/C-u K remain.
These are the accepted native routes for this increment. M/s/A/G/S no longer
claim the reference moon/save-all/append/time-grid/sunrise effects. No missing
reference operation is added merely to occupy a key. Graph-date remains named;
the relocated actions retain names and explicit old user access.

Agenda source actions continue to use the selected qualified source rather
than incidental editor point. View changes do not mutate source or grant consent.

## Accepted D02 Git placements and quick access

P14-P19 put selected/ordered cherry-pick on History A, retire History P/F/f,
and add direct Refs f/P fetch/push while retaining &/M f and p/M p. The precise
owner amendment preserves quick displaced History access through c a amend,
c f fixup and + paging. All names and explicit old overrides remain valid.
History c becomes a complete prefix family; no bare c action is introduced there.

Current Status c continues opening literal commit text. Refs aliases retain
native direct review rather than claiming Magit transient equivalence. Fetch
requires a selected configured remote; push requires a current exact local
branch or qualified source mark and upstream review. No generic native pull
or History transport owner is added by a shortcut assignment.

Other native panels, directed ranges, ordered revision lists, mainline review,
stale-source refusal, process ownership and operation-specific confirmation
remain intact in this increment. The follow-through specification requires
proper stock prefix/menu/transport workflows later; these current differences
are not declared permanently acceptable substitutes.

## Displaced and retained access

| Changed old route | Access under the accepted map |
| --- | --- |
| Org C-c C-x C-c clock report | org-clock-report by name; C-c C-x C-r remains radio-button behavior. |
| Org C-c C-v h header insertion | j/C-j under the same prefix and named command; h stays unbound without invented help. |
| Agenda d graph-date | org-habit-graph-date by name. |
| Agenda M/s/A/G/S | v M, v /, v A, v G, v S respectively and the same named commands. |
| History A amend | History c a and git-commit-amend by name. |
| History P cherry-pick | History A and git-cherry-pick by name. |
| History F fixup | History c f and git-commit-fixup by name. |
| History f paging | History + and git-log-next-page by name. |

Omitted settings inherit compiled defaults; explicit old assignments retain
normalized replacement and original context/profile ownership. No upgrade
rewrites personal maps or changes the configuration format.

## Accepted D03 findings and functional follow-through

The matrix remains a source-verified inventory. Its adaptation-required rows
are requested functional work, with whole-buffer/paragraph marking, open-line,
preceding-line join, paragraph/sexp endpoints, region indentation, command repeat
and complete editing/selection/kill-yank/history/prefix/M-x workflows first.
The existing fragment-filtering ticket is reused, without a duplicate ticket.

Related Org TODO-heading/list dispatch and source-edit save/publication follow
core parity, then Git prefix/menu/transport differences. The new specification
records actual stock source/disk/history boundaries, paired reference effects,
acceptance requirements and ordered subsequent delivery preparation. It preserves
native document syntax, explicit overrides, consent, bounded work and ownership.

Current native Markdown/table/tree/calendar and bounded projection/execution
conventions remain unchanged during the shortcut-only increment. Their recorded
differences are not permission to keep arbitrary shared-capability divergence
forever. Stock context and complete observable workflow decide functional parity.
Same-terminal Emacs evidence is required where input normalization is relevant;
a generic terminal limitation cannot excuse an avoidable core difference.

## Selected legacy supersessions

Reconcile these specific rows during the corresponding accepted runtime slice,
without rewriting historical reports or validation receipts.

| Legacy owner | Exact selected delta |
| --- | --- |
| [Org Familiar interaction](../../spec/editor-org-mode.md) | Add effort; replace document clock-report key with read-only columns; move Babel h to j/C-j; apply accepted Agenda d/M/s/A/G/S changes. Current partial heading M-S-RET remains observed until its functional owner implements EIP07. |
| [Git Binding review matrix](../../spec/editor-git-integration.md) | Replace History A/P/F/f as P14-P17 define; retain amend/fixup/paging at c a/c f/+ and by name; add direct Refs f/P aliases. Update History panel guidance. Current Status c behavior remains for this increment, with EIP08 owning subsequent parity. |
| [Markdown](../../spec/editor-markdown-mode.md), tree and calendar | No current shortcut supersession; retain current point/focus conventions and carry functional differences to the parity specification. |
| [Bindings](../../../../docs/reference/bindings.md), [configuration](../../../../docs/reference/config.toml) and guide | Update exact changed factories, normalized hints and quick-access guidance in their runtime slices. |

## Delivery selection and cost

Slice 3 implements P01-P04; Slice 4 P05-P07; Slice 5 P08-P13; Slice 6 P14-P19
plus c a/c f/+; Slice 7 performs the selected terminal acceptance and guidance.
All seven branches, titles, reports, exact selected commands and gate order
remain unchanged. The controller owns integration, final full-gate execution
and successor startup. Functional additions have a separate core-first delivery
preparation and do not silently enter these shortcut-only slices.

Use one Cargo job, disabled incremental output and no dev/test debuginfo.
The final selected gate remains make lint, make test-hooks,
check-factory-shortcuts, check-documentation, build-user-guide, git diff --check
as exactly written in the plan. T7.1 produces the focused factory script; no
historical full feature gate is added. Variants share an owned build/fixture
batch and three representative PTYs.

The [prior acceptance report](../slices/rev-presentation-continuity/slice-5-acceptance.md)
records warm Cargo checks below one second, a cold private check of 1m31s,
38.95s release relink, a 5m42s release acceptance build and 12 PTYs totaling
20.155s. These separate measurements support an estimated 3-8 minutes warm or
10-20 minutes cold for the selected gate; they are not candidate totals or
changed timeouts. T7.3 records actual phases/cache conditions. No coverage,
resource bound or frozen assertion is weakened to meet an estimate.
