# Slice 5: Selection wrapping and explicit helpers

Status: reviewing
Delivery set: rev-delimiter-editing
Plan: [Automatic Delimiter Editing Delivery](../../../plan/rev-delimiter-editing.md)
Tracker: [Automatic Delimiter Editing Tracker](../../../tracker/rev-delimiter-editing.md)
Branch: `feat/rev-delimiter-editing-05-regions`
PR: pending

## Purpose

Wrap ordinary linear selections and unwrap confirmed enclosing pairs through
bounded native qualification and checked source/history publication.

## Delivered Behavior

Typing an admitted opener or closer wraps an active ordinary linear region in
one native transaction. Selected UTF-8/CRLF bytes stay unchanged. Both orientations
and positive structural counts use opener placement after the generated openers,
or closer placement after the generated closers. Wrapping deactivates the region;
undo restores its original source, point, mark and active orientation. Zero stays
inert and negative typed repetition retains its native refusal.

M-x pair-wrap asks for one admitted opening or closing character, with no guessed
default. Either answer uses opener placement; C-g cancels the prompt and retains
the selection. M-x pair-unwrap removes only the innermost confirmed enclosing pair,
including point on either delimiter, and maps point/mark through the edit. Both
helpers work with automatic editing and its individual preferences disabled.
Exactly these two unbound commands extend the existing catalog to 710 identities.

Unsafe quote transformations refuse before mutation. Quote wrapping preserves
literal bytes without silently escaping them; character quotes require one scalar,
ordinary single-line profiles reject newlines, and ambiguous repeated quote counts
refuse. Go backticks and ordinary TOML single quotes retain their raw-content
rules. Unwrapping rejects escape/nested quote forms without safe literal evidence,
unconfirmed context and delimiter spans that would split a grapheme. Rectangular,
block, modal Visual, focus and read-only owners retain their existing behavior.

## Implementation Notes

- The canonical buffer qualifier admits a complete wrapping range and count in
  one request. Structural content must retain strict balance and ordinary context
  at both ends. Quote payload work is virtual and never becomes retained syntax
  proof about the original selection. Existing pair facts retain literal-safety
  evidence for later unwrap requests, including cache reuse.
- Session captures source point, view, mark, mode and effective policy. The native
  bounded layout/input FIFO polls one ticket; cancellation and source changes
  retire uncommitted work, and later queued typing uses the published cursor.
- Wrapping publishes two insertions; unwrapping publishes two removals. The
  selected contents are retained native source, with no full-selection string
  copy or second source/history implementation. Native admission checks payload,
  read-only state, coordinates, shared points and source revision before publication.
- Structural region edits opt into native transaction selection metadata. Other
  edits retain their existing selection policy. History restores the initiating
  cursor, active orientation and mark; other shared points retain native mapping.
  Unwrap explicitly retains its already mapped initiator for exact undo/redo.
- The unchanged 707-command catalog and reviewed shortcut manifest remain frozen.
  Additive expectations independently name auto-pair-mode, pair-wrap and pair-unwrap;
  strict old-name and full-map equality checks remain in place.

## Contracts Added or Changed

The resolved wrap_selection preference now controls typed ordinary-region wrapping.
Explicit helpers retain their own eligibility checks independent of automatic
preferences. No new configuration leaf, binding, dependency or parser is added.
The shared qualifier adds complete-region and literal-safe enclosing requests;
its 256 KiB inspection, depth 256, 2 MiB buffer metadata and 256-step poll limits
remain fixed. The four-millisecond target remains cooperative scheduling evidence,
not a universal wall-clock guarantee. There is no new timer or permanent worker.

## Files of Interest

- `src/pairs/work/wrapping.rs`: bounded region and literal-safety proof.
- `src/session/pairs/regions.rs`, `regions/prompt.rs`, `regions/plan.rs`: capture, native prompt and checked publication.
- `src/buffer/edit.rs`, `edit/history.rs`, `edit/publication.rs`: opt-in selection history metadata.
- `src/command.rs`, `examples/factory_shortcuts_acceptance/matrix.rs`: two unbound helpers and independent additive inventory.
- `tests/pairs/regions.rs`, `regions/`: native placement/history, helper, ownership and qualifier cases.
- `docs/reference/delimiter-editing.md`: exact behavior, refusal and invocation policy.

## Validation

Successful implementation checks:

- `cargo fmt --all -- --check`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs regions -- --test-threads=1`: 17 passed in 31.66 seconds with a warm build, including the existing deletion-region regression and 16 new region cases.
- `git diff --check`: passed.

Supplemental checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs -- typing regions --test-threads=1`: all 34 passed in 70.26 seconds after a 53.42-second rebuild following the final eligibility extraction.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs -- --test-threads=1`: all 64 passed in 139.39 seconds. This bounded shared-owner batch also covers the previous qualification, typing, deletion and quote contracts after qualifier/history changes.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs -- regions::qualification regions::placement regions::helpers::unwrapping regions::helpers::refusal --test-threads=1`: 4 passed in 12.13 seconds after a 52.42-second source rebuild.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs -- regions::lifecycle::shapes regions::helpers::marked_unwrap regions::qualification::retained_proof --test-threads=1`: 3 passed in 2.65 seconds.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: passed; 710 commands, 332 configuration entries, 111 reachable pages, 784 links/anchors, 10 command-reference tests and 12 configuration-reference tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --locked --example factory-shortcuts-acceptance`: passed.
- `target/debug/examples/factory-shortcuts-acceptance --native tmp/pair-regions-native`: passed under an owned isolated HOME/XDG fixture, removed after process completion. All 28 maps, 29 accepted records/49 base edits and six actions remained exact in both profiles, with 707 old names plus three explicit additions. Effective factory/custom hints and selector cancellation passed; one native process thread was observed.

Initial focused failures exposed lookup termination being applied to wrapping,
a missing initiating cursor record for unwrap undo, and a missing grapheme-span
refusal. Those owners were corrected and the tests now pass without weakening
assertions or resource limits. Source-policy feedback required cohesive admission,
boundary, cached-target and typing-eligibility extraction; no suppression was added.
The staged hook caught the final eligibility complexity before any commit was
created; the extracted owner and affected typing/region batch now pass. The vendored
crossterm parentheses warning is pre-existing and non-blocking.

Personal configuration and frozen factory catalog/manifest digests are unchanged.
Full staged-source hook results follow the implementation commit. The controller
receipt will use exactly the three injected commands at clean published HEAD
after PR-reference commits. Combined acceptance remains controller-owned.

## Risks and Follow-ups

Complete literal/macro provenance, source-projection continuity and physical-input
acceptance remain in their recorded later slices. These native checks do not
claim PTY acceptance. Unproved literal contexts and unsafe quote transformations
retain conservative refusal.
