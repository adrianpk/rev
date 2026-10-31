# Slice 5: JSON and TOML key completion

Status: reviewing
Delivery set: rev-autocompletion
Plan: [Native Autocompletion Delivery](../../../plan/rev-autocompletion.md)
Tracker: [Native Autocompletion Tracker](../../../tracker/rev-autocompletion.md)
Branch: `feat/rev-autocompletion-05-formats`
PR: pending

## Purpose

Deliver the JSON/TOML subset of CMP03 and CMP05-CMP07 through the existing
retained syntax owner, bounded open-buffer observations and checked completion.

## Delivered Behavior

JSON and TOML have bundled parsers, highlight queries and explicit structural
extraction queries. JSON observes object keys at any nesting depth. TOML observes
pair-key components, including dotted/inline keys, and complete table and
array-table header paths. Only observed names are proposed; no schema or value
vocabulary is supplied.

Manual and automatic completion admit quoted structural keys with the default
string suppression enabled. Ordinary values, array elements and comments remain
quiet even for manual requests or enabled literal policies. Key/table delimiters
can trigger automatic requests without the ordinary identifier prefix minimum.
Escaped spellings decode to the same matching/deduplication identity. Literal
replacement supplies valid quotes/escapes; TOML uses bare spelling when valid.
Header completion replaces the whole path and preserves its surrounding brackets.
Quoted path segments preserve the distinction between literal and structural dots.
Acceptance remains one checked edit and one undo operation.

Current and same-language open buffers contribute actual unsaved observations.
Undoing a donor edit removes its name on the next invocation. Unopened siblings
and the other format do not contribute names. Existing revision, path, view,
syntax-generation and source-availability checks still guard publication.

## Implementation Notes

Grammar additions are pinned to tree-sitter-json 0.24.8 and tree-sitter-toml-ng
0.7.0, preserving the single Tree-sitter 0.27.1 engine and first-party safe Rust.
Each admitted buffer still owns one parser, tree and compiled query set. The
existing temporary worker derives structural context and checked replacement
ranges from that same retained tree and shared rope snapshot.

Incomplete JSON keys can make the entire object an ERROR node. Recovery uses
only the bounded current token, preceding structural delimiter and retained-tree
container/siblings. An unfinished TOML multiline-value scanner can consume text
without returning body nodes; its retained ERROR value opener suppresses key
recovery. Regression checks retain both positive incomplete-key admission and
negative unfinished-value assertions. Recovery supplies context only; names
continue to come from explicit queries on valid independent declarations.

The shared 2 MiB per-buffer/8 MiB total snapshot, 32-buffer, 4096 per-buffer/
16384 aggregate symbol, 8 MiB payload, 512-byte label and 64 KiB literal-edit
bounds remain unchanged. New token/context walks are bounded to 512 characters
or 128 retained-tree steps. There is still one active worker, one newest pending
request, cancellation and a 50 ms work deadline; no idle analysis or file scan.

## Contracts Added or Changed

The grammar enum/defaults, complete sample and reference catalog now admit json
and toml. Existing JSON/TOML filename associations select their bundled parsers;
explicit plain overrides still apply. Extraction distinguishes json-key,
toml-key and table captures from ordinary declarations and web vocabulary.
Completion and syntax references and the existing Unreleased entry describe the
new behavior. No new preference leaves, commands or dependencies beyond the two
approved grammar packages are introduced.

## Files of Interest

- `src/syntax/catalog.rs`, `json.scm`, `toml.scm`: parser/query admission and structural observations.
- `src/syntax/formats.rs`: retained-tree targets, incomplete-key recovery and value suppression.
- `src/completion/literal.rs`: bounded escape decoding, literal encoding and dotted-path spelling.
- `src/completion.rs`, `context.rs`, `job.rs`: shared context, worker and checked-edit integration.
- `tests/completion/formats.rs`: real manual/automatic workflows, escapes, errors and unsaved sources.
- `tests/syntax/preferences.rs`, `tests/configuration/boundaries.rs`: grammar settings, colors and plain overrides.

## Validation

T5.1 is commit `45233ad8f5e8e40a5d0ef391bcefe7984e32b50a`.
T5.2 is commit `6972747e5db4600bea9ce33adaaa93de2f0a86a7`.
Development `cargo check --offline` passed with only the two approved grammar
lockfile additions. `cargo test --locked --test completion formats` passed all
six integration cases after the incomplete-JSON and unfinished-TOML-value
corrections. The original admission/suppression assertions were preserved.
`cargo test --locked --test syntax preferences::grammar_policy` passed its
expanded parser/color/plain-override matrix. Development library Clippy and the
changed-owner source-quality diagnostic passed.

Focused controller validation and canonical PR references will be recorded in
the PR-reference follow-up. The final clean PR head requires this exact order:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test completion --test syntax --test configuration`
3. `git diff --check`

## Risks and Follow-ups

Names are observed syntax rather than schema validity or semantic knowledge.
Malformed or over-limit contexts may omit suggestions while preserving editing.
The remaining approved slices and integrated footprint/PTY/resource acceptance
retain their existing plan; the full delivery-set gate remains pending.
