# Command Recency and Frequency

Status: Completed
Completed: 2026-11-06
Validated candidate: `fea4fffdfa3e66d8a7d7f1cc031a6637199424af`
Gate validation: `342LB2K3MYK2Z2JS5ESVI4WSNP`
Date: 2026-11-06
Approved: product owner, 2026-11-06; specification/model and the established preparation, push and one autonomous Looper delivery through dev
Format: OpenSpec change specification
Source: owner report that reopening M-x shows the same alphabetical list instead of the last used commands; proposed four recent plus four frequent entries
Baselines: [Editor Interaction](editor-interaction.md), UI02; [Configuration](editor-configuration.md), CFG17; [Modal Editing](editor-modal-editing.md)
Persistence model: [Command Usage Model](editor-command-ranking-model.md)
Plan: [Command Ranking Delivery](../plan/rev-command-ranking.md)
Tracker: [Command Ranking Tracker](../tracker/rev-command-ranking.md)
Current reference: [Commands](../../../docs/reference/commands.md), [Focused Prompts](../../../docs/reference/prompts.md), [Defaults](../../../docs/reference/defaults.md)

## Purpose

Put the commands the user recently invoked at the beginning of M-x instead of
showing the same alphabetical beginning every time. Follow those recent entries
with frequently used commands, without duplicates or an unbounded history.
The same behavior applies to the modal colon command selector.

The owner approved this specification and its model on 2026-11-06 and requested
the established preparation, plan, slicing, push and one autonomous Looper
delivery through dev. Main promotion, release, installation and mirrors retain
their separate authorization boundaries.

## Behavior before this change

At the planning baseline, `src/input/palette.rs` filtered the native and registered
AI command names by the configured match policy, then by name. The session
already retained distinct accepted commands for M-p/M-n navigation, bounded by
`minibuffer.history_limit` with default 128. That session history did not rank
the candidate list and was not persisted.

The new usage window is a separate bounded input to candidate ranking. It
retains repeated invocations so frequency can be calculated. Existing history
navigation and its configured retention remain unchanged.

## Ordering

With an empty query, produce one continuous candidate list:

1. Up to four distinct used commands, newest first.
2. Up to four other used commands, highest frequency first.
3. All remaining admitted commands in alphabetical order.

Frequency is measured over the last 256 accepted named-command invocations.
For equal frequency, prefer the more recently used command, then its canonical
name. A command appears only once. Unused commands cannot fill a frequent slot;
if fewer used candidates exist, the ordinary remainder follows without gaps.
The visible list still obeys `minibuffer.candidate_limit`, default eight, and
available terminal geometry. The proposal does not force eight visible rows.

With a nonempty query, preserve configured match admission. An exact name is
first. Under `prefix-then-substring`, prefix matches precede substring-only
matches; under `prefix`, substring-only matches remain excluded; under
`substring`, all other admitted matches share one rank. Apply the same recent,
frequent and alphabetical ordering within each rank. Usage never makes a poor
text match outrank a better match or displays a nonmatching command.

The four/four split and 256-invocation window are approved compiled policy
values. No additional configuration is required for this capability. Existing
candidate, prompt-input and session-history settings retain their meanings.

## MODIFIED Requirements

### Requirement: UI02 Vertical command completion

M-x SHALL display its query followed by one candidate per row, using up to
`minibuffer.candidate_limit` rows as geometry permits. The modal colon selector
SHALL use the same candidate ranking. Command recency and frequency SHALL
order candidates within the text-match ranks defined above, with deterministic
alphabetical fallback. The selected candidate SHALL remain visible while
navigating; an empty match SHALL have a clear empty state.

Up/Down and C-p/C-n SHALL select candidates; M-p/M-n SHALL retain their session
command-history navigation. Tab SHALL complete the selected command name
without executing it. Return SHALL dispatch the highlighted command once,
including the first candidate when the query is empty. An unknown name SHALL
report an error. Opening M-x alone SHALL NOT execute a command. C-g SHALL cancel,
discard the draft and restore prior editing focus. Modal Escape cancellation
SHALL retain its current behavior. Rendering SHALL NOT execute commands.

#### Scenario: Filter and choose a command

- **GIVEN** multiple commands match a typed prefix
- **WHEN** the user moves to a later row and presses Return
- **THEN** the visible selected command invokes its existing Rust handler once
- **AND** command arguments continue through the existing prompt flow.

#### Scenario: Tiny prompt retains the draft

- **GIVEN** a long Unicode query and selected candidate
- **WHEN** the completion panel cannot fit candidate rows after a resize
- **THEN** the query and cancellation remain usable within available geometry
- **AND** growing the terminal restores a coherent candidate list and selection.

#### Scenario: Repeat a highlighted recent command

- **GIVEN** the latest named command is highlighted first after reopening M-x
- **WHEN** the user presses Return with an empty query
- **THEN** that command invokes its existing handler once without requiring Tab
- **AND** opening or canceling the selector alone does not invoke it.

## ADDED Requirements

### Requirement: CMR01 Rank recent commands before frequent commands

The selector SHALL implement the approved four/four ordering from the bounded
usage window. Recent entries SHALL be distinct and SHALL precede the frequent
entries even when a frequent entry has a larger count. Promoted entries SHALL
be removed from later groups. Ranking SHALL refer to currently admitted command
identities, independently of clipped labels or visible row numbers.

#### Scenario: Reopen after using a command

- **GIVEN** `find-file` was the latest accepted named-command invocation
- **WHEN** the user reopens M-x with an empty query
- **THEN** `find-file` is the first candidate, ahead of alphabetical entries
- **AND** opening and canceling the selector does not change that order.

#### Scenario: Recent and frequent entries overlap

- **GIVEN** the four most recent distinct names are `find-file`, `bookmark-jump`,
  `toggle-file-tree` and `org-indent-enable`, in that order
- **AND** `ai-explain` has the largest frequency among other used commands
- **WHEN** the user opens an empty selector
- **THEN** the four recent names lead in that order, followed by `ai-explain`
- **AND** no promoted name appears a second time in the remaining candidates.

### Requirement: CMR02 Preserve query relevance and selection stability

Text matching SHALL precede usage ranking as defined in Ordering.
Ranking SHALL use one usage snapshot for the open selector. Background state
publication or another editor instance SHALL NOT move a candidate beneath an
already selected row. Editing the query MAY recompute the candidate list using
that snapshot, preserving the existing selection-reset and Tab behavior.
Resize SHALL change geometry without changing candidate identity or ranking.

#### Scenario: Exact name beats a familiar prefix match

- **GIVEN** `org-indent-enable` was used often and `org-indent-mode` was not
- **WHEN** the user types the exact query `org-indent-mode`
- **THEN** `org-indent-mode` is the first candidate
- **AND** no usage preference introduces a nonmatching command.

#### Scenario: Stronger match outranks frequent substring match

- **GIVEN** `prefix-then-substring` matching and a frequent substring-only match
- **WHEN** the query also has a never-used prefix match
- **THEN** the prefix match precedes the substring-only match
- **AND** ties within each match rank use recency, frequency and name.

#### Scenario: A selected command survives external usage changes

- **GIVEN** an open selector with an explicitly navigated candidate
- **WHEN** another instance publishes usage or the terminal is resized
- **THEN** the selected identity remains the command the user chose
- **AND** Return dispatches that identity once.

### Requirement: CMR03 Observe explicit named invocations only

rev SHALL record one usage event when a known command accepted through M-x or
the modal colon selector is handed to the shared dispatcher. Canonical command
names SHALL identify events, including aliases that resolve to the same native
command. Registered AI commands SHALL use their admitted canonical names.

Opening the selector, typing, Tab completion, history recall, candidate movement,
unknown names and cancellation before dispatch SHALL NOT record usage. Direct
keybindings, typing, mouse actions and internal handler calls SHALL NOT inflate
the named-command statistics. The command itself may open a further prompt or
report an ordinary failure; recording its explicit invocation SHALL NOT depend
on asynchronous completion or success of that later operation.

#### Scenario: Completion is not invocation

- **GIVEN** the user completes a candidate with Tab
- **WHEN** the user cancels before accepting it
- **THEN** usage order and counts remain unchanged.

#### Scenario: Later argument cancellation

- **GIVEN** the user accepts `find-file` through M-x
- **WHEN** its path prompt opens and is later canceled
- **THEN** `find-file` has one usage event
- **AND** neither the path draft nor the cancellation is persisted as usage.

### Requirement: CMR04 Persist a bounded user-wide usage window

The normal executable SHALL load and retain at most 256 usage events using the
state path and versioned TOML contract in the model. The most recent events
SHALL displace the oldest; repeated names SHALL remain separate events for
frequency. This is user-wide state, shared across files, directories and both
editing profiles, without project registration or a workspace boundary.

Successfully published usage SHALL survive ordinary shutdown and restart.
File reads and publication SHALL occur through a bounded state owner, never
from paint, query filtering or per-keystroke input handling. Updates MAY be
coalesced and SHALL receive a final bounded publication attempt on ordinary
shutdown. A crash or forced termination need not preserve unflushed events.

Concurrent publishers SHALL use nonblocking serialization and rebase their
pending events onto the latest valid disk window before atomic replacement.
One instance SHALL NOT simply overwrite another instance's newly saved window
with its startup snapshot. Within a batch, invocation order is preserved;
concurrent batches are ordered by successful publication, without wall-clock
timestamps. An open selector retains CMR02's snapshot.

#### Scenario: Restart remembers the last command

- **GIVEN** `toggle-file-tree` was the latest named invocation and state was saved
- **WHEN** a new rev instance starts and opens an empty M-x
- **THEN** `toggle-file-tree` is first without requiring configuration.

#### Scenario: Old frequency loses influence

- **GIVEN** a command was frequent only in events older than the latest 256
- **WHEN** the window advances and that command has no remaining event
- **THEN** its former count does not promote it.

#### Scenario: Two instances publish changes

- **GIVEN** two instances loaded the same state and accept different commands
- **WHEN** their pending batches are successfully published in sequence
- **THEN** the later publisher retains the earlier published events within the
  256-event limit and appends its own pending events once
- **AND** publication does not require a persistent service or filesystem watcher.

### Requirement: CMR05 Treat unavailable history as a nonfatal state failure

Missing state SHALL produce an empty usage window and ordinary alphabetical
fallback. An unreadable, malformed, oversized or unsupported-version file
SHALL NOT prevent editing or command invocation. rev SHALL report a clear
non-modal diagnostic, use session-local usage and preserve the invalid file
without overwriting it. Unknown but structurally valid command names SHALL NOT
become commands or appear in candidates.

A busy publication lock SHALL defer publication without blocking input or
spinning. Subsequent owned idle work or ordinary shutdown MAY retry; no periodic
polling is needed. Other publication failures SHALL preserve in-memory ordering,
leave a complete record rather than a partially written file, and report a
diagnostic without blocking shutdown.
Resolving roots or merely opening the selector SHALL NOT create a state file.

#### Scenario: Corrupt history does not disable M-x

- **GIVEN** the usage file is malformed
- **WHEN** the user opens M-x and invokes `find-file`
- **THEN** ordinary command invocation works and the session learns that usage
- **AND** the malformed file remains unchanged with a state diagnostic.

#### Scenario: A removed command remains in old state

- **GIVEN** a valid saved name no longer resolves in the current command registry
- **WHEN** M-x ranks the current candidates
- **THEN** that name does not appear or execute
- **AND** other valid usage remains usable.

## Documentation and acceptance

Implementation shall document ranking and the unchanged controls in the command
reference, the focused-prompt reference and the existing User Guide chapter on
search and commands. The defaults reference shall distinguish the persisted
256-event window from the configured distinct session-history limit. Add the
state path, bounded retention and failure behavior to reference documentation;
project the Markdown guide into Spore without introducing another source.

Acceptance shall cover blank reopening, fewer than eight used names, duplicate
removal, frequency/recency ties, all match policies, exact-match precedence,
window eviction, aliases, registered/removed AI names, explicit versus bound
invocations, both profiles, current Tab/Enter/cancel/history behavior, resize,
startup/restart, invalid state, bounded reads and concurrent publication.
Use native deterministic cases for the ranking matrix and representative
terminal journeys for actual input and restart behavior.

The approved scope is the four/four policy, fixed usage window, bounded state
publication and the existing selector controls. No new command vocabulary or
settings are needed to use the default behavior.

## Completed delivery evidence

Three canonical slices through PRs #291, #292 and #293 implement UI02 and CMR01-CMR05.
The controller passed its unchanged full gate at `fea4fffdfa3e66d8a7d7f1cc031a6637199424af`,
receipt `342LB2K3MYK2Z2JS5ESVI4WSNP`: 503 selected tests, native frame/surface
ranking, both-profile input/render/restart journeys, paired real publication,
complete catalogs and the local Markdown-derived Spore guide build.
The [Slice 3 closure](../report/slices/rev-command-ranking/slice-3-acceptance.md#delivery-set-closure) records the exact command,
immutable report introductions, stage/resource observations and evidence limits.
The interaction and configuration baselines were reconciled before that gate;
their earlier validated candidates and historical results remain unchanged.
The separate keybinding-hint ticket, main promotion and publication retain their
existing boundaries.
