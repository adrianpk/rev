# Slice 2: Heading presentation

Status: reviewing
Delivery set: rev-markdown
Plan: [Markdown Delivery](../../../plan/rev-markdown.md)
Tracker: [Markdown Tracker](../../../tracker/rev-markdown.md)
Branch: `feat/rev-markdown-02-headings`
PR: pending

## Purpose

Deliver T2.1-T2.3: level-specific heading faces, source-preserving ATX symbols
and independent view controls. List and table actions retain their later slices.

## Delivered Behavior

- Six ATX levels use the approved ◉, ○, ✸, ✿, ◆ and ▶ symbols and six exact
  Tokyo Night heading colors. Setext titles use their admitted levels and retain
  literal underlines; inline colors precede ordinary heading text.
- Only parser-admitted opening hashes are substituted. Quotes/list containers,
  indentation, closing hashes, seven-hash paragraphs and literal code retain
  their source meaning. Prefix width stays canonical: spaces end in one symbol.
- markdown-pretty-mode toggles the current view; markdown-reset-presentation
  restores its immutable markdown.pretty startup preference. Both profiles and
  read-only Markdown buffers admit these named controls without settings writes.
- Splits inherit the current preference then change independently. Buffer
  switches retain it; Text mode suspends it. Disabling syntax colors retains
  observations and symbols. Explicit re-enablement can retry canceled work.

## Implementation Notes

Implementation and report introduction: `149ec09768b755353452faca071aaeea187a21f2`.

The private adapter stores source byte prefixes only for admitted ATX headings.
Setext titles that start with a literal hash cannot acquire a false ATX symbol.
The display resolves those bytes through existing checked source coordinates,
then substitutes only visible ASCII prefix cells after clipping and wrapping.
Every replacement retains the original cell count. Cursor, hit, highlight,
mark, copy and persistence owners continue to use canonical source coordinates.
No second source map or per-frame parsing is introduced.

A view retains a startup boolean and optional explicit override by value. Split,
rebind and retained-view restoration preserve that policy independently of Org.
Markdown syntax remains owned by its private adapter; no Markdown semantic rule
is added to Org syntax or the shared geometry owner. Dependency and source-work
bounds are unchanged.

## Contracts Added or Changed

- markdown.pretty defaults true with optional parsing, leaf merge/provenance,
  atomic rejection and the exact packaged sample/catalog.
- palette.markdown.heading and heading_2 through heading_6 are independent RGB
  leaves, with the six exact approved defaults and no Org palette dependency.
- Both named presentation commands join native selectors and the exact command
  description/reference inventory. No keymap overlay is introduced in this slice.
- Source view keeps colors; selection and search retain precedence and monochrome
  attributes. Repository references and Unreleased describe the usable behavior.

## Files of Interest

- src/markdown.rs, src/markdown/parse.rs and src/buffer/markdown.rs: admitted
  heading roles/prefixes and current structural observation access.
- src/display/markdown.rs and src/syntax/style.rs: visible prefix substitution
  and independent heading faces.
- src/view/markdown.rs, src/view.rs and src/session/markdown.rs: view policy and
  native command admission with explicit canceled-observation retry.
- src/command/markdown.rs and configuration owners: compiled commands/settings.
- tests/markdown/headings.rs, tests/markdown/presentation.rs and configuration
  Markdown cases: rendered cells, geometry, lifecycle and candidate boundaries.
- tests/support/surface.rs: decode legal reverse-selection SGR attributes.
- docs/reference/syntax.md, complete sample/catalogs and CHANGELOG.md.

## Validation

All four required development commands passed in canonical order on clean
implementation HEAD `149ec09768b755353452faca071aaeea187a21f2`. All 546 tests
across seven native targets passed without failures or ignored cases. This is
slice evidence; the controller owns integration and the full delivery-set gate.
Final worker acceptance requires one controller receipt on the clean published
PR HEAD after all PR-reference commits, without modifying that tested HEAD.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test commands --test display --test pointer --test org_headings --test org_indentation -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

| Target | Passed | Seconds |
| --- | --- | --- |
| commands | 205 | 603.80 |
| configuration | 167 | 257.01 |
| display | 47 | 35.10 |
| markdown | 21 | 42.16 |
| org_headings | 19 | 25.62 |
| org_indentation | 18 | 20.38 |
| pointer | 69 | 71.82 |

The native targets took 1055.89 s in total.
This measured cost does not change a timeout, command or acceptance condition.
Full workspace lint and the installed immutable-index hook pass all 1,857
first-party Rust files, both packages and 74 Cargo roots, with no blocking or
deferred diagnostics. Documentation passes 693 commands, 321 settings,
106 reachable pages and 752 local links/anchors; whitespace checks pass.

Focused development also passed the three heading cases, six presentation
cases, exact command description guard and corrected complete context matrix.
The updated overlapping search/selection plus copy/save case passed in 0.42 s.

Development corrections preserved the existing acceptance boundaries. The first
compile found the fixed command-array count and a native-job method spelling;
both were corrected. Initial full lint passed Clippy but found complexity 11
in document projection and selection dispatch. Cohesive text projection and
presentation dispatch now satisfy the unchanged limit of 10.

The initial focused presentation run passed five of six cases; its geometry
fixture attempted a cursor inside a combining grapheme. The corrected matrix
explicitly checks that rejection and round-trips valid source positions. The
next run exposed an expected-visibility error when a narrow clip ended before
the symbol; its assertion now uses both viewport edges. All six then passed.
The complete Markdown target passed 20/21: its old rejected-table fixture is a
valid Setext level-two heading, newly colored by this slice. That case now
asserts its actual heading role, and a separate invalid delimiter remains plain.
A first added delimiter fixture was itself a valid short-hyphen GFM table;
replacing its delimiter with non-delimiter text made the intended negative
case explicit. The complete context check passes. No production parser rule,
existing test identity, bound, timeout or validation command was weakened.

## Risks and Follow-ups

Physical terminal font coverage remains terminal-owned. Full PTY acceptance and
User Guide delivery remain assigned to Slice 6. Lists and table editing remain
assigned to Slices 3-5. The existing foreign crossterm unused_parens warning is
outside first-party source policy.
