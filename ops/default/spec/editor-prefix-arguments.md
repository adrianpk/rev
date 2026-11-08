# Command Prefix Arguments

Status: Completed
Completed: 2026-11-08
Date: 2026-11-07
Updated: 2026-11-08
Format: OpenSpec capability addition
Inspection baseline: `2bde2d1b1cccdaf536c17fbd86938eb42cbfc896`
Approved: product owner approved this specification, operational placement, plan and slicing on 2026-11-08
Execution order: first; Keyboard Macros is a separate future consumer
Plan: [Command Prefix Arguments Delivery](../plan/rev-prefix-arguments.md)
Tracker: [Command Prefix Arguments Tracker](../tracker/rev-prefix-arguments.md)
Baselines: [Bindings](../../../docs/reference/bindings.md), [Commands](../../../docs/reference/commands.md), [Shell Commands](../../../docs/reference/shell-commands.md)
Concern: [Pending Editor Work](../README.md)

## Purpose

Supply an argument before invoking a command through familiar Emacs gestures.
Commands interpret it according to their behavior. A movement distance, an
earlier kill-ring entry and an alternate shell operation are different meanings.
Do not turn the dispatcher into a loop that executes every command N times.

The product term is Command Prefix Arguments. The canonical action is
prefix-argument; universal-argument is a compatibility alias, with the same
description, binding identity and handler. Native Rust ownership and values
do not require Lisp representations or Emacs internal types.

## Inspection and compatibility

Rev currently has 624 compiled command names, complete descriptions, native
modal counts and isolated C-u routes. The source editor uses grapheme-aware
motion, pane-width soft wrapping, native edit/history publication and an
existing deferred navigation owner. Reuse those contracts.

The owner approved the behavior below and this set's plan. Macro recording,
its TOML library and storage layout remain a separate draft. This set creates
no persistent argument state and does not implement macros.

Published documentation supplies interaction evidence, not implementation code:

- [Numeric arguments](https://www.gnu.org/software/emacs/manual/html_node/emacs/Arguments.html)
- [Movement](https://www.gnu.org/software/emacs/manual/html_node/emacs/Moving-Point.html)
- [Killing by lines](https://www.gnu.org/software/emacs/manual/html_node/emacs/Killing-by-Lines.html)
- [Yanking](https://www.gnu.org/software/emacs/manual/html_node/emacs/Yanking.html)
- [Earlier kills](https://www.gnu.org/software/emacs/manual/html_node/emacs/Earlier-Kills.html)
- [Scrolling](https://www.gnu.org/software/emacs/manual/html_node/emacs/Scrolling.html)

Exact rev policies are defined here. Existing grapheme/visual-row units,
buffer-local undo/redo and literal incremental search remain rev behavior;
this addition does not claim every prefix variation of the reference editor.

## Entry and command identity

| Input | Meaning |
| --- | --- |
| C-u | Begin a plain prefix, numerically four |
| C-u C-u | Extend the plain prefix, numerically sixteen |
| C-u 5 | Explicit integer five |
| M-5 | Begin an explicit integer; following ordinary or Meta digits extend it |
| C-u - 3 or M-- 3 | Explicit integer minus three |
| M-- | Negative-only prefix, numerically minus one |
| C-u 6 4 C-u 1 | Terminate numeric entry, then insert the digit 1 sixty-four times |
| C-g | Discard the pending argument and its pending invocation |
| Argument followed by M-x | Supply the argument to the accepted command |

After explicit numeric entry, C-u terminates entry rather than multiplying
that integer. While a plain prefix is still being extended, another C-u
multiplies it by four. Decimal digits do not enter source until entry is
terminated. Sign changes use checked arithmetic and preserve the distinction
between negative-only and explicit integers.

| Action | Description |
| --- | --- |
| prefix-argument | Set an argument for the next command. |
| universal-argument | Alias of prefix-argument; shares its description. |
| digit-argument | Add a digit to the pending command argument. |
| negative-argument | Set the sign of the pending command argument. |

Default editor bindings are C-u, M-0 through M-9, and M-- respectively.
A digit binding supplies its digit. Named digit-argument uses a normal one-digit
prompt and returns to the originating workspace role with the updated argument.
Named invocation, complete descriptions, effective hints and configured
remapping use the existing registry. Aliases are one semantic action.

The echo area exposes the pending form/value. Drawing, resize and command
candidate ranking neither execute nor consume it. Incomplete C-x or Meta
sequences retain it. Terminal decoding happens before argument parsing.

## Transient value and invocation ownership

Retain four forms: absent, plain-prefix with depth, explicit signed integer,
and negative-only. A plain C-u and explicit numeric four remain distinguishable.
Numeric interpretation alone must not discard that information.

The input owner constructs one value. A resolved invocation captures it with
its originating role and normal target identity. Prefix-control actions can
extend or terminate it; the next ordinary accepted command consumes it once.
A later physical repeat does not inherit an already consumed argument.

M-x carries the value through completion and accepted command-name resolution.
Its query edits, arrows, Tab and Return do not become counted source operations.
Cancelling, an unknown command, a refused invocation, context departure or an
invalid key sequence retires that invocation and value. A subsequent ordinary
command must not inherit stale input. Accepted named usage is counted once,
not once per internal movement/edit step.

Expose the same typed invocation value to the future macro owner. This is a
native in-memory contract, with no serialization, state directory or new config
preference in this set.

## Complete policy at the command boundary

The following cases opt into argument meaning. All other existing compiled or
dynamic commands execute once with their current behavior and consume the
argument without a numeric transformation. This explicit one-shot default
covers the rest of the 624-name baseline; it never repeats file operations,
Git actions, process launches, AI requests, saves, quit or configuration changes.

Resolve contextual aliases and input ownership before applying a policy.
A key that means Org visibility or a terminal key is not a counted text
insertion merely because the source editor uses the same key.

### Movement and positioning

| Resolved action | Argument contract |
| --- | --- |
| forward-char, backward-char | Signed number of existing native grapheme steps; negative reverses; zero leaves point |
| forward-word, backward-word | Signed word distance through existing word boundaries |
| next-line, previous-line in source editing | Signed distance in the existing ordinary visual/logical row units; preserve desired column |
| move-beginning-of-line, move-end-of-line | Move N-1 ordinary row steps, then choose that row's existing start/end; absent means current row |
| beginning-of-buffer, end-of-buffer | Absent retains current boundary action; a supplied numeric interpretation chooses N tenths from that end, clamped to buffer bounds and admitted source boundaries |
| scroll-up-command, scroll-down-command | Absent retains page behavior; supplied N is a signed row distance, not N full pages |
| recenter-top-bottom | Absent retains cycling; supplied nonnegative N aligns point's row N rows below the content top, negative N counts from the bottom; clamp only to pane geometry |
| goto-line | Positive supplied integer selects that one-based logical line directly; no argument retains the prompt; invalid/nonpositive input refuses without moving |
| tree-next-line, tree-previous-line | Signed loaded-row distance, including the parent row; clamp at endpoints without expansion or IO |

Large movement stops at native boundaries rather than repeatedly testing a
clamped point. Word/grapheme, source revision, wrapped-row and pane-size
admission remain owned by existing native motion/layout code. A derived
fractional buffer position must be snapped to a valid existing source boundary
without normalizing or rewriting text.

Special-buffer actions not listed here retain their current local one-shot
behavior. No numeric argument turns tree-up-directory into recursive traversal.

### Editing, kill ring and history

| Resolved action | Argument contract |
| --- | --- |
| Ordinary literal character or admitted paste | Insert N copies for nonnegative N; zero does nothing; negative refuses |
| newline, insert-tab as ordinary source edits | Perform N native newline/indentation steps as one prepared edit; zero does nothing; negative refuses |
| delete-char, backward-delete-char | Signed native grapheme distance with reversed direction for negative; preserve current region precedence |
| kill-word, backward-kill-word | Signed word span, published as one directional kill; zero makes no edit or ring entry |
| kill-line | Absent retains current to-end/at-end behavior; positive N includes N logical line endings where present; zero kills to current logical line start; negative kills backward to the start of the abs(N)th preceding logical line |
| yank | Absent selects newest; positive integer N selects the Nth newest entry once; plain-prefix selects newest and exchanges the resulting point/mark orientation; nonpositive numeric values refuse |
| yank-pop | Signed rotation from the current yank through the existing bounded ring; zero leaves it unchanged; retain the exact checked insertion range |
| undo, undo-redo | Nonnegative N existing history groups; zero is inert; negative refuses; absent remains one group |

Literal insertion replaces an admitted active region once, then inserts the
complete repeated result. Counted deletion must preserve existing region
precedence rather than deleting that region repeatedly and continuing outside
it. Counted edits use one checked source preparation/publication and one undo
unit, separated from neighboring ordinary typing. They must not trigger
completion or structural observation once per inserted character.

A counted kill publishes one combined span with the existing append/prepend
chain semantics. Prefix input itself does not break a chain that the accepted
command would ordinarily continue.

Undo/redo continues to apply native existing history groups. It does not create
a new coalesced edit group, add a synthetic undo entry or introduce selective
region-only undo. At a history boundary, retain completed native group replays
and report the stop; cancellation does not claim transaction rollback.

### Existing variants and non-source owners

- Preserve C-u M-! and C-u M-| through ordinary shell insertion/filter owners.
  A prefix reaching shell-command or shell-command-on-region through a named
  invocation selects the same output-application variant once, not N jobs.
- Preserve C-u C-c C-v t for the existing single-block tangle action and
  C-u K in its current habit/agenda context.
- Existing custom continuations beginning with C-u retain active-map precedence.
  A dual-role prefix must coexist with those continuations without an extra
  confirmation key or an arbitrary timeout on ordinary count entry.
- Modal Normal/Visual C-u remains its current half-page command. Plain modal
  digits, operators and repeat retain their native count owner. Where an
  explicitly invoked editor prefix is combined with an already present modal
  count, refuse the ambiguous combination; do not multiply it silently.
- In modal Insert/Replace source editing, inherited editor prefix gestures
  follow ordinary editor admission. Modal Escape cancels pending prefix input
  through existing escape ownership; non-modal Escape keeps Meta decoding.
- Raw PTY input, argument prompts, incremental search editing and completion
  popup navigation keep their existing owners. A prefix supplied before M-x
  belongs to the selected command, not those transient prompt actions.
- Literal incremental search, query replacement, rectangle operations, Org
  structural actions, bookmark actions and Git/AI jobs remain one-shot unless
  one of the explicit contracts above applies. No regex/search language,
  rectangle repeat engine or new external execution policy is introduced.

## Bounds and cooperative execution

Numeric magnitude is at most 1,000,000. Check every digit, sign conversion and
plain-prefix multiplication before publication; reject overflow/over-limit input
with a diagnostic and clear the pending invocation. Do not clamp a requested
number silently. Keep entry storage constant-size, including leading zeroes.

A counted insertion can materialize at most 8 MiB of new UTF-8 text per
invocation, additionally respecting existing source/input admission. Check
multiplication and aggregate byte counts before allocation. Refuse an
oversized candidate without partial source mutation.

Count-aware traversal/history work services cancellation at least every 256
native steps and yields when its existing work budget is spent; use a 4 ms
cooperative target between primitives. This is not a hard real-time guarantee
inside one native atomic operation. Reuse the existing bounded deferred-layout
and input lifecycle; no permanent count worker or idle polling is needed.

While planning a large edit, source/context changes revoke the captured target.
Cancellation before publication leaves source/ring/history unchanged. A
successfully published atomic edit remains one undoable result. Native history
group replay is the explicit progressive exception described above.

## ADDED Requirements

### Requirement: ARG01 Construct checked prefix input

The editor SHALL accept the entry forms, numeric bound, digit termination,
sign and cancellation behavior above while exposing the pending value.

#### Scenario: Digits remain input state

- **GIVEN** C-u 6 4 followed by terminating C-u
- **WHEN** the user types 1
- **THEN** sixty-four copies are inserted through ordinary editing
- **AND** no argument-entry digit was inserted into source.

#### Scenario: Overflow or cancellation

- **GIVEN** pending input or an over-limit multiplication
- **WHEN** admission fails or the user cancels
- **THEN** the pending invocation is retired without wrapped arithmetic, a
  stale argument or a source edit.

### Requirement: ARG02 Carry one typed value through named invocation

The editor SHALL retain argument form and originating role through M-x/colon
selection, then consume the value exactly once at the resolved command boundary.

#### Scenario: Named movement and prompt cancellation

- **GIVEN** argument five followed by M-x forward-char
- **WHEN** the name is accepted
- **THEN** one invocation moves five native steps
- **AND** cancelling instead leaves the next command without that argument.

### Requirement: ARG03 Apply the complete command-policy table

The editor SHALL implement the explicit count, positioning, kill/history and
variant rules, with a one-shot default for all remaining actions. Native
read-only, region and source-revision checks SHALL remain authoritative.

#### Scenario: Counted kill versus repeated command

- **GIVEN** point in the middle of a logical line
- **WHEN** kill-line receives explicit one
- **THEN** it includes the first line ending where present in one kill
- **AND** ordinary no-argument kill-line retains its existing behavior.

#### Scenario: Select an earlier kill

- **GIVEN** three retained kill-ring entries
- **WHEN** the user invokes C-u 3 C-y
- **THEN** the third entry is inserted once, not the newest entry three times.

#### Scenario: One-shot command

- **GIVEN** a numeric prefix and a command without declared numeric meaning
- **WHEN** it is accepted
- **THEN** its current operation runs once and consumes the argument.

### Requirement: ARG04 Preserve contextual ownership and modal counts

The editor SHALL resolve active context, existing C-u continuations and native
modal/terminal ownership before applying editor prefix behavior.

#### Scenario: Existing variants and PTY input

- **GIVEN** an existing shell/Org prefix route, or terminal input ownership
- **WHEN** its current keys arrive
- **THEN** the existing operation or PTY bytes retain their owner
- **AND** no unrelated repeated action is introduced.

#### Scenario: Modal count conflict

- **GIVEN** both a pending editor prefix and a native modal count
- **WHEN** the next command would receive both
- **THEN** it refuses that ambiguity and clears it without applying a multiplied
  operation or changing the source.

### Requirement: ARG05 Bound work and preserve atomic edit publication

Count-aware work SHALL satisfy the numeric, generated-byte and cooperative
limits above. Source edits SHALL publish once through existing edit/history
owners; cancelled or stale preparation SHALL not partially mutate source.

#### Scenario: Large insertion or movement

- **GIVEN** a large admitted number, or an oversized generated-text product
- **WHEN** execution proceeds or rejects admission
- **THEN** cancellation remains serviced and oversized text is refused before
  allocation/publication
- **AND** idle sessions acquire no permanent work deadline.

### Requirement: ARG06 Keep discovery and future reuse coherent

The editor SHALL expose all four names with complete canonical descriptions,
effective bindings, correct alias usage identity and remapping. Its native typed
argument contract SHALL be available to a future macro owner without adding
persistence or delivering macros in this set.

#### Scenario: Helper discovery and restart

- **GIVEN** omitted configuration and the ordinary command selector
- **WHEN** the user invokes a prefix helper by name or its effective key
- **THEN** both routes provide the same behavior and description
- **AND** restart has no pending argument to restore.

## Validation and guidance

Extend the existing native Rust fixtures rather than adding a language,
dependency or separate harness. Validate literal input, all table rows,
prefix/alias/remapping entry, M-x transfer, region/read-only cases, Unicode/CRLF,
both profiles, existing C-u routes, cancellation, revision invalidation,
history/kill-ring behavior, bounds and idle resource behavior.

Keep the User Guide's first example to C-u 5 C-n. Explain distinct meanings
through a few concrete tasks; place complete policy/bounds and raw-form details
in reference material. Markdown is the source for the Spore projection.

## Delivery completion

ARG01-ARG06 are delivered through canonical PRs #314-#317. The unchanged
controller gate passed on clean immutable dev candidate
`7cfef9be78fad975040ccd78dcb9cfebe32b28cd`, receipt
`ZGGKKSNDW2ZMWGRELYH5WWFTZV`. The [Slice 4 closure](../report/slices/rev-prefix-arguments/slice-4-acceptance.md#delivery-set-closure)
records joint native acceptance, exact commands, catalogs and local User Guide
validation. This status update preserves every approved requirement, scenario,
command policy and resource bound. Keyboard Macros retains its separate draft
and approval boundary.
