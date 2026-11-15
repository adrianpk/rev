# Slice 6: Literal provenance and source projections

Status: reviewing
Delivery set: rev-delimiter-editing
Plan: [Automatic Delimiter Editing Delivery](../../../plan/rev-delimiter-editing.md)
Tracker: [Automatic Delimiter Editing Tracker](../../../tracker/rev-delimiter-editing.md)
Branch: `fix/rev-delimiter-editing-06-provenance`
PR: [#60](https://forge.adrianpk.com/rev/rev/pulls/60)

## Purpose

Preserve literal source ingress, exactly-once macro typing and native source
projection continuity around automatic delimiter editing.

## Delivered Behavior

Raw macro keys apply the current buffer's automatic typing policy once. Generated
mates do not become recorded keys. Accepted paste and completion edits replay
literal bytes, including unmatched delimiters and Unicode, with native counts
and undo groups. Accepted semantic completion replays with completion disabled
without another provider request. Existing counter, cancellation and recording
limits remain in effect. Closer skipping remains navigation: redo restores the
last source edit's point rather than subsequent navigation.

Paste, programmatic replacement, AI result application, plugin edits.apply and
visiting-file reload preserve their literal UTF-8/CRLF payloads. Native point,
mark, shared-view and history behavior remains authoritative. Save-as changes
language policy for the canonical buffer while retaining its runtime override;
stale source/mode/policy requests retire, and pending macro qualification remains
cancelable before publication.

Same-line wrapping and unwrapping now retain completed Org/Markdown heading
presentation while exact analysis refreshes. Both matched delimiter ends
contribute to native byte mapping. Arbitrary two-edit batches, mismatched ends,
line-structure changes and larger batches retain conservative display retirement.
Exact structural actions still require current analysis.

## Implementation Notes

Task/report introduction commit: `40edb1bce7e68b3d537aff9e6a87b69232887241`. T6.1-T6.3
were committed together to keep the native ingress trace, presentation correction
and independent provenance/history regressions one complete reviewable path.

- `src/session/arguments/text.rs` gates pairing on typed input. Paste uses
  `src/session/paste.rs` and the native text/edit owner; counted literal payloads
  remain literal. `Session::replace` publishes checked source transactions.
- `src/session/completion.rs` prepares accepted replacements and records their
  literal completion preconditions. Macro `Step::Key` re-enters input, `Step::Text`
  replays paste, and `Step::Completion` applies accepted native replacements.
  Recorder identity, definition format, counters and runner limits are unchanged.
- `src/session/ai/results.rs`, the plugin edits.apply ingress and
  `src/session/reload.rs` publish through their checked source owners. Tests use
  their actual offline child transports and existing native application commands;
  no network provider or provider business-logic change is required.
- The presentation owner now admits exactly two same-row matched delimiter
  insertions or removals. It borrows native change fragments, reuses canonical
  delimiter identities, checks both fragments and both complete line images,
  and accumulates both byte deltas. Counts retain the existing depth bound.
- Each fragment inspection shares the existing 32 KiB presentation budget;
  repeated edits retain the fixed eight-row mapping. Larger/multiline/unsafe
  shapes retire display evidence while ordinary source publication stays native.
  Pair qualification's 256 KiB inspection, depth 256, 2 MiB buffer metadata,
  256-step polling and cooperative scheduling rules are unchanged.

## Contracts Added or Changed

Native presentation carry now covers the two matching ends of a same-line pair
transaction. Literal input, macro provenance, source/history coordinates and
public protocols retain their existing representations. The command catalog
remains 710 identities, with unchanged factory mappings, configuration leaves,
dependencies, queue limits and worker lifecycle. Personal configuration and the
frozen factory catalog/manifest digests remain unchanged.

## Files of Interest

- `src/buffer/presentation/inspection.rs`: bounded same-row paired-fragment admission and byte deltas.
- `src/buffer/presentation/tests.rs`: independent matched/count/quote/refusal and shared-budget boundaries.
- `tests/pairs/provenance.rs`, `provenance/`: literal ingress, macros, cancellation, language/shared-view lifecycle and pending frames.
- `tests/ai/results.rs`, `tests/plugins/transactions.rs`, `tests/lsp/provenance.rs`: actual offline ingress and accepted-completion replay.
- `docs/reference/delimiter-editing.md`, `CHANGELOG.md`: provenance and visible heading continuity.

## Validation

Successful implementation checks:

- `cargo fmt --all -- --check`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs provenance -- --test-threads=1`: 11 passed in 8.04 seconds after a 52.65-second source rebuild.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- recording regions runner --test-threads=1`: 20 passed in 19.00 seconds.
- `git diff --check`: passed.

Supplemental checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test ai --test plugins --test lsp provenance -- --test-threads=1`: all three selected cases passed. The offline AI case exercised both native applications in both profiles (2.37 seconds); semantic completion and literal macro replay took 0.59 seconds; plugin RPC/shared-view history in both profiles took 14.02 seconds.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib buffer::presentation -- --test-threads=1`: all 24 passed in 1.53 seconds after a 52.79-second unit-library rebuild. Existing refusal, retirement, eight-row, arithmetic and scan-limit assertions remain intact.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings --test markdown continuity -- --test-threads=1`: 7 Markdown and 8 Org cases passed in 11.04 and 7.55 seconds after a 55.53-second native-library rebuild.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- counters capture::literal_completion --test-threads=1`: all 10 passed in 12.98 seconds.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: passed; 710 commands, 332 configuration entries, 111 reachable pages, 784 local links/anchors, 10 command-reference and 12 configuration-reference tests.

The initial native frame case reproduced two-span presentation loss. The existing
owner's refusal matrix then rejected an over-broad two-edit admission; the final
rule admits only matched delimiter insertions/removals without changing that old
assertion. New fixture expectations follow existing insertion, reload-history and
navigation-cursor contracts. Completion recording uses physical M-Tab through the
existing recorder guard. No acceptance or resource limit was relaxed.

The vendored crossterm parentheses warning is pre-existing and non-blocking.
Full staged-source hooks passed on the implementation commit: formatting, strict
Clippy with zero blocking/deferred diagnostics, and native safe-Rust, parameter,
nesting and complexity policy for all 1,953 first-party Rust files, two packages
and 77 Cargo roots. The staged tree was
`0eb23aee41f8380d1565649132c957b5e01314a9`. The cold private-cache commit
check completed in 247.46 seconds; its strict Clippy pass took 1 minute 39 seconds.
The exact four-command controller receipt follows PR-reference commits at clean
published HEAD. Existing fixtures own/reap their offline peers; no persistent service was
started. Combined delivery acceptance remains controller-owned.

## Risks and Follow-ups

Representative Linux physical-input/PTY workflows and integrated resource/frame
measurements remain in Slice 7. These native checks establish source ingress and
pending presentation behavior; they do not claim physical-input acceptance.
Unproved context and exhausted presentation budgets retain conservative fresh
analysis requirements.
