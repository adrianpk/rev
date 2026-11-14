# Slice 3: Markdown presentation continuity

Status: reviewing
Delivery set: rev-presentation-continuity
Plan: `ops/default/plan/rev-presentation-continuity.md`
Tracker: `ops/default/tracker/rev-presentation-continuity.md`
Branch: `fix/rev-presentation-continuity-03-markdown`
PR: `#43`

## Purpose

Keep verified Markdown heading and list presentation through bounded body/title
edits. Follow the [plan](../../../plan/rev-presentation-continuity.md) and
[tracker](../../../tracker/rev-presentation-continuity.md).

## Delivered Behavior

Admitted same-line body and ATX-title edits keep Markdown heading symbols,
structural level colors and neighboring list markers while exact analysis is
pending. Pretty/source policy, disabled colors, monochrome rendering and the
active palette remain independent. Current edited source is visible immediately.

Body edits require a parser-admitted paragraph, identical leading whitespace,
nonblank before/after text and a conservative structural refusal filter.
Indented code, list/quote continuation, Setext, literal/HTML contexts, table
syntax, definitions and uncertain cases reject carry. Recognized ATX title edits
preserve the existing hash/container prefix and separator, including titles
inside lists/quotes. Setext-title edits conservatively fall back to exact analysis.

## Implementation Notes

The existing complete observation stays in its owner only through an admitted
mapping chain. Parser-owned paragraph, HTML and reference-definition spans join
the existing node store; their combined definition/event count stays within the
existing 200,000 limit. No second document/index, parser or dependency is added.

Preparation uses the canonical transaction's real before/after lines and one
shared 32 KiB allowance. It reserves bounded copies, all guard/prefix passes and
both existing-node lookups (two additional allowance units per retained node).
Large observations can therefore refuse conservatively before lookup/copy.
The fixed eight-row mapping now maps complete physical-row ranges, including
dirty-row byte deltas. Prefix/item offsets use the unchanged prefix within that
row, rather than applying its title delta to its own marker. Reserved source
installation only retains or drops the precomputed state.

The private display facade borrows retained data and emits current heading/item
coordinates. Structural heading and list-marker captures use current source
bytes and active palette roles. All retained lexical captures are suppressed
while carried, including cross-row parts; unrelated inline/list-body faces can
temporarily use their default color. No inline-context continuity is claimed.

The exact action observation API is unchanged. Retained base data cannot
authorize list/table edits. Source stamps still govern coordinates, hit targets
and affected-prefix cache reuse. Markdown substitution keeps source width; its
existing geometry identity does not change on exact/carried replacement.

Explicit quit retires carried evidence and suppresses automatic retry. A new
explicit presentation request can retry; reaping an old canceled worker preserves
that request. Live incomplete/failed results retire carry. Obsolete revisions
retain their existing stamp/generation rejection without retiring a newer chain.

## Contracts Added or Changed

Implements the Markdown portion of PC01-PC03 and PC05-PC11 with reconciled MD06.
Source/history/persistence, parser syntax/options, list/table authority, fold
policy and existing worker/deadline ownership remain intact. Joint lifecycle,
resource and terminal acceptance remain pending in Slices 4 and 5.

## Files of Interest

- `src/markdown/parse.rs`: native context/definition spans in the bounded owner.
- `src/markdown/presentation.rs`: before/after admission and display-only item data.
- `src/buffer/presentation.rs`: charged format admission and complete-row mapping.
- `src/buffer/markdown/display.rs`: borrowed current display coordinates and structural captures.
- `src/buffer/markdown.rs`, `src/buffer/edit/publication.rs`: retained owner lifecycle and exact action boundary.
- `src/display/markdown.rs`: mapped heading/list substitution.
- `src/session/markdown.rs`, `src/session/prompt.rs`: cancellation/completion ownership.
- `src/buffer/presentation/markdown_tests.rs`: positive full-parser comparison, refusals and bounds.
- `tests/markdown/continuity.rs`: native intermediate frames, color/source controls and cancellation.
- `CHANGELOG.md`: extends the existing concise correction to Markdown.

## Validation

Local pre-submit required checks passed:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`: workspace formatting, strict Clippy with zero blocking/deferred diagnostics and native source policy; 1,894 first-party Rust files, two packages and all 75 Cargo roots. Final Cargo check took 1 minute 15 seconds.
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown -- --test-threads=1`: all 55 passed, including six new continuity cases; no failures or ignored tests. Warm Cargo rebuild 1.96 seconds; execution 138.20 seconds. Initial cold dependency/test compilation took 1 minute 41 seconds.
3. `git diff --check`: passed.

Focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib buffer:: -- --test-threads=1 --nocapture`
passed all 24 cases (Cargo rebuild 53.20 seconds; execution 2.11 seconds).
The Markdown matrix recorded seven accepted and 21 rejected cases. All seven
accepted cases compare with fresh full pulldown-cmark output, as do all 64
accumulated Unicode body/title edits. These cover insertion/deletion,
empty/indented/container ATX titles, mapped neighboring items, stale list/table
authority refusal and budget exhaustion without blocking source editing.

The native writer fixture checks 30 deliberately pending frames: five edits
under both pretty policies with enabled, disabled and monochrome colors. Each
samples three heading marker/title pairs and three neighboring nested/ordered
list markers, with the active palette applied, then compares exact replacement.
The remaining new cases exercise seven native refusal contexts, global lexical
retirement, cancel/re-enable both before work starts and while canceled ownership
is still pending, canonical hash copying, BOM/CRLF Unicode save and undo. Existing
context, palette, list, table, history and lifecycle assertions remain in place.

Development checks caught an invalid fixture motion name and a byte/cell sample
offset; both were corrected. A full run then passed 54 cases and exposed a lost
explicit retry behind a canceled worker. Requests now remain queued until the
existing slot is reaped, and exact publication marks the request satisfied.
The subsequent full run passed all 55 cases. No limit or acceptance assertion
was weakened. Vendored crossterm retains its existing unused-parentheses warning.
The installed hook passed complete quality checks against the immutable staged
implementation candidate; its cold private Cargo check took 1 minute 48 seconds.

Final exact clean-HEAD evidence belongs to the controller validator after the
report/PR-reference commits and push. No PTY, graphical or delivery-set gate
acceptance is claimed here. Native fixtures release their own resources; build
and hook caches remain exclusive to this worktree.

## Risks and Follow-ups

Shared lifecycle/resource acceptance remains Slice 4; terminal acceptance and
user guidance remain Slice 5.
