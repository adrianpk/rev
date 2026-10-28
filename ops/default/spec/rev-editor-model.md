# rev Editor Model

Status: implemented
Implemented: 2026-10-27; completed rev-editor delivery
Validated candidate: `b56d705c7a84b823bf069fc874bc762938d6d897`
Input/navigation completed: 2026-10-28
Input/navigation validated candidate: `dedd85064cb5565a5382a9d8e2e2902daae24d88`
Input/navigation gate validation: `PZ4OIPXA6GUBSWDH5ZS7MJUOGX`
Approved: product owner, 2026-10-26
Date: 2026-10-26
Specification: [rev Editor](rev-editor.md)
Scope: native state, text persistence, and cross-component invariants

## Purpose

The validated candidate above belongs to the original delivery. The accepted
[configuration model](editor-configuration-model.md) supplies immutable startup
settings, effective buffer policies, context maps and semantic palette roles;
its separate delivery owns the configuration/interaction acceptance evidence.
The input/navigation fields above record the later complete gate for the
current view, pointer and terminal ownership amendments.

Define the state and persistence contracts needed by the paired specification.
Tables name semantic responsibilities. They do not mandate public Rust types,
a module per record, a custom rope, a database, or a multi-crate workspace.
The specification owns observable commands and acceptance. This companion
shares its accepted status and accepted clarifications D01-D07.

## Ownership

| Owner | State and authority |
| --- | --- |
| Application | Buffer/view registries, layout, focus, ring, rectangle, input, tree, and messages. |
| Buffer | Canonical rope, text revision, file association, mark, history, save point, and syntax state. |
| View | Buffer reference, point, desired display column, vertical/horizontal scroll, geometry, selection shape and independent gutter policy/overrides. |
| Layout | One recursive tree of editing views and below/right split nodes. |
| Terminal guard | Acquired raw mode, alternate screen, paste/cursor/color restoration and independently tracked SGR/button mouse-reporting obligations. |
| Command | One bounded operation coordinated through the owning state. |
| File operation | A captured text state and an honest publication/completion result. |
| Renderer | Temporary immutable observation of application state and mutable output access. |

Stable process-local identities distinguish buffers and views. Removing a
view does not remove its buffer. A closed buffer identity cannot silently
refer to a later allocation. Small typed IDs and ordinary owned collections
are sufficient; a reusable capability framework is not required.

## Session records

### Application

| Field | Meaning and invariant |
| --- | --- |
| `buffers` | Live identity-to-buffer collection and unique display labels. |
| `views` | Live identity-to-view collection; every buffer reference is valid. |
| `layout` | Editing-view tree; each live editing leaf occurs exactly once. |
| `selected` | A live editing view, including while another pane has focus. |
| `focus` | Editor, tree, minibuffer, or active transient interaction. |
| `ring` | Linear entries, current rotation, and kill/yank continuity. |
| `rectangle` | Optional last copied/killed block, separate from `ring`. |
| `input` | Pending chords, prompt/search state, Escape deadline and cancellation origin; ordered finite native input batches. |
| `pointer` | Session enabled override and optional captured source ViewId/qualified anchor, latest drag edge and one pending deadline. |
| `tree` | Root, loaded entries, expansion, selection, and visibility. |
| `echo` | Current user-facing status or error, without changing document text. |

After initialization, the session has at least one editing view and one buffer.
Killing the last buffer creates or selects a valid empty replacement. The tree
and internal lists do not acquire authority to mutate text through a second
storage path.

### Buffer

| Field | Meaning and invariant |
| --- | --- |
| `id` | Stable buffer identity. |
| `name` | Unique live label; equal basenames receive distinguishable labels. |
| `text` | The sole canonical editable rope. |
| `revision` | Freshness stamp for the current published text; distinct from history state. |
| `file` | Optional visited-path association and last disk observation. |
| `format` | Recognized UTF-8 BOM policy and line-ending insertion preference. |
| `readonly` | Text-mutation prohibition shared by bindings and named commands. |
| `mark` | Optional mapped position and region activation associated with the buffer. |
| `history` | Per-buffer undo/redo groups and retained logical text-state identities. |
| `saved` | Clean logical text state paired with the destination it actually represents. |
| `syntax` | Optional parser/tree state, recognized mode, and its source revision. |

A newly empty buffer may have an initial clean in-memory state without any
claim that a file exists. A new visiting path records that the destination is
absent. Inserting text makes that state modified. Opening an existing file
establishes a clean state only after successful validated ingress.

Dirty state is relative to the saved logical state and its relevant file
association, not merely `revision != saved_revision`. Undo/redo publishes fresh
revisions but can return to a retained clean history state. Saving moves the
save point only after actual success. A new destination with uncertain
publication cannot reuse an old destination's clean acknowledgment.

The initial implementation may conservatively regard a different edit-history
state as modified even if independent edits recreated equal text. It MUST NOT
scan the complete rope on each key merely to recompute dirty state. Returning
to a retained saved state through undo MUST restore the clean indication.

### View

| Field | Meaning and invariant |
| --- | --- |
| `id` | Stable view identity, distinct from buffer identity. |
| `buffer` | Live buffer reference. |
| `point` | Valid rope position for this view. |
| `column` | Optional intended display column for consecutive vertical motion. |
| `top` | Valid top logical line or mapped start position. |
| `left` | Horizontal display-cell offset. |
| `area` | Validated terminal rectangle assigned by layout. |
| `shape` | Linear or rectangular selection presentation for this interaction. |
| `gutter` | Immutable startup defaults plus explicit visibility/mode/current-line/side overrides; checked final text/number geometry is derived per frame. |

On split, the new view copies initial point/scroll values but then evolves
independently. Both views reference the same buffer. On any text publication,
every affected point, mark, and viewport reference must be remapped or safely
clamped before drawing. Focused selection uses the selected view's point with
the buffer's mark; other views do not become a second global cursor.

Gutter policy follows the [gutter model](editor-gutter-model.md): a split copies
policy/override values; buffer rebind retains policy and recomputes labels;
view close releases derived state. Labels never become text, history or one
retained record per source line. Status groups observe each pane's own point.
Pointer capture uses the canonical region and source view; it cannot transfer
across panes. Only a valid outside drag owns a deadline, at most one step per
100 ms and at most min(scroll_lines, text height), without catch-up. Ready
release/cancellation is processed before a due tick. All state is session-local.

### Layout

A leaf names one live editing view. An internal node names a below or right
division and two child subtrees. The distinction records the commands' screen
effects, avoiding ambiguous horizontal/vertical axis labels.

Layout calculation subtracts minibuffer, status, borders, and optional tree
areas with checked/saturating geometry. A proposed split must fit before the
tree changes. A resize may suspend presentation, recompute rectangles, or hide
the tree, but cannot delete rope contents. Zero dimensions are valid suspended
presentation inputs. No surface-size caps from `ruru` are inherited.

Deleting a leaf reconnects its sibling subtree. Deleting other windows retains
the selected leaf. Deterministic leaf traversal defines `other-window`; the
tree pane is outside that traversal.

## Text coordinates and edit publication

### Coordinate domains

| Domain | Representation | Rules |
| --- | --- | --- |
| Rope | Character/scalar index under the chosen rope API. | Bounds belong to one buffer and source revision. |
| Byte | UTF-8 offset or row/byte-column pair. | Explicit conversion for Tree-sitter and exact file data. |
| Grapheme | User-perceived cluster boundary within the source text. | Motion/deletion do not divide a supported cluster. |
| Display | Logical line and terminal-cell column. | Width includes tab stops, wide clusters, and combining text. |
| Surface | Row/column within the assigned viewport. | Clip to validated geometry after applying scroll. |

Ranges are half-open and ordered. A prepared text range carries or is checked
against buffer identity and revision. Invalid boundaries, foreign identity,
or stale revision reject before mutation or trigger a deliberate recomputation.
There is no implicit conversion through an unqualified integer.

A cached projection is valid only for its text, geometry, horizontal offset,
and display policy. It cannot turn old byte captures into current cell ranges.
Line endings, including CRLF, must remain coherent during line movement,
deletion, indexing, and serialization.

### One edit transaction

A transaction describes one logical mutation, including multi-line rectangles,
paste, a kill, a replacement decision, or one typing group contribution.

1. Resolve the command's target buffer and initiating view.
2. Validate read-only state, source stamp, ranges, and required input.
3. Prepare replacement text, reversible history information, dependent position
   mapping, and syntax invalidation before publishing a change.
4. Apply the complete prepared mutation to a private candidate or through an
   equivalent atomic rope operation.
5. Publish text, history, mapped positions, and invalidation coherently.
6. Return the outcome and render through ordinary application flow.

A recoverable validation/preparation failure leaves confirmed text and its
history unchanged. Rectangle row edits form one transaction. This does not
require nested transactions, callbacks, a generic observer bus, or a promise
to recover from process abort or allocator termination.

For multi-range edits, the implementation must state whether ranges use base
coordinates or sequential working coordinates, validate that choice, and
apply them consistently. Applying row replacements in the wrong order must
not make later offsets refer to already shifted text. Position movement uses
the actual accepted edits, not an unrelated coarse syntax-invalidation span.

## History and selection records

### History

| Field | Meaning and invariant |
| --- | --- |
| `state` | Identity of the current retained logical text state. |
| `undo` | Completed reversible transactions for this buffer. |
| `redo` | Undone transactions available under conventional redo. |
| `typing` | Continuity needed for sensible adjacent ordinary-insertion grouping. |
| `saved` | Reference to the clean state; loss of old history must not falsely mark current text clean. |

Each transaction retains enough inserted/removed text, affected ranges, and
selection information to restore a coherent state. Position restoration must
use a live initiating view where available or the invoking view; other views
remain independently positioned through mapping/clamping. A closed view is
not revived by undo.

A new edit after undo discards redo under the retained default. Save does not
discard history. Read-only movement, selection, search, and copy do not create
text-history mutations. Empty history navigation is a user message, not a panic.
Any retention limit selected during implementation must be documented and must
never leave a partially reversible transaction. This model does not select
`ruru`'s branching history or its resource-accounting system.

### Linear selection and kill continuity

Mark and activation belong to the buffer. Region calculation orders the mark
and selected view's point. Inactive mark may remain stored for later exchange.
Point and mark must follow edits at valid boundaries; their insertion-boundary
affinities must be chosen consistently and covered by focused tests.

The linear ring retains at most 32 entries under the default. Each entry is
owned copied/killed text. Consecutive kills retain direction so forward kills
append and backward kills prepend where appropriate. A non-kill command ends
that continuity.

Yank continuity records the buffer, initiating view, current text revision,
inserted range, and ring rotation. Yank-pop is permitted only while that
capture still describes the immediate yank sequence. Each successful pop
replaces exactly that range and refreshes the capture. An unrelated command,
target change, or incompatible mutation invalidates it before reuse.

### Rectangle

| Field | Meaning and invariant |
| --- | --- |
| `rows` | Ordered copied row fragments, without synthetic trailing source padding. |
| `width` | Requested display-column span, distinct from bytes and scalar counts. |
| `policy` | The same width/tab/grapheme policy used by drawing and motion. |

Row text is independent of the linear ring. Selection derives its line range
and display columns from point/mark. Under D04, a touched wide cluster
is included whole. Copy expands all selected tab cells into retained spaces;
kill retains complete selected tabs and materializes partial edges as spaces,
without altering untouched tabs. A short line contributes only its actual selected
text. Copy does not add absent cells to that source.

Yank inserts successive rows at the selected display column, pads short target
lines only as needed, and creates additional lines when insertion requires
them. It never divides a target grapheme. Its line creation, padding, and text
insertion are one undoable operation. Retained tabs use destination tab stops;
rows are not retabulated to force their source right edge. The policy must be consistent when a
wide cluster makes the copied fragment wider than the requested cell span.

## File association and persisted representation

### File association

| Field | Meaning and invariant |
| --- | --- |
| `path` | Intended or visited destination, independent of its display label. |
| `identity` | Normalized/canonical identity used for visited-file reuse where available. |
| `presence` | Existing, not yet created, or removed since association. |
| `observed` | Conservative metadata captured after successful open/save. |
| `warning` | External-change or unresolved-publication state requiring truthful reporting. |

An index may map normalized visited paths to buffer IDs. It must be updated
with successful association changes, buffer closure, and tree rename. Missing
paths can use normalized intended paths without pretending canonicalization
of a nonexistent file succeeded. Duplicate basename labels do not define file
identity.

Canonical visited-file reuse should treat aliases consistently. The later
plan must specify symlink save behavior and occupied-path collisions before
implementing publication; it must not silently replace a symlink entry when
the user intended to save its referent. A full race-free filesystem authority
or hard-link deduplication subsystem is not required by this baseline.

### Stored text

Files contain UTF-8 document bytes, not serialized buffer IDs, undo history,
window state, mark, kill ring, or parser data. There is no database, on-disk
session schema, or migration in the baseline.

| Concern | Disk-to-model and model-to-disk contract |
| --- | --- |
| UTF-8 | Validate before editable ingress; reject invalid data rather than replace it. |
| Binary detection | Use a documented conservative policy; rejection preserves the existing session. |
| LF/CRLF | Preserve existing separators in the canonical text and on output. |
| Inserted newline | Follow prevailing style; record a deterministic tie/new-file default during planning. |
| Final newline | Preserve presence/absence except when the user actually edits it. |
| BOM | If recognized separately from visible text, record and emit it exactly once; otherwise retain exact text consistently. |
| Permissions | Preserve appropriate existing permissions where practical and disclose unsupported guarantees. |

File input can stream or stage validation; it need not allocate one string per
line. File output should stream rope chunks or use an equivalent bounded path.
Temporary whole-file strings are not a canonical store and cannot become a
per-key serialization strategy.

### Save capture and result

| Field | Meaning and invariant |
| --- | --- |
| `buffer` | Source buffer identity. |
| `revision` | Exact captured publication revision. |
| `state` | Logical text/history state represented by the captured bytes. |
| `target` | Destination chosen by save or write-file. |
| `prior` | Last observation plus any required overwrite/external-change decision. |
| `publication` | Not published, published complete, or published with unresolved failure. |
| `error` | Primary failure plus relevant cleanup/durability context. |

Synchronous I/O is sufficient. A captured rope snapshot or equivalent stable
read establishes exactly which state is written. No background worker, Ruby
capability, or general asynchronous completion protocol is required.

Under the atomic-save default, the lifecycle is:

1. Capture the selected text state and destination.
2. Check occupied-target and external-change decisions before destructive work.
3. Create temporary output in the destination directory.
4. Write the captured bytes and complete required pre-replacement steps.
5. Replace the destination only after those steps succeed.
6. Complete required post-replacement checks and cleanup.
7. Reconcile the actual outcome into the buffer and user message.

| Outcome | Destination fact | Association and saved-state effect |
| --- | --- | --- |
| Not published | This attempt did not replace the destination. | Retain prior association/save point and dirty state; report failure or cancellation. |
| Published complete | Captured bytes were published and required completion steps succeeded. | Associate the target, update disk observation and its saved logical state; clear dirty only for that state. |
| Published with unresolved failure | Replacement occurred before a later required step failed or became uncertain. | Retain the actual target/publication fact and warning; do not falsely confirm success or clear the pending unsaved state. |

The result must preserve publication even if cleanup or observation fails
later. Required save-before-close/quit succeeds only on the complete outcome.
If filesystem durability is claimed, the chosen file and directory
synchronization must justify that claim. Unsupported guarantees must be
disclosed rather than inferred from a successful write call.

Save-as does not relocate an association before publication. Once replacement
actually happened, a warning must identify that destination even if completion
failed. A successful save cannot mark a later edited state clean merely because
it wrote an earlier captured revision.

### Tree reconciliation

The tree owns path entries and loaded child lists, not document contents.
Opening an entry delegates to the same file/buffer owner as `find-file`.
Creating or refreshing an entry updates tree observations without inventing
an editable buffer unless the operation explicitly opens one.

Successful rename updates affected visited paths and labels. A directory
rename must account for visiting buffers under that directory. A successful
deletion retains open text and marks a removed association under D07. An
attempted conflicting rename or deletion canceled by the user changes neither
the association nor disk content. Refresh reconciles the visible filesystem
without silently discarding modified buffers.

## Input and presentation records

| State | Retained information | Completion/cancellation rule |
| --- | --- | --- |
| Prefix | Pending chord sequence and its focused context. | Dispatch one complete command, or clear with no document mutation. |
| Command prompt | Query, ordered candidates, chosen item, command-name history. | `RET` invokes one real command; `C-g` cancels. |
| Argument prompt | Requesting command, argument stage, editable prompt text. | Continue predictably or cancel before unapproved side effects. |
| Search | Direction, query, starting view/point, current match. | `RET` accepts; `C-g` restores the starting point. |
| Replace | Literal query/replacement, next match, confirmed progress. | Only yes mutates; no skips; quit ends remaining replacement. |
| Tree focus | Selected entry and last editing view. | Open through common file machinery; return to valid editor focus. |

Escape disambiguation has a finite, documented policy and cannot block cleanup.
Bracketed paste bypasses command decoding and contributes one text transaction.
The selected terminal's distinguishable chords define the documented platform
limitations; they do not authorize silent rebinding of required Emacs commands.

Syntax state belongs to its buffer. It contains the chosen grammar, parser,
tree/query result, and the exact source revision. Text edits translate their
old/new byte offsets and row/byte columns before incremental parse. A failed
query or parse retains accepted text and switches to safe styling. Derived
highlight ranges, cells, and viewport caches may be rebuilt or discarded;
canonical rope, history, and file association may not.

## Lifecycle consistency

| Operation | Required consistency boundary |
| --- | --- |
| Open | Validated text and identity are ready before switching away from the current buffer. |
| Text edit | Rope, history, mapped views/mark, dirty state, and syntax invalidation agree. |
| Undo/redo | One coherent transaction is restored with valid live positions and saved-state identity. |
| Split/delete view | Layout, view registry, selected view, and live buffer references agree. |
| Kill buffer | Dirty decision succeeds before closure; every referencing view gets a valid replacement. |
| Save/write-file | Actual publication and completion determine association, warning, and saved state. |
| Tree rename/delete | Disk outcome and visiting-buffer associations agree without losing in-memory text. |
| Resize | Geometry/projection changes without changing canonical document state. |
| Quit | Required decisions/saves succeed before terminal guard release. |

## Implementation decisions

The [approved delivery plan](../plan/rev-editor.md#pinned-implementation-decisions)
resolves these implementation choices before their owning slices execute:

1. Exact rope, terminal, Unicode, and Tree-sitter crate versions and compatible
   bundled Rust grammar/query.
2. Symlink destination behavior, missing-path normalization, and save-as
   collision handling when another live buffer visits the target.
3. Binary detection, mixed-line-ending tie handling, new-file newline style,
   and recognized BOM representation.
4. Mark/point insertion affinity, exact rectangle edge/padding policy under
   D04, and coherent transaction grouping.
5. Escape/Meta timing policy, concrete terminal capabilities, minimum pane
   geometry, and deterministic focus traversal.
6. History retention, prompt/ring bounds, source-quality checks, PTY fixtures,
   and representative performance measurements.

The plan records exact defaults, dependency versions, and validation owners.
These decisions do not expand functionality or defer required acceptance
behavior beyond the single approved delivery.
