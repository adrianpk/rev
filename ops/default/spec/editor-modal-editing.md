# Modal Editing

Status: approved
Approved: product owner, 2026-11-01; explicit specification approval
Date: 2026-10-28
Reviewed: 2026-11-01
Version: 0.1
Format: OpenSpec capability proposal
Source: product-owner discussion, common-Vim clarification and explicit Odra behavioral-parity direction
Baseline: [rev Editor](rev-editor.md)
Configuration: [Editor Configuration](editor-configuration.md)
Model: [Modal Editing Model](editor-modal-editing-model.md)
Plan: [Modal Editing Delivery](../plan/rev-modal-editing.md)
Delivery boundary: capability and eight-slice plan approved; controller startup and canonical slice integration into dev authorized on 2026-11-01
Readiness: approved behavior with a finite implementation matrix in the approved delivery plan

## Purpose

Provide two startup-selected interaction profiles over the same Rust editor:
Emacs-like direct editing and Vim-like modal editing. Vim-like means familiar
ordinary editing, motions, search, selections, operators and jump navigation.
It does not mean replacing rev's functions with Vim's command catalog.

The deliberate command-entry difference is that : followed by a canonical rev
command name reaches the same operation normally selected through M-x. There
is one command catalog, one editing engine and one set of buffer/file owners.

## Confirmed direction

- Select Emacs or Vim through startup configuration. The choice applies to
  the whole session and cannot be changed by a runtime command or toggle.
- Retain Emacs-like editing as the default when the preference is omitted.
- Normal/Insert/Visual transitions during Vim editing remain ordinary modal
  interaction. They do not change the selected startup profile.
- Include common Vim behavior, not only hjkl: file and line motions, counts,
  word motions, search, operators, character/line/block selection and jumps.
- C-v enters Visual Block in Vim editing. C-o/C-i traverse older/newer jump
  locations in Normal. Input meaning remains state- and context-dependent.
- Invoke rev's existing command names through :{command}, with the same
  completion, availability, dispatch and argument prompts as M-x.
- Do not create Vim counterparts, aliases or a separate implementation of
  rev's existing functions merely to reproduce Vim's Ex command names.
- Use Odra's shipped modal behavior as the reference for movements, selection,
  operators and command entry. Keep rev's startup-only profile choice.
- Keep the full shared interactive command catalog behind colon initially.
  A possible later command filter is deferred, not part of this delivery.

## Current priority and parity baseline

The owner selected and then approved this topic on 2026-11-01, temporarily deferring item 10
of the discussion's execution order. Deferral does not cancel that topic or
make it a prerequisite for modal editing. The owner subsequently approved the
eight-slice plan and tracker and authorized their Looper execution on the same
date. The controller checkpoint owns actual runtime progress.

The current Odra reference at the commit below includes Normal, Insert,
Replace, Visual Character, Visual Line and Visual Block, counts, operators,
semantic repeat, literal search and canonical colon command entry. Its current
modal reference and User Guide include the follow-ups that the original core
specification listed as deferred; use the current behavior rather than those
historical exclusions.

Behavioral parity includes these concrete coordinate rules:

- Normal and Visual j/k, arrows, 0/^/$ and their operator targets use logical
  source lines even when text wraps. The Emacs profile retains its delivered
  visual-line behavior. Rendering, gutter labels and pointer mapping continue
  to use the native source/display projection.
- Visual Character includes the destination grapheme; Visual Line includes
  complete logical lines. Repeating the active v/V/C-v shape exits Visual;
  choosing another shape converts the canonical selection. The o key exchanges
  its active end and anchor.
- Vim Visual Block follows Odra's logical grapheme-column endpoints. Existing
  Emacs rectangles retain rev's display-column behavior. Native selection and
  edit owners must distinguish these coordinate policies without a second
  document, selection store or history. Tabs and wide/combining graphemes need
  explicit acceptance examples for both profiles.
- Insert and Replace retain compatible native modified-key editing behavior,
  while focused prompts and completion lists keep their local keys. Automatic
  completion must not interpret Normal or Visual motion letters as insertion.
- Colon changes the command-entry surface, not command identity or argument
  parsing. For example, :save-buffer and M-x save-buffer invoke the same rev
  operation; Odra's different command names are not copied into rev.

Previously requested common-Vim additions, including MOD13 jump navigation,
remain explicit requirements. The finite matrix must distinguish observed
Odra parity from additional rev requirements; absence from an Odra binding
table is not permission to drop an already requested operation silently.

Reference inspection is static source/documentation evidence, not a new Odra
runtime acceptance run. No Odra files are changed by this work.

## Reference and adaptation

The conceptual reference is Odra's interaction behavior, inspected at commit
f701c830b88e2088bb8de93ba9fee2dd0cd111f1. Relevant reference artifacts are its
modal-interaction-profile and interaction-profiles-and-modal-compatibility
specifications, the Modal Follow-Up section of basic-editor-product-closure,
and the modal-interaction User Guide.

The current behavioral tables are in Odra's
`docs/reference/modal-interaction-profile/index.md` and
`docs/tutorials/user-guide/modal-interaction/work-in-modal-profile.md`.
Navigation, Visual and command-entry ownership were also inspected in its
`src/interaction/modal_navigation.odin`, `modal_visual.odin` and
`modal_command_entry.odin`.

Reuse the behavioral decisions: shared command identity, checked canonical
edits, contextual input ownership, visible modal state, bounded composition,
shared history and : entry over the ordinary command selector. Do not import
Odra's runtime profile selection, programmable catalogs or language runtime.
Rev's existing rectangle geometry and text stores remain authoritative.

The owner's common-Vim clarification also includes a native jump list. This
must have an actual owner; naming C-o is not enough to deliver navigation.
Vim's own [motion reference](https://vimhelp.org/motion.txt.html#jumplist),
[Visual reference](https://vimhelp.org/visual.txt.html#visual-block) and
[Insert reference](https://vimhelp.org/insert.txt.html#i_CTRL-O) distinguish
these state-specific behaviors. They are behavioral references, not a promise
to implement every command or setting documented there.

## Approved editing coverage

This is the required ordinary-editing direction. The approved plan records its
finite binding/behavior matrix and acceptance obligations. Do not silently reduce
the explicit C-v, C-o, search or motion requirements to a smaller hjkl profile.

| Area | Common interaction |
| --- | --- |
| States | Normal, Insert, Replace, Visual Character, Visual Line and Visual Block. |
| Enter/leave text input | i, a, I, A, o, O, R; Escape/C-[ return to Normal. |
| Character/line motions | h/j/k/l and arrows; 0, ^ and $; desired column across vertical movement. |
| Word motions | w/b/e, W/B/E and word-end variants; distinguish word and whitespace-separated WORD behavior. |
| File/line destinations | gg, G and counted logical-line destinations. |
| Local targets | f/F/t/T with a character; repeat in the same/opposite direction with ; and comma; paragraph and matching-delimiter movement. |
| View navigation | Full/half-page motion, visible-window destinations and recentering through the native viewport owner. |
| Search | / and ?; n/N; search for the word at point through the shared search owner. |
| Composition | Counts, d/c/y operators, repeated line operators, indentation and ordinary word/paragraph/delimiter text objects. |
| Ordinary edits | x/X, r, R, D/C/Y, p/P, joining, undo, redo and semantic last-change repetition with dot. |
| Selection | v, V, C-v; extend with motions; exchange active end; operate on the canonical linear or rectangular selection. |
| Jump history | C-o/C-i in Normal; a bounded view-owned history that can identify another visited file. |
| Insert continuity | Native text insertion and common Vim Insert controls, including C-o for one Normal command and return to Insert. |
| Native commands | :{canonical-rev-name}; the same registered operation, candidates and argument workflow as M-x. |

The common-Vim direction does not authorize an Ex parser, Vimscript, external
Vim configuration or plugin compatibility. Search expression compatibility,
the finite text-object catalog and detailed binding conflicts remain listed
design decisions. rev's protected C-g cancellation remains an explicit native
exception rather than an undocumented Vim mapping.

## ADDED Requirements

### Requirement: MOD01 Startup-only profile selection

The editor SHALL resolve exactly one global Emacs or Vim interaction profile
from the admitted startup configuration. Omission SHALL retain the existing
Emacs default. The selected profile SHALL remain immutable for that session;
no M-x command, colon command, binding or per-buffer/language override SHALL
switch profiles. Editing the preference SHALL affect a later startup only.

#### Scenario: Keep direct editing by default

- **GIVEN** the startup configuration omits the profile preference
- **WHEN** rev starts and the user types ordinary text
- **THEN** existing Emacs-like direct insertion and bindings apply
- **AND** no Insert transition is required.

#### Scenario: Defer a preference change until restart

- **GIVEN** a session started with the Vim profile
- **WHEN** its configuration file is changed to select Emacs
- **THEN** the current session remains Vim
- **AND** only a new admitted startup can select Emacs.

### Requirement: MOD02 Coherent configuration and default isolation

Profile configuration SHALL use the existing typed TOML loader, composition,
source diagnostics and whole-candidate validation. Unknown values SHALL NOT
partially activate another keymap. Existing Emacs behavior SHALL be preserved,
including its bindings, text fallback and Escape/Meta handling. The shipped
sample and missing-config provisioning SHALL document the profile once the
behavior is implemented; specification approval alone does not extend the running schema.

#### Scenario: Reject an unknown profile coherently

- **GIVEN** an otherwise valid candidate with an unknown profile value
- **WHEN** startup validates configuration
- **THEN** existing invalid-candidate handling selects complete usable defaults
- **AND** the diagnostic identifies the rejected preference without mixing profiles.

### Requirement: MOD03 Modal states and text admission

Vim startup SHALL enter Normal. The admitted state grammar SHALL distinguish
Normal, Insert, Replace and the three Visual selection shapes. Normal and
Visual input SHALL NOT fall through to accidental text insertion. Insert and
Replace SHALL use canonical text transactions. State transitions alone SHALL
not edit text or create an undo entry. Entry/exit cursor behavior SHALL be
specified with checked native positions and familiar Vim-visible outcomes.

#### Scenario: Enter and leave insertion

- **GIVEN** Vim Normal in an editable buffer
- **WHEN** the user presses i, types text and presses Escape
- **THEN** text enters through the ordinary edit owner and the state returns to Normal
- **AND** subsequent motion keys navigate rather than entering document text.

### Requirement: MOD04 Familiar motions over canonical positions

The Vim grammar SHALL provide the ordinary motion families in the coverage
table, including gg/G, counted line destinations and word/WORD distinctions.
Motions SHALL resolve valid native grapheme/line positions with bounded counts,
normal viewport reveal and Unicode-safe conversion. A Vim word-start motion
SHALL NOT be approximated with an existing Emacs word-end operation merely
because it has a similar name. Any missing semantic operation SHALL have one
native owner, available to normal command dispatch where appropriate.

#### Scenario: Reach a file destination and move by words

- **GIVEN** Vim Normal in a multi-line buffer containing punctuation and words
- **WHEN** the user uses gg, G, a counted line destination and word motions
- **THEN** the documented Vim-like destinations are reached through canonical positions
- **AND** no motion inserts text or splits a grapheme.

The implemented Slice 2 endpoint matrix is documented in the
[binding reference](../../../docs/reference/bindings.md#normal-navigation).
The shared Odra subset preserves b's native Unicode word start and e's exclusive
word-end boundary; w separately observes word/punctuation-class starts. WORD,
ge/gE and counted line forms are explicit rev additions. Forward search keeps
the native match-end boundary; reverse search keeps the match start. This pins
semantic endpoints without changing the existing Emacs bindings.

### Requirement: MOD05 Counts and operator composition

Counts, operator-pending input, character targets and text objects SHALL form
bounded native composition. The editor SHALL validate the complete target and
count arithmetic before committing an operation. Copy/delete/change/paste and
indentation SHALL reuse the existing text-transfer and edit owners, extending
their shared semantics when necessary rather than adding Vim-only storage.
Unsupported or stale compositions SHALL fail without a partial edit.

#### Scenario: Resolve one counted change

- **GIVEN** Vim Normal and a valid counted operator/motion target
- **WHEN** the complete composition is accepted
- **THEN** one checked semantic operation applies to the resolved range
- **AND** overflow, invalid input or a changed target cannot publish a partial result.

### Requirement: MOD06 Shared Visual selections and history

v, V and C-v SHALL enter character, logical-line and rectangular Visual
selection respectively. Selection presentation SHALL derive from existing
native selection/rectangle ownership. Vim blocks SHALL use Odra-compatible
logical grapheme-column endpoints; existing Emacs rectangles SHALL retain
display-column endpoints. Both SHALL preserve checked grapheme/tab boundaries
and one atomic edit/history transaction, not repeated independent line edits.
Profile changes SHALL NOT
be required to use undo, redo or existing rectangle commands.

#### Scenario: Select and delete a block

- **GIVEN** Vim Normal and several lines containing tabs and Unicode text
- **WHEN** the user presses C-v, extends a block and deletes it
- **THEN** the admitted rectangle is removed as one coherent native operation
- **AND** one ordinary undo restores all affected lines.

#### Scenario: Preserve the Emacs page binding

- **GIVEN** an uncustomized Emacs session
- **WHEN** the user presses C-v
- **THEN** its existing paging behavior remains effective
- **AND** the presence of Vim support does not activate a Visual state.

### Requirement: MOD07 Colon entry uses the rev command catalog

In Vim Normal and Visual states, colon SHALL open the native extended-command
selector over the same currently registered names as M-x. Filtering, vertical
candidates, completion, history, availability, dispatch and argument prompts
SHALL use the same owners. There SHALL NOT be a second catalog of Vim versions
of rev functions. A leading colon is entry syntax, not part of command identity.
Vim Ex aliases, ranges, pipelines and trailing-argument parsing SHALL NOT be
inferred from familiar spelling. Compatible M-x entry SHALL remain available.
There SHALL NOT be a colon-only command whitelist or blacklist in this delivery.
Ordinary query matching and contextual command availability SHALL remain the
same as M-x. Any later catalog filter requires a separate product decision.

#### Scenario: Invoke one operation through either surface

- **GIVEN** equivalent editing state and the registered split-window-right command
- **WHEN** one session selects it through M-x and another through :split-window-right
- **THEN** both dispatch the same native handler
- **AND** command availability and resulting layout behavior are equivalent.

#### Scenario: Request a command argument

- **GIVEN** a colon-selected command needs an argument
- **WHEN** the user confirms its canonical name
- **THEN** the ordinary argument prompt owns subsequent input
- **AND** that input neither edits the document nor becomes an Ex expression.

#### Scenario: Cancel Visual command entry

- **GIVEN** colon entry was opened from a valid Visual selection
- **WHEN** the user cancels before invocation
- **THEN** the compatible source state and selection are restored
- **AND** no command or edit is published.

### Requirement: MOD08 Search and contextual input ownership

Slash/question-mark search and n/N SHALL use the canonical search lifecycle,
including confirmed direction, cancellation and source-revision handling.
Prompt, completion, tree, confirmation and other focused surfaces SHALL own
input ahead of ordinary editing grammar. Literal colon, slash and letters
typed in an argument or Insert context SHALL not start a Normal-mode command.
Returning from a surface SHALL restore a valid editing state without retaining
an operator targeting another view or revision.

#### Scenario: Search and repeat

- **GIVEN** Vim Normal and several occurrences of a search pattern
- **WHEN** the user searches with slash, confirms and uses n/N
- **THEN** navigation follows the admitted pattern and direction
- **AND** cancellation and failures retain the native search guarantees.

#### Scenario: Complete a command without jumping

- **GIVEN** the colon command selector owns input
- **WHEN** the terminal reports Tab or an indistinguishable C-i chord
- **THEN** the selector performs its ordinary completion action
- **AND** the Normal jump list does not consume that event.

### Requirement: MOD09 Cancellation and terminal Escape

C-g SHALL retain rev's reserved context cancellation. A decoded Escape/C-[ in
Vim editing SHALL cancel pending composition and return to Normal without an
edit. Higher-priority prompts retain their own cancellation lifetime. Escape
disambiguation SHALL be bounded and profile-aware; adding Vim SHALL neither
regress Emacs Meta input nor falsely claim distinction between chords the
terminal reports identically.

#### Scenario: Cancel a pending operator

- **GIVEN** a count/operator is incomplete in Vim editing
- **WHEN** the user cancels
- **THEN** all pending composition is cleared and a valid Normal state remains
- **AND** document text, retained text and undo history remain unchanged.

### Requirement: MOD10 Profile- and state-aware bindings

Only the selected profile SHALL participate in editing-key resolution. Native
Vim bindings SHALL have explicit state/context scope so C-v, C-o, C-i and other
chords do not inherit unrelated Emacs effects. User overrides SHALL remain
declarative mappings to admitted rev operations with whole-candidate conflict
validation and protected cancellation. The existing keys owner supplies independent `keys.vim_normal`, `vim_insert`,
`vim_replace`, `vim_visual_character`, `vim_visual_line` and `vim_visual_block`
maps; editor, minibuffer, completion and local component maps retain their scope.

#### Scenario: Keep inactive bindings out of resolution

- **GIVEN** startup selected Vim with validated state-scoped bindings
- **WHEN** a Normal chord has a different Emacs meaning
- **THEN** only its admitted Vim meaning participates in resolution
- **AND** changing profile never occurs as a side effect of a key lookup.

### Requirement: MOD11 Visible state and native presentation

The existing status layout SHALL show the current modal state while Vim is
active, with unambiguous narrow-pane degradation. Cursor shape MAY reinforce
that state where the terminal supports it; failure to set a shape SHALL not
break editing. Emacs presentation SHALL preserve its existing behavior.
State presentation SHALL not consume document cells or change buffer contents.

#### Scenario: Identify text-entry state

- **GIVEN** Vim Normal in a narrow usable pane
- **WHEN** the user enters Insert and returns to Normal
- **THEN** an unambiguous state indicator follows both transitions
- **AND** editing remains usable without terminal cursor-shape support.

### Requirement: MOD12 Bounded work and shared command semantics

Modal parsing, counts, pending keys, target observations, repeat data and jump
history SHALL have finite documented limits. Waiting for composition SHALL not
block resize, cancellation, pointer input or normal event processing. Existing
input batching and repeat handling SHALL not gain a new movement backlog.
Both profiles SHALL use the same read-only admission, canonical history, file
protection and command availability, including newly delivered native commands.

#### Scenario: Reject a stale read-only target

- **GIVEN** a modal edit is pending and its source changes or becomes read-only
- **WHEN** the operation attempts publication
- **THEN** native admission rejects an invalid edit
- **AND** no partial history, text-transfer update or unrelated target edit appears.

### Requirement: MOD13 Native jump history

Normal C-o/C-i SHALL traverse older/newer admitted jump locations, including
locations in other files when they can be resolved safely. The jump list SHALL
be view-owned and session-local, distinct from text undo, the mark and user
bookmarks. Ordinary h/j/k/l motion SHALL not flood it. Jump recording and
traversal SHALL retain current locations and reject stale/unavailable targets
without guessing another position or losing unsaved buffers. Counts and the
C-i/Tab terminal alias SHALL have explicit context-aware behavior.

#### Scenario: Return from a file jump

- **GIVEN** a view has a recorded origin and a later admitted file/line destination
- **WHEN** the user uses C-o and then C-i in Normal
- **THEN** the view can revisit the older and newer locations
- **AND** buffer edits are neither undone nor discarded.

#### Scenario: Use C-o while inserting

- **GIVEN** Vim Insert owns ordinary editing input
- **WHEN** the user presses C-o and completes one admitted Normal command
- **THEN** that command runs under checked native dispatch and compatible insertion resumes
- **AND** the chord is not incorrectly treated as Normal jump-history traversal.

### Requirement: MOD14 Replacement and semantic repeat

Ordinary r/R replacement and dot repetition SHALL use native semantic edit
requests, complete graphemes and normal undo/redo. Repeat SHALL record an
admitted operation and bounded required data rather than replaying raw keys
into whichever prompt or buffer happens to have focus. Repeating SHALL resolve
the current target and recheck command/read-only admission. It SHALL NOT repeat
file writes, argument consent or an unrelated asynchronous operation merely
because one was invoked during the previous interaction.

#### Scenario: Repeat an editing change

- **GIVEN** an admitted insertion/change and a later compatible editing target
- **WHEN** the user presses dot
- **THEN** the checked semantic change is applied to the current target
- **AND** ordinary undo reverses the repeated change without replaying prompt input.

## Baseline reconciliation during delivery

Implementation slices reconcile the baseline Purpose, R01 and R24 with default
Emacs direct entry and explicit Vim text-entry states. R22/R23 use one command
registry and native selector for colon and M-x. R33 retains native owners and
qualifies completion by modal state. CFG12-CFG15 record the typed startup profile,
six independent state maps, whole-candidate fallback and immutable admission.
Historical gate evidence and unaffected clauses remain unchanged. The integrated
candidate still requires its controller-owned gate before capability closure.

## Implementation preparation obligations

The approved scope is unchanged. The approved plan makes these obligations
concrete rather than delegating silent product decisions to a worker:

1. Pin motion/text-object ranges, inclusivity, linewise transfer and count
   boundaries to the finite matrix and inspected Odra behavior. Preserve
   explicit additional rev requirements rather than shrinking the scope.
2. Keep native literal search, smart case and no wrapping. Word-at-point search
   uses checked native boundaries, not a second regexp dialect.
3. Use editor.interaction_profile and the six keys.vim_* state maps listed in
   the model. Preserve typed startup admission, local maps and C-g/M-x recovery.
4. Test Escape/Meta and transitions against actual decoded terminal input,
   Unicode, source EOL and native cursor ownership.
5. Deliver the declared jump events, 256-entry view-local capacity, checked
   edit/rename/reopen behavior and independent split evolution.
6. Validate Vim logical-grapheme blocks and existing Emacs display-cell
   rectangles across tabs, wide characters, wrapping and folds.
7. Keep Replace/repeat within native edit/history admission, the recorded
   1,024-byte text bound and checked focus/state lifetimes.

An actual product departure requires owner review. Implementation details do
not authorize an Ex language, alternate command catalog or runtime profile toggle.

## Validation direction and dependencies

Use behavioral and real-terminal acceptance for both profiles: unchanged Emacs
typing and bindings; Vim state transitions and visible cursor; gg/G and counted
destinations; word/operator examples; slash search; C-v rectangles with tabs and
Unicode; C-o/C-i and their prompt/Insert meanings; colon/M-x equivalence;
read-only/failure/cancel/undo preservation; resize, focus and input responsiveness.

The feature depends on the existing native command, input, configuration,
buffer, selection, rectangle, history, search and presentation owners. It does
not require LSP, AI or the separate bookmark feature. Future commands keep
their canonical names under either entry surface when actually registered.
The approved capability is ready for the approved delivery map. The plan and
tracker remain the planning/execution authority; the owner authorized controller
startup and canonical slice integration into dev on 2026-11-01.

## Implementation and validation map

The implementation maps each requirement to the following concrete checks.
The capability remains approved until the controller records its integrated
exact-candidate gate and closes the delivery set.

| Requirement | Concrete coverage |
| --- | --- |
| MOD01 Startup choice | `tests/modal.rs`, configuration startup tests, native isolation and both independently started PTY profiles. |
| MOD02 Configuration isolation | `tests/modal/keymaps.rs`, configuration whole-candidate/default-sample tests and native immutable-snapshot checks. |
| MOD03 Editing states | Native operators/repeat, Insert and Replace completion, PTY state transitions and cursor shapes. |
| MOD04 Logical navigation | `tests/modal/navigation.rs`, `viewport.rs`, `local_targets.rs` and source-bounded u32-max resource traversal. |
| MOD05 Composed operators | `tests/modal/operators.rs`, `objects.rs`, `operator_safety.rs` and native/PTY word deletion with exact undo. |
| MOD06 Visual geometry | `tests/modal/visual.rs`, `visual_safety.rs`, three native shapes and the 4,096-row Unicode/CRLF block measurement. |
| MOD07 Shared colon entry | `tests/modal/contexts.rs`, command-catalog tests, native colon/M-x argument equivalence and decoded PTY selector/pane controls. |
| MOD08 Context ownership | `tests/modal/search.rs`, `contexts.rs`, `context_safety.rs`, source-draft save/abort and popup-local cancellation. |
| MOD09 Cancellation and Escape | `tests/modal/boundaries.rs`, `completion.rs`, `context_safety.rs` and separately decoded Escape versus packed Meta input in PTYs. |
| MOD10 State maps | `tests/modal/keymaps.rs`, complete supported sample/configuration checks and the independent Emacs/Vim PTY settings. |
| MOD11 Native presentation | `tests/modal/presentation.rs`, display tests, narrow native resize, PTY source cursor, three cursor styles and reset. |
| MOD12 Bounds and preservation | Count overflow, large source/block, native and PTY 128-event bursts, native/server/PTY resource observations and first-party source quality. |
| MOD13 Checked jumps | `tests/modal/jumps.rs`, `jump_safety.rs`, native jump/undo ownership and decoded C-o/Tab and post-split backward traversal. |
| MOD14 Replacement and repeat | `tests/modal/repeat.rs`, `repeat_safety.rs`, native/PTY dot grouping and installed semantic acceptance in Replace with one undo. |
