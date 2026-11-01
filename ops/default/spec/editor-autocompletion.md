# Native Autocompletion

Status: Completed
Date: 2026-10-27
Reviewed: 2026-10-31
Completed: 2026-10-31
Validated candidate: `0f7cc36cb1082aa56954fa602294f47aac80e889`
Gate receipt: `DAIRXONWTTLVG6RDIQQZKBSGXA`
Approved: product owner, 2026-10-31; implementation, preparation commit/push and ordered Looper execution requested
Format: OpenSpec capability proposal
Source: product-owner discussion and approved implementation scope
Delivery boundary: native completion first, optional LSP second; separate ordered sets under one Looper sequence
Baseline: [rev Editor](rev-editor.md)
Related: [Language Server Support](editor-language-server-support.md)
Org owner: [Org Mode](editor-org-mode.md)
Model: [Native Autocompletion Model](editor-autocompletion-model.md)
Controls: [Editor Configuration](editor-configuration.md)
Readiness: native CMP01 and CMP03-CMP09 delivered; CMP02 has focused LSP Slice 6 candidate evidence, with the integrated joint gate pending
Plan: [Native Completion Delivery](../plan/rev-autocompletion.md)
Tracker: [Native Completion Tracker](../tracker/rev-autocompletion.md)

## Joint source acceptance candidate

The LSP set's [Slice 6 report](../report/slices/rev-language-server-support/slice-6-acceptance.md) records a normal-release journey
requiring a real installed Rust server's semantic candidate from unsaved text.
Controlled peers establish semantic precedence, resolved import/undo, buffer
exceptions and valid-empty/unavailable native fallback through the same
completion owner. The joint runner also repeats the native language, Org,
external grammar, resource and PTY acceptance after source switching succeeds.

The native validated candidate and gate receipt above remain the predecessor's
historical evidence. The LSP set owns the integrated full gate and joint closure;
these focused results do not replace that pending gate.

## Purpose

Provide useful autocompletion as a built-in Rust feature while preserving rev's
small CPU/memory footprint and operation without additional editor packages.

## Confirmed direction

- Completion UI, filtering, selection and edit application are part of the binary.
- The initial Tree-sitter language/format set is Rust, HTML, CSS, JavaScript,
  TypeScript, Go, C, JSON and TOML, all included from the first delivery.
  TypeScript includes both ordinary TypeScript and TSX files, using their
  separate parsers. The initial language/variant selection is closed.
  Rust remains the primary language; TOML is also rev's configuration format.
- Zig and Odin are excluded from the initial bundle; their future support belongs
  to optional external grammar/LSP mechanisms. Python, C++ and Java are not
  selected for the initial bundle either. Bundle exclusions do not prohibit
  compatible external support.
- Org context completion reuses the existing native Rust Org owners. It is an
  additional document-mode source without another Tree-sitter grammar.
- Completion is automatic by default in completable language contexts, including
  language-specific separators and identifier prefixes. A native command remains
  available for explicit invocation.
- Automatic invocation is disabled by default in comments and string literals;
  manual invocation remains available there. This context policy is configurable.
- An enabled, operational LSP eligible for the current buffer is the preferred
  semantic completion source.
- When LSP is disabled, absent or unavailable, use the admitted native source:
  Tree-sitter for the selected languages and the existing Rust Org models for Org.
- A valid empty LSP response also selects the admitted native fallback.
- Ctags and competing local indexing engines are excluded.
- Unrecognized syntax or insufficient grammar/query coverage must degrade
  assistance without preventing ordinary editing. Optional grammar updates
  and additional languages outside the executable are requested; their
  loading/update profile is defined below.
- All user-facing concerns have TOML preferences, compiled M-x controls, or
  both as appropriate. Useful factory defaults require no configuration file.

## ADDED Requirements

### Requirement: CMP01 Native and shared interaction

Completion presentation and interaction SHALL be compiled Rust functionality,
usable without an editor addon, package or scripting layer. Local and LSP
candidates SHALL use the same user-facing selection/accept/cancel workflow.

Candidates SHALL appear in a compact vertical popup anchored to the source
cursor, with a single border and full-row highlighting for the selected item.
The popup SHALL fit within the available pane area, choosing space below or
above the cursor as appropriate. It SHALL preserve source-view focus and
SHALL NOT reflow or move document text. Colors SHALL use rev's palette roles.
When a nonempty list first appears, its first candidate SHALL be selected so
the user can accept it directly. Preselection SHALL NOT modify source text.

#### Scenario: Complete without an editor package

- **GIVEN** rev, an admitted grammar and a file containing recognized symbols
- **WHEN** the user invokes built-in completion without an LSP
- **THEN** Tree-sitter-derived candidates use the native completion interface
- **AND** no editor package or language-server process is required.

#### Scenario: Present candidates near a pane boundary

- **GIVEN** an active completion session near the bottom of a text pane
- **WHEN** the candidate list is displayed
- **THEN** the popup uses the available pane area and highlights the selected row
- **AND** source text layout and editing focus remain unchanged.

### Requirement: CMP02 Explicit source precedence

An enabled, operational LSP eligible for the current buffer SHALL be preferred.
When it is disabled, absent, unavailable or manually excluded for that buffer,
the native source SHALL supply the fallback within its admitted context
coverage. A valid empty LSP result SHALL also invoke this fallback for the
current completion context, while remaining distinct from an unavailable
server. The native sources are Tree-sitter for the selected languages and the
existing Rust Org owners for Org contexts. Ctags SHALL NOT be introduced.

#### Scenario: Disable an optional server

- **GIVEN** a document with admitted Tree-sitter coverage and an enabled LSP connection
- **WHEN** the user disables LSP
- **THEN** completion uses the native Tree-sitter path
- **AND** the editor remains usable without that server's analysis work.

#### Scenario: A working server returns no candidates

- **GIVEN** an operational LSP and admitted Tree-sitter coverage
- **WHEN** the server returns a valid empty completion response
- **THEN** rev requests Tree-sitter candidates for the current completion context
- **AND** the empty response does not mark the server as unavailable.

### Requirement: CMP03 Tree-sitter context and bounded indexing

Tree-sitter symbol extraction SHALL use included grammars and explicit
per-language queries. Org contexts use the existing native Org owners under CMP08.
Extraction SHALL cover the current buffer and bounded observations of other
open buffers with the same language identity. No directory scan or persistent
project index is part of this delivery. Current unsaved revisions SHALL own
the observations. Missing grammar/query coverage SHALL be reported truthfully and
SHALL NOT imply the presence of another local analysis engine.

#### Scenario: Unsaved symbols override an older observation

- **GIVEN** an indexed file whose open buffer has modified declarations
- **WHEN** native completion observes that buffer
- **THEN** its candidates reflect the current admitted buffer revision
- **AND** stale disk observations are not presented as current local state.

### Requirement: CMP04 Canonical edits and usable controls

Accepting a completion SHALL use the canonical edit/history owner as one
coherent undo operation. Candidate ranges SHALL be validated against the
applicable text revision and cursor context. Canceling SHALL preserve text.
Tab and Enter SHALL accept the selected valid candidate while the completion
popup is active. Without an active popup and an eligible selected candidate,
those keys SHALL retain their ordinary editing behavior.
User preferences and compiled M-x commands SHALL expose appropriate controls
without changing the startup-only TOML loading contract.

#### Scenario: Cancel a completion session

- **GIVEN** a selected candidate and unsaved document text
- **WHEN** the user cancels the completion session
- **THEN** focus returns to editing and the document is unchanged
- **AND** no completion edit or undo transaction is created.

### Requirement: CMP05 Automatic and manual invocation

Completion SHALL trigger automatically by default when the language context
admits candidates, including language-specific separators and identifier
prefixes. It SHALL NOT be restricted to member methods or properties. A native
editor command SHALL also permit explicit invocation. The delivery profile
below defines trigger timing and thresholds.

By default, automatic invocation SHALL be disabled in comments and string
literals. Users SHALL be able to configure that context policy. Explicit
invocation SHALL remain available in those contexts, with candidates subject
to the active source's coverage.

#### Scenario: Complete an identifier while typing

- **GIVEN** an editable buffer and an admitted completion context
- **WHEN** the user types a completable identifier prefix or language separator
- **THEN** rev requests candidates without requiring a manual command
- **AND** manual invocation remains available.

#### Scenario: Write a comment without automatic suggestions

- **GIVEN** the default invocation policy and a recognized comment or string literal
- **WHEN** the user types a completable prefix or separator there
- **THEN** no automatic completion request is started
- **AND** the user can still request completion explicitly.

### Requirement: CMP06 Flexible filtering

Candidate filtering SHALL support matching typed characters in order without
requiring them to be consecutive. Exact prefix matches SHALL rank ahead of
matches that rely only on this subsequence matching. The delivery profile
below defines case sensitivity and remaining ranking ties.

#### Scenario: Match an abbreviated identifier

- **GIVEN** admitted candidates named `cfg_value` and `config`
- **WHEN** the user types `cfg` in a compatible completion context
- **THEN** both candidates match the filter
- **AND** the exact-prefix candidate `cfg_value` precedes `config`.

### Requirement: CMP07 Controlled degradation

Unsupported syntax, insufficient grammar coverage or failed extraction queries
SHALL NOT prevent ordinary file opening, editing or saving. Native completion
SHALL omit proposals whose context or edit target cannot be admitted reliably.
This SHALL NOT disable an otherwise eligible LSP source merely because its
language support exceeds the native grammar's coverage.

#### Scenario: Edit syntax newer than the admitted grammar

- **GIVEN** a file containing syntax not recognized by its admitted grammar
- **WHEN** native analysis cannot establish a reliable completion context
- **THEN** the affected native proposals are omitted without modifying source text
- **AND** ordinary editing and any independently eligible LSP source remain usable.

### Requirement: CMP08 Native Org context completion

Org completion SHALL reuse the existing native Rust Org structure, task,
tag/property and link observations. Its initial contexts SHALL cover effective
TODO states in heading task positions, tags, property keys, supported `#+`
directive names, recognized link prefixes and current-document heading targets.
Candidates SHALL reflect the current unsaved document and admitted configuration.

This source SHALL use the shared popup, filtering, selection, checked edits and
undo contract. It SHALL require neither an Org Tree-sitter grammar nor an LSP.
The ordinary Org Tab behavior SHALL remain available outside an active completion
popup. Automatic requests SHALL be limited to recognized structural completion
contexts; ordinary prose SHALL NOT acquire dictionary completion by implication.
Incomplete candidate prefixes SHALL be handled within the enclosing admitted
context, without treating literal/example blocks as Org directives or metadata.

#### Scenario: Complete a configured Org task state

- **GIVEN** an Org heading task position and an effective TODO workflow
- **WHEN** a compatible prefix requests completion
- **THEN** configured task states appear through the common completion interaction
- **AND** no additional grammar or language-server process is required.

#### Scenario: Complete an internal link to an unsaved heading

- **GIVEN** a current Org document containing an unsaved heading
- **WHEN** completion is requested after an admitted `[[*` prefix
- **THEN** the heading is offered as a current-document link target
- **AND** acceptance uses the ordinary checked edit and undo owner.

### Requirement: CMP09 Optional external language support

The bundled language catalog SHALL be a factory baseline rather than an
allowlist for compatible external support. Within the selected compatibility
profile, optional grammar assets SHALL be able to update a bundled language
or add an unbundled one without rebuilding rev. Additional compatible installed
LSP servers SHALL remain admissible independently of bundled grammar coverage.

External grammar admission SHALL validate the parser/query combination and
its compatibility with the available engine and platform. Expected admission
failures SHALL preserve available native fallback and ordinary editing. The
editor-facing contract SHALL remain limited to language analysis and declared
file associations; a general editor automation API is not implied.

#### Scenario: Add an unbundled language through an admitted external source

- **GIVEN** a language not included in the factory grammar bundle
- **WHEN** the user configures a compatible installed LSP server or admits its
  external grammar/query assets under the selected profile
- **THEN** the declared external support is available without rebuilding rev
- **AND** the absence of that optional source preserves ordinary editing.

## Delivery profile

These are implementation defaults for the approved scope, not claims about
already shipped behavior. The linked plan owns the exact slice boundaries.

### Invocation, interaction and ranking

- Default `completion.enabled = true`, `automatic = true`,
  `delay_ms = 150`, `min_prefix = 2`, `in_comments = false`,
  `in_strings = false`, `max_candidates = 200` and `popup_rows = 10`.
  The minimum counts Unicode identifier characters. An admitted language
  separator or Org structural prefix can trigger with an empty identifier.
  Explicit invocation bypasses the delay and minimum prefix.
- `completion-at-point` is the manual command, bound to `M-TAB` where the
  terminal delivers it. M-x remains available when the terminal intercepts it.
  `completion-mode` and `completion-auto-mode` toggle session preferences.
  Existing TOML startup semantics remain unchanged.
- A completion input context provides `completion-next` / `completion-previous`
  on Down/C-n and Up/C-p, `completion-accept` on Tab/Enter and
  `completion-cancel` on C-g/Escape. Other movement, pane changes, prompts,
  searches and buffer changes dismiss the popup before ordinary dispatch.
  Typing/deleting an admitted prefix schedules a fresh request. Cancellation
  suppresses automatic reopening until the next text edit or explicit request.
  Bracketed paste does not trigger a request per pasted character.
- Use smart case: lowercase input matches case-insensitively; an uppercase
  character requests case-sensitive matching. Rank exact prefix, then ordered
  subsequence; within a class prefer the current buffer, nearer declarations,
  source ordering and finally a stable lexical tie-break. Deduplicate equal
  insertion/range proposals without conflating distinct edits.
- Completion never inserts a snippet program, executes a command, saves a
  source file or edits another buffer. Native candidates contain literal text.
  No popup appears without usable candidates. Complete a single valid candidate
  through the same visible selection/accept path, not an unsolicited edit.

### Native coverage and work bounds

Use explicit language queries and retained revision-qualified syntax owners,
not regex token mining. Query captures identify declaration names and their
lexical scope. The current buffer wins over other same-language open buffers;
no unopened file is read to build completion context. Cross-buffer proposals
are observed names, not a promise of imports, type checking or member resolution.

| Source | Initial structural candidates |
| --- | --- |
| Rust | Functions/methods, types/traits, modules, fields, constants and local/parameter bindings. |
| C | Functions, typedef/type/enum names, fields, enumerators and local/parameter bindings; no macro expansion. |
| Go | Functions/methods, types, fields, constants and local/parameter bindings. |
| JavaScript / TypeScript / TSX | Functions, classes/types, imports, fields and local/parameter bindings; TSX uses its own parser. |
| HTML | Observed tag/attribute names and the bounded standard tag/attribute catalog. |
| CSS | Observed selectors/custom properties and the bounded standard property catalog. |
| JSON | Observed object keys in structural key contexts; no schema service. |
| TOML | Observed table/key names in structural key contexts; no schema service. |
| Org | Effective TODO states, tags, property keys, supported directives/link prefixes and current-document heading targets. |

Native suggestions may be useful syntactic names without semantic certainty.
Exclude error-dependent captures and do not manufacture properties from an
unknown receiver type. HTML/CSS catalogs are finite compiled data, not another
indexing engine. JSON/TOML keys and Org structural prefixes are admitted as
structural contexts even where their syntax includes quote characters; ordinary
string values and Org prose remain governed by the normal suppression policy.

Default native admission limits are 2 MiB per analyzed buffer, 8 MiB total
source snapshots, 32 observed buffers, 4096 symbols per buffer and 16384 total
retained symbols. Keep observation payloads within 8 MiB and individual labels
within 512 UTF-8 bytes. Proposed insertion text is limited to 64 KiB. Overflow
evicts least-recently-used other-buffer observations before current context;
an oversized current buffer omits native extraction and reports that limit.
No full scan runs during rendering or cursor movement. Use bounded, cancellable
worker jobs, at most one active and one newest pending native request. Parse
and query callbacks enforce a 50 ms job work deadline; publish only current
buffer/revision/view/grammar/session generations. A limit outcome is not a
claim that no matching name exists. Idle completion has no polling timer.

Org completion reads current native observations and configured vocabulary.
TODO candidates apply after the heading stars and initial space; tags in a
heading tag field, properties in a property drawer, directives after `#+`,
link types after `[[` and heading targets after `[[*`. Do not interpret source,
example or other literal block contents as Org metadata. Native source-edit
buffers use their declared language and the same checked-edit contract.

### Optional external grammar assets

Select precompiled native shared libraries for the current target, matching
Tree-sitter queries and explicit user registration. Use the safe
`tree-sitter-loader = 0.27.0` loading API with default features disabled;
do not embed its CLI, Wasm runtime, compiler invocation or automatic discovery.
Retain `#![forbid(unsafe_code)]` in all first-party roots. Upstream loader
internals own FFI; first-party custom FFI/raw-pointer adapters are prohibited.

Each `language.<id>.external` registration names `parser`, `symbol`,
`highlights` and `completions`. Existing `files.associations` chooses the
language. Paths relative to a registration resolve under the resolved XDG
data root's `rev/grammars` directory; explicit absolute paths are also valid.
Use versioned asset directories. No folder is scanned or installed at startup.
The UTF-8 query contract uses named declaration/scope captures documented by
the native extraction owner; external authors need not alter rev's binary.

Read registrations from startup TOML; load an explicitly selected asset lazily
on first use, off the terminal input/render loop. Validate paths, query lengths,
symbol name, parser ABI and both queries before publishing that generation.
Limit registrations to 32 and each query to 1 MiB. Cache the one admission
outcome per registration/session; do not repeatedly load a failed library.
The upstream loader retains loaded libraries for process lifetime. Therefore
there is no in-process unload, replacement or file-watching contract: install
another versioned directory, update registration and restart rev to activate it.
Never overwrite a mapped library in place. Failure retains the bundled source
when one exists, or plain editing for an unbundled language. External native
code is user-trusted code, not a sandbox. Grammar/query admission cannot
contain arbitrary native-code faults.

The approved grammar pins are those measured below: Rust 0.24.2, C 0.24.2,
Go 0.25.0, HTML 0.23.2, CSS 0.25.0, JavaScript 0.25.0, TypeScript/TSX 0.23.2,
JSON 0.24.8 and tree-sitter-toml-ng 0.7.0, on engine 0.27.1. Measure loader and
actual integrated feature bytes separately from the grammar-only experiment.

## MODIFIED Requirements

### Requirement: R31 Bundled Tree-sitter highlighting

Target: [rev Editor R31](rev-editor.md#requirement-r31-bundled-tree-sitter-highlighting).
This delta supersedes R31's fixed Rust-only selection and completion/external
grammar exclusion for this approved delivery; it preserves R35 in full.

Supported languages SHALL use the admitted bundled or explicitly registered
external Tree-sitter parser/query pair. The initial bundled catalog SHALL be
the closed selection in CMP01-CMP09. Supported captures SHALL distinguish
common keywords, strings, comments, types, functions, numbers and
punctuation/operators where provided by the grammar. Highlighting SHALL be
visual only and preserve document content. Unsupported syntax or failed
admission SHALL preserve ordinary plain-text editing. Native completion SHALL
follow CMP01-CMP09; optional semantic completion belongs to the separate LSP
delivery. No runtime grammar compiler, downloader or installer is introduced.

#### Scenario: Highlight Rust and retain plain-text fallback

- **GIVEN** a Rust file and a file with an unsupported extension
- **WHEN** the user opens and edits both
- **THEN** Rust keywords, strings and comments receive suitable styles
- **AND** the unsupported file remains editable as plain text.

## Dependencies and planning readiness

Reuse native input, checked edits/history, configuration and admitted
Tree-sitter grammars, plus the existing native Org owners. Native observations
are limited to the current and other open buffers. The optional LSP client supplies a semantic source and
must be integrated with this UI/fallback contract. Managed server installation
is not required to use an already installed server.

The delivery profile closes the implementation-readiness decisions. A native-only stage does not satisfy the full
source-switching acceptance contract. No generic programmable provider surface
or additional local indexing engine is implied.

## Validation direction

Cover current/dirty-buffer symbols, Unicode positions, index invalidation,
source switching, cancellation, atomic undo, real terminal interaction and
bounded open-buffer/idle work. The native subset passed the controller-owned
full gate; source-switching acceptance remains assigned to the LSP successor.
The following experiment retains its separate historical grammar-linkage
evidence, independently of the integrated delivery result.

## Native delivery state

The eight canonical PRs #195-#202 deliver the selected native profile: manual
and automatic interaction, ten bundled parser/query pairs, native Org contexts,
bounded open-buffer observations, checked edits and registered external assets.
The unchanged full gate passed at the validated candidate above. Exact command,
receipt, integrated release bytes, idle/resource observations and real PTY
acceptance are recorded in the [tracker](../tracker/rev-autocompletion.md#final-gate-and-closure)
and [Slice 8 report](../report/slices/rev-autocompletion/slice-8-acceptance.md#delivery-set-closure).
Completion of this native scope does not claim CMP02 semantic/native source
switching; that combined acceptance belongs to the approved LSP successor.

## Engineering evidence: executable footprint and grammar maintenance

### Question and metric

The product question is whether the selected language coverage preserves a
small executable. The measured quantity is executable file length in bytes;
source size, downloaded crates, build-directory size, compressed archives and
resident memory are different quantities and are not used as substitutes.

Let `S0` be the current release executable size and `Si` the size of variant
`i`. The normalized footprint is `ri = Si / S0`. The marginal increment is
`delta_i = Si - S(i-1)` for the stated accumulation order. This definition makes
the existing editor the unit `1.00`, rather than an empty parser benchmark.

### Experimental design

The experiment was performed on 2026-10-31 against source revision
`6e38bb95fdf0ff81cec6d48a6e8b3bdaa9130fda`. The target was
`x86_64-unknown-linux-gnu`, using Rust/Cargo 1.98.0, LLVM 22.1.8 and GCC 16.2.1.
Tree-sitter remained pinned at 0.27.1.

1. Build a clean source snapshot with the normal Cargo release profile,
   locked dependencies and two build jobs. Its executable is 13,326,584 bytes,
   equal in size to the existing release artifact.
2. Link that unchanged editor library through a small measurement entrypoint.
   Retain the normal editor path and add a separate path that exercises each
   selected parser and compiles its highlight/tag queries. An unused dependency
   is insufficient evidence because the linker can discard it.
3. Measure a Rust-only control to quantify the entrypoint and retained-query
   overhead. Add the other grammars cumulatively, keeping the release profile
   unchanged. Apply no post-link stripping or compression.
4. Execute each variant's parser/query checks before recording its file length.
   Measure TSX separately because its crate contains a distinct parser.

The grammar versions were Rust 0.24.2, HTML 0.23.2, CSS 0.25.0,
JavaScript 0.25.0, TypeScript 0.23.2 and Go 0.25.0. The separately measured TSX parser
comes from the same TypeScript crate. All parser/query checks passed.

The clean and measurement Cargo lockfile SHA-256 values were respectively
`4aafdab4a026c7ec2c89f7811886c5e4af7550383ad8f0c61bb48d16a6b17e84`
and `579e8516691b146101292300876b3e4fef546c34e4bbf473977d0b98921f3064`.
Raw measurements, build logs, the measurement entrypoint and executables were
retained in an unversioned experiment. No measurement dependency was added to
the canonical application manifest.

### Measured results

The configurations below are cumulative; their order is not a delivery order.
Go belongs to the initial delivery scope.

| Configuration | Executable bytes | `Si / S0` | Increment from preceding row, bytes |
| --- | ---: | ---: | ---: |
| Current editor, Rust grammar | 13,326,584 | 1.000000 | 0 |
| Rust-only measurement control | 13,373,376 | 1.003511 | 46,792 |
| Control + HTML | 13,398,288 | 1.005381 | 24,912 |
| Previous + CSS | 13,522,200 | 1.014679 | 123,912 |
| Previous + JavaScript | 13,950,448 | 1.046813 | 428,248 |
| Previous + TypeScript | 15,374,344 | 1.153660 | 1,423,896 |
| Previous + Go: all six languages | 15,601,840 | 1.170731 | 227,496 |
| All six + TSX parser | 17,056,008 | 1.279848 | 1,454,168 |

The six-language variant is approximately `1.17` times the current executable:
13.33 MB becomes 15.60 MB, using decimal MB. Its gross increase is 17.073062%.
The control accounts for 46,792 bytes, or 0.351118% of the baseline. The five
additional grammars and their retained queries contribute 2,228,464 bytes
relative to that control, equivalent to 16.721945% of the baseline. Reported
ratios retain the control overhead instead of subtracting it.

### Interpretation and limits

- Marginal cost is heterogeneous. In this configuration TypeScript contributes
  substantially more bytes than HTML, CSS or Go. Language count alone is not
  a useful executable-size budget.
- A language name may correspond to more than one parser. TSX adds a separate
  measured increment and is not silently included in the six-language result.
- The measurements establish grammar-linkage cost for these versions and this
  toolchain. They are not a prediction of the complete feature release: the
  future completion interface, LSP client, external-grammar loader, additional
  extraction queries and Org completion adapter are not implemented here.
- The existing editor core is retained in every variant, but the measurement
  entrypoint differs from the production entrypoint. The control exposes that
  difference. Marginal deltas also depend on linker decisions and payload reuse.
- Parser/query checks establish retained, loadable grammar payloads for the
  samples. They do not establish comprehensive language completion, performance
  bounds, version coverage or production integration correctness.
- No CPU, resident-memory, startup-latency or aggregate installation-footprint
  conclusion follows from these executable-byte measurements. Other platforms,
  compiler versions, profiles and grammar versions require fresh measurements.
- The observed ratio is evidence for a packaging decision, not an approved
  permanent size ceiling or authorization to add unmeasured languages.

### Candidate comparison and the C decision

A second experiment retained the same editor revision, toolchain, engine and
release profile, adding pinned candidate grammars through a separate adapter.
Its six-language control was 15,602,096 bytes, 256 bytes larger than the first
adapter's corresponding result. Candidate increments below are measured
against this second control, so the adapter difference is not attributed to a
candidate. Every ratio still uses the original `S0 = 13,326,584` byte unit.

| Added candidate | Grammar version | Increment over six-language control, bytes | Complete executable bytes | Complete executable / `S0` |
| --- | --- | ---: | ---: | ---: |
| C | 0.24.2 | 633,792 | 16,235,888 | 1.218308 |
| Zig | 1.1.2 | 702,800 | 16,304,896 | 1.223487 |
| Odin | 1.3.0 | 2,232,840 | 17,834,936 | 1.338298 |
| Python | 0.25.0 | 475,016 | 16,077,112 | 1.206394 |
| JSON | 0.24.8 | 7,976 | 15,610,072 | 1.171348 |
| TOML (`tree-sitter-toml-ng`) | 0.7.0 | 30,224 | 15,632,320 | 1.173018 |

The candidate lockfile SHA-256 is
`4f22d330fb451292eaaf6cd66f16d47218d76b69dd8465973f7a1391e975f762`.
The Zig crate does not export its highlight query as a Rust constant, so the
adapter retained the query from its matching upstream `v1.1.2` tag; that file's
SHA-256 is `27a7ffc77086ec6394c04e59b42f0da4c9d3c294a0225bc155c80305ee54ca86`.
Dependency inspection confirmed one shared Tree-sitter 0.27.1 engine. The
runtime probes parse samples rather than executing the source languages.

Joint profiles were also measured because summing isolated increments is an
estimate subject to linker/payload interactions:

| Profile added to the original six | Complete executable bytes | Complete executable / `S0` |
| --- | ---: | ---: |
| C | 16,235,888 | 1.218308 |
| C, JSON and TOML, without TSX | 16,274,176 | 1.221181 |
| C, JSON, TOML and TSX: final selected profile | 17,728,320 | 1.330297 |
| C, Python, JSON and TOML | 16,749,320 | 1.256835 |
| C, Zig and Odin | 19,171,560 | 1.438595 |
| All six candidates | 19,684,960 | 1.477120 |
| All six candidates plus TSX | 21,139,128 | 1.586238 |

These observations reject an assumption that all small or emerging languages
necessarily have small parser payloads. Odin's measured grammar contributes
more bytes than the TypeScript addition in the first experiment. This is a
property of the measured implementation and profile, not an intrinsic size
classification of the source language.

After reviewing these costs, the owner selected C, JSON and TOML for the initial
native coverage and confirmed TSX as part of TypeScript support. The final
ten-parser profile measures 17,728,320 bytes, approximately `1.33` times the
original baseline; its ten parser/query checks passed. It covers nine
language/format families, with a second parser for the TSX variant. Native Org
context completion remains a separate source without another grammar. Zig, Odin, Python, C++ and
Java are excluded from the initial bundle; compatible external grammar/LSP
support remains allowed. This selection reflects product fit and maintenance
priorities as well as byte cost. The delivery profile above now defines external loading.

The C increment establishes parser/query linkage cost, not compiler-equivalent
understanding. Macro expansion and complete cross-file type resolution are
not supplied by the native syntax parser; an admitted optional server may
improve those cases. The initial six-language experiment is retained as
historical evidence and is not presented as the complete current scope.

### Update strategies and compatibility

Bundling a grammar fixes the syntax recognized by that release of rev; it does
not select the compiler version used by the edited project. Language evolution
can outpace a bundled grammar. Tree-sitter represents unrecognized syntax with
error nodes and can recover around incomplete input; CMP07 requires the editor
to preserve ordinary editing and omit unreliable native proposals.

| Strategy | Independent update unit | Executable-size consequence | Compatibility boundary |
| --- | --- | --- | --- |
| Bundled grammar | A new rev build containing the parser and queries | Parser/query payload is included in the executable | Pinned engine, grammar and queries validated together |
| Optional external grammar | A compatible parser plus its matching queries | Payload is outside the executable; loader cost is not measured here | Engine/grammar ABI, platform and query compatibility |
| Optional language server | An independently installed compatible server | Server payload is outside rev; client cost is not measured here | Admitted LSP profile and server prerequisites |

External parser loading is an established pattern in Emacs and Neovim. It can
decouple many grammar updates from editor releases, but not indefinitely:
a grammar requiring an unsupported Tree-sitter ABI may still require an editor
update. ABI compatibility alone does not establish query compatibility when
syntax-node names or shapes change; parser and query admission belong together.

Moving parsers into external libraries changes where their bytes are stored.
It does not demonstrate a reduction in total installed footprint or memory use.
An LSP update can improve semantic completion independently, but the current
LSP scope does not replace or update native syntax coloring.

The delivery profile selects optional external loading while preserving R35's
first-party safe-Rust boundary. The footprint of that loader and its actual
integration remains to be measured during implementation. Expected
incompatibility rejection is distinct from native-code fault isolation; no
external-loader isolation guarantee has been validated by this experiment.

### Org as a distinct source

Org has established context-sensitive in-buffer completion in Emacs. Rev's
existing Rust Org implementation already owns structure, TODO workflows,
tags/properties and link observations. CMP08 reuses those owners rather than
adding an Org Tree-sitter grammar or a parallel document model.

The baseline executable already contains native Org functionality. Connecting
it to the shared completion interface adds adapter/catalog code whose marginal
size has not been measured. Org is therefore included behaviorally without
being counted as an additional measured grammar, and its incremental cost is not
claimed to be zero or included in the measured grammar-only profiles.

### Primary references

- [Tree-sitter error and missing nodes](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html#special-nodes).
- [Tree-sitter parser language/ABI admission](https://docs.rs/tree-sitter/0.27.1/tree_sitter/struct.Parser.html#method.set_language).
- [Emacs language grammar loading and ABI compatibility](https://www.gnu.org/software/emacs/manual/html_node/elisp/Language-Grammar.html).
- [Neovim parser and query loading](https://neovim.io/doc/user/treesitter/).
- [Org in-buffer completion contexts](https://orgmode.org/manual/Completion.html).
