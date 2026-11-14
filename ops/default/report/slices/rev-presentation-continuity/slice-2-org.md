# Slice 2: Org presentation continuity

Status: reviewing
Delivery set: rev-presentation-continuity
Plan: `ops/default/plan/rev-presentation-continuity.md`
Tracker: `ops/default/tracker/rev-presentation-continuity.md`
Branch: `fix/rev-presentation-continuity-02-org`
PR: `#42`

## Purpose

Keep Org heading markers, indentation and structural heading faces continuous
through conservatively admitted body/title edits. Follow the
[plan](../../../plan/rev-presentation-continuity.md) and
[tracker](../../../tracker/rev-presentation-continuity.md).

## Delivered Behavior

Org headings retain their markers, prefix concealment, structural margins and
palette-derived level colors during admitted same-line body/title edits, before
fresh analysis finishes. All four indentation/pretty combinations work with
colors enabled or disabled. Current edited text remains visible immediately.

Before/after admission uses the native Org classifier. Recognized title edits
must follow a byte-identical ASCII-space prefix of unchanged level. Boundary,
definition, literal, malformed/uncertain and over-budget edits fall back to
literal source. Exact structural APIs retain their current-revision checks.

## Implementation Notes

The private borrowed display facade queries the existing complete outline at
checked mapped base line starts. It retains no second document/index and clones
no whole observation. Admission shares the Slice 1 32 KiB allowance across
payload/line inspection, bounded temporary copies, native classifier scans and
prefix probes; allocation refusal drops optional carry during preparation.
Reserved source installation still only transfers the prepared fixed mapping.

All retained lexical captures are suppressed while carried: admission proves
heading geometry, not inline context. Heading faces are reconstructed from
current source bytes and the active level palette. This conservative treatment
also drops uncertain cross-row lexical captures instead of shifting stale ranges.

A geometry identity remains stable across carry and an equivalent complete
replacement. Source stamps still change and reject old hits; unchanged-prefix
wrap cache entries survive. Different/incomplete replacements invalidate the
identity. Explicit quit retires carry without changing source/history or
automatically restarting analysis; a new explicit presentation request can retry.
Mode/syntax-policy changes retire the mapping through existing owners.

## Contracts Added or Changed

Implements the Org portion of PC01-PC04 and PC06-PC11, with reconciled OGP08/OIN05.
No new command, preference, dependency, source format or fold policy is added.
Markdown remains Slice 3; broader joint lifecycle/resource and terminal
acceptance remain Slices 4 and 5.

## Files of Interest

- `src/org/structure/presentation.rs`: bounded adapter over the actual classifier.
- `src/org/visibility/presentation.rs`: complete-base context and title admission.
- `src/buffer/presentation.rs`: shared charged admission and checked row mapping.
- `src/buffer/org/display.rs`, `src/buffer/org.rs`: borrowed display evidence and structural colors.
- `src/view/wrap/cache.rs`: stable geometry identity with ordinary source invalidation.
- `src/session/prompt.rs`: explicit cancellation retirement.
- `src/buffer/presentation/org_tests.rs`, `src/view/indentation.rs`: full-owner comparison and cache/hit assertions.
- `tests/org_headings/continuity.rs`, `tests/org_headings/lifecycle.rs`: intermediate native frames and source/cancellation behavior.
- `CHANGELOG.md`: one concise user-facing Org correction.

## Validation

Local pre-submit slice checks passed:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`: workspace formatting, Clippy with zero blocking/deferred diagnostics, and native source policy; 1,890 first-party Rust files, two packages and all 75 Cargo roots. Successful Cargo check took 1 minute 9 seconds.
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib view::indentation:: -- --test-threads=1`: seven passed, including carry geometry, unchanged-prefix cache retention and stale-hit rejection. Cargo rebuild 50.51 seconds; tests 0.93 seconds.
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings --test org_indentation -- --test-threads=1`: 25 heading and 18 indentation tests passed, with no failures or ignored tests. Cargo rebuild 2.50 seconds; execution 33.11 and 22.00 seconds.
4. `git diff --check`: passed, including staged source whitespace.

Focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib buffer:: -- --test-threads=1`
passed all 21 cases after final source changes (warm Cargo 0.05 seconds; tests
2.00 seconds). The generated two-row Unicode chain asserts 64 positive carries
against fresh complete analysis. The admission matrix asserts four accepted and
ten rejected cases; every accepted case also compares mapped levels/margins
with the full owner. Budget refusal leaves canonical editing available.

The native writer fixture checks 40 deliberately pending frames: five body/title
edits for each of four indentation/pretty combinations, with colors on and off.
Each samples the marker and ordinary title position, character and foreground
for three heading levels, then compares the equal exact replacement. Additional
cases cover active palette colors, lexical retirement, literal refusal, explicit
cancel/retry, Unicode, source copying, BOM/CRLF save, readonly refusal and undo.
Existing lifecycle tests retain their original assertions.

During development the first focused integration compile used the wrong
`LineByte` field; correcting it allowed the five-case continuity selection to
pass. The first full lint rejected a function at complexity 11; factoring its
bounded line preparation met the limit of ten. No acceptance or lint limit was
relaxed. Vendored crossterm retains its existing unused-parentheses warning.
The installed hook also passed complete quality checks against the immutable
staged implementation tree, including the final strengthened lexical fixture;
its cold private Cargo check took 1 minute 37 seconds.

Exact final clean-HEAD evidence is recorded separately by the controller's
validator after the report/PR-reference commits and push. No terminal fixture,
graphical claim or delivery-set gate is included in this slice. Native tests
release their own fixture resources; build and hook caches belong to this
dedicated checkout.

## Risks and Follow-ups

Shared lifecycle/resource acceptance remains Slice 4; terminal acceptance and
user guidance remain Slice 5.
