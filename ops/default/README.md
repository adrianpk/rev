# Pending Editor Work

Status: proposed
Date: 2026-10-31
Scope: pending behavior after the completed native completion, optional LSP and modal editing deliveries
Owner decision: native completion, optional LSP and modal editing delivered; language-server installation approved for autonomous implementation on 2026-11-02

This index connects the pending tickets, capability specifications and models.
It is a planning entry point, not a delivery plan or execution tracker. The
linked artifacts retain their own status. Capturing and versioning a proposal
does not approve its open decisions or start implementation.

Emacs similarity in keybindings and behavior remains the interaction guide.
Use its equivalent behavior where applicable; preserve explicit rev product
decisions and identify cases where rev has no direct Emacs counterpart.

## Complete inventory

The inventory contains nine delivered topics, two open presentation
follow-ups and six remaining capability proposals. The five solved
tickets retain their implementation/PR evidence.
The completed delivery passed its controller-owned full gate; other capabilities
retain their own specification/model state.

| Topic | Artifacts | State and remaining decisions |
| --- | --- | --- |
| Keybinding hints in M-x | [Ticket](ticket/open/20261031095151-show-keybindings-beside-m-x-commands.md) | Open. Show assigned shortcuts beside command candidates using effective bindings; preserve command completion and invocation. |
| Tree selection highlight and active-buffer synchronization | [Ticket](ticket/open/20261031093408-highlight-tree-selection-and-sync-active-buffer.md) | Open follow-up for later. Persistent selection feedback and displayed-buffer synchronization; multi-pane and tree-navigation indication policy remain to be settled. |
| Open a file or directory from CLI/find-file | [Navigation spec](spec/editor-file-navigation.md), [model](spec/editor-file-navigation-model.md), [ticket](ticket/solved/20261028094803-open-directory-targets-from-cli-and-find-file.md) | Delivered / Completed spec / solved ticket. Slice 5 delivered directory acceptance and root precedence through PR #178. |
| Expand the initial tree root | [Navigation NAV03](spec/editor-file-navigation.md), [model](spec/editor-file-navigation-model.md), [ticket](ticket/solved/20261028153707-expand-file-tree-root-on-first-display.md) | Delivered / Completed spec / solved ticket. Slice 5 delivered one-time first-level expansion through PR #178; deliberate collapse is retained. |
| Tab completes the highlighted path | [Navigation NAV04](spec/editor-file-navigation.md), [model](spec/editor-file-navigation-model.md), [ticket](ticket/solved/20261028170748-complete-highlighted-find-file-path-with-tab.md) | Delivered / Completed spec / solved ticket. Slice 6 delivered selected-candidate Tab and separate final acceptance through PR #179; UI03 is reconciled. |
| Select the clicked tree entry | [Navigation NAV05](spec/editor-file-navigation.md), [model](spec/editor-file-navigation-model.md), [ticket](ticket/solved/20261030231016-select-the-clicked-file-tree-entry.md) | Delivered / Completed spec / solved ticket. Slice 7 delivered checked native activation and configurable selection-only through PR #180. |
| Word-aware soft wrapping | [Spec](spec/editor-soft-wrap.md), [model](spec/editor-soft-wrap-model.md), [ticket](ticket/solved/20261028224018-define-comfortable-long-line-wrapping.md) | Delivered / Completed spec / solved ticket. Default word-aware pane-width wrap and buffer-local Emacs Visual Line controls; Slices 1-4 delivered through PRs #174-#177. |
| Native bookmarks and gutter markers | [Spec](spec/editor-bookmarks.md), [model](spec/editor-bookmarks-model.md), [plan](plan/rev-bookmarks.md), [tracker](tracker/rev-bookmarks.md) | Delivered / Completed spec and model. Six slices through PRs #185-#190 and authorized repairs #191-#193 deliver named commands, checked lifecycle, persistence, listing, native markers, joint acceptance and guidance. The controller-owned combined gate passed. |
| Built-in autocompletion | [Spec](spec/editor-autocompletion.md), [model](spec/editor-autocompletion-model.md), [plan](plan/rev-autocompletion.md), [tracker](tracker/rev-autocompletion.md) | Delivered / Completed. Eight slices through PRs #195-#202 deliver automatic/manual interaction, ten bundled parsers, native Org contexts, bounded observations, registered external grammars and native/resource/PTY acceptance. The native gate passed; the subsequent integrated LSP gate also passed CMP02 semantic/native switching and joint acceptance. |
| Optional native LSP client | [Spec](spec/editor-language-server-support.md), [model](spec/editor-language-server-support-model.md), [plan](plan/rev-language-server-support.md), [tracker](tracker/rev-language-server-support.md) | Delivered / Completed. Six slices through PRs #203-#208 and authorized repairs #209/#210 deliver disabled-by-default stdio support, unsaved sync, UTF-8/UTF-16 mapping, shared processes, buffer exclusions, checked semantic edits, fallback and real Rust-server acceptance. The unchanged controller full gate passed; LSP01-LSP06 and joint CMP01-CMP09 are closed. |
| Simple optional LSP installation | [Spec](spec/editor-language-server-installation.md), [model](spec/editor-language-server-installation-model.md), [plan](plan/rev-language-server-installation.md), [tracker](tracker/rev-language-server-installation.md) | Approved. Seven slices for guided existing/custom setup, association, explicit choice, managed recipes and lifecycle. Any compatible language/server is eligible independently of bundled grammars or the initial catalog. Execution startup and delivery evidence remain controller-owned. |
| Native AI commands via provider API | [Spec](spec/editor-ai-commands.md), [model](spec/editor-ai-commands-model.md) | Draft. Discrete explain/implement/transform operations and reusable declarative commands. Context grants, API profile, acceptance and persistence remain open. |
| Independent optional Pi backend | [Spec](spec/editor-ai-pi-integration.md), [model](spec/editor-ai-pi-integration-model.md) | Draft. Pi owns its authentication and discovery; rev owns editor snapshots and checked edits. Protocol and enforceable read-only authority remain open. |
| Startup-selected Emacs/Vim editing | [Spec](spec/editor-modal-editing.md), [model](spec/editor-modal-editing-model.md), [plan](plan/rev-modal-editing.md), [tracker](tracker/rev-modal-editing.md) | Delivered / Completed. Eight slices through PRs #212-#219 deliver immutable startup profiles, checked modal navigation/operators/selections/jumps/repeat and shared colon/M-x/completion workflows. The unchanged controller full gate passed MOD01-MOD14, native/installed-server/PTY acceptance and guidance. |
| Org heading glyphs and terminal fonts | [Spec](spec/editor-org-heading-presentation.md), [model](spec/editor-org-heading-presentation-model.md) | Draft. Six reference glyphs captured; prefix/source fidelity required. Depth overflow, defaults and reveal/geometry details remain open. Fonts and ligatures belong to the terminal. |
| Integrated terminals | [Spec](spec/editor-integrated-terminal.md), [model](spec/editor-integrated-terminal-model.md) | Draft. Lower panel by default, direct buffer alternative and live relocation, Rust alacritty_terminal direction. Exact dependency pin, settings, IO integration and limits need preparation. |
| External programs/extensions | [Spec](spec/editor-external-extensions.md), [model](spec/editor-external-extensions-model.md) | Optional exploration only. Explicit configuration, stdio and one child per invocation are the refined draft profile. Adoption itself is undecided; no scheduled delivery or baseline dependency. |

## Selected execution order

The owner selected optional language-server installation on 2026-11-02 and
authorized the seven-slice plan and autonomous Looper implementation through
dev. Existing-client support is already delivered. The selected set adds
ergonomic setup and management; it does not limit LSP languages to the native
grammar list or acquisition catalog. Other pending capabilities retain their
own state.

The owner selected startup-only Emacs/Vim modal editing on 2026-11-01,
temporarily deferring discussion item 10, and authorized its eight-slice delivery.
The specification/model and plan/tracker are now Completed after PRs #212-#219
and the unchanged controller full gate. Shared native editing, colon/M-x entry
and completion retain their existing owners. Item 10 remains pending; closure
does not start another capability.

The owner selected these five interaction topics for one approved Looper cycle,
with soft wrapping first:

1. Word-aware soft wrapping.
2. File/directory opening.
3. Initial tree-root expansion.
4. Highlighted path completion with Tab.
5. Clicked tree-entry selection.

The [Wrap and Navigation plan](plan/rev-wrap-navigation.md) and
[tracker](tracker/rev-wrap-navigation.md) define the approved eight-slice map
for one autonomous Looper cycle. The owner requested startup.
The completed interaction cycle does not schedule additional capabilities;
external extensions remain optional exploration without an implementation commitment.

Completed delivery: named persistent bookmarks, checked navigation and gutter
markers. The [Bookmarks plan](plan/rev-bookmarks.md) and
[tracker](tracker/rev-bookmarks.md) record six integrated slices and three
authorized repairs. The unchanged controller-owned combined gate passed;
the delivery and its operational documentation are closed.
The owner requested startup on 2026-10-31. The two presentation follow-ups remain
separate pending tickets.

## Actual dependencies

The proposed order is not a chain of prerequisites.

- Directory opening supplies explicit root/file-parent context for the full
  AI experience and eventual terminal launch defaults. It does not grant AI
  permission to transmit content or settle file-only discovery boundaries.
- First expansion and Tab completion share navigation observations with
  directory opening. Tree clicking follows the accepted activation preference;
  retain its checked row identity independently of root initialization.
- Bookmarks reuse the existing gutter and canonical location/edit owners.
  They are not prerequisites for completion, LSP, AI or modal jump history.
- Native completion owns the shared UI, checked edits and Tree-sitter source.
  The LSP client supplies the optional semantic source. Plan and validate that
  integration together; a native-only stage does not satisfy semantic-source
  switching. Native observations use current/other open buffers, with no directory
  index; LSP resolves roots independently.
- The LSP client can use already installed compatible servers. Managed
  installation depends on its launch/configuration contract, not the reverse.
- AI must verify actual root, buffer, command, job and structural interfaces.
  LSP enrichment is optional; bookmarks, the installer and external extensions
  are not AI prerequisites. A Pi-managed API key is Pi authentication and does
  not satisfy rev's independent native API implementation.
- Soft wrap extends shared source/display projection, logical gutter identity
  and existing Org folding. Heading glyphs and modal motions must use those
  same accepted coordinates/semantics when combined, rather than create
  competing selection or movement owners.
- Modal editing reuses native edits, commands, search, history and rectangles.
  It does not require bookmarks, AI or LSP. Its startup-only profile selection
  is distinct from ordinary Vim Normal/Insert/Visual transitions.
- Heading presentation builds on delivered native Org recognition/projection.
  Integrated terminals build on native layout, focus, buffers and nonblocking
  jobs, with their own PTY/process lifetime. Neither requires AI or plugins.
- Optional external commands remain independent exploration. Native AI, Pi,
  LSP and terminals keep their own contracts without waiting for this draft.

## Approved completion sequence

The owner approved commit/push and startup on 2026-10-31 for
[Native Autocompletion](plan/rev-autocompletion.md), eight slices, followed by
[Optional Language Server Support](plan/rev-language-server-support.md), six
slices, through one pinned controller sequence. Both sets are Completed with
their own canonical trackers, exact-candidate gates and closure evidence.
The second set completes combined semantic/native source switching. The other
capability proposals retain their separate approval and planning boundaries.

## Readiness and next planning boundary

The soft-wrap and file-navigation specifications and models are Completed. All
five selected tickets are solved after verified implementation integration through
PRs #174-#181 and authorized fixture repairs #182-#183. The controller passed the
pinned full gate at `6b2afd4e07f269c979cd26faf60b6863314e69e9` in receipt
`F7OQU2CWIOPB3VW234VVECCQNG`; the [tracker](tracker/rev-wrap-navigation.md#final-gate-and-closure)
and [Slice 8 report](report/slices/rev-wrap-navigation/slice-8-interaction-acceptance.md)
record exact validation and documentation closure. The bookmark specification,
model, plan and tracker are also Completed. Its six slices and three authorized
repairs passed the unchanged full gate at
`41ba5f89465b2be02adc613cd796271169c245e0` in receipt
`7WOA4YIWYISUYIDTDT3ZC5SI5A`; the
[bookmark tracker](tracker/rev-bookmarks.md#final-gate-and-closure) and
[Slice 6 report](report/slices/rev-bookmarks/slice-6-acceptance.md#delivery-set-closure)
record the exact command and acceptance evidence.
Native completion's specification/model, plan and tracker are Completed for
the native scope. Its eight integrated slices passed the unchanged full gate
at `0f7cc36cb1082aa56954fa602294f47aac80e889`, receipt
`DAIRXONWTTLVG6RDIQQZKBSGXA`. The
[completion tracker](tracker/rev-autocompletion.md#final-gate-and-closure) and
[Slice 8 report](report/slices/rev-autocompletion/slice-8-acceptance.md#delivery-set-closure)
record the exact command and integrated resource evidence. Optional LSP's
specification/model, plan and tracker are also Completed. Its six slices and two
authorized repairs passed the unchanged integrated full gate at
`610f2e019a52e6b1314931b11fa070d320ecf809`, receipt
`YWRNPLD3FXNU6APAKXES5AS7I5`, establishing CMP02 semantic/native source
switching and joint native/LSP acceptance. The
[LSP tracker](tracker/rev-language-server-support.md#final-gate-and-closure) and
[Slice 6 closure](report/slices/rev-language-server-support/slice-6-acceptance.md#delivery-set-closure)
record exact validation, resources and owned cleanup. Modal editing's
specification/model, plan and tracker are Completed after eight slices through
PRs #212-#219. Its unchanged full gate passed at
`e046746acab9514117520eaa68b454f7145f6212`, receipt
`2MIARCIE3HXRZVLKHS5FZLPIJB`, with 2,114 top-level tests and native/semantic/PTY
acceptance. The [modal tracker](tracker/rev-modal-editing.md#final-gate-and-closure)
and [Slice 8 closure](report/slices/rev-modal-editing/slice-8-acceptance.md#delivery-set-closure)
record exact validation, resources, guide output and owned cleanup.
Of the six remaining capability proposals, language-server installation is now
Approved with a seven-slice plan and tracker. Its implementation and integrated
acceptance are pending. The other proposals, including optional external-program
exploration, retain their prior state. Earlier completed gates keep their
historical identities.

Before planning a selected delivery, close only its relevant product decisions,
reconcile the named delivered baseline clauses, and obtain implementation-scope
approval through the established workflow. Recheck current implementation and
external protocol/dependency references at that point. Captured historical
reference versions are not claims about current upstream releases.

Completed Org and earlier editor delivery records retain their historical
status and gate evidence. Pending specs belong here rather than in the shipped
User Guide or changelog until their behavior is actually delivered.
