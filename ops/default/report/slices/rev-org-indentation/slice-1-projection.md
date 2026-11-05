# Slice 1: Projection and checked mappings

Status: reviewing
Delivery set: rev-org-indentation
Plan: [Org Indentation Delivery](../../../plan/rev-org-indentation.md)
Tracker: [Org Indentation Tracker](../../../tracker/rev-org-indentation.md)
Branch: `feat/rev-org-indentation-01-projection`
PR: `#276`

## Purpose

Provide the internal checked geometry for OIN01, OIN03 and OIN05 before the
configured presentation becomes available in Slice 2.

## Delivered Behavior

- Complete, current native outlines identify the enclosing heading of body,
  blank and literal-block lines. Preamble has level zero; sibling and shallower
  headings replace the current level. Closed source, example, export and comment
  blocks own star-like contents before native outline boundaries are paired.
- Heading first rows add N-1 cells; body rows and heading continuations add 2N.
  Each margin saturates and is capped at max(0, width-2) before text allocation.
  Source spans, source-star positions and absolute tab columns remain intact.
- Wrapped fragments carry their virtual margin and checked projection identity.
  Virtual hits resolve to the fragment's real source start. Horizontal clipping
  and Visual Line destinations use the same source-column translation.
- Cache identity includes effective policy and complete structural evidence.
  Source edits, undo, mode suspension, policy changes and new observations retire
  affected geometry. Retained fragments reject stale source, policy or evidence
  before resolving a hit or navigation destination. Hits also require the
  observed pane width.

## Implementation Notes

T1.1-T1.3 are implemented in `54395f4feea63a7cc0de34582f4d18c3840c09de`. Native ownership stays in
`org::Outline`; lookup uses its sorted heading observations rather than scanning
source lines during a frame. Literal-block classification uses two bounded,
cancellable worker passes over the existing row observations.

The buffer owns an internal policy flag, initially false. No public setting or
command enables this incomplete presentation layer. Slice 2 will admit startup
configuration and integrate rendering, cursor and overlays together.

Wrap builders retain margins across cooperative suspension and checkpoint
restarts. Existing glyph work charges, the 8192-fragment ceiling and the 1 MiB
cached-text ceiling remain in force. Virtual padding is included in cached text
bytes; no second mapping cache, source parser, dependency or idle deadline was
introduced.

## Contracts Added or Changed

- Current native enclosing-heading evidence distinguishes heading starts, body
  and preamble. Incomplete or stale structure produces coherent literal layout.
- `view::Fragment` adds virtual margin metadata while `columns` continues to
  mean absolute source-cell columns. Width and structural-policy observations
  qualify derived hits and destinations; virtual columns never become scalars.
- Disabled indentation preserves existing wrapped and heading geometry.

## Files of Interest

- `src/org/visibility.rs`, `src/org/structure/rows.rs`, `src/buffer/org.rs`:
  native ownership and revision admission.
- `src/view/indentation.rs`, `src/view/wrap.rs`, `src/view/wrap/cache.rs`:
  margins, checked fragments, cooperative layout and retention.
- `src/view/hit.rs`, `src/view/visual.rs`, `src/view/visual/scroll.rs`:
  source-only hits and visual destinations.

## Validation

The following implementation checks passed locally:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib view:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings --test wrapping -- --test-threads=1`
- `git diff --check`

The library filter passed 17 tests, including eight new projection, cooperative
layout and retention tests. Heading and wrapping regressions passed 16 and 26
tests respectively. Native cases cover parent/deeper/sibling transitions,
preamble, tables/lists/drawers, literal stars, source immutability, tabs, wide and
combining graphemes, continuation restarts, clipping and stale observations.
A 200-level heading and long text make finite source progress at widths 1, 2,
3, 8 and 20. Repeated policy/reflow tests reach both existing cache ceilings.

Additional focused parser regression passed 11 tests:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_source structure -- --test-threads=1`.

Observed command times after initial compilation: format 2.70s, Clippy 15.20s,
heading/wrapping 78.76s, and focused structure 2.27s. The library filter spent
40.60s building and 4.98s executing tests. Compilation dominates the cold checks.

Looper records the authoritative exact clean-HEAD receipt after the final
PR-reference commit. This report's local checks do not replace that receipt.
The controller-owned combined gate, PTY journeys and guide build have not run
for this slice.

## Risks and Follow-ups

Configured rendering and overlay integration remain Slice 2; buffer-local
commands remain Slice 3; joint terminal/resource acceptance and guidance remain
Slice 4. This report does not claim the user-visible indentation mode is enabled
or the delivery set has passed its final gate.
