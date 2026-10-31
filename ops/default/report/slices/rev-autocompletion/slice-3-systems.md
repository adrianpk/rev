# Slice 3: Rust, C and Go structural sources

Status: delivered
Delivery set: rev-autocompletion
Plan: [Native Autocompletion Delivery](../../../plan/rev-autocompletion.md)
Tracker: [Native Autocompletion Tracker](../../../tracker/rev-autocompletion.md)
Branch: `feat/rev-autocompletion-03-systems`
PR: [#197](https://forge.adrianpk.com/adrianpk/rev/pulls/197)

## Purpose

Deliver the systems-language subset of CMP03 and CMP05-CMP07 through the existing
syntax, completion interaction and canonical edit owners.

## Delivered Behavior

Rust, C and Go provide manual and automatic declaration suggestions with shared
checked literal acceptance and undo. C includes typedef/type/enum names,
functions, fields, enumerators and local/parameter declarations; Go includes
functions/methods, types, fields, constants and local/parameter declarations.
Rust adds representative destructuring, closure, conditional and enum coverage.
Explicit scope captures keep local bindings and parameters within their lexical
block/function; later bindings, closed blocks and other functions do not leak.
Incomplete source retains independent valid declarations, including local names
at an unfinished block's end. Error-dependent captures are omitted.

Current unsaved declarations rank before other same-language open buffers within
the same prefix/subsequence class. Proximity, source order and lexical ties are
deterministic; identical insertion/range proposals collapse. Other buffers
contribute their current unsaved global/member names. Unopened files and other
language identities are excluded. Proposals do not infer imports, macro expansion
or receiver types. C pointer `->` edits admit automatic requests; native comments
and strings retain configured automatic suppression.

## Implementation Notes

The syntax catalog selects pinned Rust 0.24.2, C 0.24.2 and Go 0.25.0 grammars
with one Tree-sitter 0.27.1 engine. Each buffer retains one parser/highlight query,
completion query and tree. A temporary worker borrows that owner and a shared
immutable rope snapshot; extraction and ranking stay off the input/render path.
Returned trees retain their actual revision. A changed source requires reparsing
rather than relabeling stale observations. Grammar or language-identity changes
rebuild derived syntax and advance its admission generation.

Admission enforces 2 MiB per analyzed buffer, 8 MiB total source snapshots and
32 observations. Recent source use orders other-buffer admission; current context
comes first. Extraction caps 4096 names per buffer, 16384 combined symbols, 8 MiB
symbol payload, 512-byte labels and 64 KiB literal edits. Explicit lexical scope
scratch data is bounded to 8192 nodes and 128 ancestor steps. Older other-buffer
observations are omitted before current context under aggregate pressure.
Manual notices distinguish limits from complete no-match outcomes. Cancellation,
parse/query callbacks and final worker publication check the 50 ms job deadline;
one active and one newest pending request remain the only native work slots.

Every published observation pins canonical revision, applied language identity,
grammar generation, associated path and live buffer availability. Edits, rename,
grammar/language changes and close retire the list without changing the target.
A change while work is in flight also rejects its result. Rendering reads only
retained proposals and metadata; idle completion has no timer or source scan.

## Contracts Added or Changed

The grammar enum and startup reference now admit `rust`, `c`, `go` and `plain`.
Rust/C/Go identities choose their bundled grammar by default; explicit plain
selection remains effective. Existing C/header and Go associations are reused.
Status presentation names the prepared grammar, including source-edit buffers.
Numeric captures from C/Go use the existing number palette role. The complete
sample, configuration/default catalog, syntax/completion references and the
existing Unreleased completion entry describe the delivered behavior.

## Files of Interest

- `src/syntax/catalog.rs`, `rust.scm`, `c.scm`, `go.scm`: admitted parser/query payloads.
- `src/syntax/completion.rs`, `scope.rs`: explicit names and lexical scopes.
- `src/completion/source.rs`, `job.rs`: source pins, finite snapshots and worker ownership.
- `src/session/completion/sources.rs`: current-first admission and source recency.
- `src/display/syntax.rs`: current grammar captures projected into terminal cells.
- `tests/completion/systems.rs`: native language, scope, context and limit cases.
- `src/session/completion/tests.rs`: invalidation, in-flight edits and aggregate snapshots.

## Validation

Development checks passed:

- `cargo test --locked --test completion systems`: seven real systems-language
  cases passed before the final incomplete-local extension.
- `cargo test --locked --test completion systems::incomplete_source`: the extended
  incomplete declarations/local-at-end case passed for all three grammars.
- `cargo test --locked --test configuration`: all 136 configuration cases passed.
- `cargo test --locked --lib completion`: ten ownership, admission and observation
  cases passed before adding the in-flight edit regression and identity case.
- `cargo tree --locked -i tree-sitter --depth 1`: one Tree-sitter 0.27.1 engine.

The early systems fixture replaced every `sam` substring in its expected text,
including a declaration's `sample` name. Its expectation now replaces the actual
source token only. Configuration fixtures retain explicit `plain` behavior and
reject an unsupported grammar instead of the newly admitted Go grammar. A
read-only publication check now preserves its explicit read-only diagnostic
before stale-source checks. These corrections preserve source/history assertions
and the configured resource limits.

The first controller batch `VNCSYH6KHB7G2HW3BLY6OZQII2` passed all 11 internal
completion cases and 136 configuration cases at `185fd05e1442cda8cc7ef45bbba0ec2f606f5057`,
then stopped in Clippy on a nested conditional and redundant integer cast.
Both style diagnostics were corrected without changing admission, deadlines or
observable assertions; fresh exact-candidate evidence follows.

Receipt `IWQSP6Z6OBVMOLMHOFYP7DCYW2` then passed the same 147 tests and
library Clippy at `dda24bcb6de0df4663a0186a9bafbc5f6055e155`. Its expanded
source check reported `ingress` (16) and `file_argument` (11) complexity in
`src/session/file.rs`; both failures were reproduced on unchanged primary dev
`bc9dbdd08526ad81dacc3f2c7a6fca3205dfa700`. The extra recency hook in file
ingress was removed, leaving that file identical to dev. Completion requests,
observation reuse and buffer selection retain source recency ownership; the
focused source command covers the final changed owners.

Required receipt `QGQIRGCBMFKOKMBO2PYWGA7CAZ` passed formatting and all
23 completion cases at `b7f3acb3cf5295d336e1c579a6b837c5767d1eb9`, then found
an obsolete Go-as-Text expectation in coordinate policy. Coordinate, visiting
alias and grammar-policy fixtures now expect the admitted Go/C modes; explicit
plain fallback, text/history immutability and coordinate assertions are retained.
The grammar-policy cases use valid representative source for each parser.

Required receipt `HBXJKGTLZ5QA3CHLHCG3JFLNPO` passed formatting, all 23
completion cases and all 24 coordinate cases at
`20ae9357ba3d43557a9054ccd4bec0038b26c86d`, then exposed an alias fixture
that resolved a custom C++ association without installing its snapshot in the
registry. It now uses `Buffers::with_settings` and also checks the buffer's actual
editing policy. The C++ plain fallback and first-alias/text assertions remain
intact; configured grammar identity now drives the real owner under test.

Required receipt `VFCMBMOVFGC4H2FTZQNHUURO4U` passed formatting and all
23 completion, 24 coordinate and 74 file cases at
`b001f8fd5d9351399a2c3d596d176d4608e56e22`. The syntax grammar-policy
assertion then exposed a real Rust-only color projection guard. The renderer
now consults the buffer's current syntax owner, admitting C/Go colors through
the same grapheme/cell projection. The systems interaction case also asserts
colors in the actual session frame, retaining the direct query and edit/undo
assertions. Fresh focused and configured evidence cover this correction.

T3.1 is commit `fcf66225fbcec203ad32c431a87a55ce6a2fc489`.
T3.2 is commit `c082cef64cf465d4ee2d0c2cb69381bf6c022ea4`.
Fresh focused controller receipt `RCTNIXKPM7NUW5YMGGSKLDET5M` passed at clean
candidate `1cab4b0b6549e4825b6c211f0ab85406b3b858e5`, including the renderer correction:

1. `cargo test --locked --lib completion`: 11 tests passed.
2. `cargo test --locked --test configuration`: 136 tests passed.
3. `cargo clippy --locked --lib -- -D warnings`: passed.
4. `cargo run --locked --quiet --example source-check -- --file src/buffer.rs --file src/buffer/policy.rs --file src/buffer/syntax.rs --file src/completion.rs --file src/completion/job.rs --file src/completion/matching.rs --file src/completion/source.rs --file src/config.rs --file src/config/resolve.rs --file src/session/buffers.rs --file src/session/completion.rs --file src/session/completion/sources.rs --file src/syntax.rs --file src/syntax/catalog.rs --file src/syntax/completion.rs --file src/syntax/scope.rs --file src/syntax/style.rs --file src/display/syntax.rs`: passed.
5. `cargo run --locked --quiet --example documentation-check`: passed.
6. `git diff --check`: passed.

The documentation check admitted 326 commands, 198 configuration entries,
60 reachable pages and 347 local links/anchors. Canonical PR #197 references the
full immutable report-introduction commit. Controller receipt
`YY3FXCH6SEAT74Q3OKMFYPD2UQ` passed all three configured checks and 151 tests at
final clean PR head `1469a344ebb18d067119951774e2447ab6e941b9`. Canonical PR #197
merged through Rebase + Fast-forward at that same commit. Its ordered list was:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test completion --test syntax --test files --test coordinates`
3. `git diff --check`

## Risks and Follow-ups

Syntactic observations do not establish semantic imports, macro expansion or
member types. Integrated PTY/resource acceptance and the remaining language
families retain their approved slices. The full delivery-set gate remains pending.
