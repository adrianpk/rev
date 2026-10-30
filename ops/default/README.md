# Pending Editor Work

Status: proposed
Date: 2026-10-31
Scope: pending behavior after the completed native Org delivery
Owner decision: directory opening is first; other ordering remains proposed

This index connects the pending tickets, capability specifications and models.
It is a planning entry point, not a delivery plan or execution tracker. The
linked artifacts retain their own status. Capturing and versioning a proposal
does not approve its open decisions or start implementation.

## Complete inventory

The fifteen topics below are pending. Broad capabilities have draft specs and
models; bounded navigation requests remain individual open tickets under one
shared behavior contract. No topic is marked delivered by this documentation.

| Topic | Artifacts | State and remaining decisions |
| --- | --- | --- |
| Open a file or directory from CLI/find-file | [Navigation spec](spec/editor-file-navigation.md), [model](spec/editor-file-navigation-model.md), [ticket](ticket/open/20261028094803-open-directory-targets-from-cli-and-find-file.md) | Draft/open. Accept-versus-descend gesture and full root-policy precedence. First priority selected by the owner. |
| Expand the initial tree root | [Navigation NAV03](spec/editor-file-navigation.md), [model](spec/editor-file-navigation-model.md), [ticket](ticket/open/20261028153707-expand-file-tree-root-on-first-display.md) | Draft/open. First level only; retain deliberate later collapse. Coordinate with directory opening. |
| Tab completes the highlighted path | [Navigation NAV04](spec/editor-file-navigation.md), [model](spec/editor-file-navigation-model.md), [ticket](ticket/open/20261028170748-complete-highlighted-find-file-path-with-tab.md) | Draft/open. Selected/default-first candidate, continued directory completion and separate final acceptance; reconcile delivered UI03. |
| Select the clicked tree entry | [Navigation NAV05](spec/editor-file-navigation.md), [model](spec/editor-file-navigation-model.md), [ticket](ticket/open/20261030231016-select-the-clicked-file-tree-entry.md) | Draft/open. Selection requested; same-click activation and any preference remain undecided. |
| Word-aware soft wrapping | [Spec](spec/editor-soft-wrap.md), [model](spec/editor-soft-wrap-model.md), [ticket](ticket/open/20261028224018-define-comfortable-long-line-wrapping.md) | Draft/open. Visual direction approved; defaults, motions, structured Org regions and display limits remain open. No hard newline insertion. |
| Native bookmarks and gutter markers | [Spec](spec/editor-bookmarks.md), [model](spec/editor-bookmarks-model.md) | Draft. Naming/scope, edit anchors, navigation, persistence and marker collision policy. |
| Built-in autocompletion | [Spec](spec/editor-autocompletion.md), [model](spec/editor-autocompletion-model.md) | Draft. Native UI; operational LSP preferred, Tree-sitter fallback, no ctags. Trigger/coverage/index scope and limits remain open. |
| Optional native LSP client | [Spec](spec/editor-language-server-support.md), [model](spec/editor-language-server-support-model.md) | Draft. Completion first; protocol, launch/sharing, synchronization, encoding and failure limits remain open. |
| Simple optional LSP installation | [Spec](spec/editor-language-server-installation.md), [model](spec/editor-language-server-installation-model.md) | Draft. Rust first; secondary language list includes Odin. Actual recipes, prerequisites, acquisition/verification and managed/external precedence remain open. |
| Native AI commands via provider API | [Spec](spec/editor-ai-commands.md), [model](spec/editor-ai-commands-model.md) | Draft. Discrete explain/implement/transform operations and reusable declarative commands. Context grants, API profile, acceptance and persistence remain open. |
| Independent optional Pi backend | [Spec](spec/editor-ai-pi-integration.md), [model](spec/editor-ai-pi-integration-model.md) | Draft. Pi owns its authentication and discovery; rev owns editor snapshots and checked edits. Protocol and enforceable read-only authority remain open. |
| Startup-selected Emacs/Vim editing | [Spec](spec/editor-modal-editing.md), [model](spec/editor-modal-editing-model.md) | Draft. Common Vim motions/operators/search/Visual/jumps; canonical rev commands through colon. Finite binding matrix and edge semantics remain open. |
| Org heading glyphs and terminal fonts | [Spec](spec/editor-org-heading-presentation.md), [model](spec/editor-org-heading-presentation-model.md) | Draft. Six reference glyphs captured; prefix/source fidelity required. Depth overflow, defaults and reveal/geometry details remain open. Fonts and ligatures belong to the terminal. |
| Integrated terminals | [Spec](spec/editor-integrated-terminal.md), [model](spec/editor-integrated-terminal-model.md) | Draft. Lower panel by default, direct buffer alternative and live relocation, Rust alacritty_terminal direction. Exact dependency pin, settings, IO integration and limits need preparation. |
| External programs/extensions | [Spec](spec/editor-external-extensions.md), [model](spec/editor-external-extensions-model.md) | Optional exploration only. Explicit configuration, stdio and one child per invocation are the refined draft profile. Adoption itself is undecided; no scheduled delivery or baseline dependency. |

## Proposed execution order

Only directory opening's first position is an owner decision. The remaining
positions retain the earlier recommendation; they are not an approved delivery
set, slice count or instruction to launch Looper.

1. File/directory opening, coordinated with first-level expansion and selected
   Tab completion through the shared navigation contract.
2. Native bookmarks.
3. Native completion and its Tree-sitter path.
4. Optional LSP client, integrated with completion source switching.
5. Language-server installation and discovery.
6. AI commands with independent native API and optional Pi execution paths.

Tree pointer selection, soft wrapping, modal editing, heading presentation and
integrated terminals have no assigned position yet. Their dependencies below
allow a later scheduling decision without implying they must wait for AI.
External extensions have no implementation commitment or assigned position.

## Actual dependencies

The proposed order is not a chain of prerequisites.

- Directory opening supplies explicit root/file-parent context for the full
  AI experience and eventual terminal launch defaults. It does not grant AI
  permission to transmit content or settle file-only discovery boundaries.
- First expansion and Tab completion share navigation observations with
  directory opening. Tree-click activation is an independent product choice;
  do not silently select an activation default while implementing navigation.
- Bookmarks reuse the existing gutter and canonical location/edit owners.
  They are not prerequisites for completion, LSP, AI or modal jump history.
- Native completion owns the shared UI, checked edits and Tree-sitter source.
  The LSP client supplies the optional semantic source. Plan and validate that
  integration together; a native-only stage does not satisfy semantic-source
  switching. Directory roots are required only for a selected directory index.
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

## Readiness and next planning boundary

All twelve capability specs and their twelve models remain drafts; all five
linked tickets remain open. The artifacts now capture behavior, ownership,
failure/validation direction, dependencies and explicit unresolved decisions.
There is no new plan, tracker, slice allocation or controller configuration.

Before planning a selected delivery, close only its relevant product decisions,
reconcile the named delivered baseline clauses, and obtain implementation-scope
approval through the established workflow. Recheck current implementation and
external protocol/dependency references at that point. Captured historical
reference versions are not claims about current upstream releases.

Completed Org and earlier editor delivery records retain their historical
status and gate evidence. Pending specs belong here rather than in the shipped
User Guide or changelog until their behavior is actually delivered.
