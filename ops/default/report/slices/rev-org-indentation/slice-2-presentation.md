# Slice 2: Configured editing presentation

Status: reviewing
Delivery set: rev-org-indentation
Plan: [Org Indentation Delivery](../../../plan/rev-org-indentation.md)
Tracker: [Org Indentation Tracker](../../../tracker/rev-org-indentation.md)
Branch: `feat/rev-org-indentation-02-presentation`
PR: pending

## Purpose

Make the complete configured Org indentation path usable for OIN01, the startup
preference in OIN02, OIN03 and OIN05. Runtime commands remain Slice 3.

## Delivered Behavior

- `org.indent` is a typed boolean, default true, with ordinary include/override
  composition, assigning-source provenance and whole-candidate failure. The
  frozen startup snapshot supplies each buffer's policy. Explicit false restores
  the previous geometry; plain text and disabled Org remain unindented.
- Rendering, cursor placement, wrapped and horizontally clipped source hits,
  Visual Line destinations and drag autoscroll share the source-cell translation.
  Heading first rows and continuations use the native margins from Slice 1.
  Source tabs retain their original stops, and narrow panes retain finite progress.
- Syntax, linear selections, rectangles and search highlights start after the
  virtual prefix. Completion popups follow the projected cursor; image previews
  retain their logical row and use the same outer margin. Gutters, bookmark
  markers and folds retain their logical source identities.
- Source/pretty heading choices remain independent per view. Copy, rectangular
  copy, save and edit history consume source only. Structural edits and undo
  retire old evidence and reflow from current observations without padding source.
- The exhaustive sample, current configuration reference and Unreleased notes
  describe the usable startup preference. Existing heading and wrap geometry
  fixtures explicitly select `org.indent=false`; enabled native frames have
  separate session coverage rather than weaker legacy assertions.

## Implementation Notes

T2.1-T2.3 are implemented in `66e6654eeba004dd997c74144c55253aca5bc9b6`. Configuration extends the existing
partial model, leaf merge/provenance and snapshot projection. No new dependency
or configuration owner was introduced.

`view::Projection` describes a row's source-cell window, visible virtual prefix
and available source width. Wrapped fragments and horizontal viewports use its
translation. Frame-local source rows carry that same geometry for search;
selection, syntax and pretty-prefix painting offset their existing source spans.
This metadata is bounded by the visible pane rows and is not a retained cache.

The existing native observation worker, fragment work charges, 8192-fragment
ceiling and 1 MiB cached-text ceiling remain unchanged. No new idle deadline or
background owner was added. Pending structural evidence renders coherent literal
rows until the current observation arrives.

## Contracts Added or Changed

- The admitted `org.indent` startup leaf is independent of `org.headings` and
  syntax coloring. Buffer Org activation gates its effective value.
- Projected surface columns include the virtual prefix; logical `Cell`, source
  spans, tab stops, rectangle columns and saved text do not.
- Pretty prefix replacement starts after virtual padding and preserves each
  original source-star cell. Search and selection never paint the virtual prefix.
- Older source/pretty and pre-indentation wrap fixtures now name their original
  policy explicitly, including the existing heading acceptance driver.

## Files of Interest

- `src/config.rs`, `src/config/merge.rs`, `src/config/project.rs`,
  `src/buffer/policy.rs`: admission, provenance and shared startup policy.
- `src/view/indentation.rs`, `src/view/hit.rs`, `src/view/wrap.rs`, `src/view.rs`:
  shared source/surface translation, hits and reveal.
- `src/display.rs`, `src/display/headings.rs`, `src/display/search.rs`,
  `src/display/preview.rs`, `src/session/pointer/autoscroll.rs`: rendering,
  overlays, result anchors and pointer destinations.
- `tests/org_indentation.rs`, `tests/configuration/org_indentation.rs`:
  real native sessions and typed configuration boundaries.

## Validation

The following implementation checks passed locally:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_indentation --test org_headings --test wrapping --test display --test pointer --test gutter --test configuration -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The required native batch passed 346 tests: configuration 155, display 39, gutter
36, headings 16, indentation 9, pointer 65 and wrapping 26. The new session cases
cover both interaction profiles, enabled/disabled and source/pretty geometry,
continuations, tabs and wide/combining graphemes, clipped and narrow panes,
source-only linear/rectangle copy, search/syntax offsets, completion and image
anchors, structural edits/undo, split views, folds, gutters and bookmark identity.
Configuration cases verify include provenance, explicit false, frozen snapshots,
invalid included types and unsupported language overrides.

Additional focused regression passed 17 tests:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib view:: -- --test-threads=1`.
This includes the native projection, cooperative work and cache-retention bounds
introduced in Slice 1.

Documentation validation passed 593 commands, 281 configuration entries,
85 reachable pages and 554 local links/anchors. Observed warm command times were
format 2.72s, Clippy 1.35s, required native tests 171.38s, documentation 1.27s and
additional view regression 48.47s. The view filter included a 43.30s first build;
the configuration suite accounted for 92.92s of the required batch.

Looper records authoritative exact clean-HEAD evidence after the final
PR-reference commit. These implementation checks do not replace that receipt.
The controller-owned combined gate and real PTY journeys have not run for this
slice; the Markdown-derived Spore build remains in the approved later work.

## Risks and Follow-ups

Buffer-local toggle/enable/disable commands remain Slice 3. Joint terminal and
resource acceptance, the Org learning path and specification/model reconciliation
remain Slice 4. This report does not claim full delivery-set acceptance.
