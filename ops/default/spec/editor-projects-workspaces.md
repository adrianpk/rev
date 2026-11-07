# Files, Directories, Projects and Workspaces

Status: Completed
Completed: 2026-11-07
Validated candidate: `6bbc6138ac9227beca51e6b86f759bbf39e36c94`
Gate receipt: `ZS6AIYOX66WRNHE5ICPVA46GIP`
Approved: product owner, 2026-11-07; core specification/model and the established preparation, plan, slicing, push and single Looper delivery through dev
Date: 2026-11-06
Updated: 2026-11-07
Presentation refinement: owner-approved shared buffer background and wider logo/menu spacing, 2026-11-07; original delivery evidence remains historical
Format: OpenSpec capability addition with explicit baseline reconciliation
Authority: approved core delivery; the linked plan/tracker govern execution; P19 remains a separate future specification and set
Source: owner-supplied `rev-projects-workspaces-spec.md`, 482 lines; SHA-256 `a84ef8a7f8a3f112caee98deb817cc5e164923cfd836e77f3e8f101b1e28d589`
Inspection baseline: `fabf9d4370fd52bf1c93709bfd645ee2264cd7e6`
Model: [Project and Workspace Model](editor-projects-workspaces-model.md)
Plan: [Projects and Workspaces Delivery](../plan/rev-projects-workspaces.md)
Tracker: [Projects and Workspaces Tracker](../tracker/rev-projects-workspaces.md)
Artwork: [Compact startup banner](../../../src/session/startup/rev-logo-small.ans)
Original artwork: [Preserved full-size banner](../../../src/session/startup/rev-logo.ans)
Baselines: [File navigation](editor-file-navigation.md), NAV01-NAV06; [Interaction](editor-interaction.md); [Configuration](editor-configuration.md)
Related contracts: [Pi integration](editor-ai-pi-integration.md), PI03/PI08; [Language servers](../../../docs/reference/language-servers.md); [Integrated terminals](../../../docs/reference/integrated-terminal.md)

## Purpose

Make opening a file, browsing a directory and returning to named editing work
ordinary parts of rev. A Project remembers one directory and its editing session.
A Workspace remembers an ordered collection of directory roots and its own
session. Both use the existing command selector, file picker, buffers and tree.

Temporary work remains temporary. Registering a Project or Workspace is an
explicit choice. Neither Git nor a language manifest is required. The feature
provides editing-context management rather than a tool-execution framework.

## Authority and evidence

The owner's reference distinguishes eighteen confirmed product requirements
from proposed defaults. This draft preserves that distinction: source-confirmed
means classified that way by the supplied reference, not implementation approval
or evidence that the repository already provides the behavior.

The source-confirmed direction covers direct directory opening, a native
dashboard, standalone files, temporary directories, named one-root Projects,
restorable buffers/positions, application-owned state, multi-root Workspaces,
promotion versus duplication, lifecycle commands and explicit membership.
Agent context aggregation is outside the feature.

The owner's additional startup-screen requirement includes a rev wordmark made
from Unicode text art. The supplied `rev-logo-small.ans` is the active artwork: UTF-8,
58 columns by 8 rows, 12,975 bytes, using U+2580 upper-half blocks and RGB
foreground/background SGR sequences on black. Its SHA-256 is
`4922a0ce647b3a52976d80266ec12b4ac3f58687bbf48716123a617f57c5b567`.
The full-size `rev-logo.ans` remains alongside it, unchanged.
The source contains color/reset sequences, not cursor movement or terminal
commands. The owner requested that its surrounding background fit the artwork.

Comparative review supports explicit identity, directory-first operation,
separate promotion, ordered composition, checked transitions and clear errors.
This proposal intentionally uses application-owned state and independent
directory-root membership. It does not require local Project metadata,
registered-only Workspace members, trust promotion or an all-roots-available
activation rule inherited from another editor.

The owner approved the complete core baseline P01-P18 on 2026-11-07. The register
retains the origin of each decision. P19 records the agreed future parameter;
its implementation remains deferred.

| ID | Decision | Origin |
| --- | --- | --- |
| P01 | `rev` shows the dashboard by default; reserve optional last-session restoration for the later P19 specification | Owner confirmed dashboard-first startup and deferred full startup restoration on 2026-11-07 |
| P02 | An explicit CLI target bypasses the dashboard | Supplied defaults |
| P03 | Opening a directory uses ordinary file navigation without registering a Project or replacing its saved session; Project selection explicitly resumes that session | Owner accepted this initial behavior on 2026-11-07, provided it stays simple; assess its value after use |
| P04 | Workspace members own copied paths and optional Project provenance, not authoritative Project references | Supplied defaults |
| P05 | Promotion/duplication allocate a fresh Workspace ID | Supplied defaults |
| P06 | Names are trimmed, Unicode-preserving and case-sensitive; uniqueness is per artifact type | Supplied defaults |
| P07 | Project and Workspace sessions remain independent after composition/duplication | Supplied defaults |
| P08 | A Workspace may be empty, including after removing its last root | Supplied defaults |
| P09 | Existing save/discard/cancel decisions protect dirty and unnamed buffers; no new unsaved-content recovery | Supplied defaults |
| P10 | Forget removes editor metadata and saved session only | Supplied defaults |
| P11 | Explicit relocation preserves IDs and never moves source directories | Supplied defaults |
| P12 | Restore buffer order, active file, cursor and source-anchored viewport; defer split-layout restoration | Supplied defaults |
| P13 | Stale concurrent writers report conflicts instead of replacing newer state | Supplied defaults |
| P14 | Use one versioned aggregate `contexts.toml` in the existing state root, with existing TOML, locking and atomic-publication facilities | Engineering baseline approved on 2026-11-07; no additional dependency |
| P15 | Apply the model's finite catalog, root, session, byte and staged-work profile; validate ordinary and boundary costs in delivery | Engineering baseline approved on 2026-11-07 |
| P16 | Reuse existing public command names and add only distinct context operations | Repository reconciliation |
| P17 | Resolve overlapping root ownership by the most specific containing root, then stored order | Review proposal |
| P18 | Center the supplied banner/action group on the configured buffer background, adapting its black-matted edges; prefer three blank rows before the menu while preserving actions on small terminals | Owner-approved presentation refinement on 2026-11-07; source artwork retained |
| P19 | Reserve TOML `startup.restore_session`, default `false`, for a later Startup Session Restoration specification and delivery set | Name/default approved; state capture, restoration behavior and implementation explicitly deferred on 2026-11-07 |

The owner confirmed dashboard-first startup for this delivery. The core scope
is file/directory contexts, named Projects/Workspaces with their basic file
sessions, and the startup screen. Complete automatic restoration is a later
capability with an agreed setting name, not a prerequisite for this core.

### Deferred specification: Startup Session Restoration

P19 records the approved future setting `startup.restore_session`, a startup-only
TOML boolean with default `false`. Its intended opt-in behavior is to reopen the
last closed editing context, including standalone files, directories, Projects
and Workspaces. The planned specification is `editor-startup-session-restoration.md`;
it has not been authored yet and is not a dependency of this delivery set.

That future specification will define capture timing, exact saved state,
pane layout, tree visibility/expansion, restoration failures, concurrency,
storage and bounds. It will also own activation of the setting in runtime and
the configuration catalog. This core reserves its name and default without
introducing an inactive setting or a partial automatic-restoration mechanism.
Basic Project/Workspace file sessions defined by PW15/PW16 remain in this core.

### Approved core baseline

The supplied reference explicitly treats its proposed defaults as concrete
working decisions. P02/P04-P13/P17 retain that provenance and were approved as
part of the complete core on 2026-11-07. P03 is accepted
for the initial delivery: ordinary directory opening does not save over a
registered Project's session. Reuse the existing directory-opening path and
introduce no extra setting or persistence layer for this distinction.

P14/P15 have an approved storage and capacity contract in the model. Names,
atomic publication, same-artifact conflicts, partial-root restoration and source
decisions form one implementation baseline.

## Current behavior and required reconciliation

| Inspected owner | Current behavior | Proposed change or preservation |
| --- | --- | --- |
| [CLI](../../../src/main.rs) and [reference](../../../docs/reference/cli.md) | Zero or one path; no options; no argument opens scratch; absent file becomes a visiting buffer | Preserve path parsing/creation; replace only no-target startup with P01 dashboard |
| [Tree root](../../../src/tree/root.rs) and NAV02 | One explicit directory root survives file visits; otherwise file-parent/cwd policy applies | Add explicit context identity and multi-root ownership; preserve explicit root intent within temporary contexts |
| [Find-file](../../../docs/reference/prompts.md) and NAV01/NAV04 | One path prompt admits files and directories; Tab completes without accepting | Retain that prompt and distinguish a file visit from an accepted directory/context replacement |
| [Exit coordinator](../../../src/session/quit.rs) | Existing save/discard/cancel decisions protect modified buffers | Reuse decisions for context replacement; do not route switching through process exit |
| [Configuration roots](../../../docs/reference/defaults.md#startup-configuration) | Startup config, data, state and cache have independent XDG roots | Store context state in the admitted state root; do not require project-local configuration |
| [Commands](../../../docs/reference/commands.md) | Native M-x and modal colon selector share named operations | Register new operations there; retain bindings, ranking, hints and canonical identities |
| [View](../../../src/view.rs) and [folding](../../../src/view/fold.rs) | Source coordinates and view-owned projections are distinct | Serialize source locations, not display offsets or temporary handles |

NAV01-NAV06 remain the navigation baseline. Approval must reconcile the new
standalone-file policy and explicit directory/context changes with NAV01/NAV02;
do not rewrite historical completed specifications during this draft. The
pending tree-selection proposal must eventually identify a selected root as
well as a selected file. Its delivery is not silently absorbed into this work.

Dashboard action order comes from the supplied written contract. The startup
presentation below uses rev's native palette and input behavior. The rev banner
is owner-supplied artwork, not a dependency on an external rendering tool or font.

## Product model

| Concept | Root and identity | Lifetime |
| --- | --- | --- |
| Standalone editing | File-backed and unnamed buffers; no registered root | Current context only |
| Temporary directory | Exactly one explicitly chosen directory; no durable artifact ID | Current context only |
| Project | One directory, stable Project ID and display name | Registration plus saved session |
| Workspace | Stable Workspace ID, ordered root-entry IDs and display name | Registration, membership and independent session |
| Dashboard | Navigable startup/selection view; may retain a return context | UI lifetime; not an editable document |

One instance edits one active context at a time. The dashboard may suspend its
input focus without discarding that context. Preparing a replacement does not
publish it. Global command usage, bookmarks and kill-ring behavior keep their
existing ownership; context changes do not delete them.

Project and Workspace here name durable editor artifacts. Existing rendering
uses of workspace and LSP protocol terminology do not acquire those identities
merely because their names are similar.

The model companion owns durable fields, identity, revisions, path references,
session coordinates, atomic conversion and capacity admission. Configuration
preferences and machine-local state remain separate concerns.

## ADDED Requirements

### Requirement: PW01 Distinguish context identity from presentation

The editor SHALL explicitly distinguish standalone editing, temporary directory,
Project and Workspace contexts. Tree visibility SHALL NOT determine context
kind. Opening a Project's path as a temporary directory SHALL NOT restore its
registered session under P03. Opening another standalone file SHALL NOT promote
the file's parent or discover a Project implicitly.
Ordinary buffer selection and context-wide save/close operations SHALL address
the active context rather than expose buffers retained from unrelated work.

Opening a directory SHALL NOT create or promote a Project. Registration requires
an explicit Project creation/promotion operation, regardless of path matching.

#### Scenario: Same directory, different intent

- **GIVEN** a registered Project and an explicitly opened matching directory
- **WHEN** the user browses that temporary directory
- **THEN** no Project session is restored or overwritten
- **AND** explicit Project selection remains available.

#### Scenario: Tree visibility is not registration

- **GIVEN** a Project or Workspace with its tree hidden
- **WHEN** the user continues editing
- **THEN** its identity, membership and persistence semantics remain unchanged.

#### Scenario: Buffer navigation after switching

- **GIVEN** a completed switch from one persistent context to another
- **WHEN** `switch-to-buffer` or `list-buffers` opens
- **THEN** it lists the destination's live buffers, without leaking the previous
  context's documents or temporary result handles into that inventory.

### Requirement: PW02 Preserve ordinary CLI targets and add dashboard startup

Under P01/P02, `rev` SHALL show the dashboard by default and an explicit
file/directory SHALL open directly. Automatic reopening of the last closed
context belongs to the deferred Startup Session Restoration specification,
not to this requirement's implementation or acceptance gate.
Capture invocation cwd once; resolve relative paths and `.` from
it. Preserve one optional OS path argument, absent-file visiting behavior and
current CLI error/terminal-restoration rules. This feature SHALL NOT invent CLI
flags or interpret shell syntax.

#### Scenario: Open the invocation directory

- **GIVEN** the shell is in `notes`
- **WHEN** it launches `rev .`
- **THEN** a temporary directory context opens at that directory without registration.

#### Scenario: Open a direct file or directory

- **GIVEN** a valid relative or absolute file/directory argument
- **WHEN** rev starts
- **THEN** it enters the corresponding standalone/temporary context directly
- **AND** a missing file remains an unsaved visiting buffer until normal save.

#### Scenario: No target or invalid target

- **GIVEN** no argument, or an invalid/unreadable/unsupported target
- **WHEN** startup dispatch runs
- **THEN** no argument with default settings shows the dashboard, while an invalid target reports the
  existing CLI failure and leaves terminal state restored.

### Requirement: PW03 Provide a native dashboard with reversible selection

The dashboard SHALL offer, in order, New Buffer, Open…, Open Project and Open
Workspace. Open… SHALL use the existing path selector to accept either a file
or a directory. Select the first action initially. Reuse native selection,
keyboard/pointer admission, palette roles and cancellation in both input profiles.
It SHALL be navigable UI rather than editable text. Opening it over existing
work SHALL retain that work until a replacement succeeds.

The dashboard SHALL present the supplied `rev-logo-small.ans` above its actions as
built-in, static artwork. Preserve the source Unicode blocks, RGB foreground/background
colors and internal spacing. Adapt the black-matted foreground/background colors
to the configured canvas when painting, keeping saturated logo colors and the
original source asset intact. Interpret its admitted color/reset sequences as
native styled cells during asset preparation or loading; do not send raw file
escapes outside the renderer. This requires no general terminal emulation or
external conversion program at startup. The artwork SHALL be passive: keyboard
selection and pointer activation address actions, not banner rows or spacing.
No Nerd Fonts setting is required to display its ordinary Unicode blocks.

Under P18, give the startup content area the same configured background as text
buffers, including the area around the banner. Keep editing-buffer palette
preferences unchanged. Place the banner
and action list together as one compact centered group when the terminal fits
them. Keep action labels left-aligned within the group and use existing
foreground/selection roles. Measure and clip by terminal
cells and complete graphemes, not byte lengths. If the artwork cannot fit,
replace it with a plain `rev` label before sacrificing action rows. Do not wrap
or distort the artwork to force it into the viewport. With insufficient height
for all actions, keep the list navigable and the selected action visible.
Resize SHALL retain action identity and any retained return context.

Use the existing input conventions for moving between actions, accepting one,
opening a command selector and canceling a picker in each editing profile.
Prefer three blank rows between the banner and actions, reducing that gap on
short terminals to preserve the existing full-banner and action visibility bounds.
The screen SHALL remain usable with color disabled and pointer input disabled.
With `NO_COLOR`, omit the full artwork and use a plain `rev` label instead:
removing colors from the supplied half-block image would produce a solid
rectangle, not a readable wordmark. Use the ordinary color-disabled renderer
and selection rather than forcing the artwork's background.

#### Scenario: Select or cancel an action

- **GIVEN** the dashboard over an existing context
- **WHEN** the user cancels a picker or returns from the dashboard
- **THEN** the existing buffers, positions and identity remain available
- **AND** no registry entry has been created by browsing a picker.

#### Scenario: Empty Project or Workspace catalog

- **GIVEN** no entries of the selected artifact type
- **WHEN** its picker opens
- **THEN** it explains the empty state and exposes the corresponding creation command
- **AND** the four-action dashboard need not become a management screen.

#### Scenario: Banner is presentation rather than an editing target

- **GIVEN** the startup screen and the supplied rev text banner
- **WHEN** the user navigates its actions in either editing profile
- **THEN** navigation skips artwork and spacing and Enter invokes the selected
  action through the same operation as its named command
- **AND** the banner does not become a file, buffer, selectable command or stored
  session entry.

#### Scenario: Small terminal and resize

- **GIVEN** an action selected below the full banner
- **WHEN** the terminal becomes too narrow or short for that presentation
- **THEN** a plain `rev` label and an actionable list replace the full artwork
- **AND** the same action remains selected, keyboard navigation remains usable
  and restoring size restores the full artwork without changing context.

#### Scenario: Ordinary terminal without decorative fonts or colors

- **GIVEN** Nerd Fonts and pointer input are disabled and `NO_COLOR` is active
- **WHEN** the startup screen appears
- **THEN** its plain `rev` label remains readable and
  native selection distinguishes the current action without relying on color.

### Requirement: PW04 Keep standalone and directory workflows temporary

Dashboard Open… SHALL use ordinary `find-file` acceptance. File visits SHALL
retain the current context; from standalone editing they SHALL NOT add an
automatic tree or associated browsing root. Files outside an active context's
roots are loose buffers, not new roots.
Explicit directory acceptance SHALL enter a temporary directory context through
the checked replacement path. Leaving a temporary context SHALL NOT register it
or create a named context session.

An explicit tree request from standalone editing SHALL retain that context and
reuse the existing file-parent/cwd browsing policy. The tree's display root does
not become a Project or Workspace root. Hide/show does not register an artifact,
replace its saved session or change the active context kind.

#### Scenario: Browse without creating a Project

- **GIVEN** an unregistered directory
- **WHEN** it is opened, edited and later left
- **THEN** no Project/Workspace or project-local metadata is created.

#### Scenario: Show the tree while editing a standalone file

- **GIVEN** standalone editing with the tree hidden
- **WHEN** the user explicitly shows the tree
- **THEN** existing file-parent/cwd policy supplies its browsing root while the
  file buffers and standalone context remain intact
- **AND** no registration or named session is created or replaced.

#### Scenario: Open a loose file within existing work

- **GIVEN** an active Project or Workspace
- **WHEN** `find-file` opens a file outside its roots
- **THEN** the context remains active, the file becomes a loose buffer and the
  tree membership is unchanged.

### Requirement: PW05 Register or promote a one-root Project explicitly

Project creation SHALL choose an existing directory, propose its basename as
the name, validate identity/path/name and persist before reporting success.
Promotion of the active temporary directory SHALL retain live buffers, active
file, positions, viewports and dirty text. A duplicate canonical root SHALL
select/offer the existing Project without overwriting its saved session.

#### Scenario: Promote current work

- **GIVEN** a temporary directory with several open buffers
- **WHEN** promotion is accepted and publication succeeds
- **THEN** one Project is registered for that root and the live session remains intact.

#### Scenario: Cancel, conflict or failed publication

- **GIVEN** a proposed Project or a root already registered
- **WHEN** input is canceled, validation/publication fails or an existing record is found
- **THEN** the source context remains usable and no duplicate registration or
  silent session replacement is reported as successful.

### Requirement: PW06 Switch persistent contexts through a staged transition

Selection SHALL resolve artifacts by typed ID, not display label. Prepare the
destination, resolve affected dirty buffers using P09, save the source persistent
session and activate only after essential preparation succeeds. Reopening the
already active artifact SHALL be a no-op. Dashboard display alone SHALL NOT
close the source or checkpoint a different context over it.

#### Scenario: Return to a Project by name

- **GIVEN** a saved Project and another active context
- **WHEN** its name/path is selected through the Project picker
- **THEN** its root, ordered buffers, active file and positions are restored
- **AND** supplying its absolute path again is unnecessary.

#### Scenario: Cancel a dirty-context replacement

- **GIVEN** modified or unnamed buffers in the source
- **WHEN** the user cancels replacement
- **THEN** the source identity and live text remain available and the destination
  is not activated. Previously confirmed file saves are not undone.

#### Scenario: Select the already active artifact

- **GIVEN** an active Project or Workspace
- **WHEN** that same ID is opened again
- **THEN** rev retains live buffers and cursor positions instead of replaying saved state.

### Requirement: PW07 Rename and forget Projects without changing source files

Renaming SHALL preserve Project ID, root and session. Forgetting SHALL remove
only registration and saved state under P10. An active forgotten Project SHALL
continue as a temporary directory with its existing live buffers; an inactive
forget SHALL not affect current editing. Source files and independent Workspace
roots SHALL remain intact.

#### Scenario: Rename a Project

- **GIVEN** a selected Project
- **WHEN** a valid replacement name is committed
- **THEN** its labels change while its identity, root and session remain the same.

#### Scenario: Forget active or inactive registration

- **GIVEN** a Project that may also have supplied a Workspace root
- **WHEN** forgetting succeeds
- **THEN** only its registration/session are removed; active work is retained as
  temporary directory work and the Workspace directory remains usable.

### Requirement: PW08 Relocate metadata without moving directories

Explicit Project/root relocation SHALL validate an existing replacement path,
preserve the affected ID and remap in-root file references by relative path where
possible. Project relocation SHALL reject a root owned by another Project;
Workspace relocation SHALL reject duplicate membership. Never move source files
or silently retarget other artifacts. Preserve the prior record on failure.

#### Scenario: Relocate a missing root

- **GIVEN** a retained Project or Workspace-root identity whose directory moved
- **WHEN** the user selects a valid replacement
- **THEN** that identity is retained, resolvable relative files follow it and
  missing files are reported individually without filesystem moves.

### Requirement: PW09 Create an independently named Workspace

A Workspace SHALL own its ID, ordered directory-root entries and session.
Under P08 it MAY start empty and remain valid after its last root is removed.
It SHALL admit plain directories and directories selected from Projects without
requiring all roots to become Projects or replacing them with a common parent.

#### Scenario: Create from scratch

- **GIVEN** a valid available Workspace name
- **WHEN** creation succeeds
- **THEN** an empty named Workspace opens with an add-root command/hint.

#### Scenario: Restore several peer roots

- **GIVEN** ordered `backend` and `frontend` roots
- **WHEN** the Workspace opens
- **THEN** both appear as independent top-level tree nodes in stored order.

### Requirement: PW10 Add roots without consuming Projects or their sessions

Add-directory SHALL append a validated root without implicit registration.
Add-project SHALL use the Project picker and append its directory, optionally
recording provenance under P04. Neither operation SHALL import that Project's
buffers/session or mutate its registration. Root changes SHALL become durable
before successful membership publication.

#### Scenario: Add a registered Project

- **GIVEN** a Project with saved buffers and an active Workspace
- **WHEN** it is added
- **THEN** its directory appears as a root, its Project remains registered and
  the Workspace keeps its own session.

#### Scenario: Duplicate or overlapping paths

- **GIVEN** a Workspace root
- **WHEN** an equivalent normalized/symlink path is added
- **THEN** duplicate membership is rejected; distinct ancestor/descendant roots
  remain allowed and the same directory may belong to another Workspace.

### Requirement: PW11 Remove or update root membership without destructive side effects

Removing a root SHALL retain buffers still covered by another root. Buffers
which lose their only covering root SHALL use ordinary close/save/discard/cancel
before membership is committed. Cancel SHALL retain the root and affected buffers.
Removal SHALL not unregister a Project or delete files. Preserve stored root
order; insertion order suffices for initial delivery and interactive reorder is optional.

#### Scenario: Remove a root with dirty buffers

- **GIVEN** dirty buffers covered only by a selected root
- **WHEN** closing them is canceled
- **THEN** membership remains unchanged and their live contents remain available.

#### Scenario: Remove the last or an overlapping root

- **GIVEN** valid Workspace membership
- **WHEN** removal succeeds
- **THEN** covered buffers survive, affected uncovered buffers close and removing
  the last root leaves an empty valid Workspace under P08.

### Requirement: PW12 Convert a Project to a Workspace atomically

Promotion SHALL create one fresh Workspace under P05, copy the Project directory
and live/saved session with remapped root identities, and remove the standalone
Project registration as one logical durable operation. Active promotion SHALL
retain dirty live text without visibly reopening buffers. Failure or cancellation
SHALL leave the original Project valid, with no half-converted registry.

#### Scenario: Successful active conversion

- **GIVEN** an active Project with open buffers
- **WHEN** promotion commits
- **THEN** it disappears from Project selection, appears as a Workspace and
  retains the live editing session without copying source files.

#### Scenario: Interrupted conversion

- **GIVEN** an admitted conversion
- **WHEN** publication fails or the process stops before its commit point
- **THEN** restart observes the original complete state or the complete converted
  state, never both registrations or neither as an accidental partial result.

#### Scenario: Convert an inactive Project

- **GIVEN** another context is active and the selected Project is inactive
- **WHEN** conversion commits from that Project's saved session
- **THEN** its catalog entry becomes a Workspace while the active context's
  live buffers, positions and identity remain unchanged.

### Requirement: PW13 Duplicate a Project as a separate Workspace

Duplication SHALL retain the Project, create a fresh Workspace ID and initialize
its session from the active or saved source. Under P09 resolve dirty source
buffers and affected buffers of the currently active context before creating
or activating the destination; cancellation creates nothing.
Afterward their sessions SHALL be independent while source files remain shared.

#### Scenario: Independent duplicate

- **GIVEN** a Project with a saved session
- **WHEN** duplication succeeds
- **THEN** both artifacts remain selectable, their IDs/sessions differ and no
  source files are copied. Later saved edits use existing external-change handling.

#### Scenario: Cancel duplication with unsaved text

- **GIVEN** a dirty active Project
- **WHEN** the user cancels save/discard resolution
- **THEN** the Project remains active and no Workspace registration is created.

### Requirement: PW14 Rename and forget Workspaces independently

Renaming SHALL change only the display name. Forgetting an inactive Workspace
SHALL remove only its metadata/session. Forgetting an active Workspace SHALL
resolve dirty buffers and return to the dashboard after publication, without
choosing an arbitrary root as a replacement directory or deleting source files.

#### Scenario: Forget an active multi-root Workspace

- **GIVEN** an active Workspace and accepted source-buffer decisions
- **WHEN** forgetting commits
- **THEN** the dashboard appears, Workspace state is removed and every source
  directory and separately registered Project remains intact.

### Requirement: PW15 Capture restorable file sessions with explicit ownership

Under P12, persistent sessions SHALL contain ordered file-backed buffer
references, the active reference, logical cursor positions and supported
source-anchored viewports. Include loose files; exclude raw process handles,
credentials and unnamed unsaved text. Capture before leaving and on clean exit;
registry/membership mutations SHALL persist immediately. Debounced checkpoints
are optional and SHALL not promise every last movement after a crash.

#### Scenario: Save a mixed file session

- **GIVEN** in-root files and an explicitly opened loose file
- **WHEN** the persistent context is left successfully
- **THEN** its saved state preserves file order and positions using root-relative
  references or an absolute loose-file reference as appropriate.

#### Scenario: Temporary and transient state

- **GIVEN** scratch, terminal, result and ordinary file buffers
- **WHEN** a session snapshot is captured
- **THEN** only the admitted restorable file session is saved; temporary directories
  and standalone contexts do not acquire registered context sessions.

### Requirement: PW16 Restore safely despite changed files and unavailable roots

Restoration SHALL reopen admitted files, restore order and the available active
file, clamp logical positions and respect native Unicode coordinates. Missing
files SHALL produce diagnostics rather than empty replacement files. A missing
Project root SHALL retain registration and expose relocation/forget. A Workspace
SHALL admit available roots and preserve unavailable membership and saved entries.

#### Scenario: Changed or missing file

- **GIVEN** a saved file shrank, changed Unicode content or became unreadable
- **WHEN** restoration runs
- **THEN** positions are clamped safely, unavailable buffers are reported/skipped
  and other buffers restore without invalid offsets or new files.

#### Scenario: Partially unavailable Workspace

- **GIVEN** one root is unavailable
- **WHEN** the Workspace opens and is later checkpointed
- **THEN** available roots remain usable and unavailable membership/session entries
  survive rather than being erased by the partial restoration.

#### Scenario: All roots unavailable

- **GIVEN** every saved Workspace root is unavailable
- **WHEN** it is selected
- **THEN** its identity and management commands remain available with diagnostics
- **AND** rev does not substitute cwd, a parent directory or a new root.

### Requirement: PW17 Publish durable state without lost concurrent updates

State SHALL live in the admitted application state area, using versioned,
validated records and one defined logical commit point for conversions. Reject
stale revisions or obtain checked exclusive ownership; independent registrations
SHALL not be silently lost and competing session writes SHALL not claim success
while replacing newer state. Report publication separately from a later durability
warning. Keep unknown/corrupt records intact.

#### Scenario: Two instances update state

- **GIVEN** one instance publishes after another captured its expected revision
- **WHEN** the stale instance attempts a registry/session write
- **THEN** it rebases only independently safe operations or reports a conflict
- **AND** a newer session is never silently discarded.

#### Scenario: Storage fails

- **GIVEN** a creation, membership change or source-session save
- **WHEN** storage cannot publish the required record
- **THEN** rev reports the operation/cause, retains usable live work and does
  not report the requested transition as complete.

### Requirement: PW18 Validate names, paths and typed identity independently

Under P06, trim names, preserve Unicode, reject empty/conflicting names within
each type and allow the same name across types. Names SHALL not be storage
paths. Canonical existing paths SHALL detect duplicates and aliases; missing
stored paths SHALL remain representable. Respect OS case rules and native path
types rather than globally lowercasing or assuming UTF-8 filenames.

#### Scenario: Name and path disambiguation

- **GIVEN** Projects/Workspaces with similar names or root basenames
- **WHEN** a picker filters by name/path
- **THEN** labels disambiguate entries and selection resolves the stored ID,
  while a conflicting same-type name keeps validation input open.

#### Scenario: Filesystem availability changes

- **GIVEN** a directory validated moments before use
- **WHEN** access fails or its physical identity changes
- **THEN** the operation revalidates/reports the failure without trusting an
  old display label or overwriting another Project's root.

### Requirement: PW19 Preserve source work on cancellation and malformed state

Preparation SHALL remain separate from publication. Picker/name/path cancellation
SHALL not create partial records. Malformed or unsupported state SHALL report
its scope and preserve input rather than replacing it with an empty catalog.
Isolate a damaged session from otherwise valid registrations where feasible.
Previously confirmed ordinary saves retain their normal effects.

#### Scenario: Corrupt registry or session

- **GIVEN** an invalid catalog, unsupported schema or malformed saved session
- **WHEN** it is read
- **THEN** original bytes are preserved, the diagnostic identifies the affected
  record and usable unrelated editing is retained.

### Requirement: PW20 Reuse named commands and preserve existing service authority

Dashboard and selector actions SHALL reach the same checked domain operations.
New operations SHALL be callable through M-x and modal colon; existing bindings,
completion, usage identity and hints SHALL remain authoritative. Opening a
context SHALL not run tools, enable LSP, load local executable configuration or
grant AI access across its roots. Existing per-file/root consumers retain their
own checked authority, cancellation and process ownership.

#### Scenario: Equivalent entrypoints

- **GIVEN** the same destination selected through dashboard or command palette
- **WHEN** opening succeeds
- **THEN** both publish the same context behavior and persistence semantics.

#### Scenario: Transition while service work exists

- **GIVEN** pending source requests, terminal sessions or other owned jobs
- **WHEN** context replacement is prepared
- **THEN** existing close/cancel ownership rules settle affected work and stale
  completions cannot mutate the replacement context through recycled handles.

### Requirement: PW21 Render multi-root trees without duplicating canonical buffers

Workspace tree nodes SHALL use root-entry identity plus path, keep peer roots in
stored order and preserve each root's admitted expansion. Reuse native decorated/
minimal labels, Nerd Fonts preference, hit geometry and lazy discovery. A file
reached through overlapping roots/aliases SHALL have one live canonical buffer
in that context. Under P17 source membership uses the most specific root.

#### Scenario: Reach one file through two roots

- **GIVEN** overlapping roots expose the same physical file
- **WHEN** either route is activated
- **THEN** the same live buffer is reused while the two tree routes remain distinct.

#### Scenario: Large roots and narrow geometry

- **GIVEN** several large roots and a narrow/resized terminal
- **WHEN** the Workspace is shown or navigated
- **THEN** loading is selective, identities remain stable and drawing performs
  no recursive filesystem scan.

### Requirement: PW22 Bound preparation, storage and retained lifetimes

Before Ready, the model SHALL define finite admitted capacities for artifacts,
roots, session entries, encoded bytes and staged work. Restore/checkpoint work
SHALL yield or use existing owned background facilities, preserve responsive
input/rendering and allocate no unbounded catalog or directory snapshot.
Record overflow SHALL reject the mutation atomically without truncating the
previous snapshot. Release staged records and obsolete view/process references
on cancellation, replacement and shutdown.

#### Scenario: Capacity or superseded preparation

- **GIVEN** a candidate exceeds an admitted bound or a newer request replaces it
- **WHEN** admission/preparation finishes
- **THEN** no partial context or record is published, owned temporary work is
  released and current editing remains usable.

### Requirement: PW23 Document ordinary workflows and verify the combined contract

Delivery SHALL document simple startup/opening, temporary versus persistent
work, promotion versus duplication and session restoration through Diataxis.
Use short CLI examples and the existing User Guide navigation; project its
Markdown to Spore. Keep exact state schemas, command inventories, limits and
failure behavior in reference material. Verify the requirement matrix through
native domain/storage tests and representative real terminal journeys.

#### Scenario: Learn then look up detail

- **GIVEN** a reader opening a directory or returning to a Project
- **WHEN** they follow the guide
- **THEN** ordinary commands suffice without state-file management, and detailed
  persistence, conflict and recovery contracts remain linked separately.

## Command surface for review

These are proposed names, not claims about existing commands. `find-file`,
`switch-to-buffer`, `kill-buffer` and `save-buffers-kill-terminal` retain their
existing identities and bindings. Prefer familiar naming over the reference's
placeholder `file-open`/`buffer-new` aliases.

| Operation | Proposed public command |
| --- | --- |
| Show dashboard / leave context | `dashboard-open` / `context-close` |
| New standalone buffer from dashboard | `generate-new-buffer`, using the native buffer-creation operation |
| Visit file within context | Existing `find-file` |
| Open temporary directory | `open-directory`; directory acceptance in `find-file` uses the same operation |
| Create / promote current directory | `project-create` / `project-promote` |
| Select / list Projects | `project-switch-project` / `project-list` |
| Rename / forget / relocate Project | `project-rename` / `project-forget` / `project-set-root` |
| Convert / duplicate as Workspace | `project-promote-to-workspace` / `project-duplicate-as-workspace` |
| Create / select / list Workspaces | `workspace-create` / `workspace-open` / `workspace-list` |
| Rename / forget Workspace | `workspace-rename` / `workspace-forget` |
| Add directory / Project | `workspace-add-directory` / `workspace-add-project` |
| Remove / relocate root entry | `workspace-remove-root` / `workspace-set-root-path` |

An ineligible command reports its unmet context through existing conventions
without mutation. No separate modal Ex parser or new binding prefix is implied.

The dashboard uses ordinary list navigation and Enter. C-g returns to the
retained context, as does the existing modal cancellation gesture when applicable.
At initial startup, with no return context, cancellation leaves the dashboard
available; the normal quit command remains usable. Banner rows never receive
selection or consume an action identity.

## Source acceptance traceability

The supplied A01-A40 scenarios are retained through these requirements and their
scenarios. This map is coverage intent, not passing implementation evidence.

| Source acceptance | Requirement coverage |
| --- | --- |
| A01-A05 | PW02-PW04: startup dispatch, dashboard, direct targets and cancellation |
| A06 | PW01/PW04/PW15: temporary directory lifetime |
| A07-A08 | PW05/PW19: promotion preserves source or cancels intact |
| A09 | PW06/PW15/PW16: named restoration |
| A10-A12 | PW07: rename and inactive/active forget |
| A13-A14 | PW09/PW21: empty and ordered multi-root Workspaces |
| A15 | PW10/PW15: add Project directory without session import |
| A16-A17 | PW10/PW18/PW21: aliases, overlap and independent membership |
| A18-A19 | PW12/PW17: atomic conversion and failure |
| A20-A21 | PW13: duplication and dirty cancellation |
| A22-A24 | PW11: safe removal, cancellation and empty remainder |
| A25 | PW07/PW10: source Project removal leaves copied membership usable |
| A26 | PW14: active Workspace forget |
| A27-A28 | PW16: Unicode/clamping and missing files |
| A29-A31 | PW08/PW16: missing Project roots and partial/all-unavailable Workspaces |
| A32 | PW21: canonical buffer reuse |
| A33 | PW01/PW04: explicit temporary opening of registered path |
| A34 | PW06/PW19: source-preserving cancellation |
| A35-A36 | PW17/PW19: stale writers and preserved corrupt input |
| A37 | PW03/PW20: common dashboard/selector operations |
| A38-A39 | PW05/PW08/PW17: relocation and failed registration publication |
| A40 | PW06: same-ID opening is a no-op |

Additional acceptance covers both input profiles, tree presentation modes,
NO_COLOR, startup banner/action geometry, unavailable metadata without a provider, independent view state,
owned child/request retirement, storage limits and both sides of the conversion
commit point. Use real temporary storage and fault injection at owned boundaries;
avoid repeating the whole application journey for every policy variant.

## Delivery gate

The core specification and model are approved together. The model defines source
coordinates, one atomic TOML catalog and finite bounds; this specification defines
standalone tree requests and dashboard cancellation. P19 is deferred and is not
part of core implementation, slicing or validation.
The supplied banner's geometry, source RGB colors and palette-matched presentation
are part of PW03 acceptance. Its asset does not require a different context
model or an additional delivery set.

Reconcile NAV01/NAV02, startup/configuration/reference wording and relevant root
consumers without broadening AI/LSP/tool authority. Execute the combined approved
scope, including startup presentation, through one ordered delivery plan/tracker
and one retained Looper run. Gate and closure evidence remain delivery outputs;
specification approval does not claim implementation or validation success.


## Delivery-set closure

The approved core is Completed through eight canonical slices (#298-#305).
The controller passed the unchanged combined gate at `6bbc6138ac9227beca51e6b86f759bbf39e36c94`,
receipt `ZS6AIYOX66WRNHE5ICPVA46GIP`, on 2026-11-07. [Slice 8 closure](../report/slices/rev-projects-workspaces/slice-8-guidance.md#delivery-set-closure)
records the exact command, 1,036 selected Rust tests, native/PTY/resource
measurements, local Markdown/Spore validation, cleanup and limits of the evidence.
The specification's PW01-PW23 and the model's fixed capacity/publication contracts
remain unchanged. Completed refers to this core; P19 full startup-state restoration
remains deferred to its separate specification and delivery.
