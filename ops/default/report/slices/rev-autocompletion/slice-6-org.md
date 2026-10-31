# Slice 6: Native Org context completion

Status: reviewing
Delivery set: rev-autocompletion
Plan: [Native Autocompletion Delivery](../../../plan/rev-autocompletion.md)
Tracker: [Native Autocompletion Tracker](../../../tracker/rev-autocompletion.md)
Branch: `feat/rev-autocompletion-06-org`
PR: pending

## Purpose

Deliver CMP08 with the shared completion interaction, checked edits and bounded
worker required by CMP01 and CMP04-CMP07.

## Delivered Behavior

Org completion proposes effective TODO states at the first heading word,
observed heading/file/declaration tags in a heading tag field, and observed
property keys inside a native property drawer. Valid file-local TODO declarations
replace configured vocabulary through the existing workflow model. Incomplete
property keys retain the parser's actual header/placement fact while malformed
text remains excluded from observed property names.

A finite catalog supplies 32 supported metadata/block/table/export directive
names after `#+` and ten admitted native link types after `[[`. Current-document
heading titles appear after `[[*`, without TODO/priority/tag metadata. Their
labels remain readable while insertion uses the native literal target encoder.
Another open Org document cannot contribute heading targets. Unsaved headings
become available on the next invocation without saving or scanning files.

Manual and automatic requests share structural admission. Heading spacing and
structural punctuation can bypass the ordinary identifier minimum. Missing
required colons are supplied once; existing colons and surrounding link brackets
remain in place. Ordinary prose, property/directive values, comments, inline
code/verbatim and literal block bodies remain quiet, including unfinished blocks.
An incomplete block-opening directive retains its own directive position.

Selection and cancellation preserve source/history. Acceptance is one checked
literal edit with one-step undo. Edits and obsolete source/view/session identities
retire proposals. Tab retains native Org visibility behavior outside an active
popup. Source-edit drafts complete their declared bundled language; draft undo
and checked one-transaction return/undo in the Org source remain available.

## Implementation Notes

The existing completion worker owns Org requests in the same one-active/one-newest-
pending slot as bundled language requests. An immutable rope snapshot and native
StructureScan feed the existing task/tag/property models and heading/link helpers.
No Org grammar, dependency, parser implementation or persistent completion index
is introduced. The worker's shared cancellation flag and 50 ms deadline also
bound native Org scan/model steps. Rendering still projects retained candidates
without completion extraction or an idle polling timer.

The parser retains an internal property-candidate flag from its native header and
placement rules. An incomplete key may otherwise classify that drawer as ordinary.
The flag admits only the structural slot; observed names still come from valid
property records. A shared heading-title helper keeps stored links and completion
consistent. Native Org source qualification additionally pins mode identity.

The 2 MiB buffer cutoff, 8 MiB shared snapshot limit, 4096 per-buffer/16384 total
symbol limits, 8 MiB symbol payload, 512-byte labels and 64 KiB edit limit remain.
Org observes only its current document. Its existing native scan profile retains
200000 nodes and 256 structural levels within the completion byte/deadline bound.
Current-line context is capped at 4096 UTF-8 bytes and the replacement token at
512 bytes. Limit/cancellation outcomes preserve source and history.

## Contracts Added or Changed

Native Org mode is eligible for the shared completion request lifecycle. Its
vocabulary and contexts use existing configuration and source metadata; no new
commands, preference leaves or packaged grammars are added. Source-edit buffers
retain their declared language rather than becoming outer Org metadata.
The completion/Org references and existing Unreleased entry describe these
outcomes. Other language sources and their bounded same-language reuse remain.

## Files of Interest

- `src/org/completion.rs`, `completion/context.rs`, `names.rs`, `vocabulary.rs`: native adapter and finite context catalogs.
- `src/org/structure.rs`, `structure/elements.rs`: native property-slot qualification.
- `src/org/property.rs`, `visibility/tags.rs`, `visibility/tags/query.rs`: existing metadata and title-model reuse.
- `src/completion/job.rs`, `source.rs`, `src/session/completion/sources.rs`: shared work slot, source identity and snapshot admission.
- `tests/completion/org.rs`: actual manual/automatic/source-draft workflows and suppression.
- `src/session/completion/tests.rs`: read-only publication coverage for both Rust and Org.

## Validation

T6.1 is commit `9a72abd26783c566c923dc6fe8c3b6535a4e5818`.
T6.2 is commit `5a37a49e73b948d33e6fc56d1f4ccd22dcf95504`.
Development `cargo check --offline` passed without lockfile/dependency changes.
`cargo test --locked --test completion org::` passed all six integration tests.
The property-context regression first confirmed the native ordinary-drawer
classification; its original Property assertion remains for recorded validation.
The workflow cases also caught incomplete block-opener suppression and deletion
of existing link brackets. Corrections preserved the original candidate and
literal edit/undo assertions. Changed-owner source-quality diagnostics passed.

Focused controller evidence and the canonical PR reference will be recorded in
the PR-reference follow-up. The final clean PR head requires this exact order:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test completion --test org_editing --test org_source --test configuration --test commands`
3. `git diff --check`

## Risks and Follow-ups

Org vocabulary remains limited to configured/native observed metadata and the
finite supported catalogs. Over-limit or unreliable contexts may omit proposals
while preserving ordinary editing. The remaining approved slices and integrated
footprint/PTY/resource acceptance retain their plan; the full gate remains pending.
