# Slice 1: Recognition and colors

Status: delivered
Delivery set: rev-markdown
Plan: [Markdown Delivery](../../../plan/rev-markdown.md)
Tracker: [Markdown Tracker](../../../tracker/rev-markdown.md)
Branch: `feat/rev-markdown-01-recognition`
PR: `#34`

## Purpose

Deliver MD01 recognition, MD02 semantic colors and MD06 observation bounds for
T1.1-T1.3. Heading decoration and list/table actions retain their approved later
slice boundaries.

## Delivered Behavior

- `.md` and `.markdown` select native Markdown through existing user association
  precedence. Both selectors expose markdown-mode; text-mode suspends native
  behavior without changing source, saved state, points or history.
- Emphasis/strong emphasis, links/images, code, quotes, lists and GFM tables use
  six independent semantic colors. Nested inline styles precede enclosing blocks.
  Fenced and indented code protect Markdown-looking content as literal source.
- Markdown works offline without a language server. Disabled syntax colors keep
  native recognition; disabled markdown.enabled refuses explicit native entry.
  Read-only buffers, BOM/CRLF, Unicode and shared views retain canonical source.
- One temporary native worker observes admitted visible revisions. Edits, undo,
  reload, mode generations, hiding and cancellation retire obsolete results.
  Failed or over-limit work publishes no partial colors and leaves no idle timer.

## Implementation Notes

Implementation commit: `98cda39921c31c00997b4400cc967ec9bad4abad` (T1.1-T1.2 and native regressions).
Canonical PR: https://forge.adrianpk.com/rev/rev/pulls/34
Report introduction: `1541cc22aac2f6116ffbeaad73bd3aa2bd9e741f`.
Controller integration: PR #34 merged into dev at validated commit
`7db1c13153289e71276544e3a32f829d01a6cdb3`.


The private Markdown adapter retains original source byte ranges, heading levels,
logical list depth and literal/block ownership. External parser/event types stay
inside src/markdown/parse.rs. Each admitted generation moves a cheap rope snapshot
to one existing native job slot; the worker makes one bounded contiguous copy.
It publishes editor-owned observations only after stamp and generation checks.
Split views reuse the canonical buffer's observation.

Limits are 8 MiB, 200,000 parser events/retained nodes, 256 structural levels and
two seconds of cooperative work. A lower nonzero syntax.max_bytes wins.
Cancellation is checked while copying chunks, around parser construction and
between events. Individual parser steps are bounded by the admitted source;
there is no incremental-parsing claim or preemptive interruption inside a parser
step. Coloring and observation availability remain separate policies.

The dependency diff adds exactly pulldown-cmark 0.13.6 and unicase 2.10.0.
Existing bitflags/memchr versions remain unchanged. Default features are disabled
and only Options::ENABLE_TABLES is selected. THIRD-PARTY-NOTICES.md retains the
exact upstream MIT notices; Rev remains GPL-3.0-only.

## Contracts Added or Changed

- markdown.enabled defaults true with partial parsing, include/root merge,
  provenance, atomic rejection and packaged sample/catalog coverage.
- palette.markdown has emphasis, link, code, quote, list and table RGB leaves,
  independent of Org and generic syntax colors. Defaults match the approved model.
- Markdown mode is buffer-owned; source publication invalidates old observations
  through the canonical edit boundary. Native syntax is excluded from Tree-sitter
  admission while language identity and optional LSP policy remain unchanged.
- Reference command/binding/configuration catalogs and Unreleased describe the
  usable increment. No SDK, persistence, transaction or input-profile format changes.

## Files of Interest

- src/markdown.rs and src/markdown/parse.rs: private source observations/adapter.
- src/buffer/markdown.rs and src/session/markdown.rs: admission, worker ownership
  and qualified publication; shared source edits and input deadlines integrate them.
- src/config.rs, palette, merge, project and validation owners: admitted settings.
- src/display/syntax.rs and src/syntax/style.rs: native role/cell projection.
- tests/support/surface.rs: the existing cell decoder, including legal monochrome resets.
- tests/markdown.rs and its context/lifecycle/palette modules; configuration
  Markdown cases; existing syntax/configuration/command regressions.
- docs/reference/syntax.md, complete sample/catalogs and THIRD-PARTY-NOTICES.md.

## Validation

Final clean published HEAD `7db1c13153289e71276544e3a32f829d01a6cdb3` passed
controller receipt `HKRXV4E57IEJINKYEFTGERL7BN`, all four ordered checks,
from 2026-11-13T16:21:58Z to 2026-11-13T16:37:48Z. All 413 tests passed
without failures or ignored cases; full lint, documentation and whitespace passed.

All required development commands below passed. The combined native command
passed all 413 selected tests without failures or ignored cases. Acceptance
belongs to the controller receipt for the final clean PR HEAD, recorded after
all PR-reference commits; the controller retains its exact ordered logs. These
slice checks do not substitute for the later integrated full gate.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test syntax --test commands -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

The successful combined development batch on the report-introduction candidate
passed 205 command tests in 589.64 s, 166 configuration tests in 253.70 s,
12 Markdown tests in 25.33 s and 30 syntax tests in 40.71 s. Documentation and
whitespace checks then passed. The native targets took 909.38 s in total;
the measured cost is not a timeout or a broader acceptance claim.

Full first-party lint passed for 1,852 Rust files, two packages and all 74 Cargo
roots. The installed immutable-index hook also passed the implementation commit.
The independent documentation check passed 691 command names, 314 configuration
entries, 106 reachable pages and 752 local links/anchors.

The first required native batch stopped after the command target: 204 passed and
one failed in 611.73 s. The missing markdown-mode description in the existing
approved inventory was added with its exact compiled text and current counts.
The focused palette::catalog_reference check then passed in 0.01 s. No runtime,
acceptance assertion, timeout or gate definition changed for this correction.

The next required batch passed all 205 command tests in 585.88 s and all 166
configuration tests in 250.01 s, then passed 11/12 Markdown cases in 26.01 s.
The monochrome test incorrectly required contiguous source text inside ANSI
output. Its correction reuses decoded terminal cells and retains RGB suppression
and canonical-source assertions. The existing cell decoder now accepts empty
SGR reset parameters while retaining strict parsing for other parameters.
The focused monochrome case passed in 0.37 s. Complete Markdown then passed
12/12 in 24.18 s and syntax passed 30/30 in 37.35 s. Full lint and documentation
checks passed again on those corrected bytes. No case, acceptance condition,
timeout or required command was removed or weakened.

The first focused native run passed 10 Markdown tests in 23.62 s. The subsequent
candidate added disabled-entry and monochrome cases. Initial lint found a borrowed
parser-event improvement and complexity 11 in syntax admission; both were
corrected locally without changing the configured lint thresholds.

## Risks and Follow-ups

Heading faces/presentation and structural list/table commands remain assigned to
Slices 2-5. Complete PTY/editor acceptance and User Guide delivery remain assigned
to Slice 6. This slice supplies native terminal-cell evidence, not physical font
or final delivery-set acceptance. The pre-existing foreign crossterm unused_parens
warning remains outside first-party source policy.
