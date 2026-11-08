# Macro Library and State Ownership

Status: Completed
Date: 2026-11-07
Updated: 2026-11-08
Inspection baseline: c86c11383f7de33d56dec17422d87ca2fbfcdc6f
Approved: owner delegated reference-compatible decisions on 2026-11-08
Completed: 2026-11-08; [delivery-set closure](../report/slices/rev-keyboard-macros/slice-8-acceptance.md#delivery-set-closure)
Specification: [Keyboard Macros](editor-keyboard-macros.md)
Dependency: [Command Prefix Arguments](editor-prefix-arguments.md)
Plan: [Keyboard Macros Delivery](../plan/rev-keyboard-macros.md)
Tracker: [Keyboard Macros Tracker](../tracker/rev-keyboard-macros.md)

## Storage ownership

Resolve the existing independent absolute XDG roots through native Roots.

| Root | Default | Responsibility |
| --- | --- | --- |
| Configuration | ~/.config/rev/ | User preferences and declarative associations |
| Data | ~/.local/share/rev/ | Durable data and installed assets under existing contracts |
| State | ~/.local/state/rev/ | Application-managed records retained between sessions |
| Cache | ~/.cache/rev/ | Recomputable disposable results |

The macro owner reads STATE/macros/library.toml and coordinates explicit writes
through STATE/macros/library.toml.lock. Create only its private macros directory
on the first admitted write; a missing library is empty and creates nothing.
Saved macro definitions are valuable user-authored content, not disposable cache.
No new home directory or repository-local configuration/state discovery is added.

Existing owners remain unchanged: STATE/command-history.toml, contexts.toml and
its lock, ai/pi/sessions.toml and sessions/, ai/pi/index.lock, lsp/mutation.lock;
DATA/bookmarks.toml, DATA/lsp payloads/installations.toml and DATA/grammars.
Do not relocate or reinterpret them to normalize this directory shape.

Each new state concern declares its owner, relative namespace, version, triggers,
limits, retention, lock, concurrency and recovery. Reuse concrete filesystem
admission/publication primitives when contracts match, not a generic state database,
global schema, global write lock or service. Record names never select paths.

## Session model

| Entity | Fields and lifetime |
| --- | --- |
| Definition | Stable identity, checked revision, profile, entry state, ordered steps, counter state and format |
| Ring | Bounded ordered definition identities; selected/head at zero; shared by buffers; session-only |
| Named snapshot | Unique command name, optional description/binding, pinned definition, expected stored revision; session-only until saved |
| Recorder | Origin, bounded pending steps, suspended selector/prompt capture, counter state and append snapshot |
| Runner | Pinned call frames, origin, outer count/iteration, step cursor, source anchors, shared limits and native stop/wait state |
| Manual pause | Query/automatic-pause return target, native owner and profile state; at most four frames |
| Definition draft | Pinned target id/revision, editable TOML and normal buffer state |
| Recent input | Last 300 recordable logical inputs, at most 256 KiB, memory-only |
| Macro menu | Stable row identities, bounded marks/flags/selection, no source data authority |

Counter state contains current, previous, iteration_start and format. The session
also owns the last globally inserted macro-counter value and the default format
for new recordings. Arithmetic is checked signed 64-bit. Playback changes this
session state only; it never rewrites disk or saves a continuation on exit.

Named snapshots and ring entries are independent values, with shared immutable
storage permitted only when retained-memory admission accounts for it. Editing a
ring entry cannot mutate an earlier named snapshot. Calls pin each admitted
revision for their lifetime; no retained reference follows a mutable ring index.

The two new configuration leaves are macros.ring_limit (default 8, integer 1..32)
and macros.execute_before_append (default true, boolean). They are preferences,
not state. Keep sample configuration and its complete catalog synchronized.

## Library schema, version 1

| Aggregate field | Stored meaning |
| --- | --- |
| schema_version | Integer 1; unsupported versions are preserved |
| revision | Positive checked aggregate revision |
| macros | At most 256 explicitly saved named snapshots |

| Record field | Stored meaning |
| --- | --- |
| id | Opaque stable 32-character lowercase hexadecimal identity |
| revision | Positive checked record revision |
| name | Unique 1..64 ASCII lowercase letters/digits/hyphens, starting with a letter |
| description | Optional UTF-8 text, at most 512 bytes; absent displays nothing |
| profile | non-modal or modal |
| entry_state | editing for non-modal; normal/insert/replace/visual-character/visual-line/visual-block for modal |
| binding | Optional admitted native key sequence; included only by an explicit prefixed save/export |
| counter | Table with signed current, previous, iteration_start and bounded format |
| steps | Ordered typed declarative values |

Reject duplicate identities/names, inconsistent profile/state, unknown tags or
fields, invalid revisions, nonregular files and over-limit aggregates. Record
order does not imply automatic execution. Names share the admitted command/action
namespace; compiled, prompt and AI collisions cannot be overwritten. Name/identity
collisions on load are reported and quarantined after whole-file structural
validation; valid independent records may remain available. Binding collisions do
not remove the definition: leave that binding inactive with a diagnostic.

## Step representation

| kind | Required payload |
| --- | --- |
| key | key: one normalized admitted native key notation, including logical function keys |
| text | text: exact UTF-8 literal insertion/paste |
| command | name: canonical admitted command identity; argument: typed prefix table; arguments: ordered role-qualified accepted values |
| completion | text, relative checked replacement contract and source text precondition, without popup ordinal/provider dependency |
| search | direction, accepted literal pattern and native acceptance/termination behavior |
| counter | operation, typed argument and admitted format where applicable |
| query | mode: confirm or edit; no recorded answers or consent |

A typed argument table has form absent, plain-prefix with positive depth, integer
with checked signed value, or negative-only. Preserve plain-prefix versus numeric
four. Prompt argument entries contain a native role and literal accepted value;
credential/consent data is never a replayable argument. A command step owns its
accepted arguments. Do not also store selector navigation or duplicated key input
for that accepted invocation. An incomplete prompt cannot be published as a
completed command step. Key/text steps retain current-keymap semantics on replay.

Relative completion contracts are anchored at the current invocation point,
include checked expected source and replacement units, and use native coordinates;
never persist absolute buffer ids, renderer coordinates or raw parser pointers.
Represent named macro calls as command names and pin resolved definitions at run
admission; detect cycles through the active call stack. Counter/format operations
and query markers retain their meaning without an expression evaluator or DSL.

Example of a saved record; all steps are inert data:

    schema_version = 1
    revision = 1

    [[macros]]
    id = "4f967f8112b34ea98a7a2e9900dfc601"
    revision = 1
    name = "prefix-task"
    description = "Add a task marker to the next line."
    profile = "non-modal"
    entry_state = "editing"
    counter = { current = 0, previous = 0, iteration_start = 0, format = "%d" }

    [[macros.steps]]
    kind = "key"
    key = "C-a"

    [[macros.steps]]
    kind = "text"
    text = "- [ ] "

    [[macros.steps]]
    kind = "key"
    key = "C-n"

The implementation must supply ordinary typed encoders/decoders for every variant
and document their exact fields in the user-facing state reference. The table
above is a semantic contract, not permission for alternate unvalidated encodings.
A definition-editor draft is a versioned single-definition projection of this
same schema. It has no new executable syntax and cannot publish another target's
identity/revision. insert-kbd-macro emits a valid single-record schema-1 document.

## Persistence lifecycle

Naming/binding/replay/ring mutation/editor exit perform no implicit library write.
Explicit save snapshots the selected name and counter state; raw prefix presence
controls inclusion of its binding. An unprefixed replacement omits a previously
stored binding rather than claiming to have saved it. Announce that consequence
in the save confirmation when replacing a record that currently has a binding.
Definition editing updates only its session target; saving afterward is explicit.

Startup and kmacro-load read the fixed library without execution. Reload preserves
unsaved named changes and reports conflicts; it is not an implicit discard command.
Session-only definitions disappear on exit without an invented export prompt.
Explicit deletion removes only the selected stored record under revision checking,
never source files, other stores or independent ring/session snapshots.

## Atomic publication and concurrent editors

Use existing private directory/file admission: new directories 0700 and files 0600,
no symlink substitution and no nonregular library/lock entries. Acquire only this
concern's nonblocking lock, reread the bounded current aggregate, validate it, check
the expected identity/record revision and apply the one admitted mutation.
Synchronize a same-directory staged file and atomically replace the library using
existing native publication primitives. Release the lock on every outcome.

Unrelated concurrent record changes merge through the fresh aggregate; stale
same-record updates and same-name additions conflict. Aggregate/record revisions
never wrap. An uncertain publication is verified against the same candidate
identity/revision before retrying; do not duplicate an addition. Preserve prior
valid disk state, local session definition and drafts on failure. No lifetime lock,
background service or last-writer-wins replacement is introduced.

## Limits, retention and invalid input

| Resource | Ceiling |
| --- | --- |
| Named session snapshots / saved records | 256 each, with admitted names sharing the 256-name ceiling |
| Ring | Default 8, configurable 1..32 |
| Definition | 4,096 steps and 256 KiB encoded |
| Encoded library | 8 MiB |
| Description | 512 UTF-8 bytes |
| Recent logical input | 300 entries and 256 KiB |
| Counter format / rendered counter | 256 / 512 UTF-8 bytes; integer width/precision at most 128 |
| Macro-owned retained memory | 48 MiB, accounting for shared or copied admitted values |
| Call depth / manual pauses | 16 / 4 |
| Original region-line anchors | 65,536 |
| Outer playback | 1,000,000 logical steps and iterations; 64 MiB generated text; 60 active seconds |

Enforce encoded and decoded aggregate limits before unbounded allocation. Check
normal source/edit admission independently. A malformed/unsupported/oversized store
is not rewritten as an empty default. There is no automatic format migration,
backup tree, cache eviction or unrelated state cleanup. Saved records remain until
explicit deletion; removing cache cannot remove them. User-facing backup guidance
must identify macros/library.toml as valuable saved content and distinguish it
from session-only recordings and recomputable caches.
