# Project and Workspace Model

Status: Completed
Completed: 2026-11-07
Validated candidate: `6bbc6138ac9227beca51e6b86f759bbf39e36c94`
Gate receipt: `ZS6AIYOX66WRNHE5ICPVA46GIP`
Approved: product owner, 2026-11-07; core model together with its behavioral specification
Date: 2026-11-06
Updated: 2026-11-07
Specification: [Files, Directories, Projects and Workspaces](editor-projects-workspaces.md)
Plan: [Projects and Workspaces Delivery](../plan/rev-projects-workspaces.md)
Tracker: [Projects and Workspaces Tracker](../tracker/rev-projects-workspaces.md)
Authority: approved core persistence contract; P19 startup restoration remains deferred
Source identity: owner reference SHA-256 `a84ef8a7f8a3f112caee98deb817cc5e164923cfd836e77f3e8f101b1e28d589`

## Ownership and invariants

1. A Project owns exactly one root and one independent session reference.
2. A Workspace owns zero or more ordered root entries under P08 and its own
   independent session reference. Membership does not transfer Project authority.
3. Project, Workspace and root-entry identities are distinct typed values.
   Names and paths are mutable metadata, not primary keys or record filenames.
4. One live context owns its admitted buffers/tree/views. A staged replacement
   cannot mutate it before publication. The dashboard may retain one return
   context without becoming a persistent artifact.
5. Filesystem aliases can share physical identity while tree routes remain
   distinct. A session does not create duplicate live buffers for one file.
6. Publication is guarded by expected revisions and one defined commit point.
   An operation that consumes a Project and creates a Workspace is atomic at
   that point, including recovery after process interruption.
7. Source directories and file bytes are never registry-owned payloads.
   Forgetting, converting and relocating metadata do not delete or move them.

## Logical records

These fields specify the information and invariants that must survive storage.
P14 defines a versioned aggregate TOML representation using existing
dependencies. The writer/reader must represent this model losslessly; a
file-per-record design alone would not establish atomic multi-record conversion.

### Registry

| Field | Meaning and constraints |
| --- | --- |
| `schema_version` | Supported positive format version; unknown versions are preserved and rejected for mutation |
| `revision` | Checked monotonically advancing publication revision; overflow rejects further mutation |
| `projects` | Bounded Project records indexed by typed Project ID |
| `workspaces` | Bounded Workspace records indexed by typed Workspace ID |
| `sessions` | One selected snapshot per registered owner, addressed by the owning record's `session_ref` |

Lookup indexes by normalized name and existing physical Project-root identity
are derived and rebuildable. A changed label never creates a new entity.
Catalog enumeration and empty pickers are read-only operations.

### Project record

| Field | Meaning and constraints |
| --- | --- |
| `id` | Stable opaque Project ID, distinct from Workspace IDs |
| `revision` | Expected Project-record revision for update/forget/relocation |
| `name` | Valid bounded display name under P06 |
| `roots_revision` | Revision of root/path membership; unchanged by a display-name update |
| `root` | One owned Root record; Project-root identity survives relocation |
| `session_ref` | Reference to the selected versioned session snapshot, or an admitted empty session |

Project uniqueness is by Project ID, normalized name and current canonical root.
Two separately registered Projects cannot own the same resolved directory.
Unavailable stored roots retain their last normalized path and ID; inability
to canonicalize does not make them new free identities or justify overwriting
their registrations.

### Workspace record

| Field | Meaning and constraints |
| --- | --- |
| `id` | Stable opaque Workspace ID; fresh on conversion or duplication under P05 |
| `revision` | Expected Workspace-record revision for membership/update/forget |
| `name` | Valid bounded display name, unique among Workspaces under P06 |
| `roots_revision` | Revision of ordered root/path membership, independent of display-name changes |
| `roots` | Bounded ordered Root records; empty is valid under P08 |
| `session_ref` | Workspace-owned snapshot, independent from source Projects |

There is no nested Workspace member type. The same physical directory can
appear in multiple Workspaces. Ancestor/descendant roots are allowed within one
Workspace, while equivalent physical roots are rejected as duplicates.

### Root record

| Field | Meaning and constraints |
| --- | --- |
| `id` | Stable Root ID scoped to its owning Project/Workspace |
| `path` | Lossless normalized absolute OS path; never a name-derived storage path |
| `label` | Bounded display label; default directory basename, disambiguated by path in UI |
| `origin_project_id` | Optional non-authoritative provenance for a Workspace root selected from a Project |

An origin ID is provenance, not a foreign-key dependency. A forgotten source
Project does not invalidate the root. Renaming/relocating the source Project
does not silently rewrite Workspace labels/paths. A stale provenance ID can be
retained as history or cleared by a checked Workspace update; it cannot grant
trust, configuration or process authority.

Availability is observed state, not a replacement root or a persistent deletion
marker. Preserve membership when a directory cannot currently be observed.

### Session snapshot

| Field | Meaning and constraints |
| --- | --- |
| `schema_version` | Supported session shape; may evolve independently of the registry |
| `owner` | Typed Project-or-Workspace ID; no temporary or standalone durable owner |
| `revision` | Snapshot publication revision, checked independently from labels |
| `roots_revision` | Membership/root revision against which relative references were captured |
| `files` | Bounded ordered File entries, one per canonical file at capture |
| `active_file` | Optional File reference resolving to an entry in this snapshot |

An empty Workspace may have no roots and still contain explicitly opened loose
file buffers. Empty membership does not require deleting those loose files or
converting them into root entries.

### Deferred startup-restoration model

The future setting name `startup.restore_session` and default `false` are agreed.
The [deferred Startup Session Restoration specification](editor-projects-workspaces.md#deferred-specification-startup-session-restoration)
will define its context checkpoint, capture timing, exact saved state, pane/tree
restoration, failure handling, concurrency, storage and capacities. No startup
checkpoint record or automatic-restoration implementation is required by this
core model, and those later decisions do not block its approval.

Project/Workspace file snapshots above remain part of this delivery. They are
restored when an artifact is explicitly selected and remain separate from the
future automatic startup route. Temporary files/directories do not acquire
registered identities or sessions through this core.

### File entry and position

| Field | Meaning and constraints |
| --- | --- |
| `file` | `RootRelative(root_id, path)` or `LooseAbsolute(path)` |
| `cursor` | Logical source position; 1-based line and 1-based Unicode-scalar column |
| `viewport` | Optional logical source anchor plus admitted horizontal cell scroll; no rendered row number |

Use 1-based logical source lines and 1-based Unicode-scalar columns for saved
cursor and viewport anchors. Convert through existing checked native coordinate APIs.
Tabs, combining characters, wide glyphs, wrapping, gutters and Org virtual
indentation do not change the meaning of a source position. Clamp a position
when its source shrinks; never use a column as an unchecked UTF-8 byte offset.

`RootRelative` paths are relative to the identified root and cannot escape it
through absolute components or unchecked parent traversal. Physical aliases
must still resolve through native file identity. Use P17's most-specific
component-wise containing root to choose one stored reference for overlap;
stored order breaks remaining ties. `LooseAbsolute` records intentionally
opened files outside roots, not automatic membership.

The snapshot contains no raw buffer/pane/process handles, document text,
undo/redo entries, selections, kill-ring content, credentials or AI conversation.
P12 defers split layout and per-view fold restoration. Terminal/result buffers
retain native live lifetimes and are not serialized as file buffers.

## Names and paths

Names are display strings. P06 proposes Unicode-preserving trimming followed
by exact case-sensitive comparison within each artifact type. Do not convert
names into filenames or invent filesystem-name restrictions. Unicode
normalization beyond trimming remains an explicit schema decision if needed;
do not silently add case folding or merge distinct names.

Paths use native OS path types in memory. A selected wire representation must
round-trip non-UTF-8 paths on supported systems without lossy substitution;
tagged native bytes or another documented reversible encoding is admissible.
Foreign-platform paths can remain stored while being reported unavailable.
The P14 layout below uses the existing bookmark path codec for lossless
UTF-8 strings or native-byte arrays.

Canonicalization verifies currently existing objects and duplicate roots;
retained normalized absolute paths represent unavailable objects. Do not treat
a failed observation as proof that a previously stored path is unregistered.
Recheck identity at use/publication boundaries because directories can change
between validation and access. Display labels never select filesystem objects.

## Context and transition lifetime

| State | Owned information and allowed effects |
| --- | --- |
| Active standalone | Live file/unnamed buffers; no durable context session |
| Active temporary directory | One checked root and live session; no registration |
| Active Project | Project ID/revisions, one root, selected saved snapshot and live session |
| Active Workspace | Workspace ID/revisions, ordered roots, snapshot and live session |
| Dashboard | Action identity, list viewport, banner projection and optional return-context token; no stored entity |
| Prepared replacement | Destination descriptor, expected revisions, staged observations and owned cancellation token |

Dashboard selection is an action identity, not a row index into banner text.
The built-in banner is presentation input rather than user state. Terminal size
determines its full or compact projection without changing the selected action
or return-context token. Artwork, cell measurements and banner row offsets are
not serialized into the registry or a Project/Workspace session.
The supplied asset is 100 by 13 cells with RGB foreground/background runs. Its
black startup content background is a surface presentation choice, not a mutation
of buffer palette preferences. Color-disabled presentation uses the plain `rev`
label rather than flattening the colored half-block artwork into monochrome.

Preparation stages are resolve, validate, source-decision, capture, durable
publication and live activation. An operation can return before activation if
validation or source decisions fail. Once publication occurred, report that
fact separately from a later directory-sync/durability warning; do not pretend
the previous record is still current or retry creation with a fresh ID blindly.

Opening the dashboard alone retains the current editing context. Choosing a
replacement and canceling its picker returns without losing that context.
An ordinary directory can have the same path as a registered Project without
activating that Project's session owner. Directory-mode navigation does not
checkpoint or replace that registered session. Reuse existing directory/file
operations for this initial distinction rather than adding another state store.
Closing a context or quitting resolves the affected dirty/unnamed buffers with
native decisions. Explicitly approved saves can already have modified disk;
canceling a later step does not roll those ordinary saves back.

A context generation/token rejects obsolete asynchronous results after
replacement. It is a live lifetime value, not a durable buffer ID. Settle or
retire affected terminal/job/request owners through existing close/cancel
contracts before freeing their source resources. Preserve global command usage,
bookmarks and kill-ring state independently.

## Mutations and commit points

| Operation | Required durable effect | Preserved values |
| --- | --- | --- |
| Register Project | Add one unique Project and initial snapshot reference | Directory contents; originating live text |
| Promote temporary directory | Register its exact root/session, then activate its ID | Buffers, positions, dirty state |
| Rename | Advance selected record revision and replace name | Artifact/root IDs, paths and snapshot |
| Relocate | Replace selected root path and checked relative references | Artifact/root IDs, name, source filesystem |
| Forget Project | Remove Project and its owned snapshot references | Source files, independent Workspace roots; live session becomes temporary if active |
| Create Workspace | Add one fresh Workspace with admitted empty session | All existing artifacts and source files |
| Add root | Append new unique root and advance membership revision | Existing root IDs/order and current session; source Project untouched |
| Remove root | Remove selected membership and explicitly resolved uncovered session entries | Source files, Projects, overlapping/loose buffers |
| Promote Project to Workspace | Atomically add Workspace/new root mapping/session and remove Project registration | Source files, live text, unrelated Workspaces |
| Duplicate Project as Workspace | Add Workspace/new root mapping/copied session only | Source Project identity and saved session |
| Forget Workspace | Remove Workspace and its owned snapshot references | All source files/Projects; inactive current context |

Conversion must remap source root IDs in every saved file/active reference.
Its commit includes the catalog change and the session reference needed to
interpret the new Workspace. Session objects prepared before publication are
not discoverable registrations; abandoned candidates must be reclaimable.

Initialize a new artifact's record/root revisions as supported positive values.
Advance `revision` for each metadata mutation. Advance `roots_revision` only
when ordered membership or root paths change; a display-name update does not
make an unchanged session stale. A root mutation publishes a matching session
reference with that roots revision, even if its remaining file entries are
otherwise unchanged. Reject revision overflow rather than resetting counters.

When removal leaves a file covered by another root, remap its RootRelative
reference through the remaining most-specific root and preserve its canonical
buffer, position and order. A file losing all coverage follows the specified
close decision; it is not silently converted into a loose file. Already loose
entries keep their absolute references. Preserve unavailable retained entries
unless their own membership was explicitly removed.

P14 selects one aggregate record: registration and its selected
snapshots are committed by the same atomic replacement of `contexts.toml`.
Do not implement unrelated replacements followed by best-effort deletion and
call that atomic conversion. Recovery must distinguish
not-published, published and unsupported/corrupt states without manufacturing
missing source records.

## Concurrency and publication

Use expected catalog, selected-record and session revisions to define conflict
scope. Independent additions can be rebased after re-reading current state if
uniqueness and resource admission still hold. A same-artifact session conflict
requires rejection or explicit user resolution; merging cursor state by a
timestamp does not establish user intent.

IDs are allocated before candidate publication and retained through an
ambiguous-result retry. Inspect whether that exact operation committed before
resubmitting; do not create duplicate Projects or Workspaces after a lost
acknowledgment. Revision counters use checked arithmetic and cannot wrap.

Writers use existing private staging and state-path rules where suitable.
Readers never execute metadata, follow record-provided executable hooks or use
an unvalidated relative state filename. Reject symlink/path substitution at
owned storage boundaries using existing filesystem primitives. Preserve
corrupt, unsupported and oversized input instead of overwriting it with empty
defaults. User-requested metadata removal is not permission to remove arbitrary
files selected by a malformed record.

## State placement and recovery

Use the admitted state root. On Linux this is `$XDG_STATE_HOME/rev`, with the
existing absolute-HOME fallback `$HOME/.local/state/rev`. Configuration remains
under its existing config root. Do not create `.rev/` or require local Project
records, and do not place machine-local sessions into source repositories.

Catalog reads and temporary directory/file use do not create registry files.
First explicit registration creates only the required owned state. Existing
bookmark/command-history records keep their current schema and locations.
### Physical layout

P14 uses `contexts.toml` in that state root, initially with `schema_version = 1`.
It contains `schema_version`,
`revision` and the `projects`, `workspaces` and `sessions` collections above.
Each registration references exactly one included snapshot by typed owner and
revision; all root/session references must validate before publication. A
conversion writes the complete new catalog, including its remapped session,
before one atomic replacement makes it visible. No database or transaction
service is introduced.

Reuse the existing command-usage store's nonblocking publication lock,
private staging and file/directory synchronization patterns. Use one owned
`contexts.toml.lock` file; hold no permanent instance-wide lock. Validate expected
owner/session revisions after rereading under the publication lock. Unrelated
updates may be applied to that fresh catalog; same-owner conflicts are reported.

Use the existing bookmark path codec: exactly one UTF-8 `path` or lossless
native `path_bytes` representation per stored path. TOML strings must use the
existing encoder's escaping rather than unchecked interpolation. Stored integer
revisions use checked positive values representable by the TOML format.

The aggregate scheme trades bounded whole-catalog writes for a single visible
commit point and no orphan snapshot collection. Store only one selected session
per artifact. Capture on accepted context departure/clean exit and metadata
mutations; no periodic autosave or per-keystroke catalog rewrite is needed.
The later startup-restoration specification remains independent of this layout.

Restore from the selected owner's snapshot. Preserve saved entries for an
unavailable root across checkpoints unless membership is explicitly removed.
For available roots, a failed file open can be diagnosed/skipped without
creating a new empty file. A missing Project root blocks its activation while
retaining registration. A Workspace with all roots unavailable remains a
manageable Workspace descriptor with diagnostics.

The initial version introduces no migration from an existing rev Project
registry because none was found at the inspected baseline. Future unsupported
versions need a separately defined migration path; this draft does not invent
cross-editor import or execute another editor's configuration.

## Queries and operation failures

| Query | Input and result contract |
| --- | --- |
| List Projects/Workspaces | Bounded immutable catalog descriptors with typed IDs, names, root summaries and captured revisions |
| Filter a picker | Existing candidate matching against admitted names and paths; display labels do not replace selected IDs |
| Resolve a selected artifact | Exact typed ID and expected revision, with copied root/session diagnostics |
| Find a duplicate Project root | Checked physical path identity plus retained unavailable-path observations; never string basename equality |
| Determine file membership | Component-wise containment and canonical identity; most-specific root and stored-order tie under P17 |
| Inspect a session | Selected owner, snapshot revision and file references, without opening files or executing services |

Refresh a stale picker snapshot before publication instead of mutating a new
record selected through an old row index. Availability checks occur in owned
preparation, not paint or every matching keystroke. Newly unavailable entries
remain identifiable for relocation/forget rather than silently selecting the
next row as if it were the original target.

Errors distinguish invalid input, name/root conflict, missing source,
unavailable root/file, stale revision, unsupported/corrupt state, capacity,
publication failure and post-publication durability warning. Return the
operation and affected identity/path safely through native diagnostics.
Cancellation is a separate outcome and does not report successful mutation.

## Resource admission

P15 uses the following approved initial engineering profile. These are state
and scheduled-work limits, not new configuration options. Required outcome on overflow is atomic
rejection with preserved previous state, not silent truncation.

| Dimension | Admission and evidence |
| --- | --- |
| Catalog entries | At most 256 Projects and 128 Workspaces; complete encoded input/output at most 8 MiB |
| Workspace roots | At most 64 members per Workspace; zero remains valid under P08 |
| Session files | At most 512 file entries and 2 MiB of encoded data per snapshot, including retained unavailable entries; total catalog cap still applies |
| Names and native paths | At most 256 UTF-8 bytes per display name and 4096 native bytes per path, matching the existing bookmark path bound; account separately for expanded TOML encoding |
| Staged snapshots | One active context, at most one prepared replacement and one owned state-publication job; superseded preparations are retired |
| Listing/restoration work | At most 16 file-admission steps per cooperative pass; use existing owned loading/IO facilities for expensive work and reject stale generation results |
| Storage ownership | One nonblocking lock attempt per requested publication; busy is reported with source work intact, not an unbounded wait or retry loop |

Counts and encoded-byte limits apply together; simultaneous per-record maxima
need not fit the aggregate cap. Do not truncate an existing session to make it
fit. Native file-loading/source-text limits remain owned by their current
contracts rather than being bypassed by session restoration.

For scale, 50 artifacts with 20 files each and roughly 128 path bytes plus 128
bytes of coordinates/format overhead per file produce about 250 KiB before
catalog/root overhead. This is a design estimate, not a measurement. At the
8 MiB ceiling, several input/candidate/output copies can cost tens of MiB before
decoded records; delivery must measure those retained copies and idle behavior.
Do not load source files merely to show the catalog. Exercise ordinary use and
boundary admission with the existing native/PTY/resource tooling during the set,
and record results before treating this profile as validated.

Charge copies and encoded payloads, not only element counts. Do not recursively
pre-scan every root to restore a small session. Do not checkpoint every key.
Debounced session checkpoints remain optional; adding them requires owned
coalescing and shutdown behavior, not a permanent busy poll.

## Model validation matrix

Verify typed-ID separation, record uniqueness, exact owner/revision linkage,
root-relative confinement, native path encoding, coordinates, aliases, overlap,
empty membership, deleted provenance, duplicate/conversion root remapping,
partially unavailable snapshots, stale writers and both sides of the commit
point. Check preservation of source bytes, current dirty text and unsupported
metadata through cancellation, failed writes and restart recovery.

Use real isolated storage and deterministic native cases for policy/format
boundaries. Reserve real TUI journeys for representative opening, switching,
restoration and conversion rather than multiplying all record variants through
the terminal. This matrix defines expected evidence, not passing test results.


## Delivery-set closure

The approved core is Completed through eight canonical slices (#298-#305).
The controller passed the unchanged combined gate at `6bbc6138ac9227beca51e6b86f759bbf39e36c94`,
receipt `ZS6AIYOX66WRNHE5ICPVA46GIP`, on 2026-11-07. [Slice 8 closure](../report/slices/rev-projects-workspaces/slice-8-guidance.md#delivery-set-closure)
records the exact command, 1,036 selected Rust tests, native/PTY/resource
measurements, local Markdown/Spore validation, cleanup and limits of the evidence.
The specification's PW01-PW23 and the model's fixed capacity/publication contracts
remain unchanged. Completed refers to this core; P19 full startup-state restoration
remains deferred to its separate specification and delivery.
