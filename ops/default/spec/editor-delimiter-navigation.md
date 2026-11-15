# Delimiter Navigation

Status: Completed
Completed: 2026-11-15
Approved: product owner, 2026-11-15; continuation to the next ordered delivery set
Plan: [Delimiter Navigation Delivery](../plan/rev-delimiter-navigation.md)
Tracker: [Delimiter Navigation Tracker](../tracker/rev-delimiter-navigation.md)
Date: 2026-11-15
Format: OpenSpec capability specification
Inspection baseline: 0a05c82da5babc23f2fcd58176e35b99823b6c3a
Review scope: explicit mate jumps, structural groups and expression motions
Model: [Pair Preferences and Native State](editor-delimiter-pairs-model.md)
Shared qualification: [Delimiter Pairs](editor-delimiter-pairs.md), DPR04
Related behavior: [Interaction Parity](editor-emacs-interaction-parity.md), EIP04

## Purpose

Navigate source structure independently of automatic edits and color. Retain
DPR02 from the original pair proposal while distinguishing a typed mate jump,
structural group movement and a syntax-aware expression motion.

## Delivery progress

All five slices are integrated through canonical PRs #62-#66. They deliver the
seven commands, six additive base-editor chords and shared bounded source owner.
[Slice 5](../report/slices/rev-delimiter-navigation/slice-5-acceptance.md)
delivers the complete native acceptance batch, two representative Linux PTYs
and final guidance. The controller's unchanged combined gate passed at
`27957128f7626ee16b5dbdc50af52e9cee27f3c8`, receipt
`YEU4CDCDB3RP65D5PG67RRYTYF`, completing DPR02, DPN01 and DPN02. The
[delivery-set closure](../report/slices/rev-delimiter-navigation/slice-5-acceptance.md#delivery-set-closure)
records full-gate evidence and the subsequent documentation-only relationship.
Pair presentation and paragraph parity retain their separate authority.

## Approved commands and endpoints

| Command | Binding | Observable target |
| --- | --- | --- |
| matching-delimiter | Unbound shared command | The other delimiter of the confirmed typed pair at point. |
| forward-pair | C-M-n | Forward across mode-defined list/group structure. |
| backward-pair | C-M-p | Backward across mode-defined list/group structure. |
| up-pair | C-M-u | Backward out through an enclosing opening delimiter. |
| down-pair | C-M-d | Forward into a nested group. |
| forward-sexp | C-M-f | Forward over one mode-defined expression; specializes EIP04. |
| backward-sexp | C-M-b | Backward over one mode-defined expression; specializes EIP04. |

These sequences have no factory leaves at the current baseline. Their addition
is approved for the existing base-editor inheritance, with explicit assignments,
input focus and profile/context ownership preserved.
All commands have canonical named invocation, descriptions and truthful effective
hints. They are usable on read-only source without enabling pairs.enabled or
pair presentation. Terminal/prompt/projection input retains its existing owner.

The direct jump prefers a delimiter at point, then one immediately before point.
It lands on the mate's source boundary. At a delimiter endpoint, jumping twice
returns to that endpoint; adjacent caret positions do not invent an extra round
trip position. With no qualifying delimiter, report that condition instead of
searching later through the line. Missing, mismatched or unconfirmed typed mates
leave point and mark unchanged. The command is non-counted: a supplied numeric prefix is rejected by native
unsupported-argument admission before movement; it never repeats a mate jump.

Group motions follow the declared mode's forward-list/backward-list/
backward-up-list/down-list behavior, including signed counts and normal file
boundaries. Zero is inert. They do not substitute the destination of a symbol,
string or number for a requested group operation. Mode-dependent grouping can
be looser than typed mate matching: its result must not assert a typed pair for
editing or color. Truly uncertain lexical/source qualification refuses before
movement; an ordinary reference end-of-buffer result is not that failure.

The native modal % retains its current-line delimiter search and operator,
selection and jump-history contracts. Existing inner/around pair/quote objects
retain their entry, escape and selected-range behavior. Reuse qualified facts
only where those contracts agree; no silent rebinding or replacement is implied.

Expression motions specialize EIP04 from the interaction-parity spec; that
spec retains paragraph requirements and the broader workflow acceptance. A
delivery of this capability can discharge its expression-motion portion only,
without claiming paragraphs or the rest of functional parity. Expressions include
admitted atoms and strings as well as groups; these are not aliases for mate
jumps or group-only motions.

## Approved profile, region and work contract

Group syntax follows the declared mode's opening/closing syntax classes for
parentheses, brackets and braces. It is not strict mate-type validation: [a) can
have a qualified group endpoint without ever becoming a confirmed typed pair.
Strings and comments are scanned as classified units, not raw punctuation.
Angle brackets have no generic typed/group-mate rule.

| Native profile | Group/expression policy and conservative boundaries |
| --- | --- |
| Plain and unconfigured languages | Standard text syntax for ordinary words/symbols, double strings and ()/[]/{} groups. Preserve the observed text-mode hyphen/underscore/word-apostrophe behavior. Ordinary Lisp-compatible forms in the reference apply; do not claim Lisp reader semantics or invent semicolon comments in unknown text. |
| C | C-mode ordinary syntax-class/group/string/comment behavior, including documented prefix/punctuation and numeric endpoints. A grammar node spanning a decimal is not authority to replace the reference's syntax-class endpoint. |
| Rust and Go | Declared C-family ordinary atom/group behavior plus existing native language quote/comment exclusions. Current native evidence can admit language-specific source units; unproved lifetime/character/raw states retain conservative refusal. This does not install a new external reference mode or claim complete language-package parity. |
| JavaScript, TypeScript and TSX | Script identifier/string/comment classes, including the observed js-mode dollar identifier boundary; current qualified JSX/template/interpolation only, otherwise conservative refusal. |
| JSON | C-family ordinary atom/group behavior and JSON double strings; invalid/unproved quote syntax is not silently accepted as a typed pair. |
| TOML | Ordinary atom/group and declared single/double raw-string classes; unproved triple/multiline source remains conservative. |
| CSS | CSS ordinary atom/string/comment classes, including a hyphenated identifier as one expression. |
| HTML | Qualified attribute strings and confirmed tag-token expression boundaries, as observed for an opening tag. This is not an angle-delimiter mate or whole-element/tag-jump command. Unknown embedded context remains conservative. |
| Org and Markdown | Source-relative prose forms/atoms with current native literal/embedded boundaries. Proved source-edit buffers use their recognized language. Do not guess structure inside unqualified code/verbatim or embedded language. |

The independent reference matrix records exact observed mode/version/input and
endpoint/error/mark state. Mode-specific examples apply through the table above,
not by treating every stock major mode as Plain. The grammar-free Rust/Go/JSON/
TOML fallback is explicit; syntax colors or a highlight capture do not invent an
external language's command grammar. Navigation remains available with editing
and quote preferences disabled; those preferences do not change source syntax.

Preserve the active ordinary linear mark and its anchor on successful group/sexp
movement, including its native empty-region state. Refusal leaves point, mark
and selection unchanged. Do not create a source edit or undo item; retain native
typing barriers and source-motion/jump ownership. The direct mate command uses
the source-jump owner; group/sexp operations use ordinary motion ownership.
Rectangular/block, modal operators and Visual geometry keep their existing owner.

Counts use native bounded signed arguments once. Zero is inert for group/sexp;
negative counts reverse each operation according to the recorded list/up/down/
sexp behavior. Ordinary file-boundary results are admitted. A malformed or
unproved counted destination refuses atomically without an intermediate cursor
publication. A completely inspected known unterminated ordinary C string may
produce the recorded expression-to-EOF boundary; it never proves a typed mate.
Unproved raw/triple/template states are different and remain conservative.

All published targets must be native grapheme boundaries. If a reference scalar
endpoint falls inside a grapheme, normalize in the movement direction to the
nearest native boundary and record that necessary difference in the native case.
Do not split a CRLF pair or combine terminal cells with source scalar positions.

The existing aggregate limits apply: 256 KiB inspected bytes per request, depth
256, 2 MiB retained qualified metadata per buffer, at most 256 lexical steps per
cooperative poll and the native four-millisecond scheduling target. Counts share
one request budget; cached facts remain source/revision-qualified. Cancellation
and captured source/view/point/mark/focus changes retire work before publication.
The plan records an optimized native baseline and the new engine must measure
its actual work/retention/latency before final acceptance.

### Factory and focus matrix

The six approved C-M chords are base-editor additions. Existing inheritance
covers Editor, VimInsert, VimReplace, Markdown, Org, OrgCapture, OrgSourceEdit,
MacroDefinition and GitCommit default maps; existing local overlay/availability
rules remain authoritative. No new C-M leaves are added to VimNormal or Visual
filters, terminal passthrough, Minibuffer/M-x, Tree, Agenda, Calendar or read-only
Git projection maps. Named invocation retains native source/focus admission;
it cannot silently navigate another buffer behind a component.

Canonical source buffers, including owned editable source/commit buffers, use
their admitted source owner. Explicit local/profile/shared assignments keep
existing precedence and truthful effective hints. The original modal percent
and inner/around objects remain unchanged. No new key is assigned to the shared
matching-delimiter command.


## Reference observations

Controlled GNU Emacs 31.1 buffers, without personal init, produced the following
zero-based character positions. Electric pairing was enabled for the companion
editing probes; no automatic edit is part of these motion observations.

| Mode, source and initial point | Sequence | Observed result |
| --- | --- | --- |
| Lisp, (ab) tail, point 0 | C-M-n | Point 4, after the group. |
| Lisp, (ab) tail, point 4 | C-M-p | Point 0, before the group. |
| Lisp, (a (b) c), point 4 | C-M-u | Point 3, on the inner opener. |
| Lisp, (a (b) c), point 0 | C-M-d | Point 1, inside the outer group. |
| Lisp, (a), point 0 | C-u 2 C-M-n | Point 3, normal end of buffer. |
| Lisp, (a), point 3 | C-u -1 C-M-n | Point 0. |
| Lisp, (a), point 1 | C-u 0 C-M-n | Point 1, unchanged. |
| Lisp, (a (b) c), point 4 | C-M-f | Point 5, after symbol b; this is expression motion. |
| Lisp, (a (b) c), point 4 | C-M-n | No next group; point remains 4. |
| C mode, [a), point 0 | C-M-n | Point 3 under that mode's group scan; this is not proof of a confirmed typed [] mate. |

For the approved strict mate jump, (ab) tail at its opener instead targets the
closer's offset 3. That entry policy is the approved Rev contract, distinct from the
reference's group and expression endpoints. Keep malformed-mode cases explicit
rather than silently imposing one language's delimiter rules on all modes.

Reference manuals: [Parenthesis structure](https://www.gnu.org/software/emacs/manual/html_node/emacs/Moving-by-Parens.html)
and [Expressions](https://www.gnu.org/software/emacs/manual/html_node/emacs/Expressions.html).
Batch reference effects are not physical terminal acceptance or an implemented
Rev result. Native paired fixtures and real emitted input remain required.

## ADDED Requirements

### Requirement: DPR02 Navigate matched delimiters in both directions

The shared direct jump SHALL use confirmed typed mate endpoints qualified by
canonical source and revision. It SHALL preserve source/history, point/mark on
refusal, read-only access and the documented endpoint policy.

#### Scenario: Multiline pair

- **GIVEN** an admitted balanced typed pair across multiple logical lines
- **WHEN** matching-delimiter is invoked at either delimiter endpoint
- **THEN** it reaches the opposite delimiter without editing source or selecting a popup row
- **AND** jumping again returns to the original endpoint.

#### Scenario: No pair at point

- **GIVEN** ordinary text at and immediately before point, with a pair later on the same line
- **WHEN** matching-delimiter is invoked
- **THEN** point/mark/source/history remain unchanged and the condition is reported
- **AND** the separate modal % entry contract is unaffected.

### Requirement: DPN01 Preserve motion units, mode syntax and counts

Group motions SHALL follow their declared mode/reference endpoints, signed
counts and normal boundary behavior, independently of sexp motion and typed
mate validation. No group result SHALL become a fabricated typed pair.

#### Scenario: Symbol and group differ

- **GIVEN** (a (b) c) with point at character offset 4 in the admitted Lisp profile
- **WHEN** forward-pair is invoked
- **THEN** it reports no next group and retains point 4
- **AND** the distinct forward-sexp operation reaches 5 under EIP04.

#### Scenario: Bounded request and source authority

- **GIVEN** a stale revision, exhausted scan budget or uncertain lexical profile
- **WHEN** a motion prepares or publishes its destination
- **THEN** it refuses the unproven result without source changes or a stale cursor jump
- **AND** cancellation and subsequent ordinary navigation remain usable.

### Requirement: DPN02 Specialize syntax-aware expression motion

C-M-f/C-M-b SHALL implement the mode-dependent expression endpoints and signed
counts required by EIP04, including atoms, strings and balanced forms. A matching
delimiter offset or group-only search SHALL NOT substitute for an expression.
Source/history and the native mark/region contract SHALL be preserved.

#### Scenario: Forward after the whole form

- **GIVEN** (ab) tail with point at zero in the admitted Lisp-compatible profile
- **WHEN** C-M-f is entered
- **THEN** point reaches 4 rather than the closer's offset 3
- **AND** C-M-b from 4 returns to 0 without a source edit.

#### Scenario: Move past an atom

- **GIVEN** (a (b) c) at offset 4 under the same profile
- **WHEN** C-M-f is entered
- **THEN** point reaches 5 after b
- **AND** the group-only no-next-group case remains a separate accepted result.

## Approval and acceptance

The owner authorized this next ordered delivery after manually accepting editing
on 2026-11-15. The seven command identities, six base chords, profile/entry/count/
region/resource policy above and five behavior-first slices define the approved
scope. The controller integrates slice PRs into dev; main promotion is separate.
Presentation/depth colors follow in their own independently started delivery.

The plan freezes 82 independently reproduced reference observations and native
mate-entry cases before runtime branches. Acceptance covers exact source-unit
endpoints, no-match/malformed/limited refusal, readonly source, active mark,
Unicode/CRLF and cooperative/cancelled/stale requests. Validate complete emitted
input, effective maps/custom precedence, legacy modal behavior, shared editing
contracts and owned native/PTY/frame/resource workflows. Preserve frozen factory
snapshots and old receipts while extending independent current expectations.
