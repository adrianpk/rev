# Slice 2: Fixed symbols and font flag

Status: reviewing
Delivery set: rev-better-gutter
Plan: [Better Gutter Delivery](../../../plan/rev-better-gutter.md)
Tracker: [Better Gutter Tracker](../../../tracker/rev-better-gutter.md)
Branch: `feat/rev-better-gutter-02-markers`
PR: `#283`

## Purpose

Deliver the approved fixed bookmark symbols, blue default and typed font-support
flag for BMK03, BTG01 and BTG04 after Slice 1's outside layout integration.

## Delivered Behavior

- `gutter.nerd_fonts` defaults to false and selects the compiled square `■`
  (U+25A0). True selects the compiled flat bookmark `` (U+F02E). Both occupy
  one terminal cell under the existing display policy.
- The existing bookmark palette role defaults to `#7aa2f7`. Explicit colors,
  including the old yellow default in existing files, retain precedence.
- The boolean follows ordinary include/root precedence and source provenance.
  Invalid types reject the complete candidate; a valid root cannot conceal an
  invalid included value.
- Numbering commands and reset retain the immutable startup choice. Splits and
  buffer rebinds retain it together with independent marker visibility.
- Legacy `bookmarks.marker` strings are accepted and discarded during decoding.
  They cannot change the symbol, acquire effective provenance or reject a
  candidate because of grapheme count, width or control characters. Other input
  types remain invalid.

## Implementation Notes

T2.1-T2.3 are implemented in
`c59fea87a73273887736a3a362f0cd19db18875f`. The typed API change, renderer and
updated default assertions form one cohesive implementation commit.

The gutter's existing startup policy owns `nerd_fonts`. Its fixed marker method
returns one of two compiled strings. No font override, detection, installation,
icon-name resolver or runtime font command is added.

Effective bookmark settings and views no longer contain an arbitrary marker
string. The partial input field retains only legacy string-type admission; its
deserializer returns no value. Merge and projection omit that retired leaf.
The documentation catalog excludes this one compatibility input from current
settings while still checking every active setting against the live model and
packaged sample.

Frame painting keeps the existing bookmark palette role, revision-qualified row
index and geometry. Bookmark identity, source coordinates, storage format and
history owners remain unchanged.

The complete sample, value catalog, gutter reference and migration guidance now
describe the boolean and compiled blue. The existing Unreleased bookmark outcome
was revised once. Historical bookmark acceptance helpers now expect the changed
factory square; their PTY gate was not run for this slice.

## Contracts Added or Changed

- The public startup choice is a boolean, with false distinct from an omitted
  source leaf during composition and provenance reporting.
- Effective settings expose no custom glyph. Legacy strings are input
  compatibility only, and no configuration rewrite or bookmark migration occurs.
- A running view retains its font choice and explicit palette even after the
  file changes; a newly prepared startup receives the new boolean.

## Files of Interest

- `src/config.rs`, `src/config/merge.rs`, `src/config/project.rs`:
  typed boolean composition and publication.
- `src/config/bookmarks.rs`, `src/config/validate.rs`:
  legacy decoding and removal of arbitrary-glyph validation.
- `src/gutter/policy.rs`, `src/view.rs`, `src/display/gutter.rs`:
  fixed selection and view lifetime without a retained custom string.
- `src/config/palette.rs`: compiled bookmark blue.
- `tests/configuration/`, `tests/bookmarks/markers.rs`, `tests/gutter/policy.rs`:
  defaults, layering, rejection, renderer cells, immutable startup and source state.
- `docs/reference/config.toml`, `docs/reference/defaults.md`,
  `docs/reference/gutter.md`, `examples/documentation_check/catalogs.rs`:
  current preference inventory and migration guidance.

## Validation

All required commands passed locally, in the approved order:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration --test bookmarks --test gutter -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The suites passed 250 tests: bookmarks 58, configuration 156 and gutter 36,
with no ignored tests. Native frames verify both code points, one-cell width,
both gutter sides, the default blue and an old explicit color. Cases also
preserve record identity, source revision/state, saved bytes and empty history.
Existing split/rebind checks use the Nerd Fonts choice, retain a changed-on-disk
startup value through reset and verify a newly prepared startup separately.

Documentation validation passed 596 commands in each combined inventory,
281 configuration entries, 85 reachable pages and 561 local links and anchors.
Observed command times were 3.60s for formatting, 88.96s for all-target Clippy,
139.24s for the suite command, 1.48s for documentation and 0.01s for diff hygiene.
The initial focused configuration run compiled the isolated target in 1m59s
and passed three bookmark-configuration tests.

Before handoff, Looper must record these five commands against the final clean
PR-reference HEAD. Its retained receipt identifies that candidate; local results
do not replace the exact-HEAD evidence or the controller-owned final gate.
Tests release their owned temporary fixtures.

## Risks and Follow-ups

Native evidence establishes emitted code points, geometry and renderer colors;
it does not establish the appearance or availability of a physical terminal font.
Joint terminal journeys, complete guidance, Spore build and baseline reconciliation
remain Slice 3. The retained controller owns integration, the frozen final gate
and successor startup. No final-gate or public deployment result is claimed.
