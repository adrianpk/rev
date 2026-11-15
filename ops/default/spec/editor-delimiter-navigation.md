# Delimiter Navigation

Status: Draft
Date: 2026-11-15
Format: OpenSpec capability proposal
Inspection baseline: dd1b569dd9c6e480ae4dd82a1f298bcc8478072f
Review scope: explicit mate jumps, structural groups and expression motions
Model: [Pair Preferences and Native State](editor-delimiter-pairs-model.md)
Shared qualification: [Delimiter Pairs](editor-delimiter-pairs.md), DPR04
Related behavior: [Interaction Parity](editor-emacs-interaction-parity.md), EIP04

## Purpose

Navigate source structure independently of automatic edits and color. Retain
DPR02 from the original pair proposal while distinguishing a typed mate jump,
structural group movement and a syntax-aware expression motion.

## Proposed commands and endpoints

| Command | Proposed binding | Observable target |
| --- | --- | --- |
| matching-delimiter | Unbound shared command | The other delimiter of the confirmed typed pair at point. |
| forward-pair | C-M-n | Forward across mode-defined list/group structure. |
| backward-pair | C-M-p | Backward across mode-defined list/group structure. |
| up-pair | C-M-u | Backward out through an enclosing opening delimiter. |
| down-pair | C-M-d | Forward into a nested group. |
| forward-sexp | C-M-f | Forward over one mode-defined expression; specializes EIP04. |
| backward-sexp | C-M-b | Backward over one mode-defined expression; specializes EIP04. |

These sequences have no factory leaves in the inspected base maps. They remain
proposals: respect explicit assignments, input focus and profile/context ownership.
All commands have canonical named invocation, descriptions and truthful effective
hints. They are usable on read-only source without enabling pairs.enabled or
pair presentation. Terminal/prompt/projection input retains its existing owner.

The direct jump prefers a delimiter at point, then one immediately before point.
It lands on the mate's source boundary. At a delimiter endpoint, jumping twice
returns to that endpoint; adjacent caret positions do not invent an extra round
trip position. With no qualifying delimiter, report that condition instead of
searching later through the line. Missing, mismatched or unconfirmed typed mates
leave point and mark unchanged. A supplied numeric prefix does not repeat this
single mate jump.

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

For the proposed strict mate jump, (ab) tail at its opener instead targets the
closer's offset 3. That new entry policy is a Rev proposal, distinct from the
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

## Review and acceptance

The proposed names, C-M-n/p/u/d/f/b defaults, exact mode-admission matrix and direct
jump entry policy require review. Existing custom bindings and modal semantics
remain authoritative. Navigation does not depend on accepting automatic editing
or a presentation style. The source/mode grouping distinction above is part of
review, not a request to color mismatched punctuation as a valid pair.

Before a plan, extend independent paired observations for mixed/nested groups,
strings/comments, malformed syntax, signed counts, boundaries, Unicode/CRLF and
selection state. Use existing coordinates/input/modal/native frame and owned
PTY fixtures; compare exact endpoints, source/history and complete key sequences.
Preserve resource bounds and validate shared/language/custom map precedence.
Reconcile additive catalog/keymap acceptance in the owning delivery without
rewriting historical factory-audit snapshots or their exact-commit receipts.
