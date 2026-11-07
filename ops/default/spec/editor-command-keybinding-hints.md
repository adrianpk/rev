# Command Keybinding Hints

Status: Completed
Completed: 2026-11-06
Validated candidate: `ca39b8f14e03d3753f071811b1151b8299bc3560`
Gate validation: `BV6KPQKMFAI37JMO2HBOQOISIM`
Date: 2026-11-06
Approved: product owner; proceed with the established delivery process on 2026-11-06
Format: OpenSpec capability addition
Source: [TKT-20261031095151](../ticket/solved/20261031095151-show-keybindings-beside-m-x-commands.md); owner requested a formal specification before implementation
Baselines: [Editor Interaction](editor-interaction.md), UI02; [Command Recency and Frequency](editor-command-ranking.md); [Configuration](editor-configuration.md); [Modal Editing](editor-modal-editing.md)
Current reference: [Commands](../../../docs/reference/commands.md), [Bindings](../../../docs/reference/bindings.md), [Focused Prompts](../../../docs/reference/prompts.md)
Plan: [Command Keybinding Hints Delivery](../plan/rev-command-keybinding-hints.md)
Tracker: [Command Keybinding Hints Tracker](../tracker/rev-command-keybinding-hints.md)
Presentation revision: owner-approved direct-dev correction on 2026-11-06; the candidate/receipt above describe the original two-slice delivery
Description authoring: [TKT-20261106202027](../ticket/ready/20261106202027-complete-command-catalog-descriptions.md) and [Command Descriptions](editor-command-descriptions.md), approved separately from the delivered layout

## Purpose

Show a command's effective shortcut beside its name in M-x so users can learn
the binding while discovering and invoking the command. Apply the same
presentation to the shared modal colon selector.

The annotation describes a binding that reaches that command from the context
where the selector was opened. It is derived from admitted runtime keymaps,
including configured overrides and local precedence, rather than a second
handwritten shortcut catalog.

This specification formalizes the existing ticket. The owner's continuation
approves delivery preparation and implementation through the established
plan, tracker and Looper workflow.

## Planning baseline and ownership

`src/display/completion.rs` paints one canonical command name per candidate row.
`src/input/palette.rs` owns query matching, recent/frequent ranking, selection,
completion and accepted command identity. The effective maps and normalized
sequences live under `src/input/keymap/`; focused input owners decide which map
and handlers apply.

At the approved planning baseline, the selector displayed no shortcut
annotation. Looking up the active
minibuffer map after opening it would describe selector controls instead of
bindings available in the originating editor, Org, Git or tree context. Capture
the originating dispatch context before the selector takes focus.

Hints are transient presentation data keyed by command identity. They introduce
no persistent model, state file, history field, configuration leaf or binding.
The existing command-usage model remains unchanged.

## Presentation

Keep command names at their current left edge. Use three fields: name, nearby
left-aligned shortcut and optional description. Derive shared column starts
from the visible candidate block, not from the panel's right edge. Separate
fields by at least two blank cells. With factory non-modal bindings:

```text
save-buffer  [C-x C-s]
```

An unbound command leaves its shortcut field blank. Missing descriptions leave
the description field blank, without dashes or other placeholders. Catalog
summaries are optional metadata; writing every summary is the separate linked
ticket, not a prerequisite for this layout. An annotation is not part of a name.
Use the project's existing readable key notation, such as `C-x C-f` and `M-%`,
with normalized terminal aliases represented once.

When a command has multiple reachable bindings, display one: prefer the
sequence with the fewest normalized chords, then use stable lexical ordering
of the canonical formatted sequence. This is the deterministic selection policy,
not a new configurable preference. One displayed hint does not remove other
valid bindings.

Command-name readability has priority. Clip descriptions at terminal-cell
boundaries and omit them when their column cannot fit. In narrow geometry,
retain a complete bracketed shortcut immediately after its own name when the
shared column cannot fit it. If the complete name, gap and bracketed hint still
cannot fit, omit the hint and retain ordinary name clipping. Never wrap a
candidate or truncate a shortcut into a different-looking sequence.

Use existing palette roles: `palette.syntax.function` for unselected command
names, `palette.status_text` for shortcuts and descriptions, and existing
selection foreground/background across the selected row. The function accent
applies to command names independently of source syntax highlighting. These
roles make names and shortcuts visibly distinct with rev's factory palette.
Under `NO_COLOR`, separation and selection remain readable without foreground
color. No new palette schema is required.

## ADDED Requirements

### Requirement: KBH01 Describe effective bindings from the originating context

M-x and the modal colon selector SHALL annotate a candidate when a reachable
key sequence resolves to that same canonical command in the originating
context. Discovery SHALL respect the active editing profile and state,
context-local maps, inherited maps, user overrides, shadowed leaves/prefixes
and focused input ownership through the same resolution rules as dispatch.

The context SHALL be captured before command entry activates the minibuffer.
The annotation SHALL NOT describe how to navigate the selector itself. A
binding whose input is consumed by another owner SHALL NOT be advertised as an
editor shortcut. Binding reachability describes dispatch, not a guarantee that
the command can complete on a read-only buffer or unavailable external resource.

#### Scenario: Factory shortcut

- **GIVEN** a non-modal editing view with factory bindings
- **WHEN** M-x displays `save-buffer`
- **THEN** its row shows `C-x C-s`
- **AND** that sequence resolves to `save-buffer` in the originating view.

#### Scenario: Configured replacement

- **GIVEN** valid configuration rebinds `C-x C-s` to a different command
- **AND** `C-c s` is the only effective shortcut for `save-buffer`
- **WHEN** the selector displays `save-buffer`
- **THEN** the hint is `C-c s`, not the displaced factory shortcut.

#### Scenario: Local input shadows an editor binding

- **GIVEN** the tree owns input and a chord expands a directory there instead
  of invoking its ordinary editing motion
- **WHEN** M-x is opened from that tree
- **THEN** the chord is not advertised beside the shadowed editing command
- **AND** a valid local command may show it through the same resolution rules.

### Requirement: KBH02 Select one normalized hint without hiding commands

Hint discovery SHALL use currently admitted command identities and normalized
key sequences. Equivalent terminal spellings SHALL not create duplicate hints.
Multiple reachable sequences SHALL use the shortest-then-lexical
policy, independently of map iteration order.

A command without a reachable binding SHALL remain a normal candidate with no
shortcut annotation. Its optional description retains the same metadata rule.
This includes registered AI commands without configured shortcuts.
Do not invent a binding, infer one from a similar command's effects or label
invocation through M-x itself as that command's shortcut.

#### Scenario: Multiple bindings

- **GIVEN** two different effective sequences reach the same command
- **WHEN** the selector is opened repeatedly with unchanged context and maps
- **THEN** it displays the same primary hint, chosen by the defined policy
- **AND** both original bindings retain their dispatch behavior.

#### Scenario: Unbound or user-defined command

- **GIVEN** an admitted command has no binding in the originating context
- **WHEN** the user filters for it and accepts it
- **THEN** its row contains its name without a fabricated shortcut
- **AND** ordinary named invocation remains available.

### Requirement: KBH03 Preserve command matching, ranking and identity

Shortcut hints and descriptions SHALL remain separate from canonical names. Matching,
exact-name admission, the four-recent/four-frequent ordering, candidate selection,
Tab completion and Return dispatch SHALL continue to use command identities and
names only. Query text SHALL NOT match a candidate solely through its hint or description.

Annotations SHALL follow their command through filtering and scrolling rather
than attaching to a prior row index. Tab SHALL insert only the canonical name.
An accepted invocation SHALL enter command usage exactly once under its normal
identity; hints add no usage events or persisted text. Existing cancellation,
Visual-selection ownership and argument prompts SHALL remain unchanged.

#### Scenario: Complete a bound candidate

- **GIVEN** the selected row contains a command name and shortcut annotation
- **WHEN** the user presses Tab and then Return
- **THEN** the prompt receives only the command name and dispatches it once
- **AND** usage contains no shortcut label or additional invocation.

#### Scenario: Filtering changes the visible rows

- **GIVEN** several annotated candidates in recent/frequent order
- **WHEN** the query removes some entries and the user scrolls the result
- **THEN** each remaining hint still belongs to its own command
- **AND** hint text does not change matching or usage ranking.

### Requirement: KBH04 Keep a single readable row under clipping and resize

Each candidate SHALL retain one row within the existing candidate panel and
`minibuffer.candidate_limit`. Name, shortcut and description SHALL use shared
nearby columns, terminal-cell geometry and existing palette/selection ownership.
The shortcut SHALL be bracketed and left-aligned near its name rather than
attached to the panel's right edge. It is shown only when its complete sequence
fits under the presentation policy.

Narrow or zero geometry SHALL preserve query, command identity and cancellation.
Growing the terminal SHALL restore complete hints when they fit, without
changing ranking or dispatch. Where existing pointer selection supports a
candidate row, the hint field SHALL refer to that same row identity rather
than execute the displayed shortcut as a separate action.

#### Scenario: Hint does not fit

- **GIVEN** a bound command and a candidate area too narrow for both fields
- **WHEN** the row is painted
- **THEN** the complete hint is omitted and the name uses existing clipping
- **AND** the candidate remains selectable and invocable.

#### Scenario: Selection without color

- **GIVEN** `NO_COLOR` and an annotated selected row
- **WHEN** the user moves between candidates or resizes the terminal
- **THEN** the name and hint remain distinguishable by layout when they fit
- **AND** existing reverse-video selection remains readable across the row.

### Requirement: KBH05 Prepare bounded hints without a second dispatch system

Discovery SHALL inspect only finite admitted maps and their existing normalized
sequences, whose current bound is eight chords. It SHALL NOT enumerate arbitrary
key combinations, numeric arguments or possible editor interactions. Prepare
hints once for the captured selector context and reuse them during filtering
and painting. Rendering SHALL perform no filesystem reads or command execution.

Effective keymap resolution remains the authority; implementation may expose
read-only introspection from that owner, but SHALL NOT maintain a parallel table
of default shortcut strings. A changed origin SHALL follow the existing input
owner's retirement rules rather than silently advertise another context's
bindings. Reopening the selector SHALL observe the current valid origin and
admitted registered commands.

#### Scenario: Both editing profiles

- **GIVEN** the same named command is available in non-modal editing and in a
  modal state with different effective bindings
- **WHEN** the shared selector is opened from each origin
- **THEN** each displays a valid hint for its own origin
- **AND** command entry does not substitute minibuffer bindings for either.

#### Scenario: Painting retains the prepared annotation

- **GIVEN** an open selector with a selected command and prepared hint
- **WHEN** frames repaint, usage is published or the terminal is resized
- **THEN** the annotation remains associated with the captured command/context
- **AND** repainting performs no binding mutation, command execution or I/O.

### Requirement: KBH06 Consume optional descriptions without placeholder text

The selector SHALL consume optional single-line catalog descriptions without
duplicating command identity or dispatch. Missing/empty descriptions SHALL
leave the field blank, with no dash, placeholder sentence or inferred text.
Description metadata SHALL not enter filtering, prompt completion or usage.
Catalog-wide summary authorship remains the separate linked ticket.

#### Scenario: Available description

- **GIVEN** a candidate has a description in its command metadata
- **WHEN** the row has room for its description column
- **THEN** it is displayed and clipped only at complete terminal-cell boundaries
- **AND** name, shortcut and description retain one selection identity.

#### Scenario: Missing description

- **GIVEN** a candidate has no nonempty description
- **WHEN** its row is painted
- **THEN** the description field stays blank without placeholder marks
- **AND** the name, effective shortcut and named invocation remain available.

## Documentation and acceptance

Document hints and primary-binding selection in the command reference and the
existing search-and-commands User Guide chapter. Include one short annotated
row and clarify that an absent hint means no shortcut in that origin, while
named invocation remains available. Keep the Spore guide projected from the
Markdown source. Existing binding/configuration references remain authoritative.

Acceptance shall cover factory and configured maps, inherited and shadowed
bindings, local Org/Git/tree contexts, input-owning terminal views, both editing
profiles and modal states, multiple sequences, normalized Meta aliases, unbound
and registered AI commands, and context capture before minibuffer activation.
Compare hinted sequences with actual dispatch through the existing resolver.

Also verify query/ranking independence, Tab/Return, usage identity, scrolling,
selection, Unicode/key-label widths, narrow/zero geometry, resize and NO_COLOR.
Use deterministic native cases for resolution and layout, plus representative
input/render journeys for the shared selector. No new dependency, persisted
model or configuration surface is required by this presentation feature.


## Completed delivery evidence

Canonical PRs #294 and #295 deliver KBH01-KBH05, preserving UI02 and CMR01-CMR05.
The interaction baseline UI02 was reconciled before the gate; its historical
candidate and validation evidence remain unchanged. The controller passed the
frozen full gate at `ca39b8f14e03d3753f071811b1151b8299bc3560`, receipt
`BV6KPQKMFAI37JMO2HBOQOISIM`, with 362 selected tests, native origin/layout/
identity acceptance, both-profile bounded PTY journeys and canonical
Markdown-derived Spore guidance.

The [Slice 2 closure](../report/slices/rev-command-keybinding-hints/slice-2-acceptance.md#delivery-set-closure)
records the exact successful command, immutable report introductions, timings,
resource observations and local evidence limits. The source ticket is solved
with resolution fixed. This closure changes no requirement, runtime, test,
guide, dependency, configuration or persistence assertion.

The owner subsequently requested adjacent shortcut/description columns directly
on dev. The revised Presentation section, KBH03/KBH04 and new KBH06 define that
post-delivery correction. Its validation belongs to the correction commit;
the historical receipt above is not reused as evidence for the revised layout.
