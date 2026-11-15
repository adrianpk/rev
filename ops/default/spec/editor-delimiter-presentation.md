# Delimiter Presentation

Status: Approved
Date: 2026-11-15
Format: OpenSpec capability proposal
Inspection baseline: 744497bab434cc38204f091398c89e21d5147476
Owner decision retained: depth coloring by default, restrained match alternative; 2026-11-08
Approval: product-owner continuation to the final delimiter delivery, 2026-11-15
Delivery: [Delimiter Presentation](../plan/rev-delimiter-presentation.md)
Review scope: independently controlled pair color and active-pair presentation
Model: [Pair Preferences and Native State](editor-delimiter-pairs-model.md)
Shared qualification: [Delimiter Pairs](editor-delimiter-pairs.md), DPR04
Baseline: [Presentation Continuity](editor-presentation-continuity.md)

## Purpose

Recognize structural nesting and the active pair without editing source, moving
point or requiring automatic pair insertion. This document retains DPR03 from
the original delimiter-pairs proposal. Navigation and automatic editing keep
their separate specifications and controls.

## Presentation choices

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

The default depth palette uses a short cycle drawn from the resolved editor
palette, including user palette overrides. Explicit pair-color overrides use
the same existing RGB configuration format, not an unrelated fixed theme.
Only delimiters receive depth color; strings/comments retain their language
meaning and other syntax remains intact. Users can select the restrained match
style or disable presentation while retaining automatic pairs. Selection, search, cursor and diagnostic
precedence must remain readable, with explicit overlay ordering.

A pulse is one temporary mark, not periodic blinking: no terminal blink attribute,
caret relocation, scroll jump or focus transfer. The default duration is 250 ms, configurable within 50..1000 ms.
Typing, changing pair, source revision or view cancels/replaces the pending mark.
Rendering/resizing alone must not restart its lifetime. Once cleared, the pair
feature owns no deadline. No ordinary keystroke may wait for that duration.

If a mate is offscreen, preserve the view and report its line/column unobtrusively;
do not move point to demonstrate the match. A confirmed mismatch may use the
existing warning role. An unfinished or uncertain parse is not a confirmed error.
Without color, active pairs can use supported underline/reverse attributes; depth
color degrades without manufacturing extra punctuation. Respect terminal and
NO_COLOR capability. Nerd Fonts are not required for any pair function.

## Ownership, palette and continuity

Presentation belongs to each view. The startup style is a default; toggling a
view does not change a sibling view, the shared buffer policy or a config file.
pair-highlight-mode toggles off/the last non-off style. pair-highlight-style
accepts depth, match, pulse or off through an ordinary bounded prompt. Both
commands are unbound catalog commands.

Keep source selection/search foregrounds and backgrounds authoritative. Pair
marks cannot erase the cursor or existing diagnostic attributes. Outside those
higher-priority cells, the active-pair mark adds background/underline while
retaining depth foreground. In match/pulse, match_text applies to the marked
structural delimiters; ordinary syntax remains unchanged elsewhere. Quote/string
contents do not become rainbow structural delimiters merely because they
contain punctuation. A mode-qualified active quote pair can be marked without
recoloring its body or replacing its existing string foreground.

Rendering reads bounded source-qualified results. Repainting and moving point
must not scan or recolor an arbitrary whole buffer. An edit may advance retained
spans to a new revision only through an explicit verified source mapping that
proves their classification and nesting unchanged. Otherwise invalidate the
unproven portion; never paint stale source coordinates or invent nesting depth.

Preserve completed-frame continuity during admitted same-line edits, including
Org/Markdown pretty and indented views. Pair presentation must not make stable
headings, prefix geometry or unchanged pair colors alternate while asynchronous
analysis catches up. This is an observable contract, not a mandated renderer
or a promise to keep unqualified old syntax forever.

Cooperative pair work shares the native input budget. Continuous typing and
held navigation must retain cancellation and timely completed frames. Use the
existing interaction-repeat/release budgets and owned PTY infrastructure for
measurement; freeze the workload with the delivery plan. In steady match/depth
display, and after a pulse clears, this feature owns no wake-up deadline.

## Approved operating contract

The presentation delivery adds global and optional language-level pairs.highlight
and pairs.pulse_ms leaves through the existing configuration/merge/origin owners.
Missing language leaves inherit the resolved global values. Runtime view overrides
win without saving settings. Existing automatic-editing leaves and navigation
admission retain their independent meaning. The only catalog additions are
pair-highlight-mode and pair-highlight-style; no factory shortcut changes.

The six-role fallback depth cycle is syntax.function, syntax.number,
syntax.keyword, syntax.type, syntax.string and syntax.punctuation, resolved after
palette merging and repeated modulo its length. Optional palette.pairs overrides
admit match_background, match_text, mismatch and depth_colors with the shared
model's opaque RGB format and 1..8 cycle entries. Defaults use selection,
foreground and warning for the first three roles. Only source-qualified
structural delimiters receive depth color; eligible quote endpoints retain their
string foreground. No inferred angle pairs or unproved raw-string boundaries.

Use the existing matching-delimiter entry rule: a qualified endpoint at point,
then immediately before point. Point inside an ordinary body does not trigger an
arbitrary enclosing-pair search. A confirmed mismatch is distinct from unfinished,
uncertain and resource-limited source. A visible endpoint with an offscreen mate
keeps the viewport and exposes a lower-priority native line/column hint without
replacing prompts, errors or explicit command feedback.

Source selection/search own their existing foreground/background cells; cursor
and diagnostic attributes remain authoritative. Elsewhere depth colors provide
the foreground and active marks add background/underline. Match/pulse use
match_text only on marked structural endpoints. Capability and NO_COLOR handling
retain supported non-color attributes and never manufacture punctuation.

Pulse starts only for an accepted input or successful explicit pair motion that
identifies the active endpoint pair. Rendering, resize, unsuccessful/no-op motion
and polling do not extend its lifetime. A replacement generation owns one finite
expiry; source/view/point changes retire or explicitly requalify that generation.
Depth, match, off and an expired pulse own no pair wake-up deadline.

Presentation reuses the existing buffer-owned typed qualification and immutable
source-qualified facts. It cannot retire an editing/navigation request, promote
a loose group result to a typed mate or create a second mutable source cache.
Source mapping must prove both endpoint transformation and unchanged lexical
classification/nesting before retained colors can advance to a new revision.
Otherwise the unproved range is invalidated rather than painted at stale offsets.

Limits are 256 KiB inspected source per shared request, nesting depth 256,
8,192 eligible visible delimiter spans per view and 2 MiB total retained pair
metadata per buffer, counting shared and per-view charges exactly once. Polls
process at most 256 tokens under the native four-millisecond scheduling target.
Exhaustion remains limited/uncertain, without guessed depth or mismatch. Existing
input/frame/release scheduling remains authoritative. The delivery plan and its
independent acceptance contract freeze the workloads and physical-input budgets.

## ADDED Requirements

### Requirement: DPR03 Default to depth color and offer restrained matching

Depth coloring SHALL be the default, with restrained match, pulse and off
alternatives. Presentation SHALL be independently configurable and view-local,
without changing source or ordinary editing policy. Palette/capability/overlay
precedence SHALL remain readable without persistent blinking or cursor relocation.

#### Scenario: Color-averse presentation

- **GIVEN** match or pulse rather than depth presentation
- **WHEN** a pair is identified
- **THEN** only the two delimiter spans are marked
- **AND** a pulse releases its deadline after expiry.

#### Scenario: Independent views and editing

- **GIVEN** automatic editing is disabled and two views share a buffer
- **WHEN** one view uses depth and the other uses off
- **THEN** source bytes, source history and editing policy remain unchanged
- **AND** each view retains its own active-pair and presentation state.

### Requirement: DPV01 Retain presentation continuity and bounded input service

Every completed frame SHALL use current or explicitly advanced pair spans.
Stable admitted pair presentation SHALL not disappear and reappear solely
because analysis is pending. Pair work SHALL preserve bounded input service,
existing document projections and cancellation without whole-buffer work per key.

#### Scenario: Typing inside a stable nested form

- **GIVEN** a confirmed nested form and an ordinary same-line edit that leaves its delimiters and lexical classification unchanged
- **WHEN** input and asynchronous analysis produce successive completed frames
- **THEN** proved depth colors and active-pair geometry remain stable through the edit
- **AND** Org/Markdown heading presentation and canonical source mapping remain correct.

#### Scenario: Pulse retirement and navigation release

- **GIVEN** a qualified pulse and a burst of input or a view/source change
- **WHEN** the mark expires, is replaced or is cancelled
- **THEN** no obsolete mark/deadline remains
- **AND** the feature causes no navigation after input stops, recurring idle work or waiting for pulse duration on a key event.

## Review and acceptance

The owner authorized this final delimiter delivery on 2026-11-15. The operating
contract above and shared model settle the 250 ms default pulse, palette fallback,
overlay ordering and resource bounds. The delivery plan freezes measurement and
continuity workloads before implementation. Editing and navigation are Completed;
presentation remains Approved until its exact-candidate delivery gate passes.

Use source/frame and completed-frame observer regressions plus representative
owned PTYs: nested/deep/malformed input, offscreen endpoints, Unicode, wrapping,
Org/Markdown projections, selection/search, split views, NO_COLOR, source reload,
rapid typing, held navigation, cancellation and idle expiry. Verify bytes and
cursor/viewport invariants as well as visible colors. Add the approved controls,
roles and limits to reference and User Guide material at delivery.
