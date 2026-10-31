# Native Bookmarks

Status: Completed
Completed: 2026-10-31
Validated candidate: `41ba5f89465b2be02adc613cd796271169c245e0`
Gate validation: `7WOA4YIWYISUYIDTDT3ZC5SI5A`
Date: 2026-10-27
Reviewed: 2026-10-31
Format: OpenSpec capability proposal
Source: product-owner discussion; named persistent bookmarks confirmed on 2026-10-31 using Emacs as the behavioral reference
Approved: product owner, 2026-10-31; complete bookmark profile and explicit request to start its six-slice Looper delivery
Delivery boundary: one approved six-slice Looper delivery through dev
Gutter owner: [Editor Gutter](editor-gutter.md)
Model: [Native Bookmarks Model](editor-bookmarks-model.md)
Plan: [Bookmarks Plan](../plan/rev-bookmarks.md)
Tracker: [Bookmarks Tracker](../tracker/rev-bookmarks.md)
Readiness: BMK01-BMK07 delivered through six slices and three authorized repairs; controller-owned full gate passed

## Purpose

Return to named file locations across editing sessions and identify those
locations in the terminal gutter. Bookmark state belongs to a native Rust
owner, separate from selection marks, views and rendered cells.

## Confirmed direction and reference

The owner selected named bookmarks and persistence between sessions. Emacs is
the reference for supported interactions; differences require an explicit
policy rather than an assumption that Emacs does or does not support them.

The [GNU Emacs manual](https://www.gnu.org/software/emacs/manual/html_node/emacs/Bookmarks.html)
documents setting, jumping, listing, deleting and saving bookmarks, named
persistence, contextual relocation, and graphical fringe indicators. This
capability adopts those ordinary file-position workflows. It specifies its own
native storage and terminal presentation; it does not import executable
bookmark handlers, annotations or alternative bookmark-file workflows.

## Accepted concrete profile

The owner approved these concrete policies with the request to start the
six-slice Looper delivery on 2026-10-31.

- One user-wide collection, independent of the directory currently opened.
  Opening a bookmark does not replace an explicitly chosen tree root.
- Bookmark names are case-sensitive, nonempty UTF-8, at most 256 bytes, with no
  control characters. Do not silently trim, normalize or change identity.
  Names are unique across this collection. The visited basename is the initial
  prompt default; an entered name can distinguish locations in the same file.
- File-backed buffers, including modified and intended new files, are eligible.
  Reject unnamed and synthetic result buffers before mutation. Bookmarks never
  save source text implicitly. A reference to unsaved content may be unresolved
  after restart or discard; report that outcome rather than create the content.
- Load the collection lazily on its first use, including marker demand for a
  file-backed view. The native session prepares this outside frame painting.
  Save changed records on successful ordinary quit by default,
  with an explicit save command. Optional save-on-change and manual policies
  are available. No polling, directory scan or background daemon is required.
- Use `$XDG_DATA_HOME/rev/bookmarks.toml`, falling back through the existing XDG
  resolver to `~/.local/share/rev/bookmarks.toml`. These are user-created data,
  separate from configuration and disposable cache. Permit an explicit path.
- Keep a current live anchor while its buffer is open. Reopening uses stored
  context to check or relocate a position. Ambiguous or missing evidence is
  reported, not replaced with a guessed line number.
- Add next/previous navigation within the current file, without new default
  shortcuts. This is a rev convenience beyond the manual's core command set.
- Represent a resolved logical source line with one configurable gutter glyph,
  independently of number visibility. The graphical Emacs fringe is a behavior
  reference, not a terminal API or a claim of identical implementation.

## Commands and bindings

| Native command | Factory binding | Behavior |
| --- | --- | --- |
| `bookmark-set` | `C-x r m` | Prompt for a name; create or replace that name at point. |
| `bookmark-set-no-overwrite` | `C-x r M` | Same prompt; reject an existing name without changing it. |
| `bookmark-jump` | `C-x r b` | Complete names from the whole collection and visit the selected location. |
| `list-bookmarks` | `C-x r l` | Open the native bookmark list. |
| `bookmark-delete` | None | Complete a name and remove that record. |
| `bookmark-save` | None | Save the current collection explicitly. |
| `bookmark-reload` | None | Re-read the configured store, confirming before discarding unsaved bookmark changes. |
| `bookmark-next` | None | Visit the next resolved bookmark in the current file. |
| `bookmark-previous` | None | Visit the previous resolved bookmark in the current file. |
| `bookmark-markers-mode` | None | Toggle marker presentation for the current editing view. |

All commands participate in the compiled catalog and configured keymaps.
Prompts use existing vertical completion, explicit acceptance and cancellation.
The rectangle commands already under `C-x r` retain their bindings. No shortcut
hint display is included; that is an independent captured ticket.

## ADDED Requirements

### Requirement: BMK01 Canonical bookmark ownership

A native owner SHALL retain checked name, file identity and location evidence.
The gutter and list SHALL derive observations from that owner. Bookmark state
SHALL remain independent of transient editing marks and source bytes. Name
replacement SHALL be atomic; cancellation or failed admission SHALL retain the
previous collection.

#### Scenario: Replace a name without editing text

- **GIVEN** an existing named bookmark and an eligible current point
- **WHEN** the user accepts that name through `bookmark-set`
- **THEN** that record refers to the new point
- **AND** source bytes, modified status, selection and undo history are unchanged.

#### Scenario: Refuse accidental replacement

- **GIVEN** an existing bookmark name
- **WHEN** it is accepted through `bookmark-set-no-overwrite`
- **THEN** the editor reports the occupied name and retains the original record.

### Requirement: BMK02 Coherent edits and navigation

The location owner SHALL map live anchors through the canonical edit publication
path, including ordinary edits, compound replacements and retained structural
moves. Insertions exactly at an anchor use left affinity, matching the existing
native anchor boundary. A replacement that removes its source position SHALL
mark the target stale rather than silently treat the replacement text as its
original location. Deletion ending exactly at the point maps the boundary.

Undo/redo SHALL restore or reapply anchor outcomes while the corresponding
source transaction remains in history. A removed bookmark SHALL NOT reappear
because source undo ran; a bookmark created later SHALL NOT inherit another
record's saved history slot. Evicting text history SHALL NOT retain whole
source documents merely for bookmark recovery.

Jumping SHALL prefer an already open canonical buffer, including unsaved text.
Otherwise it SHALL prepare the file visit and resolve location evidence before
publishing the target view. Failures SHALL preserve the originating view and
point. Reveal folded Org ancestors and the wrapped target through the existing
navigation path, without changing fold or source identity globally.

#### Scenario: Edit and undo around a location

- **GIVEN** a live bookmark in one buffer shown in two panes
- **WHEN** text before it is inserted, removed and undone
- **THEN** both marker projections follow the same canonical anchor outcome
- **AND** source undo restores that bookmark's earlier valid location.

#### Scenario: Deleted or externally changed target

- **GIVEN** a bookmark whose original point was removed or whose file changed
- **WHEN** a jump cannot find unambiguous admissible location evidence
- **THEN** the editor reports a stale or unavailable target
- **AND** it retains the bookmark and the originating view without guessing.

### Requirement: BMK03 Derive current gutter markers

The gutter SHALL reserve one marker cell while its per-view marker policy is
on, whether or not number labels are shown. Use `gutter.side` for placement;
retain the existing number format and separator contract. A row with multiple
bookmarks SHALL show one glyph, while all names remain individually navigable.
Markers SHALL appear only on the first visual fragment of a resolved logical
source line, never on continuation rows, filler, hidden folded bodies or a
surrogate heading. A target inside a fold appears after navigation reveals it.

Marker lookup SHALL use a current buffer/revision index, not scan every stored
record for every terminal row. Paint SHALL perform no filesystem IO, relocation
or persistence. Hiding markers, numbering changes, scrolling, focus and resize
SHALL NOT change bookmark identity. Under narrow geometry preserve the existing
three-column text minimum, omit the marker before sacrificing available number
geometry, and restore the requested presentation when space returns.

#### Scenario: Independent numbers and markers

- **GIVEN** a resolved bookmark in a file-backed view
- **WHEN** the user disables line numbers or switches between absolute and relative
- **THEN** its enabled marker retains the same logical line identity
- **AND** disabling markers later does not remove the bookmark.

#### Scenario: Wrapped and folded source

- **GIVEN** a bookmarked line with wrapped continuations and a folded descendant
- **WHEN** the view is rendered, scrolled and resized
- **THEN** at most the visible first fragment gets the appropriate source marker
- **AND** hidden or unrelated lines do not acquire a marker.

### Requirement: BMK04 Provide native user controls

The command/binding table SHALL define the supported public surface. Named
prompts SHALL preserve ordinary completion and cancellation. Jump completion
SHALL use names, not list-row positions. Next/previous SHALL use source order
within the current file, wrap at the ends, and break same-position ties by
name. If the current point equals several targets, command repetition SHALL
visit each stable record before wrapping; point movement or edits reset that
iteration observation. No eligible target SHALL report a clear empty result.

`list-bookmarks` SHALL present a native read-only `*Bookmarks*` view with name,
file and checked status; names are sorted lexically. Opening the list SHALL
NOT open or read every referenced source file. Use arrows, `C-n`/`C-p`, `n`/`p`
for rows, Return to jump, `d` to mark a record for deletion, `x` to execute the
marked deletions, `u` to unmark and `q` to return. Cancellation and quit discard
unexecuted marks. Row actions SHALL carry stable identities and revalidate the
collection generation before publication. These local keys SHALL be exposed in
an appropriate native keymap context and the complete sample configuration.

#### Scenario: Navigate and delete through the list

- **GIVEN** multiple bookmarks, including two names at one source position
- **WHEN** the user navigates the list, marks one record and executes deletion
- **THEN** only the selected stable record is removed
- **AND** invoking a remaining record resolves that record rather than its old row.

### Requirement: BMK05 Named persistent bookmarks

Names and stored location evidence SHALL survive normal sessions. Persistence
SHALL use the versioned model below, bounded reads and checked atomic writes.
Default save timing SHALL be on exit after ordinary document save/discard
choices finish; explicit and on-change saves SHALL use the same owner. Saving
bookmarks SHALL NOT save source buffers. In manual mode quit SHALL NOT claim
unsaved bookmark changes were stored.

An invalid, unknown-version, oversized, unreadable or externally changed store
SHALL NOT be silently replaced with an empty collection. Retain the file and
in-memory changes with a diagnostic. New native in-memory operations MAY remain
usable; persistence stays blocked until a successful explicit retry can safely
reconcile the expected observation. `bookmark-reload` SHALL validate a fresh
disk candidate, then ask before replacing dirty in-memory records. Rejection
or cancellation retains both states. A quit-time save failure SHALL retain the
session and offer retry, explicit discard of pending bookmark changes, or cancel.

Concurrent writers SHALL serialize the compare-and-publish operation with a
bounded interprocess lock using the Rust standard library's file-lock facility.
Validate the last loaded/published disk observation while holding that lock.
Reject conflicting replacements rather than implement an implicit merge.
The lock serializes cooperating rev instances; it does not prevent arbitrary
external programs from modifying the store. Retain the existing file owner's
checked-observation and unresolved-publication contracts for those changes.

#### Scenario: Restart and return to a location

- **GIVEN** a named bookmark saved without changing the source file
- **WHEN** another rev session loads it and the user selects its name
- **THEN** it resolves and visits the recorded file and position
- **AND** the directory from which rev was started does not limit the collection.

#### Scenario: Preserve another instance's update

- **GIVEN** two sessions that loaded the same collection
- **WHEN** one saves a change and the other later attempts a conflicting save
- **THEN** the second detects the changed disk observation under its write lock
- **AND** it preserves both the newer disk data and its unsaved in-memory changes.

### Requirement: BMK06 Checked reopening and file lifecycle

After reopen, first check the saved scalar position against its bounded prefix
and suffix. If that fails, search the configured scalar window around the saved
point. Admit exactly one matching context boundary; reject multiple candidates,
missing evidence and incomplete searches. Report relocation as contextual
resolution, not proof of historical identity: duplicated text can limit it.
Support BOF, EOF and empty files with explicit boundary evidence.

A canonical file rename performed through rev SHALL update associated records.
Save-as SHALL follow the existing canonical buffer destination once publication
succeeds. External renames are unavailable targets unless the owner explicitly
sets the bookmark again; do not scan directories for them. Closing a buffer
retains its record but discards transient revision identity. If unsaved changes
are discarded, discard their live anchor and resolve the saved file afresh.
Failed save, close, rename or jump SHALL NOT publish a partial bookmark update.

#### Scenario: Context relocation after a small external edit

- **GIVEN** a stored bookmark with valid context and an edit before that context
- **WHEN** the reopened file has exactly one matching boundary in the allowed window
- **THEN** the jump uses that checked position
- **AND** ambiguous or exhausted matching leaves the current view intact.

### Requirement: BMK07 Bounded resources and complete configuration

Add the settings below through existing startup TOML, merge/validation, defaults,
reference and catalog owners. Existing user configuration remains valid. All
new keys and command bindings SHALL appear in the complete sample. The native
marker command supplies a per-view override; other settings load at startup.

| Setting | Default | Contract |
| --- | --- | --- |
| `bookmarks.file` | Unset | Resolve the user data path; explicit value must be an absolute path. |
| `bookmarks.save` | `"on-exit"` | `"on-exit"`, `"on-change"` or `"manual"`. |
| `bookmarks.markers` | `true` | Initial marker visibility for each editing view. |
| `bookmarks.marker` | `"●"` | One printable, non-control grapheme of terminal width one. |
| `bookmarks.max_entries` | `1024` | Integer 1..4096; no silent eviction. |
| `bookmarks.context_chars` | `64` | Integer 16..256 per side; scalar count, including boundary evidence. |
| `bookmarks.search_chars` | `262144` | Integer 0..1048576 total scalar search window; zero disables relocation. |
| `palette.gutter.bookmark` | `"#e0af68"` | Semantic marker foreground within the existing palette. |

Hard admission bounds: 256-byte name, 4096-byte encoded path and 16 MiB store.
No on-disk read is allowed to allocate beyond that store bound. Context search
SHALL be cancellable, process at most 16384 source scalars per event-loop pass,
and revalidate its source revision before applying a result. Input changes,
buffer close and cancellation invalidate pending results. No idle polling or
whole-document copy per keystroke is permitted. Record measured behavior for
1024 bookmarks, a large source, resize/reversal and idle operation.

#### Scenario: Resource limit or invalid configuration

- **GIVEN** a full collection, an invalid glyph or an over-limit serialized store
- **WHEN** creation, configuration admission or loading is attempted
- **THEN** the appropriate owner rejects it before partial publication
- **AND** existing records, source text and ordinary editing remain available.

## Dependencies and acceptance

Delivered prerequisites: canonical file/buffer ownership, prepared native edits,
retained structural mappings and undo, vertical prompts, configurable keymaps,
logical gutter geometry, soft wrap and native Org fold reveal. No new dependency
is required. Extend those owners; do not create a provider/plugin framework.

The six-slice plan covers every requirement. It includes isolated native/PTY
restart and failure journeys, source/dirty/undo preservation, alias/rename,
wrapped/folded locations, both gutter sides, disabled numbering, narrow panes,
store conflicts, limits and cancellation. The final gate belongs to one Looper
lifecycle; earlier delivered runs and unrelated pending tickets remain untouched.

## Approval and delivery state

The owner approved named persistence, the concrete profile, limits, terminal
adaptation and six-slice plan, and explicitly requested Looper startup.
The controller owns implementation, PR integration, the exact-candidate gate
and closure. The six canonical slice PRs #185-#190 and authorized repair PRs
#191-#193 are integrated. The unchanged full gate passed at the validated
candidate above; exact commands, receipt and native/resource/PTY evidence are
recorded in the tracker and Slice 6 report. The accepted behavior and limits
remain the delivered contract.
