# Emacs Interaction Parity

Status: Prepared for subsequent delivery slicing
Expression navigation status: Approved through rev-delimiter-navigation
Paragraph navigation status: Pending
Date: 2026-11-14
Direction approved: product owner, 2026-11-14; core-first functional parity and related Org/Git follow-through
Decision evidence SHA-256: `cc915b2d4af14a6ad0da34a517724e9664f59436b40d1004877e8a539af21939`
Format: OpenSpec behavioral requirements; workspace specification contract
Native inspection source tree: `442765af9faac4351c208b2bfc5ca02c1b896331`
Current delivery: EIP04 expression portion through [Delimiter Navigation](editor-delimiter-navigation.md), [plan](../plan/rev-delimiter-navigation.md), [tracker](../tracker/rev-delimiter-navigation.md)
Evidence: [semantic matrix](../report/rev-factory-shortcuts/semantic-matrix.md), [contextual review](../report/rev-factory-shortcuts/mode-review.md), [functional reference](../report/rev-factory-shortcuts/parity-reference.json)
Baselines: [Interaction](editor-interaction.md), [Modal Editing](editor-modal-editing.md), [Prefix Arguments](editor-prefix-arguments.md), [Org](editor-org-mode.md), [Git](editor-git-integration.md)

## Purpose

Make shared editing and command workflows use the same finger choreography
and observable effects when moving between Rev and stock Emacs. Nonmodal
editing is the compatibility target. The selected modal profile keeps its
explicit state-specific operations. Similar command names, broad purposes or
successful key parsing do not establish parity.

The accepted decision requests the missing behavior, with core gaps before
module extensions. The current seven-slice shortcut delivery owns only its
accepted existing-operation map and this preparation input. Functional additions
belong to the subsequent delivery sequence below, whose formal plan and exact
checks must be recorded before implementation. This document starts no second
controller and does not change the current seven identities or frozen gate.
It introduces no durable model, persistence schema or dependency choice.

## Verified baseline and evidence limits

The frozen catalog has no whole-buffer/paragraph mark or nonmodal open-line
command. [Region commands](../../../src/command/region.rs) expose set/exchange,
while [region dispatch](../../../src/session/region.rs) owns activation and quit.
The current newline inserts the preferred separator and advances point;
modal open-above/open-below changes editing state and cannot stand in for C-o.

[Native line editing](../../../src/session/editing/lines.rs) joins the following
line and shifts indentation by a configured unit. [Structure navigation](../../../src/coordinates/navigation/structure.rs)
uses blank logical lines for paragraphs and reaches the matching delimiter
itself; it does not implement syntax-aware sexp endpoints. [Repeat](../../../src/session/repeat/replay.rs)
requires modal Normal state and replays a retained semantic change. These
owners are reusable boundaries, not equivalent substitutes for the stock
nonmodal operations.

[parity-reference.py](../report/rev-factory-shortcuts/parity-reference.py)
reproduces 17 stock key workflows under GNU Emacs 31.1 / Org 9.8.7 with `-Q`.
It records source hashes for simple.el, paragraphs.el, lisp.el and org-src.el,
zero-based character positions, point/mark/region, source text, kill payloads
and prompt depth. Quit separates input bursts because it ends a keyboard macro.
The owned Org fixture additionally verifies actual parent-buffer and disk save
while retaining the source draft. No personal init or external transport runs.
These are batch key workflows, not physical terminal evidence or Rev parity
acceptance. Existing native probes establish the current differences separately.

| Observed stock sequence and fixture | Reference result |
| --- | --- |
| C-x h within `one\ntwo\n` | Point 0, mark 8, active whole-buffer region, source unchanged. |
| M-h within `one two\nline\n\nnext\n` | Point 0, mark 13, active paragraph region, source unchanged. |
| C-o at offset 2 in `abcd` | `ab\ncd`, point remains 2; C-u 2 C-o inserts two separators there. |
| M-^ in `prev\n  current\n  next` | `prev current\n  next`, point 4; C-u M-^ instead joins the following line, point 14. |
| M-} / M-{ around the second paragraph | Both observed endpoints are offset 4 for their respective starts; movement does not edit. |
| C-M-f / C-M-b on `(ab) tail` | Forward reaches 4, backward from 4 reaches 0; matching-delimiter offset 3 is different. |
| C-x h C-M-\ on a Lisp progn | Indents both body lines by two spaces; retains the syntax and matches reference point/mark/activation. |
| C-d C-x z on `abcd` | Deletes twice to `cd`; not modal last-change replay. |
| C-SPC M-f M-w C-g C-e C-y | Copies `alpha`, then yanks at end to `alpha betaalpha`. |
| C-SPC M-f C-w C-e C-y | Kills `alpha`, then yanks at end to ` betaalpha`. |
| C-u 2 C-f; M-x forward-char RET | Both command routes have their own observed point effects and no source edit. |
| M-x delete-char C-g | Retains source and point, with no active prompt or command invocation. |
| Org source-edit Save after inserting x | Parent and disk contain the edited block; parent is unmodified after save; draft remains selected. |

The table is a concrete seed, not the full acceptance matrix. Extend paired
fixtures for Unicode, syntax, counts, errors and complete undo/redo sequences
through the affected owners. Preserve actual reference observations when a
case fails; do not regenerate expectations from the Rev implementation.

## Requirements

### Requirement: EIP01 Compare complete workflows

For shared nonmodal capabilities, Rev SHALL match stock key sequences and
observable text, point/mark, region activation, count, prompt, kill/yank and
undo/redo behavior in the corresponding point and mode context. Explicit user
overrides SHALL remain authoritative. Mode-specific differences SHALL be
identified against stock mode behavior rather than an arbitrary global alias.

#### Scenario: Same keys, different primitive

- **GIVEN** a native primitive has a similar name but a different endpoint,
  direction, selection effect or interaction state
- **WHEN** a familiar sequence is considered
- **THEN** the required operation is implemented through its appropriate owner
- **AND** the primitive is not advertised as equivalent by rebinding alone.

### Requirement: EIP02 Whole-buffer and paragraph marking

C-x h and mode-appropriate M-h SHALL establish the reference point, mark and
active region, including repeated invocation/count behavior where supported.
They SHALL preserve source and edit history. A following copy, kill, indentation
or quit SHALL act on that selected region through existing native ownership.

#### Scenario: Mark before copying

- **GIVEN** point is inside the first paragraph of a multi-paragraph buffer
- **WHEN** M-h then M-w is entered
- **THEN** the copied source range and region state match stock text-mode
- **AND** source bytes remain unchanged; C-g can retire the selection safely.

### Requirement: EIP03 Open-line and preceding-line join

C-o SHALL insert the counted line separators at point while retaining the
reference point/mark effects and editing profile. M-^ SHALL join the preceding
line by default; its explicit prefix SHALL select the reference following-line
behavior. Spacing, beginning/end boundaries, read-only refusal and undo/redo
SHALL match the supported stock operation and preserve native document syntax
and line-separator policy.

#### Scenario: Open without moving into the next line

- **GIVEN** `abcd` with point at character offset 2
- **WHEN** C-u 2 C-o is entered
- **THEN** the source is `ab\n\ncd` and point remains at 2
- **AND** undo/redo restores and reapplies that complete operation.

#### Scenario: Join direction is observable

- **GIVEN** point is on the indented current line after `prev`
- **WHEN** M-^ is entered without a prefix
- **THEN** it joins that current line to `prev`, not to its following line
- **AND** the explicit-prefix case is verified separately against the reference.

### Requirement: EIP04 Paragraph and sexp navigation

M-{/M-} and C-M-b/C-M-f SHALL use correct mode-dependent paragraph and syntax
boundaries, direction, counts and endpoints. Strings, comments, nested pairs,
empty/blank boundaries and malformed syntax SHALL have explicit reference
cases. Movement SHALL preserve source/history and follow stock mark/region
behavior; native coordinates and Unicode boundaries remain valid.

The expression-motion portion is specialized by [Delimiter Navigation](editor-delimiter-navigation.md),
DPN02, alongside distinct typed-mate and structural-group commands. Its shared
qualification/configuration model is [Pair Preferences and Native State](editor-delimiter-pairs-model.md).
Paragraph motion and the complete functional-parity delivery remain owned here;
implementing delimiter navigation does not by itself complete F2 or this spec.

Expression progress: forward-sexp/backward-sexp, C-M-f/C-M-b and their admitted
mode/count/region behavior are implemented in navigation Slice 4. Its focused
evidence is recorded in the [Slice 4 report](../report/slices/rev-delimiter-navigation/slice-4-expressions.md).
The [Slice 5 report](../report/slices/rev-delimiter-navigation/slice-5-acceptance.md)
records representative physical-input and complete native acceptance for this
portion. Controller integration, the combined gate and expression closure remain
pending; local evidence does not complete the requirement.
Paragraph motion remains pending; EIP04 and F2 retain that separate scope.

#### Scenario: Forward past a balanced form

- **GIVEN** Lisp text `(ab) tail` with point at 0
- **WHEN** C-M-f is entered
- **THEN** point reaches 4, past the form
- **AND** a separate matching-delimiter motion may retain its native endpoint.

### Requirement: EIP05 Region indentation and command repetition

C-M-\ SHALL indent the selected region using the admitted mode's indentation
contract, rather than shifting each line by one arbitrary native unit.
C-x z SHALL repeat an eligible preceding command with the reference argument
policy and refusal behavior. General command repetition and modal semantic
last-change replay SHALL retain separate ownership and state contracts.

#### Scenario: Repeat a nonmodal deletion

- **GIVEN** `abcd` at point 0 in nonmodal editing
- **WHEN** C-d then C-x z is entered
- **THEN** the result is `cd`, with reference point and undo behavior
- **AND** no modal state change is required.

### Requirement: EIP06 Shared editing and command selection

Existing editing, mark/exchange, kill/copy/yank/yank-pop, history, prefix
arguments, prompts and M-x SHALL be tested as complete workflows. Cancellation
SHALL retain the source and retire pending input; a suffix SHALL never supply
consent to a different operation. Full-name and single-fragment matching,
ranking, completion, invocation and effective hints SHALL remain coherent.

The existing [name-fragment ticket](../ticket/open/20261114010915-filter-commands-by-multiple-name-fragments.md)
owns whitespace-separated fragment filtering in both shared selectors. Integrate
that ticket's acceptance into this subsequent work; do not create a duplicate
or claim stock default completion supplies that additional native requirement.
The current [palette](../../../src/input/palette.rs) uses contiguous contains/
prefix matching, so `mark ind it` is not already implemented by the baseline.

#### Scenario: Cancel before invoking

- **GIVEN** an M-x command draft that could edit the current buffer
- **WHEN** C-g cancels it
- **THEN** no command executes, source/history remain unchanged and editing
  focus resumes without consuming later fresh input as a stale suffix.

### Requirement: EIP07 Org contextual insertion and source save

After the core increments, Org SHALL follow stock contextual TODO-heading/list
insertion for M-S-RET through its checked structure owners. Existing checkbox
list support SHALL not stand in for the heading branch.

Source-edit Save SHALL publish checked draft content to the parent and perform
the explicit parent save while retaining the draft, matching stock successful
behavior. Draft exit and abort SHALL remain distinct operations. Failed/stale/
read-only/conflicting saves SHALL retain recoverable draft/source state and
report actual disk outcomes through native save/consent guarantees.
[Native draft lifecycle](../../../src/session/source_edit/lifecycle.rs) currently
refuses Save, exits to an unsaved parent, and discards the draft on abort.

#### Scenario: Save without exiting source editing

- **GIVEN** an edited source draft and a current writable owned parent file
- **WHEN** C-x C-s is entered in the draft
- **THEN** checked parent text and saved disk reflect the edit
- **AND** the draft stays selected; cancellation/failure cannot imply a save.

### Requirement: EIP08 Git prefix, menu and transport behavior

After core parity, the related Git command families SHALL compare complete
reference sequences, selected identities, menus, prompts and operation effects.
The current History c a/c f/+ and Refs f/P routes are the accepted shortcut
increment, not completion of reference transient or transport parity. Status c
currently opens literal composition; it SHALL be reviewed against reference c
prefix behavior in the separate functional increment.

Reference-style prefixes SHALL prepare the proper family and selected resource.
They SHALL preserve operation-specific review/confirmation, exact revisions,
local/remote scope and bounded execution. A second letter SHALL not become an
unintended message character or supply execution consent. Owned local fixtures
SHALL verify cancel/refusal and source/index/history/disk outcomes before any
transport test. No new dependency is selected by this requirement.

#### Scenario: Cancel a transport family

- **GIVEN** an owned repository and a selected revision or branch
- **WHEN** a reference-family prefix is opened and then cancelled
- **THEN** no transport or repository mutation occurs
- **AND** current selection and the explicit native consent boundary survive.

### Requirement: EIP09 Terminal and profile acceptance

Relevant ambiguous control/Meta sequences SHALL compare stock Emacs in the
same terminal, with the same emitted input, before attributing a difference
to the terminal. Native key normalization SHALL preserve reachable reference
operations. Nonmodal, modal state, language, prompt and projection transitions
SHALL be checked explicitly, retaining all user override and data guarantees.

#### Scenario: A control sequence differs

- **GIVEN** a key works in stock Emacs in the chosen terminal
- **WHEN** Rev interprets it differently
- **THEN** acceptance records emitted input and both observable effects
- **AND** fixes the avoidable native difference rather than declaring a generic
  terminal limitation.

## Subsequent delivery preparation and ownership

These concrete ordered increments are preparation for one subsequent formal
delivery. They do not append slices to the active seven-slice set. Its final
plan/tracker must pin branches, reports, exact checks and resource cost before
functional implementation; no competing controller starts from this document.

| Order | Bounded functional increment | Owning source and acceptance focus |
| --- | --- | --- |
| F1 | Whole-buffer/paragraph marking, C-o and preceding-line join | Region/edit/history owners; EIP02/EIP03; paired key-to-selection/edit and undo/redo cases, boundaries and explicit overrides. |
| F2 | Mode-aware paragraph/sexp navigation | Coordinates/navigation with admitted mode syntax; EIP04; exact endpoints/counts, strings/comments/malformed input and Unicode. |
| F3 | Region indentation and general command repetition | Mode indentation and command-history/repeat owners; EIP05; region state, eligible/ineligible commands, prefix policy and undo groups without altering modal dot. |
| F4 | Complete shared editing/prompt/history workflows and existing fragment ticket | Input/palette/session/kill/history owners; EIP01/EIP06/EIP09; whole sequences and representative same-terminal input, both selectors and focus transitions. Core completion precedes F5/F6. |
| F5 | Org heading/list branch and retained source-draft save | Org structure/source-edit/native save owners; EIP07; exact parent/draft/disk outcomes, stale failure, cancel and undo; inherited profiles and overrides. |
| F6 | Git command prefixes/menus and transport selection | Git input/composer/Refs/process owners; EIP08/EIP09; complete family sequences, selection, cancelled review and bounded owned local outcomes. |

Reuse existing native command, configuration, modal, Org and Git regression
roots and owned PTY infrastructure. Select focused cases at the changed owner;
then define one finite integrated gate for the subsequent delivery. Preserve
current data/consent/resource assertions and do not combine historical full
feature gates by default. No functional result is claimed by this preparation.

The audit's D03 dispositions remain accurate observations of current differences.
They do not authorize keeping arbitrary shared-operation differences permanently.
The current shortcut plan links this owner and closes its own mapping scope;
this specification keeps the requested core-first implementation discoverable.
