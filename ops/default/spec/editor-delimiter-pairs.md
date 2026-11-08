# Delimiter Pairs

Status: Draft
Date: 2026-11-08
Format: OpenSpec capability proposal
Inspection baseline: e2432a0ceec9
Review scope: canonical proposal, reconciled after the keyboard-macro delivery closed
Implementation authority: none; the visual default is settled, other proposed details remain under review
Model: [Pair Preferences and Native State](editor-delimiter-pairs-model.md)
Baselines: [Bindings](../../../docs/reference/bindings.md), [Commands](../../../docs/reference/commands.md), [Configuration](../../../docs/reference/config.toml)
Prerequisites: [Command Prefix Arguments](editor-prefix-arguments.md), [Keyboard Macros](editor-keyboard-macros.md), both delivered

## Purpose

Make paired punctuation easy to insert, navigate and recognize without requiring
extensions. Automatic editing, matching navigation and visual presentation are
independent capabilities. A user may disable automatic insertion, all extra
color, or both while retaining explicit pair navigation.

The requested scope covers automatic opening/closing pairs, movement between
matched endpoints in either direction, and an optional visual pairing aid.
The initial proposal also includes wrapping a selection and removing an outer
pair. More extensive structural transformations are a separate scope decision;
this proposal does not silently adopt a complete structural programming system.

## Observed implementation

- The modal matching-delimiter action already exists, bound to %. It finds a
  starting parenthesis/bracket/brace at or after point on the current logical
  line, then scans forward/backward for its mate, including across later lines.
- Current matching counts same-kind raw punctuation. It does not provide a
  context-aware general pair service, explicit finite scan admission, automatic
  insertion or pair-specific presentation.
- Modal inner/around pair and quote objects already exist; their escape-aware
  lexical scanning is a separate current contract.
- Native syntax snapshots are source-revision-qualified. Source mutation/history,
  grapheme/cell projection, palette, wrapping and modal input already have owners.

Evidence: [matching](../../../src/coordinates/navigation/structure.rs),
[objects](../../../src/coordinates/navigation/objects/pairs.rs),
[syntax](../../../src/syntax.rs), [projection](../../../src/display/syntax.rs),
[palette](../../../src/config/palette.rs).
This draft does not claim these new capabilities are implemented.

## Proposed ordinary behavior

### Automatic pair editing

The proposed public toggle is auto-pair-mode. In source text entry, typing an
admitted opening delimiter inserts its closer and leaves point between them.
For example, typing ( in an empty eligible context produces (|), where | marks
point and is not document text. Typing the matching closer immediately before
its already admitted mate moves over it instead of inserting a duplicate.

Backspace between an adjacent empty admitted pair removes both delimiters in
one native undo step. Content, including whitespace, prevents paired deletion.
Deleting a closing delimiter otherwise remains an ordinary single deletion.
Region precedence, read-only checks, Unicode/grapheme boundaries and native
history remain authoritative. Explicit wrapping and automatic wrapping each
publish one checked edit containing both ends; undo restores the original region
and selection orientation.

With an active ordinary linear selection, typing either side of an admitted pair
wraps the selection when wrap_selection is enabled. It preserves the selected
bytes; it does not silently escape or reformat their contents. Quote wrapping
that cannot meet the admitted literal contract must refuse with a useful message.
Rectangular/block selections retain their existing typed-input behavior unless
an explicit future multi-row wrapping contract is approved.

Automatic insertion considers lexical context, escaping and local balance. Do
not create surplus closers where an existing unmatched closer supplies the mate,
or skip a closer when that would leave its opening delimiter unmatched. At an
uncertain balance/context boundary, insert the user's literal character normally.
Do not impose strict balance by preventing ordinary manual edits.

Paste, completion edits, AI output, external reload and programmatic source
replacement are literal source operations, not simulated typing. They must not
recursively trigger pair insertion. Explicit numeric prefixes reuse native
counted-edit admission; any generated mate expansion must remain one checked
undo unit and cannot multiply itself through callbacks. The exact repeated-opener
rule is retained as a review item before approval.

### Language and string contexts

Provide built-in () [] {} profiles. Admit double/single quotes and backticks
only where the recognized language/text context supports that use. Escaped quotes
and apostrophes within words stay literal. Ambiguous character-versus-lifetime
syntax, raw/triple strings, template strings and interpolation must be handled
by their declared language profile or fall back to literal input.

Generic angle brackets are not ordinary pairs: comparisons and type/tag syntax
cannot share an unconditional insertion rule. Tag-name completion, arbitrary
markup constructs and a general pair-definition language are not implied.

Use current native syntax information when available, including bundled and
admitted external grammars; no language server is required. Never apply stale
syntax coordinates. A bounded, conservative lexical fallback supports plain
punctuation without claiming semantic knowledge of an unavailable language.
Document the distinction between confirmed, unmatched and uncertain results.

### Explicit navigation and structural helpers

| Proposed command | Proposed binding | Description |
| --- | --- | --- |
| auto-pair-mode | Unbound | Toggle automatic delimiter insertion and empty-pair editing in this buffer. |
| pair-highlight-mode | Unbound | Toggle pair presentation in this view. |
| pair-highlight-style | Unbound | Choose match, depth, pulse or off presentation. |
| matching-delimiter | Unbound shared command | Move to the other delimiter of the pair at point. |
| forward-pair | C-M-n | Move forward past a balanced group. |
| backward-pair | C-M-p | Move backward before a balanced group. |
| up-pair | C-M-u | Move out through an enclosing opening delimiter. |
| down-pair | C-M-d | Move into a nested balanced group. |
| pair-wrap | Unbound | Wrap the active region in an admitted delimiter pair. |
| pair-unwrap | Unbound | Remove an enclosing pair while preserving its contents. |

All functions are invocable by name in both editing profiles, with descriptions
and effective key hints. Default shortcuts are proposals, subject to existing
key ownership. Existing modal % and pair text objects must keep their documented
entry-point, selection/operator and jump-history contracts. Share matching
primitives where those contracts agree; do not silently replace modal semantics.

The new bidirectional jump prefers a delimiter at point, then one immediately
before point, and lands on the other delimiter's native source boundary. If
neither qualifies, it reports that no pair is at point rather than searching
arbitrarily through later source. The source is unchanged and jumping again
returns to the first endpoint. Read-only source permits navigation.

Directional group motions use signed prefix counts; zero is inert. The direct
mate jump runs once rather than inheriting generic repeat-loop semantics. Missing,
mismatched or unconfirmed matches preserve point and mark and explain the reason.
Navigation, highlighting and explicit edits use the same source-qualified pair
result wherever their entry contracts agree.

### Presentation choices

Owner decision, 2026-11-08: depth coloring is the default. Color structural
delimiters by nesting depth and mark the active pair discretely. The match style
is the restrained alternative: mark only the active pair's two delimiter spans
with a subtle background and underline, retaining normal text elsewhere. Neither
style selects or recolors the entire enclosed expression.

| Style | Presentation |
| --- | --- |
| depth (default) | Color structural delimiters by nesting depth; add the active-pair mark. |
| match | Restrained alternative: mark only the two active delimiters. |
| pulse | Briefly mark an identified pair after relevant input or an explicit jump, then clear it. |
| off | No extra pair presentation; automatic editing and navigation stay available. |

The depth palette uses a short cycle drawn from the existing editor palette.
Only delimiters receive depth color; strings/comments retain their language
meaning and other syntax remains intact. Users can select the restrained match
style or disable presentation while retaining automatic pairs. Selection, search, cursor and diagnostic
precedence must remain readable, with explicit overlay ordering.

A pulse is one temporary mark, not periodic blinking: no terminal blink attribute,
caret relocation, scroll jump or focus transfer. Proposed duration is 250 ms.
Typing, changing pair, source revision or view cancels/replaces the pending mark.
Rendering/resizing alone must not restart its lifetime. Once cleared, the pair
feature owns no deadline. No ordinary keystroke may wait for that duration.

If a mate is offscreen, preserve the view and report its line/column unobtrusively;
do not move point to demonstrate the match. A confirmed mismatch may use the
existing warning role. An unfinished or uncertain parse is not a confirmed error.
Without color, active pairs can use supported underline/reverse attributes; depth
color degrades without manufacturing extra punctuation. Respect terminal and
NO_COLOR capability. Nerd Fonts are not required for any pair function.

## Proposed configuration and runtime scope

Configuration uses the existing startup TOML root and per-language overrides.
Runtime editing policy is buffer-local and shared by its views. Presentation is
view-local because each pane has its own point, viewport and active pair. Changes
made through commands do not rewrite config.toml.

The model contains the proposed preference leaves and colors. The full sample
configuration must document every delivered leaf, including disabled options.
The owner selected depth presentation as the default and match as the restrained
alternative; pulse and off remain available choices. The proposal also favors
automatic pairing, selection wrapping, empty-pair deletion and closer skipping
enabled, with quotes admitted by language profile. Those editing defaults remain
proposals rather than an approved implementation contract.

## Work, ownership and interactions

Use one native, revision-qualified pairing owner. Reuse syntax/coordinate
classification, source transactions and display projection rather than duplicating
a parser or introducing an LSP dependency. Bounded fallback matching must not
scan an arbitrary whole document on every key or every frame.

Proposed bounds: 256 KiB of fallback source scan per request, nesting depth 256,
8,192 painted delimiters per view, 2 MiB retained pairing metadata per buffer,
and cooperative work slices of at most 256 tokens with a four-millisecond target
between native primitives. Exhaustion yields an uncertain/limited result, never
an invented mate. The implementation must measure representative large files
before finalizing these proposed bounds. No permanent idle worker or timer.

Pair edits must compose with the completed prefix-argument contract and the
completed macro recorder. Record/replay must distinguish ordinary raw
input from accepted literal edits so generated closers are not inserted twice.
Future pair delivery must preserve the macro command, history and literal-input
contracts rather than changing them as an implicit side effect.

## ADDED Requirements

### Requirement: DPR01 Insert and remove admitted pairs through native edits

Eligible typed input SHALL use the effective buffer pairing policy, preserving
selection, read-only, count, literal-input and history contracts.

#### Scenario: Empty pair

- **GIVEN** eligible editable source with automatic pairing enabled
- **WHEN** the user types an opener, its closer and an empty-pair Backspace
- **THEN** insertion, skip and deletion follow the declared policy without duplicate closers or unrelated source changes.

### Requirement: DPR02 Navigate matched delimiters in both directions

Shared pair commands SHALL use source-qualified endpoints and preserve existing
modal entry/operator behavior. Unconfirmed targets SHALL leave point unchanged.

#### Scenario: Multiline pair

- **GIVEN** an admitted balanced pair across multiple logical lines
- **WHEN** matching-delimiter is invoked at either endpoint
- **THEN** it reaches the opposite endpoint without editing text or selecting a popup row.

### Requirement: DPR03 Default to depth color and offer restrained matching

Depth coloring SHALL be the default, with restrained match, pulse and off
alternatives. Presentation SHALL be independently configurable and runtime-switchable without
changing source or normal editing policy. It SHALL preserve overlay/capability
precedence and avoid persistent blinking or cursor relocation.

#### Scenario: Color-averse presentation

- **GIVEN** match or pulse rather than depth presentation
- **WHEN** a pair is identified
- **THEN** only the two delimiter spans are marked and any pulse releases its deadline after expiry.

### Requirement: DPR04 Preserve syntax, input and bounded-work ownership

The owner SHALL reject stale results, use declared syntax/lexical fallbacks,
respect native input owners and stop bounded searches without false matches.

#### Scenario: Incomplete context or literal insertion

- **GIVEN** stale/incomplete syntax, an escaped quote, pasted source or macro-generated literal edits
- **WHEN** the pairing path observes that operation
- **THEN** ordinary literal behavior remains usable without recursive pair generation or fabricated diagnostics.

## Decisions retained for review

1. Confirm the initial editing scope: automatic pairs, selection wrapping and
   unwrap; consider additional structural expansion/contraction separately.
2. Confirm proposed enabled-by-default editing. Default depth coloring and the
   restrained match alternative are already settled by the owner.
3. Finalize quote/language policy, whitespace-skipping behavior and counted opener
   semantics against actual native input and syntax boundaries.
4. Confirm bindings, offscreen feedback and the optional pulse duration.
5. Freeze measured resource bounds and exact integration acceptance before a plan.

## Later acceptance and documentation

Use existing native edit/input/history, source/syntax, modal, display and pointer
fixtures. Cover nested/multiline/malformed pairs, strings/comments/escapes,
apostrophes/lifetimes, stale parses, UTF-8/CRLF, selections and multiple views.
Verify literal paste/completion/macro behavior, undo, configured/runtime modes,
monochrome, overlay precedence, pulse cancellation and idle release.

Markdown User Guide demonstrates one short pair edit and jump. How-to covers
presentation and language overrides; reference owns exact policies/bindings,
configuration and limits. The Markdown-derived guide must remain aligned.
This canonical draft is separate from the completed macro delivery. Remaining
editing and resource decisions require review before an implementation plan.
