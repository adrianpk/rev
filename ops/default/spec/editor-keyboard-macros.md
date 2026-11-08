# Keyboard Macros

Status: Approved
Date: 2026-11-07
Updated: 2026-11-08
Format: OpenSpec capability addition
Inspection baseline: c86c11383f7de33d56dec17422d87ca2fbfcdc6f
Approved: owner delegated remaining behavior decisions to the documented reference workflow on 2026-11-08
Execution authorized: finish specification/model, plan, slicing and one Looper run
Dependency: [Command Prefix Arguments](editor-prefix-arguments.md), Delivered / Completed
Model: [Macro Library and State Ownership](editor-keyboard-macros-model.md)
Plan: [Keyboard Macros Delivery](../plan/rev-keyboard-macros.md)
Tracker: [Keyboard Macros Tracker](../tracker/rev-keyboard-macros.md)
Baselines: [Commands](../../../docs/reference/commands.md), [Bindings](../../../docs/reference/bindings.md), [State roots](../../../docs/reference/defaults.md#startup-configuration)

## Purpose

Record a real editing interaction and repeat it through native input, command,
source, history and prompt owners. Keep familiar record/end/replay gestures,
macro-ring navigation, append, counters, queries and editing. Named macros are
ordinary commands. Naming and binding are session-local; explicit saving retains
selected definitions as inert TOML in the application state namespace.

The owner settled named-command identity and state ownership earlier, and now
requests the closest practical reference behavior without further product quizzes.
All remaining decisions below are resolved. Storage, bounded execution and modal
entry contracts adapt that behavior to the existing native editor. No executable
language, third-party source or external macro runtime is introduced.

## Commands and default gestures

Every command below has the stated concise English description in M-x and modal
colon. Aliases share identity, usage and description. Existing user keymap overrides
remain effective. A supplied count uses the completed prefix-argument contract.

| Command | Default gesture | Description |
| --- | --- | --- |
| kmacro-start-macro-or-insert-counter | F3 | Start recording, or insert the current macro counter. |
| kmacro-start-macro | C-x (, C-x C-k C-s | Start or append to a keyboard macro. |
| kmacro-end-macro | C-x ) | Finish recording a keyboard macro. |
| kmacro-end-or-call-macro | F4 | Finish recording, or run the current keyboard macro. |
| kmacro-end-and-call-macro | C-x e | Finish recording and run the current keyboard macro. |
| kmacro-end-or-call-macro-repeat | C-x C-k C-k | Run the current macro and enable ring repeat keys. |
| kmacro-call-ring-2nd-repeat | C-x C-k C-l | Run the second macro and enable ring repeat keys. |
| kmacro-cycle-ring-next | C-x C-k C-n | Select the next older keyboard macro. |
| kmacro-cycle-ring-previous | C-x C-k C-p | Select the next newer keyboard macro. |
| kmacro-delete-ring-head | C-x C-k C-d | Remove the current macro from the session ring. |
| kmacro-swap-ring | C-x C-k C-t | Exchange the first two macros in the ring. |
| kmacro-redisplay | C-x C-k d | Refresh the display during macro playback. |
| kmacro-insert-counter | C-x C-k C-i | Insert a macro counter value. |
| kmacro-set-counter | C-x C-k C-c | Set or reset the current macro counter. |
| kmacro-add-counter | C-x C-k C-a | Add to or restore the current macro counter. |
| kmacro-set-format | C-x C-k C-f | Set the integer format for macro counters. |
| apply-macro-to-region-lines | C-x C-k r | Run the current macro on logical lines in the region. |
| kbd-macro-query | C-x q | Add a confirmation or editing pause to a macro. |
| exit-recursive-edit | C-M-c | Resume a macro after its manual editing pause. |
| abort-recursive-edit | C-] during a macro editing pause | Abort a macro editing pause and its playback. |
| kmacro-name-last-macro | C-x C-k n | Give the current macro a session command name. |
| kmacro-bind-to-key | C-x C-k b | Bind a session macro to a key sequence. |
| kmacro-save | Unbound | Save a named macro in the persistent library. |
| kmacro-load | Unbound | Reload saved macros without running them. |
| kmacro-delete | Unbound | Explicitly remove a macro from the saved library. |
| insert-kbd-macro | Unbound | Insert a named macro definition as TOML text. |
| kmacro-edit-macro | C-x C-k Enter | Edit the current keyboard macro definition. |
| kmacro-edit-macro-repeat | C-x C-k C-e | Edit the current macro and enable ring repeat keys. |
| edit-kbd-macro | C-x C-k e | Select a named or bound macro to edit. |
| kmacro-edit-lossage | C-x C-k l | Make a macro from recent recordable input. |
| kmacro-step-edit-macro | C-x C-k Space | Execute and edit a macro one step at a time. |
| list-keyboard-macros | Unbound | List and manage the session macro ring. |
| edmacro-finish-edit | C-c C-c in a macro definition buffer | Apply a checked macro definition draft. |
| edmacro-insert-key | C-c C-q in a macro definition buffer | Insert the next key sequence in macro notation. |
| edmacro-set-macro-to-region-lines | C-c C-r in a macro definition buffer | Use complete selected definition lines as the macro body. |

The definition and menu controls have named native actions and descriptions.
Step-edit and query choices are role-qualified prompt answers, not standalone
commands that bypass their prompt owner. The additional menu inventory is:

| Command | Menu key | Description |
| --- | --- | --- |
| kmacro-menu-edit-position | # | Change the selected macro's ring position. |
| kmacro-menu-transpose | C-x C-t | Move the selected macro above its predecessor. |
| kmacro-menu-edit-counter | c | Change the selected macro counter. |
| kmacro-menu-edit-format | f | Change the selected macro counter format. |
| kmacro-menu-edit-keys | e | Edit the selected macro definition. |
| kmacro-menu-edit-column | Enter | Edit the selected macro property. |
| kmacro-menu-flag-for-deletion | d | Flag selected macros for ring deletion. |
| kmacro-menu-do-flagged-delete | x | Remove flagged macros from the ring. |
| kmacro-menu-mark | m | Mark selected macros for menu operations. |
| kmacro-menu-do-copy | C | Duplicate selected macros in the ring. |
| kmacro-menu-do-delete | D | Confirm removal of selected ring macros. |
| kmacro-menu-unmark | u | Clear selected macro marks and deletion flags. |
| kmacro-menu-unmark-backward | Backspace | Clear a macro mark and move backward. |
| kmacro-menu-unmark-all | U | Clear all macro menu marks and flags. |
| kmacro-menu-refresh | g | Refresh the macro menu. |
| kmacro-menu-quit | q | Close the macro menu and restore its source view. |
| kmacro-menu-help | ? | Show the macro menu commands and bindings. |

All entries are native command identities, even when their default key is local.
Do not add global register shortcuts: this editor has no general
register owner, and macro naming/binding already supplies direct reusable access.

## Recording, replay and repeat maps

Recording performs real edits immediately and displays a compact Def indicator.
F3 starts a fresh recording; while recording it inserts the counter. F4 ends
recording without replay. Unprefixed C-x ) also ends without replay. C-x e while
recording ends and runs the completed definition once more. A positive count N
on C-x ) includes the recording pass, so N=4 adds three executions.

After C-x e, bare e repeats until another input leaves that temporary repeat map.
Ring actions enable the corresponding C-k/C-l/C-n/C-p/C-d/C-t continuations.
An unrelated key leaves the repeat map and is processed normally, not swallowed.
C-g retires pending counts, repeat maps and capture/playback; it never rolls back
edits already made. Aborted/empty/over-limit capture preserves the previous valid
ring head rather than publishing a partial recording.

Any supplied start prefix appends to the current definition. A plain C-u runs
that definition once before appending; a double plain C-u appends without running
it. macros.execute_before_append defaults to true; false skips the pre-execution.
Pre-execution failure prevents append capture. Named snapshots remain independent
of later ring edits. The shared ring defaults to eight entries, newest/current at
position zero; macros.ring_limit accepts 1..32. It is shared by all buffers and
memory-only. Eviction never deletes a named or saved snapshot.

No argument replays once. A positive argument requests that many iterations.
Zero or a negative argument repeats until a native stopping condition, C-g or the
finite run budget; it does not reverse the definition. Empty macros cannot enter
this form. A failed search, end boundary that signals failure, read-only refusal,
invalid target or lost owner stops the current run with iteration/step information.
A saturated cursor movement that reports ordinary success must not be mistaken
for a stopping condition. Budget exhaustion remains an explicit stop.

Playback uses the ordinary native edit/history mechanism. Each outer iteration
is one undo unit in each affected source buffer, separated from adjacent typing
and other iterations. A partly completed iteration closes its native undo unit;
prior edits stay real and undoable. Multi-buffer iterations are not cross-buffer
transactions. Recording itself keeps ordinary native history grouping. Macro
editing, naming and library publication do not add source-history entries.

## Recording representation and input ownership

Store normalized logical key input and exact UTF-8 text/paste, not terminal escape
bytes, timestamps, physical releases, renderer rows or pointer coordinates. Key
steps use the current effective keymap on replay. Named M-x/colon acceptance is
different: store the canonical resolved command name and its typed prefix once,
with accepted role-qualified prompt arguments. Do not also record selector keys,
query abbreviations, candidate navigation or the command twice. Missing commands
stop playback; changed candidate order cannot retarget a saved invocation.

Capture literal accepted prompt values for file/buffer selection, goto, searches,
replacement and ordinary native text arguments. Preserve the invocation's input
origin; prompts cannot consume its prefix accidentally. Search failures propagate
as typed stopping outcomes. Record manual completion acceptance as its checked
literal edit text/range contract, not a row number or dependency on a live server.
Validate the same current source applicability before replaying that edit.
Clipboard/yank operations retain ordinary kill-ring semantics; they are explicit
commands against the current ring, not a hidden snapshot of another application.

All execution goes through normal admission. Internal typed step completion,
wait, stop and consent outcomes belong to native owners; never infer failure by
matching human echo strings. Native file visits may wait for existing source or
layout work under the run deadline. Save/read-only/overwrite/process decisions
requiring fresh consent pause for the user; consent is not captured for automatic
reuse. Core source, window and buffer commands are recordable. Externally running
AI, Git mutations, shell jobs, raw terminal sessions and context replacement are
not advertised as synchronous macro steps: encountering one stops recording or
playback before starting that operation, with a useful diagnostic. This prevents
an asynchronous launch from being treated as a completed source transformation.

Recorder and runner pin startup profile and source entry state. Shared controls
work in both profiles without taking over native modal counts or Escape handling.
A macro recorded in modal Normal must begin there; Insert and Replace are distinct
states. A conflicting entry profile/state refuses before effects. A changed end
state prevents another iteration unless the definition itself restores its entry
state. Manual query edits use the current native mode and checked owner on resume.

Named macro calls inside recordings store the stable name, not expanded bodies.
Playback pins each admitted called definition and shares one outer budget. Reject
recursive name cycles and excessive nesting. No expanded playback events enter
recording or recent-input history. A user invocation records one command-usage
event; its internal expanded commands do not flood M-x ranking.

## Counters and formats

Each new recording starts current/previous/iteration-start counters at zero and
captures its initial format. Inserting a counter updates previous, then adds the
explicit numeric increment or one by default. A plain prefix inserts the previous
value without changing current. Setting with a numeric prefix assigns it; a plain
prefix during playback restores the iteration-start value. Adding uses the numeric
argument; a plain prefix restores the last counter value inserted by any macro.
Checked signed 64-bit arithmetic stops on overflow, never wraps.

An empty format prompt restores %d. Support one integer conversion d, o, x or X,
literal surrounding text, %%, ordinary sign/alignment/alternate/zero flags, and
bounded decimal width/precision up to 128. Formats are at most 256 UTF-8 bytes;
one rendered counter is at most 512 bytes. Reject unsupported conversions without
changing the previous format. A format set outside capture/playback affects new
recordings; a format set inside is retained at that point in that definition and
does not rewrite other macros. Existing macros retain their own formats. Saved
snapshots retain counter state at explicit save; running them does not rewrite disk.

## Region lines, queries and manual pauses

apply-macro-to-region-lines runs once at the beginning of each logical source line
whose beginning is inside the original half-open region. A region ending at the
next line's beginning excludes that line. Wrapped rows add no targets. Capture
checked original line anchors, at most 65,536, and map them through edits. Skip
anchors whose source line was deleted; do not drift to a replacement line or run
newly inserted lines. Source/owner changes outside admitted work stop the run.

C-x q records a query without interrupting the initial recording. At replay,
Space/y continues, Backspace/n skips the rest of this iteration, and Enter/q ends
the remaining run. C-r enters manual editing; C-M-c returns to the same query.
A supplied prefix on C-x q records an unconditional editing pause and enters that
pause both during recording and replay. Manual edits during the pause are not
recorded. C-M-c resumes; C-] or C-g aborts the active pause/run. Pause frames are
bounded to four, remain visible and introduce no polling timer.

## Definition editing, step editing and menu

The macro definition buffer shows editable declarative TOML steps with its pinned
target identity/revision. C-c C-c validates and replaces only that session target;
invalid, stale or oversized drafts stay editable and preserve the prior macro.
Normal buffer closing can discard the draft. Capture-key and selected-whole-line
helpers use the same admitted step notation. They do not evaluate user expressions.
Naming/editing never implicitly persists. Recent-input editing uses at most the
last 300 recordable logical inputs, bounded to 256 KiB and kept only in memory;
exclude protected prompts, external terminal input and expanded playback events.

Step editing shows the next logical command and ? help. Space/y executes it;
n/d/Backspace removes it; f skips it for this execution only; Tab executes an
adjacent homogeneous text group; c continues without questions; C-k removes the
rest. q/C-g cancels definition changes while keeping real source edits. i/I inserts
input before the step, r/R replaces it, a inserts after it, and A appends after
executing the remainder. Lowercase forms read through C-j, uppercase I/R read one
key sequence. Successful finish replaces the pinned session definition once.

The read-only macro menu lists ring position, optional name, current counter,
format, rendered value and a clipped definition. Native rows retain stable
identities through reorder. # edits position, C-x C-t transposes upward, c/f/e
edit counter/format/definition and Enter edits the current column. d flags deletion;
x applies flags; m marks; C duplicates; D deletes with confirmation; u/Backspace/U
clear marks in the usual current/previous/all scopes. Region selection takes
precedence over marks, then the current row. Multiple-target duplication and all
immediate deletion require confirmation. Ring deletion does not delete named or
saved copies. g refreshes, q closes the menu, ? shows local bindings. Menu controls
use native commands with descriptions and matching key hints.

## Naming, bindings and persistent state

C-x C-k n prompts for a command name and snapshots the current macro for this
session. Reject collisions with compiled commands, prompt actions or admitted AI
commands. Replacing another session macro requires confirmation. Later ring append
or eviction does not change that snapshot. A missing description renders nothing.
C-x C-k b reads a native key sequence; a single digit or uppercase letter expands
to its reserved C-x C-k slot. Other occupied bindings require explicit confirmation;
reserved cancellation and raw-terminal ownership cannot be overridden.

kmacro-save explicitly selects a named snapshot and atomically publishes it under
STATE/macros/library.toml, including its admitted binding only when the caller
supplies a prefix. Naming/binding/exit never saves implicitly. kmacro-load reloads
this fixed library without execution, reporting conflicts with unsaved session
changes instead of silently replacing them. kmacro-delete explicitly removes a
saved record after confirmation and revision checking; ring deletion is separate.
insert-kbd-macro inserts a standalone TOML representation of the named snapshot
through normal source editing; a prefix includes its binding. Export is inert and
does not publish a library record. The model owns encoding and concurrency.

Startup reads the fixed bounded library and admits valid definitions without
executing them. A newly conflicting binding is left inactive with a diagnostic;
a conflicting command name is quarantined, never substituted. Other independent
valid records can be admitted only after the complete structural store validation
passes. Invalid/unsupported/oversized stores stay untouched and do not affect
ordinary editing or other persistence owners.

## Finite work and justified adaptations

Definitions have at most 4,096 steps and 256 KiB encoded size; the saved library
holds at most 256 named records and 8 MiB. Session named definitions share the
256-record ceiling. Ring capacity is configured in 1..32. Nesting is at most 16;
recursive name cycles stop. Aggregate macro-owned retained memory is capped at
48 MiB, including ring, admitted names, recent input and active draft/run copies.

Each outer run shares ceilings of 1,000,000 dispatched logical steps, 1,000,000
iterations, 64 MiB generated text and 60 seconds of active automatic work. Existing
8 MiB single-edit limits still apply. Explicit user pauses do not consume active
time; native I/O keeps its own shorter deadline. Poll cancellation at least every
256 logical steps and yield on a four-millisecond cooperative target between
native primitives. A single existing atomic primitive is not a hard realtime
operation. Exhaustion reports its cause and preserves already published effects.

No persistent idle worker/timer is introduced. Pinned definitions cannot mutate
mid-run. Async/protected contexts and macro-only pause frames have the explicit
boundaries above. TOML storage/editing replaces an executable representation;
integer-only formatting replaces a runtime-dependent general formatter. These
are concrete native ownership/representation adaptations, not a new extension API.

## ADDED Requirements

### Requirement: KMA01 Record and execute ordinary keyboard macros

The editor SHALL implement the declared start/end/replay gestures, Def indicator,
temporary repeat maps and cancellation through native owners.

#### Scenario: Record and repeat

- **GIVEN** editable source and a recording with movement and insertion
- **WHEN** F4 ends it and a positive counted replay runs
- **THEN** the body executes exactly the requested additional times without its control gestures.

### Requirement: KMA02 Reuse prefix arguments and original source-line targets

The editor SHALL preserve raw prefix meaning, finite nonpositive repetition and
original logical region-line targeting, independent of visual wrapping.

#### Scenario: Edited region

- **GIVEN** a selected multiline source region and a macro that inserts/deletes lines
- **WHEN** apply-macro-to-region-lines runs
- **THEN** only surviving admitted original line beginnings are processed once.

### Requirement: KMA03 Name and explicitly persist independent snapshots

Naming and binding SHALL create session-only snapshots. Only explicit library
operations SHALL publish the concern-owned TOML store; no runtime macro state
SHALL be written into config.toml.

#### Scenario: Name, save and restart

- **GIVEN** two named session macros and only one explicitly saved snapshot
- **WHEN** the editor restarts
- **THEN** only the saved definition is restored without execution.

#### Scenario: Failed publication

- **GIVEN** an existing valid library and a usable named session snapshot
- **WHEN** an explicit save fails or conflicts
- **THEN** both prior states remain available and no durable success is claimed.

### Requirement: KMA04 Preserve profiles, owners, bindings and updates

Replay SHALL pin definitions and check entry profile/state, native command
admission, source applicability, consent and collisions. Undo SHALL be grouped
per outer iteration per buffer; completed effects SHALL not be rolled back.

#### Scenario: Changed state or consent

- **GIVEN** a saved recording and a changed entry state, keymap or protected action
- **WHEN** replay reaches an inapplicable step
- **THEN** it stops or requests fresh native consent before that effect.

### Requirement: KMA05 Retain named-command identity and accepted input

Accepted named invocations SHALL retain canonical names, raw typed arguments and
role-qualified accepted prompt data once, independently of selector presentation.

#### Scenario: Different ranking and completion source

- **GIVEN** a recording with a named command and a literal accepted completion
- **WHEN** ranking or available providers change
- **THEN** the same command and checked literal edit are used without a replacement row selection.

### Requirement: KMA06 Supply ring, append and counter workflows

The editor SHALL implement the declared ring repeat maps, independent snapshots,
append variants and checked stateful counters/formats.

#### Scenario: Append and count

- **GIVEN** an existing recording with a counter and a separately named snapshot
- **WHEN** append extends the ring recording and repeated playback inserts numbers
- **THEN** the sequence follows its saved counter operations and the named snapshot is unchanged.

### Requirement: KMA07 Permit query choices and bounded manual editing

Queries SHALL support continue, skip, stop and manual pauses with unrecorded
manual input and explicit native resume/abort behavior.

#### Scenario: Vary an iteration

- **GIVEN** a counted macro with a query and an unconditional manual pause
- **WHEN** the user skips one iteration, edits another and resumes
- **THEN** the decisions affect that invocation without becoming recorded automatic consent.

### Requirement: KMA08 Edit and manage definitions without execution on admission

Definition, step and menu workflows SHALL preserve valid snapshots on invalid,
stale or cancelled edits, expose native named actions and avoid implicit saving.

#### Scenario: Stale draft

- **GIVEN** an open definition draft and a changed target revision
- **WHEN** the draft is accepted
- **THEN** the conflict preserves the target and leaves the draft available for inspection.

### Requirement: KMA09 Bound execution, storage, cancellation and idle cost

All stated resource limits SHALL be enforced before uncontrolled allocation or
publication. Stop outcomes SHALL be typed, and cancellation SHALL release runner
ownership without an idle deadline or background service.

#### Scenario: Nonpositive run or malformed library

- **GIVEN** an unending source transformation or an oversized/invalid library
- **WHEN** its applicable bound is reached
- **THEN** the concern stops with a diagnostic while ordinary editing and unrelated stores remain usable.

## Acceptance and guidance

Use native integration fixtures and real isolated storage. Cover both profiles,
recording, named/prompt/completion capture, history, ring/append/counters, queries,
region anchors, nested calls, fresh consent, restart, collisions and concurrent
publication. Include representative real PTY function-key and C-x journeys, a
large-buffer cancellation/resource probe and explicit idle-release assertions.

Beginner documentation teaches one short recording and replay. Task guides cover
counts/regions, naming and explicit saving. Reference owns all commands, defaults,
TOML state, formatting and limitations; explanation owns native recording/replay
semantics. Markdown remains the source of the Spore guide. KMA01-KMA09 and the
model are the approved delivery contract; there are no unresolved product questions.
