# Modal Editing Model

Status: approved
Approved: product owner, 2026-11-01; paired promotion with the specification
Date: 2026-10-28
Reviewed: 2026-11-01
Version: 0.1
Specification: [Modal Editing](editor-modal-editing.md)
Existing models: [Editor](rev-editor-model.md), [Configuration](editor-configuration-model.md)

## Ownership

One admitted startup profile selects input behavior for the session. Native
buffers, revisions, text positions, selections, rectangles, text transfer,
history, search and layouts retain their existing ownership. A modal grammar
does not own a second rope, undo stack, command registry or register store.

Profile is not modal state: choosing Vim is a startup configuration decision;
moving between Normal, Insert, Replace and Visual states is ordinary runtime
editing. There is no runtime operation that changes the selected profile.

## Persistent preference

| Field | Meaning | Persistence contract |
| --- | --- | --- |
| Interaction profile | Emacs-like direct editing or Vim-like modal editing. | One validated startup preference. Omission resolves to Emacs; the admitted value is immutable until the process exits. |

Approved TOML spelling for the implementation delivery:

```toml
[editor]
# Startup interaction profile: "emacs" or "vim". Restart rev after changing it.
interaction_profile = "emacs"
```

This is an approved extension, not a key implemented by the v0.2.0 runtime.
Use the existing root/include precedence, provenance and invalid-candidate
handling. Do not introduce project/language/view profile overrides or a
separate preference file. Missing-config provisioning and the complete sample
gain the key only when the feature is supported. Modal state is not written
back to TOML. No preference reload or runtime setter is introduced.

Extend the existing keys owner with vim_normal, vim_insert, vim_replace,
vim_visual_character, vim_visual_line and vim_visual_block maps. Existing
keys.editor overrides remain scoped to Emacs; Vim factories explicitly retain
compatible native modified-key controls rather than inheriting arbitrary Emacs
overrides. Local component maps apply whenever they own input in either profile.
Inactive profiles cannot shadow the active editing map.

## Ephemeral records

| Record | Fields and contract |
| --- | --- |
| Session profile | Admitted enum value, startup origin and native factory/override resolution. Read-only for the session. |
| Editing state | Normal, Insert, Replace, Visual Character, Visual Line or Visual Block; current editing/focus owner and checked resume observation. Restore only a compatible origin; otherwise retire its transient state into a valid selected context. |
| Pending composition | State, count/operator/character-target kind, finite input, checked arithmetic and originating view/buffer/revision. Cancel or reject when the target is no longer valid. |
| Visual observation | Canonical mark/point or rectangle reference and selection shape. Native owners compute ranges; no private modal selection is authoritative. |
| Command entry | Shared catalog/query/candidate/history owner plus source state, focus and compatible selection observation. Colon is an entry route, not another command identity. |
| Search observation | Existing search session/pattern/direction/revision ownership. Modal bindings select its operations rather than maintain parallel matching state. |
| Jump list | At most 256 ordered locations and a traversal position per view, using native document/location evidence. Session-local and independent of undo/bookmarks. Splits copy the bounded observation, then evolve independently. |
| Jump location | Buffer/file identity, checked logical position or native anchor, source observation and resolution status. Stale locations must never be silently reused as another target. |
| Semantic repeat | Last admitted repeatable operation, target descriptor, checked count and at most 1,024 retained text bytes. An explicit repeat count overrides the recorded count. No raw event, consent or provider/tool replay. |
| One-command Insert continuation | Original compatible insertion state and one bounded Normal-command lifetime. It cannot become a nested command loop or redirect input into a newer prompt. |

These are semantic records, not mandated Rust types or a public extension API.
The interaction implementation should use the existing native owners and
introduce only missing state with a demonstrated lifetime.

## Positions, selection and editing

Canonical byte, character, grapheme, logical-line and display-cell coordinates
remain distinct. Familiar Vim cursor/motion behavior must map to valid native
positions; it does not justify byte-oriented cuts or a second cursor model.
Vim word/WORD and motion-end rules may need new native semantic operations;
reusing a similarly named Emacs motion is insufficient evidence of equivalence.

The owner's Odra parity direction fixes Vim Visual Block endpoints in logical
grapheme columns. Existing Emacs rectangles retain display-column semantics.
Native selection ownership must distinguish those coordinate policies and
preserve checked tab and wide/combining-character boundaries without a private
modal selection store. Linear and block changes use prepared atomic edits and
ordinary history; failure does not publish earlier lines of a multi-line operation.

Vim Normal and Visual line motions use logical source lines while the Emacs
profile retains its existing visual-line behavior. Both project through the
same native wrapping, gutter and pointer coordinate owners.

Colon uses the complete shared interactive catalog initially, with the same
query matching and contextual availability as M-x. No persisted colon-specific
allowlist, denylist or alternate command identity is introduced.

Copy/delete/change/paste and repeat must preserve required selection shape and
linewise intent through shared native ownership. Do not add a second modal
kill ring or an implicit full Vim register namespace.

## Implemented navigation endpoints

Slice 2 routes semantic descriptors and their checked count through the native
command/deferred-layout owner. The coordinate owner resolves graphemes,
word-class runs, logical-line finds, paragraph boundaries and delimiter targets;
the existing view owner projects logical vertical, wrapped viewport and fold
geometry. Native Emacs word commands share the counted Unicode word scanner
with modal b/e and retain their prior endpoints.

Shared Odra endpoints are h/l whole graphemes, j/k logical rows with desired
display column, 0/^/$ logical boundaries, w word/punctuation-class starts,
b Unicode word starts, e exclusive word-end insertion boundaries, gg BOF and
G EOF. WORD, ge/gE and counted destinations remain explicit rev requirements,
with endpoints documented in the [binding reference](../../../docs/reference/bindings.md).
No byte decrement or standard-Vim e endpoint is inferred from the key's name.

Pending syntax observes the exact native view and point stamp/position. Find
repetition observes view and source revision plus the matched grapheme and its
landing position, so till repetition progresses. Confirmed query/direction and
whole-word continuation extend the canonical search owner and literal matcher.
Each resolver bounds traversal by the source/view; missing counted matches
publish no partial point. Idempotent targets keep their single destination.

## Implemented operator and transfer ownership

Slice 3 composes one editing request from operator/count, native navigation or
text-object descriptor and an optional Unicode character. Count multiplication
checks u32 before range resolution. Pending state stays qualified by the exact
native source point and shares the 16-stroke/96-byte grammar limits. Operator
search previews retain that origin; confirmation supplies one qualified range.

Resolved transfer ranges carry linear or complete-logical-line intent in the
existing kill-ring owner. Copy and failed/read-only edits do not alter history;
retained text is published only after an admitted deletion/change succeeds.
Small grapheme deletion and replacement preserve the retained entry. Native
linear yank and modal paste share storage; linewise paste handles destination
LF/CRLF and EOF separators without a register store. Repeated transfer output
has a 64-MiB materialization bound checked before allocation; this does not turn
the native history retention target into a single-edit limit.

The native half-open word and character-target endpoints remain explicit in the
[binding reference](../../../docs/reference/bindings.md#normal-operators-and-transfer).
Shared native coordinate resolution owns inner/around word, WORD, paragraph,
paired delimiters and logical-line quotes. Indentation/joining publish ordered,
disjoint prepared edits in one canonical history unit and map every shared view.

## Lifetimes and recovery

Startup prepares and validates the complete selected profile before editing.
Invalid settings follow the existing complete-default fallback. After startup
the profile cannot change; only editing state and transient contexts evolve.

Prompts, candidates, tree and confirmations own their input while active.
Suspension/resumption must check the originating view, buffer and revision.
Changing buffer cannot leave a Visual selection or pending operator targeting
the previous document. A profile-specific grammar cannot consume a local
confirmation response or treat completion Tab as jump navigation.

C-g retains native cancellation. Escape handling must distinguish decoded
events from terminal ambiguities and keep the Emacs path unchanged. There are
no idle modal scans or background services. Counts and pending data use the u32
count ceiling, 16-stroke grammar and 96-byte target bound. Jump/repeat storage
obeys its limits above; large counts cannot become count-sized UI loops.

Jump recording/traversal needs its own declared event and invalidation policy.
Reuse checked native locations where possible without making the separate
user-bookmark feature mandatory. Closing, renaming, editing or reopening a
file must not turn an old location into an unrelated current one. Split
inheritance and retention follow the bounded view policy and plan acceptance cases.

## Readiness

The approved specification and [delivery plan](../plan/rev-modal-editing.md)
define finite coverage, keymaps, search, location ownership and bounds.
Implementation must verify cursor/Escape, selection edges and state restoration
against native tests and the inspected reference. There is no persisted
jump/repeat archive, runtime profile setter or executable configuration.
The owner approved the implementation plan and authorized controller startup
and canonical slice integration into dev on 2026-11-01.

## Delivered Visual selection ownership

Visual shapes extend the native buffer region mode. The shared mark anchors a
checked view point; Character materializes an inclusive grapheme endpoint and
Line materializes complete logical lines. Block distinguishes logical grapheme
columns from existing display-cell rectangles. Highlighting and edits resolve
those same row fragments through native coordinate projection. A session retains
only the source stamp qualifying Visual lifetime, without copying its endpoints.
Same-buffer view changes preserve the shared shape; buffer changes or source
publication retire it. Pointer drag resets the shared anchor in the active shape.

Visual d/c/y and linear indentation use prepared native edits/history. Block
transfer shares one native rectangle allocation with the bounded kill-ring entry
and exposes an LF-separated linear text view. Modal paste reads the retained
shape and native rectangle insertion preserves its coordinate policy. Counts
repeat block rows horizontally within the shared 64-MiB expanded-text bound;
copy, rejection and cancellation never publish partial row edits. Linear Visual
objects are limited to inner/around word and paragraph and publish Character
endpoints through the same mark/point owner.

## Delivered jump and continuation ownership

Each view owns a 256-entry jump history and traversal index. Splits copy bounded
native location handles and then evolve independently; rebinds retain that view's
history. A weak native location registry releases unreferenced entries on ordinary
capture/edit/close work, without idle polling. Each location contains a checked
source point, optional native file identity and 64 source scalars of evidence on
each side. Paths retain the native 4096-byte limit. The shared source-context
primitive belongs to file locations; named bookmarks keep their separate storage,
settings, context-search policy and persistence.

Canonical prepared edits admit the live location set, map endpoints with left
anchor affinity and record before/after outcomes in their ordinary payload budget.
Removed or joined-away boundaries become stale. Structural permutations and
undo/redo restore checked saved outcomes rather than guessing nearby cursors.
Close prepares all source evidence before changing either location owner. Closed
visits require matching file observations, exact bounded context and a native
grapheme boundary; changed/replaced/missing targets do not advance traversal.
Canonical rename carries closed paths, while open dirty buffers retain identity.

Explicit line/buffer/paragraph/viewport destinations, accepted search and file
navigation record successful origin/destination pairs. Ordinary cursor/word steps,
previews, drawing and traversal itself do not append entries. Backward traversal
captures the newest current point; older locations stay authoritative after edits
and undo. Only a new admitted jump truncates a forward branch.

Insert C-o retains one checked view/source continuation while the native Normal
command lifetime consumes counts, operator/character syntax and component-owned
prompts. An admitted edit may change that source revision; incompatible external
source, buffer or view changes retire it. Completion restores Insert only in a
valid native editing context, before later deferred input is dispatched. Explicit
Visual entry remains Visual, and local prompts/completion retain the Tab/C-i alias.

## Implemented replacement and repeat ownership

Slice 6 adds Replace to the typed input state and its independent factory map.
Native text admission replaces complete graphemes on the current logical line,
inserts at EOL and retains the line separator. Joining input extends the previous
cluster. Replace uses an underline cursor and ordinary command/input ownership.

One session-local record retains an entry descriptor, a complete semantic edit
request, Visual geometry or an admitted transfer payload, with at most 1,024
retained text bytes. Visual geometry stores a whole-grapheme count, logical-line
count or logical-block height/width. A completed record can resolve a fresh target
in another compatible pane/buffer. No stored source offset becomes authoritative.
An explicit count replaces the original count; a Visual override selects the
linear/line count or block width while retaining block height.

A pending entry observes its exact native view/point/revision and first history
stamp. Native connected replacements compose reversible fragments and before/
after anchor snapshots into one ordinary history unit. An empty top block row
can insert before the first removed row in that same owned group. Independent
movement, changed source or another view ends its authority. Native undo payload
accounting and whole-group pruning remain authoritative for retained history.

Successful publication supplies the repeat text from the native grouped result;
there is no raw-event transcript. A line opening or linewise change separates its
native line separator from user payload. Exceeding the text limit retires the
entire repeat candidate without truncating input or undo. Destructive entry edits
outside that insertion, source-bound accepted-search ranges and unrelated native
workflows retire repeatability. Failed edits and failed repetitions preserve the
last admitted record. Prompt, consent, provider and asynchronous work is never a
repeat operation.

## Implemented command and completion ownership

Colon in Normal and every Visual shape binds the existing extended-command
enum. Its vertical catalog, query, history, availability, argument stages and
native handlers are exactly those used by M-x. An observed Visual command origin
contains only view, stamped point, mark and shape. Native selection remains the
authority while the selector owns input. Cancellation retains it only if every
observation still matches; changed evidence retires it without copying offsets.
Selector cancellation precedes outer component cancellation, so an Org source
draft remains active until its native save/abort command executes.

Directional pane commands resolve overlapping screen geometry and select through
the native view owner. Balancing retains bounded overrides for live layout forks,
reserves separator cells and allocates same-axis pane units. Deletion prunes dead
forks; later splits keep the admitted startup ratio. Neither operation writes
configuration or owns source text, cursor positions or history.

Completion captures editing state with its ordinary invocation generation and
native source/view/point evidence. Automatic requests require Vim Insert/Replace;
manual canonical invocation remains available. Changed state retires requests,
popups and selected resolution before polling. Existing generation, source,
provider and permission checks govern returned edits and imports. Native atomic
publication updates modal context and retires unsupported repeat ownership.
Insert C-o remains suspended through an active completion request and resumes
only after the complete asynchronous workflow has finished compatibly.

Tree, prompts, inventories, bookmarks and native Org contexts keep their input
priority. Popup cancellation stays local, Tab/C-i keeps its active component
meaning and held Return cannot become a later edit. Source drafts share the
immutable startup profile. No worker, queue, input-batch or deadline limit changes,
and modal state alone introduces no idle polling.

## Acceptance and resource ownership

A native Rust acceptance driver constructs isolated startup snapshots and source
files, exercises shared editing/history and launches the installed Rust server in
an owned offline project. The semantic child has a 60-second outer deadline and
35-second readiness window; selected work and server cleanup retain five-second
bounds. Server RSS and CPU observations are separate from the native driver.

The existing controlling-PTY owner supplies finite input/output and cleanup.
Each PTY keeps the eight-second and 1-MiB output limits, observes complete native
frames and requires terminal attributes, alternate screen, paste mode and cursor
restoration. Cursor-style bytes precede the frame delimiter, so a read-only
observer uses the same bounded stream. Explicit Escape is separated from a later
undo key; packed Escape-x remains terminal Meta input.

Large-count and logical-block observations use native targets and prepared
history. Count overflow preserves source, point and history. A 4,096-row block
removes complete tab/wide/combining graphemes with one exact undo. Native and PTY
128-event bursts retain source and command order. Idle observations require no
native deadline, terminal output or modal polling. Existing 32-event/four-ms
input batching and runtime worker/retention limits remain unchanged.

Executable size is observed under normal release flags with two build jobs and
compared to the recorded predecessor release; the external server is excluded.
Measurements describe their actual candidate and host, without creating another
runtime service or inferring universal performance guarantees. Canonical Markdown,
its ordered guide index and the Spore tree remain one documentation source.
