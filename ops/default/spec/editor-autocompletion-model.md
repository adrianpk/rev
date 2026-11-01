# Native Autocompletion Model

Status: Completed
Date: 2026-10-31
Reviewed: 2026-10-31
Completed: 2026-10-31
Validated candidate: `0f7cc36cb1082aa56954fa602294f47aac80e889`
Gate receipt: `DAIRXONWTTLVG6RDIQQZKBSGXA`
Joint acceptance completed: 2026-11-01
Joint validated candidate: `610f2e019a52e6b1314931b11fa070d320ecf809`
Joint gate receipt: `YWRNPLD3FXNU6APAKXES5AS7I5`
Completion boundary: native interaction, sources and joint semantic/native acceptance delivered
Approved: product owner, 2026-10-31; implementation and ordered Looper execution requested
Specification: [Native Autocompletion](editor-autocompletion.md)
Semantic source: [Language Server Support](editor-language-server-support.md)
Org owner: [Org Mode Model](editor-org-mode-model.md)
Existing owners: [rev Editor Model](rev-editor-model.md), [Editor Configuration Model](editor-configuration-model.md)

## Joint source acceptance

The [LSP Slice 6 report](../report/slices/rev-language-server-support/slice-6-acceptance.md) records real semantic readiness separately
from transport readiness and native fallback. The same canonical edit/history
owner accepts the semantic primary and admitted additional edits. Source and
connection replacement retire the original invocation; empty or unavailable
semantic results obtain current native observations. Shared-root lifetime and
exclusions preserve buffer and view ownership.

The predecessor candidate and receipt above remain unchanged. Native source,
Org, external-grammar, resource and terminal cases passed the joint runner.
The LSP controller's unchanged full gate at
`610f2e019a52e6b1314931b11fa070d320ecf809`, receipt
`YWRNPLD3FXNU6APAKXES5AS7I5`, completes CMP02 and joint CMP01-CMP09.
The [Slice 6 closure](../report/slices/rev-language-server-support/slice-6-acceptance.md#delivery-set-closure)
records executed acceptance and resource observations.

## Ownership

The native completion owner selects, presents and accepts candidates. Candidate
sources supply proposals; canonical buffers still own edits and history.
Tree-sitter supplies structural candidates within admitted grammar/query
coverage. An enabled, operational LSP eligible for the buffer supplies the
preferred semantic source.
Org context completion reuses the existing Rust Org owners instead of adding
an Org Tree-sitter grammar or a second document model. Its fallback uses those
native owners when LSP is ineligible, unavailable or returns no candidates.
There is no ctags source or mandatory server process.
Initial Tree-sitter coverage is Rust, HTML, CSS, JavaScript, TypeScript, Go, C,
JSON and TOML, all in the selected delivery scope, with Rust as the primary
language. TypeScript includes the separate TypeScript and TSX parsers; the
initial language/variant selection is closed. External assets use the bounded
native-library/query profile in the specification. Zig, Odin, Python, C++
and Java are excluded from the initial bundle, which does not restrict the
catalog of compatible external grammar assets or installed LSP servers.

## Semantic records

| Concept | Evidence | Contract |
| --- | --- | --- |
| Completion session | Invocation, view/buffer identity, revision, point/prefix context and status. | Input/presentation lifetime is separate from source jobs and buffer ownership. |
| Source choice | Admitted language/grammar or Org context, buffer activation eligibility/generation, LSP readiness, selected source and reason. | Buffer exclusion, unavailable server and valid empty result are distinct; each selects the admitted native fallback for the current context. |
| Candidate | Stable session/source identity, display text, ordering/filter evidence and proposed replacement. | Display labels are not automatically insertion text; ranking remains an explicit design choice. |
| Proposed edit | Checked native range, replacement text and original buffer/revision/context evidence. | Encoding conversion and read-only/range admission precede publication; unsupported edit forms are explicit. |
| Grammar admission | Language, grammar/query identity, generation, recognized context and failure evidence. | Insufficient coverage suppresses unreliable native proposals without owning canonical text or disabling an independently eligible LSP source. |
| External language assets | Language identity, explicit registration, parser/symbol/query paths, admission outcome and generation. | Versioned native libraries use the safe upstream loader, XDG data and startup registration; lazy one-time admission, no runtime compilation, unload or hot reload. |
| Org observation | Canonical revision, admitted structural/prefix context, Org coverage and effective task/tag/property/directive/link facts. | Reuse current native Org observations and configuration; no new parser, canonical store or implicit project scan. |
| Structural observation | Canonical open-buffer identity, revision, language/grammar generation, scope and extracted symbols. | Current and other same-language open buffers only; no disk/project index or persistent symbol cache. |
| Source work | Request/job identity, cancellation state and finite input/result/work budget. | Late replies do not revive a cancelled or incompatible completion session. |

## Lifecycle and queries

Automatic language-context triggers are enabled by default for completable
separators and identifier prefixes; explicit command invocation remains
available. The default context policy suppresses automatic requests in comments
and string literals while preserving manual invocation there; users can change
that policy. An admitted trigger captures point and revision, resolves the eligible
source and requests bounded candidates. The native UI owns filtering, selection,
acceptance and cancellation. Candidate arrival is not permission to edit.
Tab and Enter accept an eligible selected candidate in the active popup;
outside that context, their ordinary editing behavior is preserved.
Acceptance revalidates the source target and applies an admitted edit as one
ordinary undo operation. Cancellation/failure leaves canonical text untouched.

Presentation is a compact vertical popup anchored to the source cursor. It uses
a single border, full-row selection highlighting and rev palette roles, fits
within the pane, and preserves source-view focus and document text layout.
The first candidate is selected when a nonempty list initially appears;
selection alone does not apply an edit.

Filtering supports typed characters in order without requiring consecutive
matches. Exact prefix matches rank ahead of subsequence-only matches.

Org candidates use the same session and edit admission. Recognized structured
prefixes select effective task states, tags, property keys, supported directive
names and link prefixes/current-document heading targets. Ordinary prose and
literal regions do not select this source merely because they contain similar
characters. Outside an active popup, ordinary Org Tab behavior retains ownership.

Queries needed by the capability are prefix/context candidate lookup,
revision/grammar invalidation and source-readiness selection. Smart-case prefix
then subsequence ranking uses current-buffer/proximity/source/lexical tie-breaks.
The specification defines trigger defaults, native/Org coverage and finite budgets.
Changing buffer,
revision or source availability must invalidate or reconcile stale observations
under a documented policy, not reuse unrelated candidates by row number.

## Storage and readiness

Completion sessions, candidate lists and source observations are disposable.
Optional grammar assets updated outside the executable can replace a bundled
grammar or add an unbundled language within the admitted profile. Their
parser/query update unit, compatibility admission and loader are defined by the
specification. Runtime registration does not invoke an installer. Loaded native
libraries remain for process lifetime; replacing an admitted generation requires
restart, and the registration count bounds that retention. No persisted symbol
index is selected. Completion results carry literal, revision-checked edits.

The completed [plan](../plan/rev-autocompletion.md) and
[tracker](../tracker/rev-autocompletion.md) record eight integrated native
slices and the successful controller-owned full gate at the validated candidate
above. Native session, candidate/edit, grammar generation, Org observation and
bounded source-work contracts are delivered. The subsequent integrated LSP gate
recorded above establishes optional semantic readiness and source precedence,
completing CMP02 without replacing native closure evidence.
