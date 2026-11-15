# Delimiter Pairs

Status: Completed
Completed: 2026-11-15
Date: 2026-11-08
Updated: 2026-11-15
Format: OpenSpec capability specification
Inspection baseline: dd1b569dd9c6e480ae4dd82a1f298bcc8478072f
Review scope: automatic pair editing and the shared qualified pairing contract
Approved: product owner, 2026-11-15; first capability, slicing and Looper execution
Implementation authority: automatic pair editing and shared qualification only
Plan: [Automatic Delimiter Editing Delivery](../plan/rev-delimiter-editing.md)
Tracker: [Automatic Delimiter Editing Tracker](../tracker/rev-delimiter-editing.md)
Model: [Pair Preferences and Native State](editor-delimiter-pairs-model.md)
Baselines: [Bindings](../../../docs/reference/bindings.md), [Commands](../../../docs/reference/commands.md), [Configuration](../../../docs/reference/config.toml)
Prerequisites: [Command Prefix Arguments](editor-prefix-arguments.md), [Keyboard Macros](editor-keyboard-macros.md), both delivered

## Purpose

Make automatic paired punctuation editing available through checked native edits:
insert a mate, skip an admitted existing closer, delete an empty pair, wrap a
linear selection and explicitly remove an enclosing pair. Manual literal editing
remains available; the feature does not enforce a permanently balanced document.

## Capability ownership

The owner authorized separating these concerns on 2026-11-15. The existing
proposal path and requirement identities are retained; DPR02 and DPR03 now have
one authoritative owner in the corresponding documents below.

| Capability | Authoritative specification | Independent controls |
| --- | --- | --- |
| Automatic editing and shared pair qualification | This specification; DPR01/DPR04 | auto-pair-mode and editing settings. |
| Mate jump, group and expression navigation | [Delimiter Navigation](editor-delimiter-navigation.md); DPR02 | Named motions and effective keymaps, usable on read-only source. |
| Depth color, active-pair mark and finite pulse | [Delimiter Presentation](editor-delimiter-presentation.md); DPR03 | View-local presentation controls. |

All three share the existing model companion. Disabling automatic editing does
not disable navigation or color. Presentation can be approved or delivered
without enabling automatic editing. Shared qualification has its own acceptance;
the approved editing delivery does not authorize navigation or presentation implementation.

[Interaction Parity](editor-emacs-interaction-parity.md), EIP04, remains the owner
of paragraph and syntax-aware expression (sexp) behavior. It does not own automatic
pair insertion or colors, and a matching-delimiter jump is not a substitute for
forward-sexp/backward-sexp.

## Observed implementation at the inspection baseline

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
At the inspection baseline, the startup schema has no pairs settings or pair
palette, and language overrides admit editing indentation and grammar fields,
not pairing leaves. The editing TOML requires explicit schema work; it
is not configuration accepted by the installed binary.

Approval authorizes implementation; it does not claim these capabilities are delivered.

## Shared pair contract

This family reuses native source, syntax, coordinates, history and input owners.
A qualified result includes the canonical buffer/revision, source spans, lexical
profile, result kind and work provenance. Never retain raw tree nodes across a
revision. A stale or bounded-out query cannot become a fabricated match.

Typed mate lookup, mode-dependent group motion and syntax-aware expression
motion have distinct result contracts. Share classification and source-qualified
facts where their policies agree; do not equate their endpoints or turn a loose
mode grouping result into a confirmed typed mate. The shared
[model](editor-delimiter-pairs-model.md) keeps these results separate.

Language classification must use the admitted language identity even when syntax
coloring is disabled. Current syntax is reusable evidence; unavailable or stale
syntax requires the declared bounded lexical policy. Ambiguous strings,
comments, Rust lifetimes/raw strings and template/interpolation boundaries stay
literal or uncertain unless the profile proves the requested operation safe.
Plain text can admit ordinary () [] {} without pretending to parse a language.

The approved editing limits are 256 KiB of fallback source inspection per
request, depth 256 and 2 MiB retained pair metadata per buffer. Cooperative work
uses at most 256 tokens per slice and the existing four-millisecond native input
scheduling target. Counted work shares one admitted request budget. One unusually
large token may not bypass the byte bound or cancellation checkpoints. The plan
records the measured current raw-scan baseline and resource sketch; Slice 2 must
measure and verify the new service before typing integration. These hard limits
are approved boundaries, not claims about future measured capacity. Presentation
span limits belong to the separately scheduled visual capability.

No permanent idle worker, per-delimiter timer, filesystem discovery, language
server or new parser dependency is required. Presentation consumes bounded,
qualified cached results rather than running a whole-file scan per frame.

## Approved ordinary behavior

### Automatic pair editing

The public toggle is auto-pair-mode. In source text entry, typing an
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
undo unit and cannot multiply itself through callbacks. Positive counts apply the admitted typed operation that many times within one
checked native undo unit. A zero count is inert; negative typed repetition
retains the existing refusal contract. Generated mates are source effects, not
additional input events. In an empty eligible buffer, C-u 3 ( produces ((()))
with point after the three openers. Budget exhaustion refuses the whole counted
edit before publication; it cannot leave half an admitted pair operation.

Wrapping placement follows the controlled reference: typing an opener
around an active linear selection leaves point immediately after the outer
opener; typing its closer leaves point after the outer closer. The region is
deactivated. Symmetric quote wrapping uses opener placement when admitted by
the language profile. The native undo contract restores source, point/mark and
selection orientation. Explicit pair-wrap uses the opener placement. pair-unwrap removes
only the confirmed enclosing delimiters and maps point/mark through that edit;
strings/quotes that cannot be unwrapped without changing their literal meaning
refuse before mutation.

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

### Approved initial language admission

Every profile admits () [] {} only in a proved ordinary source context. Punctuation inside strings,
comments and embedded literal/verbatim content stays literal unless the declared
profile proves a safe pair operation;
unknown context is not proof that punctuation is executable code. Disabled syntax
colors do not disable recognized-language policy. Confirmed current syntax can
strengthen classification; external grammar highlight names alone are insufficient
quote grammar. Fallback is conservative and bounded.

| Profile | Automatic quote admission and conservative boundaries |
| --- | --- |
| Plain/unknown text | Double quotes at eligible boundaries; word apostrophes and backticks remain literal. Structural punctuation has bounded plain-text balance, without semantic claims. |
| Rust | Ordinary double-quoted strings at confirmed boundaries. Apostrophes remain literal because character/lifetime syntax is ambiguous; unproved raw-prefix/string contexts remain literal. |
| C | Ordinary double strings and single character quotes at confirmed boundaries. Escapes/comments and character contexts are respected; backticks remain literal. |
| Go | Ordinary double strings, single character quotes and confirmed backtick raw strings. Unknown or incomplete quote context stays literal. |
| JavaScript, TypeScript, TSX | Ordinary single/double strings at confirmed boundaries. Template/backtick/interpolation contexts remain literal unless current classification proves a safe ordinary operation. |
| JSON | Double strings only, with escapes and key/value string context; apostrophes/backticks remain literal. |
| TOML | Ordinary single/double strings. Triple/raw or unproved multiline quote contexts remain literal. |
| CSS | Ordinary single/double strings in confirmed string-eligible contexts. |
| HTML | Single/double quotes only at confirmed attribute-value boundaries; other markup quote context remains literal. |
| Org and Markdown | Prose punctuation and eligible double quotes. Inline/verbatim/code markers remain literal; embedded source uses its confirmed native language profile, otherwise conservative refusal/literal behavior. |
| Other recognized or external languages | Bounded conservative structural policy. Double quotes require the plain-text boundary rule or a declared confirmed language profile; do not infer single/backtick rules from syntax colors. |

An eligible prose quote boundary is outside an escaped/quoted context and a word
interior. Never add a second apostrophe to ordinary prose or silently transform
selected string bytes. If a requested explicit wrap/unwrap cannot preserve the
selected literal contract, refuse with a useful message before mutation.

pair-wrap uses the existing cancellable native prompt to select an admitted
opening or closing delimiter, then opener-placement wrapping. It has no guessed
pair default. pair-unwrap removes the innermost confirmed pair enclosing point,
including point on either delimiter, through one checked transaction. Preserve
contents and map point/mark; unconfirmed or meaning-changing removal refuses.

## Commands and policy

| Command | Binding | Scope |
| --- | --- | --- |
| auto-pair-mode | Unbound | Toggle automatic editing in this buffer. |
| pair-wrap | Unbound | Wrap an active ordinary linear region in an admitted pair. |
| pair-unwrap | Unbound | Remove the confirmed enclosing pair, preserving literal meaning. |

All commands use the existing catalog, named invocation, descriptions and key
introspection. Prompts, searches, terminals, Git projections and other focused
input owners do not acquire source auto-pairing through a shared printable key.
Modal Normal/Visual commands and pair text objects retain their existing owners;
ordinary eligible Insert/Replace typing uses the admitted editing policy.

Approved defaults enable automatic editing, selection wrapping, empty-pair
deletion and closer skipping. Quote eligibility comes from the language profile,
not an unconditional rule for every apostrophe or backtick. The five editing
booleans default true. The independently settled depth-color default
belongs to the presentation specification.

Runtime editing policy is buffer-local and shared by its views. Commands never
rewrite startup configuration. The [model](editor-delimiter-pairs-model.md)
defines the approved editing schema extension, per-language inheritance and origins.

## Reference observations and limits

Controlled GNU Emacs 31.1 buffers with electric-pair-local-mode enabled were
examined without personal init or third-party packages. Positions below are
zero-based character offsets; the vertical bar denotes point only in prose.
These are reference observations, not Rev implementation or terminal acceptance.

| Initial text / selection | Input | Observed text and point |
| --- | --- | --- |
| Empty Lisp buffer | ( | (), point 1. |
| Empty Lisp buffer | C-u 3 ( | ((())), point 3. |
| (), point 1 | ) | (), point 2; no extra closer. |
| (), point 1 | DEL | Empty text, point 0. |
| ( ), point 2 | DEL | (), point 1; only the space is removed. |
| alpha selected in either orientation | ( | (alpha), point 1; region deactivated. |
| alpha selected | C-u 3 ( | (((alpha))), point 3. |
| alpha selected | ) | (alpha), point 7; region deactivated. |
| dont in Text mode, point 3 | apostrophe | don't, point 4; no extra quote. |
| An unmatched closer ), point 0 | ( | (), point 1; the existing closer is reused. |

The proposal retains Rev's checked source transactions, literal-input provenance
and native undo/selection restoration. A reference key workflow is not proof
that the existing Rev primitive already has that behavior. Actual native and
physical-input acceptance remains required.

References: [Matching Parentheses](https://www.gnu.org/software/emacs/manual/html_node/emacs/Matching.html),
installed elec-pair.el and the native owners linked above. The exact minor-mode
configuration is material: automatic pairing is not implied by a stock mode
with electric pairing disabled.

## ADDED Requirements

### Requirement: DPR01 Insert and remove admitted pairs through native edits

Eligible typed input SHALL use the effective buffer pairing policy and one
checked source transaction, preserving read-only, selection, count, Unicode,
line-ending and native history contracts. Paste, completion, AI output, external
reload and accepted literal replacements SHALL NOT simulate typed pair triggers.

#### Scenario: Empty pair

- **GIVEN** eligible editable source with automatic pairing enabled
- **WHEN** the user types an opener, its closer and an empty-pair Backspace
- **THEN** insertion, skip and deletion follow the declared policy without duplicate closers or unrelated source changes.

#### Scenario: Count and active region

- **GIVEN** an ordinary active linear region and a positive admitted count
- **WHEN** the user types an eligible opening delimiter
- **THEN** the selected bytes are wrapped with the counted nested pair and the declared point placement
- **AND** one undo restores the original bytes, point/mark and selection orientation.

### Requirement: DPR04 Preserve syntax, input and bounded-work ownership

The shared owner SHALL reject stale results, enforce declared syntax/lexical
policies and resource budgets, and distinguish confirmed typed pairs from
mode grouping and uncertain observations. Macro record/replay SHALL preserve
the existing raw-input versus accepted-literal-edit contract so generated mates
are not inserted a second time.

#### Scenario: Incomplete context or literal insertion

- **GIVEN** stale/incomplete syntax, an escaped quote, pasted source or macro-generated literal edits
- **WHEN** the pairing path observes that operation
- **THEN** ordinary literal behavior remains usable without recursive pair generation or fabricated diagnostics.

#### Scenario: Limited or stale request

- **GIVEN** a captured request exceeds its byte/depth/work budget or loses its source revision
- **WHEN** the owner reaches the admission or publication boundary
- **THEN** it reports an uncertain/limited result without publishing an unproven mate or partial counted edit
- **AND** cancellation and ordinary editing remain available.

## Approval and delivery boundary

The owner approved editing-first slicing and Looper execution on 2026-11-15.
Automatic editing defaults, the initial language matrix, wrapping point
placement, counted transactions and fixed bounded-work contract above define
this first delivery. The plan and tracker record seven predecessor-ordered
slices and controller-owned canonical dev integration. Navigation and visual
work remain separate subsequent sets in that order. No main/release or global
installation is implied. Preserve the completed factory-shortcut receipts.

## Acceptance and documentation

Use existing native edit/input/history, source/syntax, modal and PTY fixtures.
Cover nested/multiline/malformed pairs, strings/comments/escapes, apostrophes,
Rust lifetimes/raw strings, stale parses, UTF-8/CRLF, regions and multiple views.
Verify paste/completion/AI/reload/macro provenance, counts, undo/redo and explicit
configuration overrides without changing unrelated source or personal settings.

The User Guide demonstrates one pair edit and undo. How-to covers independent
controls and language overrides; reference owns exact policies, commands,
configuration and limits. Navigation and visual acceptance belong to their own
linked specifications, not to inferred success from an editing test.

## Editing implementation and closure evidence

The seven-slice [tracker](../tracker/rev-delimiter-editing.md) and
[Slice 7 report](../report/slices/rev-delimiter-editing/slice-7-acceptance.md)
collect the configuration, qualification, native editing, provenance and integrated
acceptance evidence for DPR01/DPR04. The implementation owners are
[src/pairs](../../../src/pairs.rs), [buffer qualification](../../../src/buffer/pairs.rs)
and [session transactions](../../../src/session/pairs.rs). The
[acceptance script](../../../scripts/check-delimiter-editing.sh) checks native
bounds/shared owners and two Linux PTYs. The
[reference](../../../docs/reference/delimiter-editing.md),
[how-to](../../../docs/how-to/configure-delimiter-editing.md) and
[editing lesson](../../../docs/tutorials/user-guide/02-edit-text.md) describe the
implemented policy.

All seven slices and DPR01/DPR04 are Completed. The controller's unchanged full
gate passed at `eb65b227dee8b42bf8d0c8a6e0ef12f5675c756b`, receipt
`QD5ALW2WJR64VCCTK4CJAOZPDK`; the
[delivery-set closure](../report/slices/rev-delimiter-editing/slice-7-acceptance.md#delivery-set-closure)
records native/PTY/resource, workspace/hook and documentation results. Closure
is a subsequent Markdown-only dev commit with identical tested bytes outside
the approved closure paths. DPR02/DPR03 and their model portions remain Draft;
their acceptance is independent.
