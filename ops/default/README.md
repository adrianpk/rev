# Pending Editor Work

Status: proposed
Date: 2026-11-06
Scope: pending behavior after the completed Better Gutter, Org visual-indentation, native Git, integrated-terminal, native shell-command, native completion, optional LSP, modal editing, server-management, Pi/reload, heading-presentation, and Files, Directories, Projects and Workspaces deliveries
Owner decision: Better Gutter is Completed after three slices and repair #285; Org visual indentation is Completed after four canonical slices; native shell commands, text filters and integrated terminals are Completed; native Git integration is Completed after fourteen slices and authorized repair #272; other proposals retain their separate approval boundaries

This index connects the pending tickets, capability specifications and models.
It is a planning entry point, not a delivery plan or execution tracker. The
linked artifacts retain their own status. Capturing and versioning a proposal
does not approve its open decisions or start implementation.

Emacs similarity in keybindings and behavior remains the interaction guide.
Use its equivalent behavior where applicable; preserve explicit rev product
decisions and identify cases where rev has no direct Emacs counterpart.

## Complete inventory

The inventory connects completed capabilities, open presentation follow-ups
and remaining capability proposals. Solved tickets retain their
implementation/PR evidence.
The completed delivery passed its controller-owned full gate; other capabilities
retain their own specification/model state.

| Topic | Artifacts | State and remaining decisions |
| --- | --- | --- |
| Command prefix arguments | [Spec](spec/editor-prefix-arguments.md), [plan](plan/rev-prefix-arguments.md), [tracker](tracker/rev-prefix-arguments.md) | Delivered / Completed. Four slices through PRs #314-#317 deliver ARG01-ARG06: checked prefix entry, command-specific movement/edit/history meaning, contextual ownership and native acceptance with User Guide/reference guidance. The unchanged controller gate passed at `7cfef9be78fad975040ccd78dcb9cfebe32b28cd`, receipt `ZGGKKSNDW2ZMWGRELYH5WWFTZV`; [Slice 4 closure](report/slices/rev-prefix-arguments/slice-4-acceptance.md#delivery-set-closure) records the exact command, 207 selected tests and local Markdown/Spore results. Keyboard Macros retains its separate draft and approval boundary. |
| File tree parent navigation | [Spec](spec/editor-tree-parent-navigation.md), [plan](plan/rev-tree-parent-navigation.md), [tracker](tracker/rev-tree-parent-navigation.md) | Delivered / Completed. One slice through PR #312 delivers TUP01-TUP04: tree-up-directory and tree-local ^ browse automatic ancestors while retaining the selected return directory and editing state. Explicit directory, Project, Workspace and configured cwd roots stay fixed. The unchanged controller gate passed at `d75db7c33994b51eeb6a3a01c0b05647105ab7c8`, log `FU4RKA7ANFGNAMEYBVMUXMDHTI`; [Slice 1 closure](report/slices/rev-tree-parent-navigation/slice-1-parent.md#delivery-set-closure) records the exact command, 107 selected tests and local Markdown/Spore results. |
| Files, directories, Projects and Workspaces | [Spec](spec/editor-projects-workspaces.md), [model](spec/editor-projects-workspaces-model.md), [plan](plan/rev-projects-workspaces.md), [tracker](tracker/rev-projects-workspaces.md) | Delivered / Completed. Eight slices through PRs #298-#305 deliver native contexts, the atomic TOML catalog, basic named sessions, independent roots, lifecycle commands and the native startup screen. The unchanged controller gate passed at `6bbc6138ac9227beca51e6b86f759bbf39e36c94`, receipt `ZS6AIYOX66WRNHE5ICPVA46GIP`; [Slice 8 closure](report/slices/rev-projects-workspaces/slice-8-guidance.md#delivery-set-closure) records exact commands, 1,036 selected tests, timings, measured costs and owned cleanup. Full automatic startup-state restoration remains [deferred](spec/editor-projects-workspaces.md#deferred-specification-startup-session-restoration); startup.restore_session stays reserved for that future set. |
| Command recency and frequency | [Spec](spec/editor-command-ranking.md), [model](spec/editor-command-ranking-model.md), [plan](plan/rev-command-ranking.md), [tracker](tracker/rev-command-ranking.md) | Delivered / Completed. Three slices through PRs #291-#293 deliver UI02 and CMR01-CMR05: recent/frequent ranking within match ranks, bounded persisted usage, native/PTY acceptance and Markdown/Spore guidance. The unchanged controller gate passed at `fea4fffdfa3e66d8a7d7f1cc031a6637199424af`, receipt `342LB2K3MYK2Z2JS5ESVI4WSNP`; [Slice 3 closure](report/slices/rev-command-ranking/slice-3-acceptance.md#delivery-set-closure) records the exact command, 503 tests, measured acceptance, cleanup and evidence limits. |
| Better file tree | [Spec](spec/editor-better-file-tree.md), [model](spec/editor-better-file-tree-model.md), [plan](plan/rev-better-file-tree.md), [tracker](tracker/rev-better-file-tree.md) | Delivered / Completed. Three slices through PRs #287-#289 deliver BFT01-BFT04: decorated default, explicit minimal labels, shared Nerd Fonts choice and Markdown/Spore guidance. The unchanged controller gate passed at `e0351ac918a760b7940e660494f2309d67bd30a4`, receipt `2WXLCTXPDKGL7ZMY7KI52IQSFP`; [Slice 3 closure](report/slices/rev-better-file-tree/slice-3-acceptance.md#delivery-set-closure) records exact evidence and font-rendering limits. |
| Native Git integration | [Spec](spec/editor-git-integration.md), [model](spec/editor-git-integration-model.md), [plan](plan/rev-git-integration.md), [tracker](tracker/rev-git-integration.md) | Delivered / Completed. Fourteen slices through PRs #258-#271 and authorized repair #272 deliver native GIT01-GIT20 workflows, contextual bindings, installed-Git ownership, separate fetch/reviewed push, both local tag kinds and bounded process/source lifetime. The unchanged controller gate passed at `343850fb6914bf801dfa248b8cdd0e1c34ae1fef`, receipt `D6BU4EM7IWRUA3UHGDBE432UK3`; [Slice 14 closure](report/slices/rev-git-integration/slice-14-guidance.md#delivery-set-closure) records exact counts and evidence limits. |
| Keybinding hints in M-x | [Ticket](ticket/solved/20261031095151-show-keybindings-beside-m-x-commands.md), [spec](spec/editor-command-keybinding-hints.md), [plan](plan/rev-command-keybinding-hints.md), [tracker](tracker/rev-command-keybinding-hints.md) | Delivered / Completed. Two canonical slices through PRs #294 and #295 deliver KBH01-KBH05: effective originating-context shortcuts, canonical identity, complete cell-safe hints and Markdown/Spore guidance. The unchanged controller gate passed at `ca39b8f14e03d3753f071811b1151b8299bc3560`, receipt `BV6KPQKMFAI37JMO2HBOQOISIM`; [Slice 2 closure](report/slices/rev-command-keybinding-hints/slice-2-acceptance.md#delivery-set-closure) records the exact command, 362 selected tests, native/PTY acceptance, measured timings, owned cleanup and evidence limits. The ticket is solved with resolution fixed. |
| Command catalog descriptions | [Ticket](ticket/solved/20261106202027-complete-command-catalog-descriptions.md), [spec](spec/editor-command-descriptions.md), [plan](plan/rev-command-descriptions.md), [tracker](tracker/rev-command-descriptions.md) | Delivered / Completed. One slice through PR #307 fills all 618 approved built-in descriptions and retains optional user-command metadata. The unchanged controller gate passed at `978deacc49f6f5f948cea2070919ce91031f67a2`, log `A6S4H3BKQFKIG4WEYTWGATYYE4`; [Slice 1 closure](report/slices/rev-command-descriptions/slice-1-catalog.md#delivery-set-closure) records the exact command, six selected tests and Markdown/Spore validation. |
| Tree selection highlight and active-buffer synchronization | [Ticket](ticket/solved/20261031093408-highlight-tree-selection-and-sync-active-buffer.md), [spec](spec/editor-tree-selection.md), [model](spec/editor-tree-selection-model.md), [plan](plan/rev-tree-selection.md), [tracker](tracker/rev-tree-selection.md) | Delivered / Completed. PR #308 delivers persistent selection and default active-buffer following, with configurable manual selection and preserved root/source ownership. The controller gate passed at `27e222f65a81bbdb0d3aece0e7279877faf997b2`, log `KGGQSFRVEFDYE7B4A3KIT2FGIO`; [Slice 1 closure](report/slices/rev-tree-selection/slice-1-follow.md#delivery-set-closure) records the exact command and evidence. The ticket is solved with resolution fixed. |
| Open a file or directory from CLI/find-file | [Navigation spec](spec/editor-file-navigation.md), [model](spec/editor-file-navigation-model.md), [ticket](ticket/solved/20261028094803-open-directory-targets-from-cli-and-find-file.md) | Delivered / Completed spec / solved ticket. Slice 5 delivered directory acceptance and root precedence through PR #178. |
| Expand the initial tree root | [Navigation NAV03](spec/editor-file-navigation.md), [model](spec/editor-file-navigation-model.md), [ticket](ticket/solved/20261028153707-expand-file-tree-root-on-first-display.md) | Delivered / Completed spec / solved ticket. Slice 5 delivered one-time first-level expansion through PR #178; deliberate collapse is retained. |
| Tab completes the highlighted path | [Navigation NAV04](spec/editor-file-navigation.md), [model](spec/editor-file-navigation-model.md), [ticket](ticket/solved/20261028170748-complete-highlighted-find-file-path-with-tab.md) | Delivered / Completed spec / solved ticket. Slice 6 delivered selected-candidate Tab and separate final acceptance through PR #179; UI03 is reconciled. |
| Select the clicked tree entry | [Navigation NAV05](spec/editor-file-navigation.md), [model](spec/editor-file-navigation-model.md), [ticket](ticket/solved/20261030231016-select-the-clicked-file-tree-entry.md) | Delivered / Completed spec / solved ticket. Slice 7 delivered checked native activation and configurable selection-only through PR #180. |
| Word-aware soft wrapping | [Spec](spec/editor-soft-wrap.md), [model](spec/editor-soft-wrap-model.md), [ticket](ticket/solved/20261028224018-define-comfortable-long-line-wrapping.md) | Delivered / Completed spec / solved ticket. Default word-aware pane-width wrap and buffer-local Emacs Visual Line controls; Slices 1-4 delivered through PRs #174-#177. |
| Native bookmarks and gutter markers | [Spec](spec/editor-bookmarks.md), [model](spec/editor-bookmarks-model.md), [plan](plan/rev-bookmarks.md), [tracker](tracker/rev-bookmarks.md) | Delivered / Completed spec and model. Six slices through PRs #185-#190 and authorized repairs #191-#193 deliver named commands, checked lifecycle, persistence, listing, native markers, joint acceptance and guidance. The controller-owned combined gate passed. |
| Better gutter | [Spec](spec/editor-better-gutter.md), [model](spec/editor-better-gutter-model.md), [plan](plan/rev-better-gutter.md), [tracker](tracker/rev-better-gutter.md) | Delivered / Completed. Three slices through PRs #282-#284 and repair #285 deliver outside markers, fixed symbols and the startup Nerd Fonts flag. The unchanged controller gate passed at `bcf705d9e6ac70caef0d2fc7c9c124d966f56627`, receipt `2VTBXR6STDPODOXBSBNFIUWESW`; [Slice 3 closure](report/slices/rev-better-gutter/slice-3-acceptance.md#delivery-set-closure) records acceptance and limits. Folding indicators and debugger behavior remain future. |
| Built-in autocompletion | [Spec](spec/editor-autocompletion.md), [model](spec/editor-autocompletion-model.md), [plan](plan/rev-autocompletion.md), [tracker](tracker/rev-autocompletion.md) | Delivered / Completed. Eight slices through PRs #195-#202 deliver automatic/manual interaction, ten bundled parsers, native Org contexts, bounded observations, registered external grammars and native/resource/PTY acceptance. The native gate passed; the subsequent integrated LSP gate also passed CMP02 semantic/native switching and joint acceptance. |
| Optional native LSP client | [Spec](spec/editor-language-server-support.md), [model](spec/editor-language-server-support-model.md), [plan](plan/rev-language-server-support.md), [tracker](tracker/rev-language-server-support.md) | Delivered / Completed. Six slices through PRs #203-#208 and authorized repairs #209/#210 deliver disabled-by-default stdio support, unsaved sync, UTF-8/UTF-16 mapping, shared processes, buffer exclusions, checked semantic edits, fallback and real Rust-server acceptance. The unchanged controller full gate passed; LSP01-LSP06 and joint CMP01-CMP09 are closed. |
| Simple optional LSP installation | [Spec](spec/editor-language-server-installation.md), [model](spec/editor-language-server-installation-model.md), [plan](plan/rev-language-server-installation.md), [tracker](tracker/rev-language-server-installation.md) | Delivered / Completed. Seven slices through PRs #222–#228 deliver guided existing/custom setup, associations, explicit source choice, managed recipes and lifecycle. Any compatible language/server remains eligible independently of bundled grammars or the initial catalog. The unchanged controller gate passed INS01-INS06 with both-profile native/PTY management, real Rust/Go semantics and grammar-independent routing. |
| Native AI commands via provider API | [Spec](spec/editor-ai-commands.md), [model](spec/editor-ai-commands-model.md) | Draft broader contract. Selected shared commands are delivered through Pi; native API transport and native context construction remain deferred. |
| Independent optional Pi backend | [Spec](spec/editor-ai-pi-integration.md), [model](spec/editor-ai-pi-integration-model.md), [plan](plan/rev-ai-pi.md), [tracker](tracker/rev-ai-pi.md) | Delivered / Completed. Eight slices through PRs #229-#236 deliver private Pi RPC sessions, guarded discovery, checked native application and reusable commands. The unchanged controller gate and separate bounded OAuth acceptance passed; native API remains deferred. |
| Configurable external file changes | [Spec](spec/editor-external-file-changes.md), [model](spec/editor-external-file-changes-model.md), [plan](plan/rev-ai-pi.md), [tracker](tracker/rev-ai-pi.md) | Delivered / Completed in the same eight-slice delivery. Checked revert, automatic clean-buffer reload, passive notification and unsaved-edit protection passed EFC01-EFC08 through the unchanged controller gate. |
| Startup-selected Emacs/Vim editing | [Spec](spec/editor-modal-editing.md), [model](spec/editor-modal-editing-model.md), [plan](plan/rev-modal-editing.md), [tracker](tracker/rev-modal-editing.md) | Delivered / Completed. Eight slices through PRs #212-#219 deliver immutable startup profiles, checked modal navigation/operators/selections/jumps/repeat and shared colon/M-x/completion workflows. The unchanged controller full gate passed MOD01-MOD14, native/installed-server/PTY acceptance and guidance. |
| Org initial visibility and drawers | [Ticket](ticket/solved/20261103111614-configure-org-drawer-visibility.md), [spec](spec/editor-org-initial-visibility.md), [model](spec/editor-org-initial-visibility-model.md), [plan](plan/rev-org-initial-visibility.md), [tracker](tracker/rev-org-initial-visibility.md) | Delivered / Completed. Slice 1 through PR #310 delivers four startup states, structural document precedence, whole-view drawer controls and startup reset while preserving showeverything and full reveal. The unchanged controller gate passed at `dc31493589c9a35eff79edebe1765a76a3fa4651`; [Slice 1 closure](report/slices/rev-org-initial-visibility/slice-1-policy.md#delivery-set-closure) records exact evidence. |
| Org heading glyphs and terminal fonts | [Spec](spec/editor-org-heading-presentation.md), [model](spec/editor-org-heading-presentation-model.md), [plan](plan/rev-org-headings.md), [tracker](tracker/rev-org-headings.md) | Delivered / Completed. Three slices through PRs #238-#240 deliver six cyclic reference glyphs, complete startup preferences, independent native view toggle/reset, source-backed prefix width and bounded projection. The unchanged controller full gate passed OGP01-OGP08 with 420 selected tests and both-profile native/resource/PTY acceptance. Fonts and ligatures remain terminal-owned. |
| Org visual body indentation | [Spec](spec/editor-org-indentation.md), [model](spec/editor-org-indentation-model.md), [plan](plan/rev-org-indentation.md), [tracker](tracker/rev-org-indentation.md) | Delivered / Completed. Four slices through PRs #276-#279 deliver OIN01-OIN05: enabled-by-default visual indentation, one TOML boolean, three native commands and buffer-local policy. The unchanged controller gate passed at `61e6e6a1b73d94577c7851aae6f200f6101d35c7`, receipt `YE523W7IGTDKXEUNGCMA3NNJBY`; [Slice 4 closure](report/slices/rev-org-indentation/slice-4-acceptance.md#delivery-set-closure) records native/resource/PTY evidence and strict local guide validation. |
| Native shell commands and text filters | [Spec](spec/editor-shell-commands.md), [model](spec/editor-shell-commands-model.md), [plan](plan/rev-shell-commands.md), [tracker](tracker/rev-shell-commands.md) | Delivered / Completed. Four slices through PRs #242-#245 and repair #246 deliver explicit command entry, bounded stdin/stdout, checked insert/replace and native controls. The unchanged controller gate passed SHL01-SHL08 with both-profile native/resource/PTY acceptance and guide projection. |
| Integrated terminals | [Spec](spec/editor-integrated-terminal.md), [model](spec/editor-integrated-terminal-model.md), [plan](plan/rev-integrated-terminal.md), [tracker](tracker/rev-integrated-terminal.md) | Delivered / Completed. Eight slices through PRs #248-#255 and authorized readiness repair #256 deliver lower-panel/direct-buffer shells, same-process relocation, multiple sessions, native input/copy and owned shutdown. The unchanged combined controller gate passed TRM01-TRM11 with native/PTY/resource acceptance and Markdown/Spore guidance. |
| External programs/extensions | [Spec](spec/editor-external-extensions.md), [model](spec/editor-external-extensions-model.md) | Optional exploration only. Explicit configuration, stdio and one child per invocation are the refined draft profile. Adoption itself is undecided; no scheduled delivery or baseline dependency. |

## Selected execution order

Command Prefix Arguments is Completed on 2026-11-08 after all four canonical
PRs (#314-#317) integrated into dev. The unchanged controller gate passed at
`7cfef9be78fad975040ccd78dcb9cfebe32b28cd`, receipt
`ZGGKKSNDW2ZMWGRELYH5WWFTZV`, with 207 selected native tests, complete
command/configuration catalogs and the strict local Markdown/Spore User Guide
build. The specification, plan and tracker are Completed; all four reports are
delivered. [Slice 4 closure](report/slices/rev-prefix-arguments/slice-4-acceptance.md#delivery-set-closure)
retains the exact command, immutable report identities and evidence limits.
Keyboard Macros and other pending topics keep their separate approval boundaries;
this completion does not start a successor delivery.

On 2026-11-07 the owner approved the Files, Directories, Projects and Workspaces
core and one eight-slice Looper delivery through dev. All eight canonical PRs
(#298-#305) are integrated and reports delivered. The unchanged controller gate
passed at `6bbc6138ac9227beca51e6b86f759bbf39e36c94`, receipt `ZS6AIYOX66WRNHE5ICPVA46GIP`, with
1,036 selected Rust tests, native/resource acceptance, both-profile PTYs and local
Markdown/Spore guidance. The core specification/model, plan and tracker are
Completed; [Slice 8 closure](report/slices/rev-projects-workspaces/slice-8-guidance.md#delivery-set-closure)
records exact evidence. Other pending drafts and full startup-state restoration
keep their separate scope and approval boundaries.

On 2026-11-06 the owner approved continuation from the formal Command Keybinding
Hints specification. The two-slice plan delivers effective annotations, then
joint acceptance and guidance, in one Looper run through dev. Both slices are
integrated and their reports delivered. The unchanged controller gate passed
at `ca39b8f14e03d3753f071811b1151b8299bc3560`, receipt `BV6KPQKMFAI37JMO2HBOQOISIM`.
The delivery is Completed and the source ticket solved with resolution fixed.

On 2026-11-06 the owner approved Command Recency and Frequency and requested
the established delivery process. The three-slice plan delivers session ranking,
bounded persistence, then acceptance and Markdown/Spore guidance. Looper owns
execution progress; the linked plan/tracker own the slice identities. Keybinding
hints are now approved separately; other pending work retains its own approval
boundary.

Better File Tree is Completed on 2026-11-06 after three canonical slices through
PRs #287-#289. The unchanged controller gate passed at
`e0351ac918a760b7940e660494f2309d67bd30a4`, receipt `2WXLCTXPDKGL7ZMY7KI52IQSFP`,
with 395 selected tests, the native presentation/configuration matrix, both-profile
PTY acceptance and local Markdown/Spore guidance. The plan, tracker and
specification/model are Completed; the [Slice 3 closure](report/slices/rev-better-file-tree/slice-3-acceptance.md#delivery-set-closure)
records exact commands, immutable report introductions, timings, cleanup and
physical-font evidence limits. The separate persistent selection/active-buffer
synchronization delivery is Completed through PR #308, with the ticket solved
and its [closure evidence](report/slices/rev-tree-selection/slice-1-follow.md#delivery-set-closure)
recorded. Other pending topics retain their current approval boundaries.

Better Gutter is Completed on 2026-11-06 after three canonical slices and
repair #285. The unchanged controller gate passed at
`bcf705d9e6ac70caef0d2fc7c9c124d966f56627`, receipt `2VTBXR6STDPODOXBSBNFIUWESW`,
with 398 selected tests, two nested child runs, native/two-profile PTY acceptance
and local Markdown/Spore guidance. The plan, tracker and specification/model
are Completed; the [Slice 3 closure](report/slices/rev-better-gutter/slice-3-acceptance.md#delivery-set-closure)
records exact evidence. Future folding/debugger capabilities and other pending
topics retain their own approval boundaries.

On 2026-11-05 the owner approved Org visual indentation and explicitly requested
the plan, slicing and one Looper startup. All four canonical slices through
PRs #276-#279 are delivered; the unchanged controller full gate passed at
`61e6e6a1b73d94577c7851aae6f200f6101d35c7`, receipt
`YE523W7IGTDKXEUNGCMA3NNJBY`. The specification/model and plan/tracker are
Completed. The linked tracker and Slice 4 closure retain exact commands,
immutable report introductions, resource/PTY observations and local guide
evidence. Other pending topics retain their current approval boundaries.

On 2026-11-04 the owner approved the complete native Git specification/model,
binding matrix, local-operation policies, bounds and fourteen-slice plan/tracker,
and explicitly requested one autonomous delivery through dev. Fourteen slices
and authorized repair #272 are now integrated; the unchanged controller gate
passed and the specification/model and plan/tracker are Completed. The linked
tracker and Slice 14 closure retain the exact candidate, receipt and local
acceptance limits. Other presentation follow-ups retain their own approval
boundaries.

On 2026-11-03 the owner requested two independent specifications and starting
the agent-selected first capability. Native shell commands and text filters
were selected first. Their specification/model and four-slice plan/tracker are
Completed after PRs #242-#245, repair #246 and the unchanged controller gate.
The owner then approved the separate interactive-terminal specification and
model on 2026-11-03, including multiple on-demand sessions, a lower-panel default,
direct main-buffer entry and live relocation in both directions. The owner then
explicitly requested Looper startup. The linked specification/model and
eight-slice plan/tracker are now Completed after PRs #248-#255, authorized repair
#256 and the unchanged combined controller gate. Exact acceptance and closure
belong to their tracker and report; the retained checkpoint owns execution state.
The larger external extension proposal remains optional.

The owner selected Org heading presentation on 2026-11-03 after the Pi/reload
delivery. Its specification/model and plan/tracker are now Completed after
canonical PRs #238-#240 and the unchanged controller full gate. One retained
controller owns the set's exact gate evidence and final checkpoint verification.
General drawer visibility is now Completed separately through PR #310 and the
[Org visibility delivery](tracker/rev-org-initial-visibility.md).

The owner approved proceeding with Pi and general external reload on
2026-11-02 after the architecture explanation. The linked eight-slice plan and
tracker place reload before the Pi increments in one autonomous delivery.
All eight slices and the frozen full gate are complete. Direct API construction
remains deferred. External reload does not grant Pi
direct source-write authority; native result application retains the approved
single-target boundary. Actual startup/progress belongs to the controller.

The owner selected optional language-server installation on 2026-11-02 and
authorized the seven-slice plan and autonomous Looper implementation through
dev. Its seven canonical slices and unchanged integrated gate are complete.
The specification/model and plan/tracker are Completed. Delivered setup and
management remain independent of the native grammar list and acquisition
catalog. Other pending capabilities retain their own state.

The owner selected startup-only Emacs/Vim modal editing on 2026-11-01,
temporarily deferring discussion item 10, and authorized its eight-slice delivery.
The specification/model and plan/tracker are now Completed after PRs #212-#219
and the unchanged controller full gate. Shared native editing, colon/M-x entry
and completion retain their existing owners. Item 10 now has its separate
approved heading-presentation specification; modal closure
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
- Selected native shell commands reuse command/prompt, checked text-edit and
  low-level process ownership. Their one-shot streams do not require terminal
  emulation, PTYs, interactive-session placement or the optional extension API.

## Playground presentation

New Rev playgrounds use isolated TOML with `gutter.nerd_fonts = true` for
screenshot-ready bookmark symbols. The user supplies a compatible terminal font;
the playground does not install fonts or change global editor configuration.

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
Language-server installation's specification/model, plan and tracker are
Completed after seven slices through PRs #222–#228. Its unchanged controller
gate passed at `0bc1ede50fd128897d80507c24cbaf9a0e670fda`, receipt
`EVGM5GWST4Q4AX4CYQWMEGUFSK`, with 412 selected top-level tests,
three opt-in cases ignored, and native/real-server/PTY management acceptance.
The [installation tracker](tracker/rev-language-server-installation.md#final-gate-and-closure)
and [Slice 7 closure](report/slices/rev-language-server-installation/slice-7-acceptance.md#delivery-set-closure)
record exact validation, resources, local guide output and owned cleanup.
Pi and external-file changes are Completed after eight canonical slices through
PRs #229-#236. Their unchanged controller gate passed at
`ecd02abcbd1f08530d8fad1395672dc6d00ee085`, receipt
`HBFDUZX54JK3RJUBGSQYIJANGO`, with 578 selected top-level tests,
six nested child cases and both-profile native/PTY/offline Pi acceptance. The
[Pi/reload tracker](tracker/rev-ai-pi.md#final-gate-and-closure) and
[Slice 8 closure](report/slices/rev-ai-pi/slice-8-acceptance.md#delivery-set-closure)
record exact validation, resources, the separate OAuth observation and local
User Guide output. Native API support remains deferred in its broader draft.
Heading presentation is Completed after PRs #238-#240. The unchanged full gate
passed at `18f625997de4c0ac2e1ae50d6fea667fe14f2c2d`, receipt
`T2QVMEBIU5HJTNBIOEXKDCUFSU`. The
[heading tracker](tracker/rev-org-headings.md#final-gate-and-closure) and
[Slice 3 closure](report/slices/rev-org-headings/slice-3-acceptance.md#delivery-set-closure)
record the 420 selected tests, native/resource/four-PTY acceptance, source-byte
fidelity and local User Guide output. Remaining capability proposals retain
their independently recorded states, including optional external-program
exploration. The separate terminal delivery is completed below.
Earlier completed gates keep their historical identities.

Shell commands and text filters are Completed after four canonical slices and
repair #246. The unchanged controller gate passed at
`736d631947d4d379cd65dd695ff2ab00fd6c964b`, receipt
`5SCPQH64UEI7UZ2VFXVFZMGKWH`, with 402 selected top-level tests, three nested
child cases, both-profile native/resource/PTY acceptance and the local guide
build. The [shell tracker](tracker/rev-shell-commands.md#final-gate-and-closure)
and [Slice 4 closure](report/slices/rev-shell-commands/slice-4-acceptance.md#delivery-set-closure)
record exact validation and owned cleanup. Six native commands, four non-modal
key sequences and five typed preferences are delivered under SHL01-SHL08.

Integrated terminals are Completed after eight canonical slices and authorized
readiness repair #256. The unchanged controller gate passed at
`52da8df25937c2165d72916603ef048712b3abbb`, receipt
`IYS47Y2HH6YVYH3AMLMDEOZHVQ`. It passed 715 selected top-level tests,
both-profile native/PTY/resource acceptance, normal/error restoration and the
local User Guide build. The [terminal tracker](tracker/rev-integrated-terminal.md#final-gate-and-closure)
and [Slice 8 closure](report/slices/rev-integrated-terminal/slice-8-guidance.md#delivery-set-closure)
record exact commands, measured resource bounds and the documentation-only
closure. TRM01-TRM11, all 37 terminal commands and seven startup preferences
are delivered; other capability proposals retain their independent boundaries.

Native Git is Completed after fourteen slices through PRs #258-#271 and
repair #272. The unchanged controller gate passed at
`343850fb6914bf801dfa248b8cdd0e1c34ae1fef`, receipt
`D6BU4EM7IWRUA3UHGDBE432UK3`: 759 top-level tests and three nested
child runs, both-profile/object-format native and PTY journeys, resource/lifetime
acceptance, complete catalogs and local Markdown/Spore validation. The configured
modal filter selected zero tests; profile coverage and that evidence limit
remain explicit in the [tracker](tracker/rev-git-integration.md#closure) and
[Slice 14 closure](report/slices/rev-git-integration/slice-14-guidance.md#delivery-set-closure).
Earlier editor and feature gates retain their original candidates.

Before planning another selected delivery, close only its relevant product decisions,
reconcile the named delivered baseline clauses, and obtain implementation-scope
approval through the established workflow. Recheck current implementation and
external protocol/dependency references at that point. Captured historical
reference versions are not claims about current upstream releases.

Completed Org and earlier editor delivery records retain their historical
status and gate evidence. Pending specs belong here rather than in the shipped
User Guide or changelog until their behavior is actually delivered.
