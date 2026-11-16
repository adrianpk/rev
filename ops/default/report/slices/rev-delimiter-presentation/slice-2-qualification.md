# Slice 2: Qualified presentation facts

Status: delivered
Delivery set: rev-delimiter-presentation
Plan: [Delimiter Presentation Delivery](../../../plan/rev-delimiter-presentation.md)
Tracker: [Delimiter Presentation Tracker](../../../tracker/rev-delimiter-presentation.md)
Branch: `feat/rev-delimiter-presentation-02-qualification`
PR: [#68](https://forge.adrianpk.com/rev/rev/pulls/68)

## Purpose

Deliver T2.1-T2.3: bounded typed spans and proved nesting for independent
presentation views, with exact source advancement and foreground priority.

## Delivered Behavior

The existing buffer-owned typed qualifier publishes immutable endpoint slices
qualified by source stamp, profile, policy and view generation. Both mates and
depth require typed lexical confirmation. Quotes retain their kind; comment,
string-body, fenced and ambiguous source never invents structural endpoints.
Malformed typed closers, missing mates, lexical uncertainty and resource limits
remain distinct outcomes. Unicode scalars and CRLF retain native coordinates.

Foreground pair/group requests keep their tickets, answers and scheduling
priority. Visual polls wait for pending foreground work. Completed pair work
can serve presentation and resume counted lookup with its original inspection
budget; cancellation returns its scanner to the foreground owner. Group answers
never become typed proof. A retained source prefix without a live request is
idle, so it cannot block visual progress.

Exact native transactions retain proved spans for ASCII word-interior edits
inside confirmed structural pairs. Every endpoint, mate and coverage boundary
is checked before publication and mapped from base to final scalar coordinates.
All 64 revisions of each Text/Org/Markdown/Rust burst expose correctly mapped
spans before another lexical poll. Undo/redo and multiple replacements use the
same native publication. Delimiter, quote, escape, newline, word-boundary,
replacement, mode/language/settings and reload changes discard unproved state.
Stale source/view generations and canceled requests cannot publish late results.
Read-only qualification and automatic-editing toggles preserve valid spans.

## Implementation Notes

One mutable lexical operation owns the buffer; there is no second parser,
source copy, timer, worker or dependency. Ordered endpoint facts extend that
owner's existing typed records and lexical checkpoints. Checkpoint restoration
removes mate claims learned after the retained prefix. Only confirmed endpoints
are cooperatively exported. Views retain a ticket/range and borrow one shared
sorted publication, capped at 8192 eligible visible endpoints per view. Extending
one viewport retains the previous publication throughout every poll. If the
extension is limited, already confirmed spans remain available. Overlapping
views extend one in-flight request without resetting its inspection budget.

Scanner allocations, stacks/checkpoints, fact capacity, pending export,
published spans, per-view capacity and inline owners count once in the shared
2097152-byte budget. Foreground polls evict visual retention before it can
reduce their native reserve. Actual allocation capacities govern admission.
Qualification and export use separate polls, each at most 256 steps with the
earlier caller deadline or existing 4 ms target. No budget resets per endpoint
or view. Byte/depth limits remain 262144/256 and exhaustion is conservative.

Native edit preparation captures a fixed proof for at most 16 replacements,
without copying endpoint arrays. Removed/inserted payloads and neighboring
source share a bounded inspection budget; quotes, escapes, delimiters and
newlines refuse. Org/Markdown additionally require an ordinary prose line;
Org refuses underscored tokens that could introduce inline source syntax. HTML
does not advance cached qualification. Known prefixes can advance while their
malformed or limited suffix keeps its conservative outcome. A publication identity rejects a
proof if another snapshot replaces it between preparation and installation.
These conservative misses rebuild through the same bounded lexical owner.

T2.1-T2.3 were introduced together in implementation/report commit
`f338d0f56f36d6c3e67740e3a597b6922a99f900`. A follow-up retains publications
during viewport extension, keeps overlapping views in one inspection budget,
and protects Org control-word qualification. Rendering and pulse integration
remain later slices.

Follow-up commit: `30096774eb1e54b615aabe28c13c26c360850446`.
Canonical review: [PR #68](https://forge.adrianpk.com/rev/rev/pulls/68).

## Contracts Added or Changed

- Buffer-owned presentation request/poll/borrow/cancel APIs and actual retention.
- Source/view-qualified endpoint slices; group results remain separate.
- Exact neutral native mapping, including history, with publication freshness.
- Existing inspection, depth, metadata and scheduling limits remain unchanged.
- Frozen acceptance contract, catalog, bindings, dependencies and hooks preserved.

## Files of Interest

- `src/pairs/presentation.rs`: endpoint, outcome and borrowed publication types.
- `src/pairs/presentation/store.rs`: shared export, view admission and accounting.
- `src/pairs/presentation/advance.rs`: bounded native transaction proof.
- `src/pairs/state.rs`: exclusive foreground/background/prefix ownership.
- `src/pairs/work/presentation.rs`: typed endpoint confirmation and checkpoint repair.
- `src/buffer/pairs.rs`: source-qualified native service API.
- `src/buffer/edit.rs`, `src/buffer/edit/publication.rs`: prepared proof/publication.
- `tests/pair_presentation/qualification.rs`: 21 nonempty focused cases.

## Validation

All three ordered required commands PASS once through looper validate at clean
published `733061bba1780466675111f86a3704fed71c977c`, integrated by PR #68.
Acta receipt: `01M7GXGZHQTC1GPDEJG15WJ0S4`. The combined gate remains
controller-owned.

- PASS: `cargo fmt --all -- --check`.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation qualification -- --test-threads=1`: 21 cases at the exact integrated HEAD.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo check --locked --lib`.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation qualification -- --test-threads=1 --nocapture`: 21 cases, 25.68 s excluding warm compilation.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs --test pair_navigation qualification -- --test-threads=1`: 21 existing pair/edit cases and two group/expression cases.
- PASS: focused native source policy over 15 changed source/test inputs, retaining
  safe Rust, parameter <=5, nesting <=3 and complexity <=10. The preserved
  staged hook remains responsible for complete workspace acceptance.
- PASS: introduction commit hook at staged tree `5f49046d143b44d5e348f257a7cebe3f2bbf653a`: full workspace Clippy with zero blocking/deferred diagnostics, 1987/1987 first-party Rust files and 81/81 Cargo roots, with no source-policy exception.
- PASS: corrected follow-up commit hook at staged tree `9f913b8ed548bd24fdce68d29bc17ae1074c31d1`: full workspace Clippy with zero blocking/deferred diagnostics and native policy across all 1987 files/81 roots.
- PASS: `git diff --check`.

Measured development observations (unoptimized, one build job, exclusive target):

| Fixture | Inspected bytes | Retained bytes | Poll slices | Maximum poll |
| --- | ---: | ---: | ---: | ---: |
| Depth 128 | 256 | 60920 | 3 | 0.218 ms |
| Depth 256 | 512 | 79352 | 5 | 0.254 ms |
| Depth 257, limited | 272 | 42488 | 3 | 0.212 ms |
| Oversized token, limited | 262144 | 32248 | 1026 | 0.245 ms |
| 5000 adjacent pairs | 10000 | 964088 | bounded lexical/export | 0.197 ms |
| Aggregate retention stress | bounded | peak 2097128 | bounded lexical/export | 0.321 ms |

An endpoint occupies 32 bytes on this target. The dense publication remains one
shared allocation: one original view, 20 additional full-range interests and a
tail viewport total 965096 retained bytes; the additional interests cost 1008
bytes. The full viewport reports its 8192-span limit; the tail reads the remaining
1808 spans without a copy. Stress cancellation leaves 528 bytes of inline owner
and retained view capacity. Expired deadlines, current/stale cancellation,
foreground continuation, lexical exclusion with/without syntax colors, native
read-only reload, transactional history and every mapped burst revision pass.
These observations are not a physical-key or terminal-frame acceptance claim.

Development checks corrected a test's Rust module paths and native source access,
extracted the transaction containment predicate to retain complexity <=10, and
corrected the reload fixture to write the actual visiting path. No acceptance
assertion, frozen oracle or resource limit was weakened. A new viewport-extension
regression first reproduced loss of the current publication, then passed after
retention was corrected. Additional cases protect overlapping request budgets,
neutral proved prefixes and Org inline-source control words. The follow-up hook
rejected staged tree `44d3eab34d4c9cd76bf27e34b95945b0ee77c7be` for one redundant Range
clone after the containment refactor; removing it preserves behavior and the
unchanged Clippy/source policy.

## Risks and Follow-ups

PR #68 integrated validated commit `733061bba1780466675111f86a3704fed71c977c`
into dev. All three required commands passed once through looper validate.
Fetched ancestry, clean state and no owned processes were verified; the integrated
worktree was removed without force. The combined gate remains controller-owned.

Neutral mapping intentionally refuses unproved lexical boundaries, HTML and
batches beyond 16 replacements and underscored Org words. Limited results may retain only a proved prefix.
Visible rendering, projection overlays and pulse lifetimes remain assigned to
Slices 3-6. The existing vendored crossterm parentheses warning is unchanged.
