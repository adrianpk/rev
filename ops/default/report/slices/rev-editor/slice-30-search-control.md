# Slice 30: Search backward with smart case and cancel

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-30-search-control`
PR: `#30`
Integrated: Rebase + Fast-forward into dev at `144155023f101134aa4fb50320049d9fe96d8bf3`.

## Purpose

Complete directional incremental search with Unicode smart case and explicit
accept/cancel lifetimes while preserving original source ranges and text state.

## Delivered Behavior

- C-r and M-x isearch-backward open a visible backward query. Backward search
  selects the last match ending at or before its anchor and places point at
  its grapheme-safe start; forward search keeps its end-point behavior.
- Repeating C-r or C-s moves before or after the current match in the chosen
  direction. Switching direction uses the adjacent match boundary. A direction
  change after failure uses the retained valid point; neither boundary wraps.
  Query edits immediately recompute from the current anchor.
- Queries without uppercase characters use Unicode full default case folding;
  any uppercase query scalar makes the entire query exact. Edits recalculate
  the policy. Expansions such as sharp s and dotted I map back to complete
  original scalar ranges; a match cannot consume half of an expansion.
- Return accepts the current position and retires search origin/result state.
  C-g cancels empty, successful, repeated, reversed or failed searches and
  restores the saved point, scrolling, intended column and recenter cycle.
  Reveal then accommodates current geometry in the original selected pane.
- Search leaves the rope, logical state and undo/redo transactions intact.
  Failed queries retain valid points, remove stale match styling and display
  `No match`; source revision changes retain the existing stale-result rejection.

## Implementation Notes

- T30.1: `c41066a63d82a1eb10961642046b60cbaa42a530` adds the backward
  command/binding, direction-aware query/session state, bounded Unicode folding
  and source-position tracking, three native behavior cases and updated reference.
- T30.2: `d03adaff8e5475a8aca97f23cbc13bf3236da4d5` expands the search
  suite to nineteen native cases and adds two real PTY journeys for backward
  continuation, failure recovery, Unicode source display and viewport cancellation.
- KMP scans borrowed rope characters across chunks in either direction. A
  reversed scan starts its borrowed iterator at the admitted range end. Only
  the bounded query/table is reversed; no document or folded-file copy is built.
- The folded stream emits at most three tokens per scalar and retains a ring
  bounded by folded query length, at most 12288 tokens. Each token carries its
  original scalar position and expansion edges. Matching rejects partial
  expansions and returns a buffer/revision-qualified Span of original text.
- Local data pins all 1585 C/F mappings from
  [Unicode 17.0.0 CaseFolding.txt](https://www.unicode.org/Public/17.0.0/ucd/CaseFolding.txt).
  The generated table includes its source hash and Unicode license notice.
  Unlisted scalars remain unchanged; Turkic mappings and normalization are not
  applied. No dependency or runtime network access was added.
- Added deterministic cases use existing native fixtures. Two bounded PTY
  journeys use complete-frame waits and resize handshakes. Only owned fixtures
  close after checks; accepted exit verifies terminal restoration and exact
  unchanged disk bytes. Existing larger-rope scans remain in the focused suite.

## Contracts Added or Changed

- `Pattern::new` now selects smart case from the raw query; `Pattern::rfind`
  admits only original scalar bounds and returns the last fully contained match.
  Full folding preserves source qualification independently of folded length.
- Search input distinguishes direction through its label and Repeat outcome.
  The catalog advertises `isearch-backward`; C-r and named invocation share
  its real session handler with the same checked result and cancellation owner.
- Continuation uses raw match boundaries, while presentation expands complete
  graphemes. Unicode case policy never substitutes folded indices for source
  positions, including before/after a real rope seam.
- Accepted search retires its saved origin. C-g restores an active search's
  entire saved View before the existing cancellation path consumes its owner.
- `docs/reference/search.md` documents directions, smart case, expansion edges,
  bounds and exact view restoration. Command/index links and the existing
  reader-facing changelog entry now describe the completed search behavior.

## Files of Interest

- `src/search.rs`, `src/search/fold.rs`: directional matching and source mapping.
- `src/search/case.rs`: pinned Unicode folding data, source hash and license.
- `src/input/search.rs`, `src/input/prompt.rs`: direction labels and query ownership.
- `src/session/search.rs`: continuation anchors, match point and saved-view lifetime.
- `src/command.rs`, `src/input/bindings.rs`: named and bound backward search.
- `tests/search/control.rs`: eight native direction, case and cancellation cases.
- `scripts/check-pty.py`: backward boundary and smart-case/cancel journeys.
- `docs/reference/search.md`, `docs/reference/commands.md`,
  `docs/reference/index.md`, `CHANGELOG.md`: product behavior and discovery.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 310 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test search`: passed; nineteen tests, zero skips. Coverage
  includes mixed Unicode case, complete expansion ranges, reverse chunk seams,
  maximum threefold query expansion, edits, boundaries, directions, named/bound
  parity, retained redo and exact cancellation after two-axis reveal.
- `python3 scripts/check-pty.py --suite terminal`: passed; sixty-two cases,
  zero skips. New journeys verify C-r/M-x backward entry, repeated search,
  BOF failure and reversal, original Unicode underline after folding, uppercase
  policy, exact document viewport restoration after resize and unchanged files.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
