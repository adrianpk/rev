# Slice 5: Terminal acceptance and guidance

Status: delivered
Delivery set: rev-presentation-continuity
Plan: `ops/default/plan/rev-presentation-continuity.md`
Tracker: `ops/default/tracker/rev-presentation-continuity.md`
Branch: `test/rev-presentation-continuity-05-acceptance`
PR: [#45](https://forge.adrianpk.com/rev/rev/pulls/45)

## Purpose

Verify PC01-PC12 through continuous native and emitted-terminal Org/Markdown
frames, preserve source authority and document bounded carry and literal fallback.
The controller retains integration and final delivery-set acceptance.

## Delivered Behavior

The narrow `scripts/check-presentation-continuity.sh` entrypoint extends the
existing Org/Markdown acceptance executables, native session/projection owners,
shared util-linux PTY lifecycle and cell decoder. It adds no framework, parser,
dependency, shipping worker, timer or runtime field. Existing historical driver
flags and the eight-second/1 MiB terminal ownership limits remain intact.

The stream owner now visits every completed writer frame in each read chunk.
A regression built from real literal/exact frame output proves that an earlier
literal update cannot hide behind the final correct update, including a split
Show footer. Partial UTF-8 input remains with the existing bounded stream owner.

The native matrix checks all four Org indentation/pretty combinations, Markdown
pretty/source, enabled/disabled colors and monochrome under both profiles. Every
pattern starts with complete native evidence. Five source publications on one
body row, one title row or two rows preserve required marker/title columns and
palette-derived heading foreground. Current edited text is checked immediately.
A separate pretty projection proves admission even when the selected view has
raw markers and disabled colors. Neighboring Markdown list markers remain;
uncertain item-text colors are lexical and can retire.

Existing native jobs hold publication pending across the five revisions. In
source/disabled-color Org cases no automatic color/pretty job is requested;
the fixture records that fact, explicitly requests analysis through the existing
pretty control after the burst and restores the selected policy. Every Org
replacement is checked at the current stamp with complete index coverage.
No claim of continuing worker CPU execution is inferred from owned publication.

## Implementation

T5.1 and T5.2 are delivered by `466b28d70c8e7fb906f087d4523c4fd14ed11936`, with per-key revision/byte
assertions in `1c37488abab754ca9fabf977e5ff161dfe16f527`. T5.3 updates the five existing reference/lesson
pages and introduces this report in `15faffec18031d71ff6be352e524b683ec336fa9`.
Canonical PR #45 retains that immutable report-introduction link.

## Admission, Source and Resources

The pre-submit matrix covers 52 policy/document combinations and 156 bursts:
780 admitted ASCII publications, plus four Unicode pastes in BOM/CRLF fidelity
cases. Each burst uses at most two changed rows and restores the exact source
and saved state through the existing undo groups. Each Unicode paste has one
undo entry; native copying yields canonical `*` or `#`, and exact BOM/CRLF bytes
are verified on save/undo/save in both profiles.

Ten negative publications (five per format) cover prefix/separator changes,
multiline replacement, literal blocks and definitions. Each retires display to
readable source, preserves the requested canonical bytes and has one-step undo.
Mandatory positives are checked independently; fallback cannot satisfy them.

| Input | Org bytes / content lines | Markdown bytes / content lines | Headings |
| --- | --- | --- | --- |
| Minimal | 59 / 6 | 83 / 10 | 3 |
| Approximately 50 KB | 50,018 / 1,287 | 50,003 / 1,290 | 3 |
| Approximately 500 KB | 500,000 / 12,825 | 500,024 / 12,829 | 3 |
| Heading density | 11,958 / 1,006 | 11,982 / 1,010 | 1,003 |
| Heading density | 37,958 / 3,006 | 37,982 / 3,010 | 3,003 |

Both profiles run body/title/two-row bursts at each larger size/density. Complete
Org replacements report source bytes, indexed rows and covered bytes; all 96
cover their entire current source. The largest has 500,005 bytes and 12,826
indexed rows after typing (the index also represents the final empty row).

The 304-byte optional eight-slot mapping, charged 32 KiB admission budget and
8,192-fragment/1 MiB per-view cache ceilings remain the existing owners' bounds.
Their direct storage, eviction, overflow, ninth-row and oversized-inspection
regressions/evidence remain in [Slice 4](slice-4-lifecycle.md) and the buffer
fixtures selected by the controller's integrated gate. This slice measures
source extent, sampled cells and end-to-end projection/input work; it does not
claim a new whole-observation heap/RSS or internal inspection-counter benchmark.
Every rendered sample uses three heading markers plus three ordinary title cells;
Markdown adds two neighboring list marker cells. All key-frame states are
recorded, then compared with exact replacement under the original policy.

## Emitted Cells and PTY Evidence

Heading identity uses fixed logical source rows; the original word locates its
ordinary title cell after append-only typing. Expected symbols come from resolved
Org preferences or the fixed native Markdown marker owner, and colors from the
active snapshot. Columns below are zero-based with gutter/bookmark markers off:

| Format / policy | Marker columns Root, Child, Deep | Title columns Root, Child, Deep |
| --- | --- | --- |
| Org flat/source | 0, 1, 2 | 2, 3, 4 |
| Org flat/pretty | 0, 0, 0 | 2, 2, 2 |
| Org indented/source | 0, 2, 4 | 2, 4, 6 |
| Org indented/pretty | 0, 2, 4 | 2, 4, 6 |
| Markdown source or pretty | 0, 1, 2 | 2, 3, 4 |

Configured heading foregrounds are `#123456`, `#234567`, `#345678` for levels
one through three, for both marker and ordinary title cells. Disabled syntax
uses active default foreground `#c0caf5`; monochrome has no RGB foreground or
background escapes anywhere in an emitted update. Pretty markers are `◉ ○ ✸`;
source uses the native final star/hash. Markdown neighbor markers are `•` and
`◦` at columns 0/2 in pretty mode and `-` in source mode.

Twelve real editor journeys cover both formats, both profiles and enabled,
disabled and NO_COLOR output. Each sends five body keys and five keys across
title/body rows, records every completed emitted update through quiescence,
checks source/pretty/reset controls, resizes 120x30 to 32x12 and verifies exact
undo/save, idle silence and TTY restoration. The two-row modal sequence remains
in Insert and uses ordinary arrows; it does not inject cancellation between keys.
All fixture children finish through the owned terminal lifecycle.

In the successful pre-submit run, all 156 five-key native bursts completed in
4,800-9,299 microseconds each, including insertion, projection and cell checks;
each was below 180 ms. There were 144 bursts with an owned publication slot held
across the five revisions and 12 unrequested Org source/disabled-color bursts.
Every case subsequently obtains exact replacement and restores its policy.
Native matrix execution took 13.504 seconds for Org and 6.335 for Markdown.
These are observations from this host/build, not a latency guarantee.

The 24 PTY bursts sent 120 input characters and observed 48 complete updates.
Five-character input batches were enqueued in 2-7 microseconds. All 48 live-thread
samples observed zero analysis threads, including zero at the settled boundary;
that does not prove a worker never ran or remained CPU-pending between samples.
The native publication hold is the deterministic pending evidence. PTY output
coalesced keys into fewer updates; no un-emitted per-key frame is invented.
Every emitted update's heading/list state and actual title/body row are recorded.

The 12 owned journeys took 1.625-1.772 seconds each, 20.155 seconds combined,
including quiescence, source controls and cleanup. Per-editor output ranged
50,992-68,522 bytes, below 1 MiB; all finished within the existing eight-second
terminal lifetime. Native and terminal color/source results remain distinct from
graphical/font acceptance.

## Documentation

The existing Org/Markdown reference, shared syntax reference and the Org and
Markdown User Guide lessons now explain ordinary body/title continuity,
conservative lexical fallback, explicit cancellation/re-enablement and exact
structural action authority. Product lessons retain source editing/copy/save
exercises; implementation budgets stay in the reference. The existing concise
changelog outcome is preserved without another slice-specific entry.

Documentation checking passes 110 reachable pages and 780 local links/anchors.
The existing producer projects 111 Markdown pages for Spore, checks strict links,
renders and derives the local guide/downloads under this worktree's ignored
`target/userguide`. No site publication or binary installation occurs.

## Validation

Local pre-submit required commands passed in canonical order:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`: complete formatting, Clippy and native policy passed across 1,902 first-party Rust files, two packages and 75 Cargo roots; zero blocking or deferred diagnostics. Warm Cargo check 0.74 seconds.
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-presentation-continuity.sh`: the real completed-frame observer regression, all 156 native bursts, four source-fidelity cases, 10 refusals and 12 PTY journeys passed. The counter-strengthened repeat also passed; its warm test preparation took 0.70 seconds and release driver relink 38.95 seconds. Native and PTY execution costs are recorded above; initial cold compilation is additional and is not reconstructed from wall-clock estimates.
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`: 110 reachable pages and 780 local links/anchors passed.
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`: 111 Markdown pages projected, strict Spore checks/render and local guide/download derivation passed.
5. `git diff --check`: passed.

Installed hooks passed the exact immutable staged source candidates. The cold
private Cargo check took 1 minute 31 seconds; the counter-strengthening candidate
reused that cache and checked in 0.79 seconds. No hooks, limits or acceptance
assertions were disabled. Vendored crossterm retains its existing nonblocking
unused-parentheses warning.

Development checks corrected module routing, a command enum name, policy-specific
worker expectations, complete-base setup after undo, modal Insert input and the
lexical/structural distinction for item text. The real PTY observer caught source
fallback caused by cancellation in a doubled-Escape/prompt sequence; ordinary
Insert-mode arrow motions now exercise two-row typing without injecting cancel.
Every completed edit update still must pass the strict marker/title/color checks.
No product runtime change was required.

The pinned validator recorded all five ordered commands passing on clean
published `2b5cde6c52878f09fe00c059b2e84c9b836e12be`, after the report/PR-reference
commits and push. The canonical PR check also passed. Fixtures released only
their owned editor children; generated targets and hook caches stayed exclusive
to their worktree. This slice evidence does not replace the integrated full gate.

## Risks and Follow-ups

Native frame-writer/cell and Linux emitted-terminal evidence do not prove font,
ligature, compositor or graphical presentation. PTY enqueue timing is not a
per-key processing latency guarantee. Completed-frame boundaries are the writer's
hide/show protocol; intermediate GPU painting within one frame is outside this
fixture. No worker CPU-pending claim is inferred from fast input or a quiet frame.

The controller must run its exact full gate on clean published dev before
closing the delivery set. Final Markdown acceptance records remain conditional
on that gate. Worktree disposition is recorded below.
No main promotion, release, tag, installation or mirror/site publication is claimed.

## Integration

Canonical PR [#45](https://forge.adrianpk.com/rev/rev/pulls/45) is merged into
`dev` at `2b5cde6c52878f09fe00c059b2e84c9b836e12be`, verified against Forgejo,
Acta and synchronized Git refs. All five slice reports are delivered; the set
remains Approved pending the controller's exact integrated full gate.

The owned worktree was clean, unlocked and inactive, with only generated Cargo
output ignored and a private generated hook cache. Its HEAD was an ancestor
of fetched dev and origin/dev. Non-forced Git worktree removal completed;
the branch and all unrelated worktrees remain preserved.

Gate preparation verified tool availability, tracked inputs, shell syntax and
the primary checkout's exclusive Cargo target. Installed hooks and the approved
gate remain unchanged. No build, native/PTY suite, guide producer or full gate
was executed during preparation.
