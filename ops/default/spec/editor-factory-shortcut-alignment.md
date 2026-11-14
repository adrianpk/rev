# Factory Shortcut Alignment

Status: Draft
Date: 2026-11-14
Approved scope: product owner, 2026-11-14; semantic audit slices 1-2 and preparation publication; target mappings and runtime delivery remain subject to the documented decision checkpoint
Format: OpenSpec capability additions; workspace specification contract
Inspection baseline: `197bf98be615ceca06a3bab0ef5cd24b3bee2ffb`
Readiness: all semantic rows reviewed; exact mapping proposal published for D01-D03 acceptance before the Slice 2 PR and runtime freeze
Source: [Factory shortcut review](../ticket/open/20261114010916-clarify-custom-bindings-for-unbound-commands.md)
Baselines: [Configuration](editor-configuration.md), CFG12-CFG14 and CFG20-CFG24; [Configuration Bootstrap](editor-configuration-bootstrap.md); [Interaction](editor-interaction.md), UI02; [Modal Editing](editor-modal-editing.md); [Prefix Arguments](editor-prefix-arguments.md)
Mode contracts: [Org](editor-org-mode.md), Familiar interaction; [Markdown](editor-markdown-mode.md); [Git](editor-git-integration.md), Binding review matrix; [Projects and Workspaces](editor-projects-workspaces.md)
Discovery: [Command Keybinding Hints](editor-command-keybinding-hints.md)
Current reference: [Bindings](../../../docs/reference/bindings.md), [Commands](../../../docs/reference/commands.md), [Configuration](../../../docs/reference/config.toml)
Plan: [Factory Shortcut Alignment Delivery](../plan/rev-factory-shortcuts.md)
Tracker: [Factory Shortcut Alignment Tracker](../tracker/rev-factory-shortcuts.md)

## Purpose

Reduce friction when moving between Rev and an Emacs-based editing environment.
An existing native operation with an equivalent purpose and context should use
the familiar shortcut for that operation, even when its command name differs.
The comparison includes related command families and contextual dispatchers,
not only one-to-one command names or commands without a direct binding.

Preserve the existing design: compiled factory defaults, validated user
overrides and focus-owned dispatch. This change defines shortcut selection and
compatibility. It introduces no durable entity, configuration schema or model
companion; the existing configuration and command models remain authoritative.

## Reference profile and evidence

Use stock GNU Emacs 31.1 and Org 9.8.7 as the inspected core reference.
Installed Markdown mode 2.8-alpha and Magit source declaring 4.5.0 provide
package-specific references. Record their revisions when reproducing a finding.
Load references without personal init files or custom bindings. They are
inspection tools, not Rev runtime or build dependencies.

The inspected native catalog contains 707 commands. Its 28 context maps contain
2,349 factory entries, including inherited duplicates and modal states. Of the
catalog names, 462 occur in factory entries; six additional targets are
minibuffer actions. These counts describe the inspection baseline, not required
future catalog sizes or the number of keyboard-accessible operations.

The [semantic matrix](../report/rev-factory-shortcuts/semantic-matrix.md) now
reviews all 707 command rows and six prompt actions: 333 core/profile rows plus
238 Org, 119 Git and 17 Markdown names. The source tree and raw catalog remain
unchanged. [Contextual evidence](../report/rev-factory-shortcuts/mode-review.md)
records operation owners, package revisions, indirect branches and deliberate
differences. The [exact mapping proposal](../report/rev-factory-shortcuts/mapping-contract.md)
is complete and still requires owner acceptance; successful audit probes do
not supply mapping or runtime approval.

Static lookup is supporting evidence only. A missing binding, a prefix result,
a wrapper name or a dispatcher that reads its next key inside a function does
not establish a missing capability. Actual behavior and point context decide.

## Proposed mapping targets

These are concrete proposals for review. Draft status does not authorize
runtime changes. Accepted entries will become the implementation's mapping
contract together with the completed semantic review.

| ID | Native operation | Proposed factory sequence | Context and profile policy |
| --- | --- | --- | --- |
| P01 | `modal-line-first-text` | `M-m` | Nonmodal editor and all six modal editing states, including inherited editing contexts. Move point without changing interaction state. |
| P02 | `balance-windows` | `C-x +` | Nonmodal editor and all six modal editing states through the common `C-x` family; existing focus admission remains authoritative. |
| P03 | `undo-redo` | `C-M-_` | Nonmodal editor, modal Insert and modal Replace. Keep existing Normal/Visual undo/redo controls and nonmodal `C-r` backward search. |
| P04 | `project-switch-project` | `C-x p p` | Nonmodal editor and all six modal editing states through the common `C-x` family. Restore a registered Project through the existing context transition. |
| P05 | `org-inc-effort` | `C-c C-x E` | Active Org editing contexts; use the existing heading/property admission and effort workflow. |
| P06 | `org-columns` | `C-c C-x C-c` | Org. Replace the current factory clock-report assignment with the property-column operation. Retain `org-clock-report` through named invocation. |
| P07 | `org-babel-insert-header-arg` | `C-c C-v j` and `C-c C-v C-j` | Org source-block context. Remove its factory `C-c C-v h` assignment; that reference chord belongs to Babel help. Do not introduce an unsupported help command. |
| P08 | `org-agenda-day-view` | `d`, retaining `v d` | Agenda. Replace the current `d` habit-graph-date assignment. Preserve `org-habit-graph-date` by name; P09-P13 specify the remaining proposed Agenda family. |
| P09 | `org-tags-view` | `v M`, remove `M` | Agenda native view family; preserve named access without claiming moon phases. |
| P10 | `org-search-view` | `v /`, remove `s` | Agenda; existing `v s` remains span, and save-all is not implemented. |
| P11 | `org-habit-toggle-show-all-today` | `v A`, remove `A` | Agenda native habit view; no append-agenda approximation. |
| P12 | `org-habit-set-graph-window` | `v G`, remove `G` | Agenda native graph-window prompt; no time-grid approximation. |
| P13 | `org-agenda-sort` | `v S`, remove `S` | Agenda native sort; no sunrise/sunset approximation. |
| P14 | `git-cherry-pick` | History `A` replaces amend | Existing selected/ordered revision owner; amend becomes named-only. |
| P15 | `git-cherry-pick` old alias | Remove History `P` | Retain `A` and named invocation; no History push owner/menu is invented. |
| P16 | `git-commit-fixup` old key | Remove History `F` | Fixup remains named; native generic pull is absent. |
| P17 | `git-log-next-page` old key | Remove History `f` | Paging remains named; fetch needs a configured remote in Refs. |
| P18 | `git-fetch` | Refs `f`, retain `&` / `M f` | Native direct operation with current selected remote and review, not a transient menu. |
| P19 | `git-push` | Refs `P`, retain `p` / `M p` | Exact local branch or qualified mark and upstream review; no History transport alias. |

P01 deliberately reuses an existing native identity with a different name from
the reference's `back-to-indentation`. P04 is an analogous project-switch
workflow, not a claim that Rev implements the reference's project action menu.
P06 exposes Rev's existing read-only property-column projection; it does not
introduce editable inline columns.

The native normalizer aliases C-j to RET, so the second P07 spelling owns the
normalized C-c C-v RET leaf. The contract records both forms explicitly.

P06 does not relocate clock reports onto `C-c C-x C-r`: that chord already
controls radio-button behavior. Users may explicitly retain old assignments in
their existing configuration maps.

## Review coverage and decisions

The review covers editor operations, prompts/completion, tree/bookmarks, macro
editing and menus, every native Git view, Org document/agenda/capture/calendar/
source-edit contexts, Markdown and all six modal states. Each family must have
a recorded disposition even when the result is to preserve current defaults.

| Area | Observed distinction | Required disposition |
| --- | --- | --- |
| Existing editor families | File/save/exit, point/mark, motion, search, replacement and pane controls already use familiar families. | Preserve assignments that satisfy the operation contract; do not rebind merely to increase coverage. |
| Org Agenda | `M`, `s`, `A`, `G`, `S` invoke tag/search/habit/sort actions where the reference uses moon phases, save-all, append agenda, time grid and sunrise/sunset. | Decide the final placement of native view and habit controls as a family. P08 specifies the separate daily-view correction. |
| Git History | `P`, `A`, `F`, `f` currently mean cherry-pick, amend, fixup and next page; the reference uses push, cherry-pick, pull and fetch families. | Review native menus, direct actions and prefixes together. Identify explicit exceptions or approved replacements, including how displaced actions remain reachable. |
| Git baseline | The completed Git specification explicitly assigns `f`, `A`, `F` and `P` to the current native actions. | Any accepted replacement must explicitly supersede the affected rows. A differing reference map does not by itself repeal an approved native contract. |
| File tree | `C-x t` is an established native tree namespace; the reference uses it for tabs. | Preserve this namespace and the approved horizontal panning/selection defaults. Do not substitute a different tree or tab interaction model. |
| Markdown | Native tables deliberately follow the existing Org-like table family. | Preserve that convention; inspect plain text, lists and tables separately before proposing additional keys. |
| Indirect operations | Return dispatchers expose Markdown continuation and table-row movement; Org dispatchers select contextual actions. | Record their actual keyboard routes. Do not count them as missing merely because a helper name has no direct map entry. |
| Related modal primitives | Join direction, paragraph boundaries, paired delimiters, indentation and repeat scope differ from superficially similar reference operations. | Leave misleading reference chords unassigned to those primitives. Any semantic adapter needs an explicit behavior contract before inclusion. |
| Named-only operations | Some reference operations have no stock global key, and some Rev functions have no corresponding reference family. | Retain named invocation unless a specific convention and context justify a factory key. |

Decision checkpoint (proposal complete; acceptance pending):

1. **D01 Agenda family:** proposed P08-P13 use daily d and a native v family,
   preserving named access and unselected defaults. The contract also presents
   retaining the conflicting native keys as explicit exceptions. The owner must
   choose; no unsupported reference operation is introduced.
2. **D02 Git family:** proposed P14-P19 remove misleading History transport
   letters, put cherry-pick on A, and add Refs direct fetch/push aliases. Other
   native panels/transients remain deliberate conventions. Retaining every
   existing Git key would require explicit resolution of provisional Slice 6;
   adding full transient behavior would require new approved scope.
3. **D03 Semantic classification:** all rows now have a disposition. Accept the
   retained Markdown/tree/native-panel conventions and explicit behavior gaps:
   heading M-S-RET, source-edit Save and the core rejected approximations.
   Native calendar, read-only columns and bounded Babel/export features retain
   their documented limits. Indirect Org/Markdown/process routes are recorded
   rather than treated as missing keys. These exclusions do not commission
   new runtime behavior.

The specification remains Draft until the owner accepts these decisions and
this exact proposal through the approved controller checkpoint. The contract
identifies each selected supersession of Org Familiar interaction and Git
Binding review matrix rows. Current defaults, legacy contracts and runtime
source are unchanged during the audit. No worker may infer approval from
complete coverage or successful parsing.

## ADDED Requirements

### Requirement: FSA01 Complete semantic classification

The review SHALL account for every native command and configurable input
action at its frozen baseline. It SHALL group aliases, variants and dispatch
branches by operation while retaining their native identities and contexts.
Each operation SHALL record its current direct or indirect keyboard route,
reference operation/family and version, behavioral evidence, proposed route,
conflict footprint and disposition.

Allowed dispositions SHALL distinguish keep, add, relocate, indirectly
reachable, named-only, adaptation required and unresolved. A command-name
comparison or a raw unbound count SHALL NOT substitute for this review.
No unresolved mapping may enter an approved implementation manifest.

#### Scenario: Equivalent behavior under a different native name

- **GIVEN** `modal-line-first-text` reaches the first non-whitespace grapheme
  without requiring modal state
- **WHEN** the review compares it with `back-to-indentation`
- **THEN** it evaluates line boundaries, blank lines, selection and arguments
- **AND** its different public name does not prevent consideration of `M-m`.

#### Scenario: Native and reference dispatchers have different names

- **GIVEN** a catalog helper has no factory entry under its own name
- **WHEN** an active Return or mode dispatcher reaches its operation
- **THEN** the review records that contextual route as existing access
- **AND** it does not propose an arbitrary additional key to fill the catalog.

### Requirement: FSA02 Match observable operation contracts

A proposed familiar binding SHALL match the supported operation's purpose,
direction, target, point/mark/selection effects and invocation context. The
review SHALL also check prefix arguments, prompts, cancellation, undo grouping
and source-versus-projection ownership where applicable.

An analogous native workflow MAY retain its existing bounded feature set when
the difference is explicit, as in P04 and P06. A materially different default
effect SHALL require a separate adapter contract or an explicit retained
exception; a familiar chord alone SHALL NOT imply unimplemented behavior.

#### Scenario: Joining in the wrong direction

- **GIVEN** native joining joins the current line with its following line
- **WHEN** the review considers the reference's default `M-^` operation
- **THEN** it records the preceding-line direction difference
- **AND** it does not expose the native primitive as an equivalent binding.

### Requirement: FSA03 Preserve inherited defaults and explicit overrides

Accepted factory assignments SHALL remain compiled defaults. Omitted user
entries SHALL inherit the installed defaults. Existing explicit overrides
SHALL retain their current normalized replacement, context precedence and
profile ownership. Upgrading defaults SHALL NOT rewrite personal configuration
or copy the exhaustive factory map into startup configuration.

The generated startup file SHALL retain commented examples. The exhaustive
reference SHALL describe the complete current defaults. Existing configuration
validation and startup failure behavior SHALL remain unchanged; this capability
does not add an unbinding sentinel or change stored configuration formats.

#### Scenario: Upgrade with one custom assignment

- **GIVEN** an existing configuration explicitly assigns `M-m` in `keys.editor`
  and omits the other accepted additions
- **WHEN** a newer Rev version loads that file
- **THEN** the explicit assignment retains its meaning in its owning profile
- **AND** the omitted keys inherit their accepted defaults without file edits.

#### Scenario: A new installation inherits later defaults

- **GIVEN** startup creates the standard configuration with commented examples
- **WHEN** a later version adds an accepted default
- **THEN** the omitted setting inherits that default
- **AND** startup does not require a regenerated full binding map.

### Requirement: FSA04 Expose accepted additions through native owners

Accepted P01-P05 assignments SHALL invoke the existing native operations in
the contexts and profiles stated in the target table. Bound and named
invocation SHALL retain one canonical command identity and the same operation
owner. Adding a key SHALL NOT silently change an existing command's semantics.

#### Scenario: Reach first text on varied logical lines

- **GIVEN** lines with spaces, tabs, mixed indentation, no text, nonbreaking
  space, wide text or combining text
- **WHEN** `M-m` runs in an admitted editing context
- **THEN** point reaches the first non-whitespace grapheme, or content end for a
  blank line
- **AND** text, dirty state and undo history remain unchanged.

#### Scenario: Balance nested editing panes

- **GIVEN** unequal native pane divisions and a selected editing pane
- **WHEN** `C-x +` runs
- **THEN** the existing balancing operation equalizes the divisions
- **AND** selected pane, buffers and startup split policy remain intact.

#### Scenario: Redo without replacing backward search

- **GIVEN** an undone native edit group in nonmodal editing
- **WHEN** `C-M-_` runs
- **THEN** the normal redo owner reapplies the next admitted redo group
- **AND** `C-r` continues to open backward incremental search.

#### Scenario: Switch a registered Project

- **GIVEN** registered Projects and current unsaved work
- **WHEN** `C-x p p` selects a Project
- **THEN** the existing Project restore workflow handles source decisions
- **AND** cancellation retains the originating work and context.

#### Scenario: Cycle the current heading's declared effort

- **GIVEN** an Org heading with admitted `EFFORT_ALL` choices
- **WHEN** `C-c C-x E` runs
- **THEN** the existing effort owner selects the next declared choice
- **AND** absent/invalid choices retain the existing diagnostic and edit rules.

### Requirement: FSA05 Resolve accepted family conflicts explicitly

Accepted P06-P08 and subsequent accepted target rows SHALL replace only their
specified factory routes. Displaced operations SHALL retain named invocation
and any unaffected aliases. New arbitrary aliases SHALL NOT be added merely
to preserve the previous number of bindings. Explicit user assignments to old
routes SHALL continue to work under the existing configuration rules.

Unresolved Agenda/Git assignments SHALL be resolved in the specification before
their implementation scope is frozen. Approved legacy tables SHALL be
reconciled only for the selected replacement rows; their historical delivery
status and validation evidence SHALL remain intact.

#### Scenario: Property columns use the column-family chord

- **GIVEN** an Org document with properties and factory mappings
- **WHEN** `C-c C-x C-c` runs after P06 is accepted and delivered
- **THEN** it opens the native property-column view
- **AND** it does not insert a clocktable or alter source through that binding.

#### Scenario: Header arguments use the Babel insertion chord

- **GIVEN** an admitted Org source block
- **WHEN** either accepted P07 sequence runs
- **THEN** it enters the existing checked header-argument workflow
- **AND** factory `C-c C-v h` no longer edits header arguments.

#### Scenario: Agenda daily view retains its existing alias

- **GIVEN** the native Agenda owns focus and P08 is delivered
- **WHEN** the user invokes `d` or `v d`
- **THEN** the same native daily-view operation runs
- **AND** habit-graph date selection remains available by its native name.

### Requirement: FSA06 Preserve contextual editing routes

Shortcut selection SHALL respect actual point context and the current input
owner. Heading, plain-list, checkbox, table, source-block, agenda and ordinary
text branches SHALL be assessed separately. Partial equivalence SHALL remain
explicit rather than being reported as either complete parity or a name-based
conflict.

Existing Markdown Return and table/list conventions SHALL remain intact.
Org `M-S-RET` SHALL NOT be described as full TODO-heading parity merely because
its checkbox-list branch matches the reference. Supporting a missing heading
branch would require an explicitly added behavior requirement.

#### Scenario: Continue a Markdown list through Return

- **GIVEN** point is in an admitted Markdown list item
- **WHEN** `RET` or `M-RET` follows its existing continuation path
- **THEN** the list owner retains its current continuation/empty-item behavior
- **AND** plain text and table Return retain their own dispatch branches.

#### Scenario: Checkbox insertion is only a partial equivalent

- **GIVEN** `M-S-RET` inserts a checkbox on a plain-list item
- **WHEN** the same chord is assessed on an Org heading
- **THEN** the review records the missing TODO-heading branch separately
- **AND** no shortcut-only change is presented as implementing that branch.

### Requirement: FSA07 Preserve profiles, focus and recovery

Every accepted target SHALL name its nonmodal/modal states and local overlays.
`keys.editor` SHALL remain independent from `keys.vim_*` overrides. Native
modal operators, counts, selections and state transitions SHALL retain their
existing meanings unless an accepted target explicitly replaces that route.

`C-g` cancellation and modal `M-x` recovery SHALL remain protected. Prompts,
completion, trees, projections and terminal input owners SHALL keep their
existing admission and consent boundaries. A new binding SHALL NOT bypass
confirmation or cause its suffix keys to answer a pending decision.

#### Scenario: Preserve state-specific Control-O

- **GIVEN** modal Normal and Insert already assign `C-o` to different actions
- **WHEN** familiar nonmodal additions are installed
- **THEN** both state-specific assignments retain their existing behavior
- **AND** the change does not substitute a merely related open-line primitive.

#### Scenario: Reject a protected-key override

- **GIVEN** a candidate map replaces `C-g` with an unrelated command
- **WHEN** configuration validation runs
- **THEN** the candidate is rejected under the existing recovery contract
- **AND** no partially validated map becomes active.

### Requirement: FSA08 Verify normalized terminal delivery

Accepted sequences SHALL pass parsing, normalized alias/prefix validation and
actual terminal input checks. Keys that collapse to another operation in the
terminal protocol SHALL NOT be advertised as distinct factory controls.
Profile and mode overlays SHALL remain valid after normalized aliases merge.

#### Scenario: Do not turn Backspace into redo

- **GIVEN** Rev normalizes `C-?` to Backspace
- **WHEN** the reference redo family is considered
- **THEN** P03 uses the checked `C-M-_` route rather than `C-?`
- **AND** ordinary deletion and the terminal's existing Meta handling remain
  unchanged.

### Requirement: FSA09 Keep command discovery and guidance accurate

Accepted assignments SHALL flow through the existing effective-map owner used
by M-x and the modal selector's shortcut hints. Hints SHALL describe the
originating active context and explicit overrides, not a second factory table.
Indirect helper access SHALL be explained as a contextual workflow without
falsely annotating a helper as the direct target of a dispatcher chord.

Bindings/reference configuration and the relevant user-guide sections SHALL
describe the delivered defaults and deliberate exceptions consistently.
Existing command names and source syntax SHALL remain stable.

#### Scenario: Discover an overridden shortcut

- **GIVEN** a user replaces an accepted factory chord in its owning map
- **WHEN** M-x is opened from that context
- **THEN** the existing hint owner reflects the effective reachable assignment
- **AND** documentation does not imply that a factory key overrides user choice.

### Requirement: FSA10 Deliver behavioral and coverage evidence

Delivery acceptance SHALL include a complete operation/context disposition
matrix, the exact accepted before/after mapping manifest, and evidence for all
changed families. Neither passing map compilation nor a documentation review
alone SHALL establish shortcut compatibility.

Verification SHALL cover meaningful dispatch effects, point/mark/selection,
prefix counts, undo/redo, read-only/rejected contexts, prompt cancellation,
overrides, normalized aliases, inherited maps and applicable modal states.
The existing repository gates SHALL remain unchanged. Specification checks
SHALL be reported separately from runtime acceptance.

#### Scenario: Detect a correct name with the wrong target

- **GIVEN** a proposed agenda binding resolves to the expected native command
  name but its handler targets the wrong source item
- **WHEN** behavioral acceptance runs from a selected projection row
- **THEN** the check fails despite successful map lookup
- **AND** the family cannot be reported as delivered until its source ownership
  and effects match the accepted contract.

## Implementation readiness and acceptance

The specification is ready for implementation planning when D01-D03 are
resolved, proposed targets are accepted or explicitly revised, and all mapped
exceptions have a recorded rationale. The final manifest must distinguish
direct additions, relocations, retained aliases, indirect access and named-only
operations. Discovery of a missing behavior must not silently expand a
shortcut-only delivery into a new editing feature or Git menu redesign.

Implementation acceptance requires:

- The complete native inventory is reconciled with the reviewed operation
  matrix; every family and context has an explicit disposition.
- Accepted P01-P08 and any additional accepted rows pass dispatch and effect
  checks in their declared states, including physical terminal input.
- Existing custom configuration bytes remain untouched; explicit overrides,
  failure recovery, source safety and named command access remain intact.
- Intentional tree/Markdown conventions and existing unaffected defaults are
  preserved; selected legacy tables and product guidance match delivered maps.
- Required repository checks and family-specific acceptance pass against the
  exact candidate, without weakening gates or presenting specification prose
  as test evidence.

## Inspection sources

- `src/command/`: canonical catalog, aliases and operation variants.
- `src/input/keymap/`: factories, context/profile ownership, availability,
  overlays, normalized sequences and recovery validation.
- `src/input/key.rs`: terminal aliases, including Control-Question/Backspace.
- `src/config/template.rs`: startup examples and exhaustive reference assets.
- `src/coordinates/navigation.rs` and `src/session/navigation.rs`: first-text
  positioning and shared dispatch outside modal state.
- `src/session/windows.rs`: pane balancing and focus restrictions.
- `src/session/contexts/projects.rs`: registered Project selection/restoration.
- `src/session/outline.rs`, `src/session/tables.rs`,
  `src/org/list/checkbox/structural.rs`: Org contextual routing.
- `src/session/markdown.rs`: Markdown Return/list/table dispatch.
- Installed stock/package maps and function behavior for the reference
  versions recorded above; personal bindings are not a factory reference.
