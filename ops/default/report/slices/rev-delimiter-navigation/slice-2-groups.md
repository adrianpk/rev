# Slice 2: Forward and backward groups

Status: delivered
Delivery set: rev-delimiter-navigation
Plan: [Delimiter Navigation Delivery](../../../plan/rev-delimiter-navigation.md)
Tracker: [Delimiter Navigation Tracker](../../../tracker/rev-delimiter-navigation.md)
Branch: `feat/rev-delimiter-navigation-02-groups`
PR: [#63](https://forge.adrianpk.com/rev/rev/pulls/63)

## Purpose

Deliver forward-pair/backward-pair as distinct bounded structural group motions,
with native signed counts, source-map inheritance and unchanged editing history.

## Delivered Behavior

C-M-n and C-M-p cross complete mode-defined groups and land after the closer or
before the opener. M-x offers the same commands. Strings, comments and native
literal/embedded exclusions follow the declared profile even with automatic
pairing and syntax colors disabled. A loose mixed group such as [a) can move
point without supplying typed matching authority.

Counts run once in a captured request; zero is inert, negative reverses direction
and ordinary EOF/BOF is allowed. Malformed, uncertain or limited requests publish
no intermediate point. Successful and refused motions preserve source, revision,
undo history and the active linear mark, including an empty active region.
Endpoints normalize to native grapheme boundaries, including combining/prepend
characters and CRLF. Groups use ordinary movement; the mate command retains its
separate source-jump behavior.

## Implementation Notes

The canonical buffer owns exactly one active typed-pair or group request. Existing
lexical classification, reader accounting and ticket retirement are shared.
Forward traversal retains a bounded structural stack and one destination;
backward traversal scans the prefix once and visits bounded completed source
groups in reverse. Neither direction rescans per count or allocates per-count
targets. Group results cannot appear through the typed pair-answer API.

The session captures revision, selected view, point, mark and editing state.
Existing cooperative layout scheduling, cancellation and deferred-input FIFO
prepare one candidate view before publication. Point/mark/focus, source or view
replacement retires stale work. No timer, permanent worker or dependency is added.

## Contracts Added or Changed

The catalog grows from 711 to 713 names. Exactly two base-editor leaves propagate
through existing source-map inheritance; Normal/Visual defaults retain their
filters. User overrides and effective M-x hints keep precedence. Macros and the
native commit-source map admit these two ordinary motions through their existing
guards. Pending modal operators reject an unrelated group chord; M-x explicitly
retires the operator through the unchanged input owner.

Hard limits remain 262,144 inspected bytes, depth 256, 2,097,152 retained bytes
per buffer and 256 lexical/search steps per poll. A signed count has magnitude
at most 1,000,000 and shares the entire request budget. The native four-millisecond
poll deadline remains a scheduling target, not a wall-clock guarantee.

## Files of Interest

- `src/pairs/groups.rs`: source-qualified group request and outcomes.
- `src/session/pairs/navigation.rs`: captured native motion publication.
- `tests/pair_navigation/groups.rs`: independent group and lifecycle coverage.
- `tests/pair_navigation/groups/keymaps.rs`: effective input, modal and macro cases.
- `examples/factory_shortcuts_acceptance/matrix.rs`: literal additive expectations.

## Validation

Focused checks are local; the controller owns the combined delivery gate.
Final clean-HEAD evidence is recorded separately through looper validate after
the PR-reference commit.

- `cargo fmt --all -- --check`: PASS.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation groups -- --test-threads=1`: PASS, 14 tests, 57.47 seconds after compilation.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation -- --test-threads=1 --nocapture`: PASS, 25 tests (14 group and 11 existing mate cases), 135.29 seconds after compilation. An initial 14-case group run had one test assumption failure: native M-x retires a pending modal operator. The corrected case verifies both operator rejection for a custom group chord and the existing M-x transition.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib -- mate_stale groups_stale --test-threads=1`: PASS, two invalidation tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs qualification -- --test-threads=1`: PASS, 21 existing qualification/cache/retirement tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal -- delimited_objects structural_targets --test-threads=1`: PASS, two existing modal percent/object tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example factory-shortcuts-acceptance -- --native target`: PASS, 713 commands, six actions and all 28 complete maps in both profiles; exactly two added base leaves and their existing inheritance.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: PASS, 713 commands, 332 configuration entries, 112 reachable pages, 792 local links/anchors, 10 command and 12 configuration reference tests. Development failures exposed the omitted packaged keymap leaves and stale documented counts; both now match the compiled Editor/Insert/Replace defaults.
- Both implementation and test commits passed their exact staged hooks: formatting, strict all-target/all-feature workspace Clippy and full native policy for 1,961 and 1,963 files respectively, with all 79 Cargo roots selected. The preserved hook uses its exclusive cache; no hook was installed or bypassed.
- `git diff --check`: PASS.

The frozen Emacs matrix supplies 31 applicable group cases, including counts,
readonly source, active/empty mark and refusal. Its Lisp semicolon-comment case
does not apply to the native Plain profile; the independent Plain semicolon case
does apply. Frozen observation bytes and historical factory artifacts are intact.

Measured unoptimized native buffer requests, one sample per direction/source
shape; allocation of the source occurs outside timing. Each row reports complete
forward/backward total time, malformed forward/backward total time and the
maximum observed poll across those four requests, in microseconds:

| Source bytes | Complete totals | Malformed totals | Maximum poll | Inspected bytes | Maximum retained bytes | Outcome |
| --- | --- | --- | --- | --- | --- | --- |
| 1,024 | 603 / 622 | 722 / 634 | 218 | 1,024 | 8,720 | target / malformed refusal |
| 65,536 | 51,137 / 83,502 | 65,099 / 64,792 | 3,143 | 65,536 | 8,720 | target / malformed refusal |
| 262,144 | 267,140 / 264,990 | 261,174 / 284,122 | 5,665 | 262,144 | 8,720 | target / malformed refusal |
| 1,048,576 | 279,522 / 292,627 | 259,887 / 292,248 | 3,449 | 262,144 | 2,576 | limited: bytes |

The 5,665-microsecond poll exceeded the scheduling target; hard work limits held.
Every request retained at most 256 steps per poll. Depth 256 succeeds, 257 refuses;
high-count backward metadata exhaustion refuses within 2 MiB. Oversized strings
and aggregate counts leave the original empty active mark and point unchanged.
Decoded C-M-n dispatch on 8,194 bytes measured 379 microseconds; completion plus
the queued ordinary key measured 35,267 microseconds. These measurements cover
native decoded input, not physical terminal transport or an optimized benchmark.

## Risks and Follow-ups

PR #63 integrated validated commit `318a65e4030c857b9457db265d46cc447f3a51aa`
into dev. Its three ordered required checks passed through looper validate.
The clean inactive integrated worktree was removed without force.
The later nesting/expression slices and representative physical-input PTYs are
delivered. The controller's unchanged combined gate passed at
`27957128f7626ee16b5dbdc50af52e9cee27f3c8`, receipt
`YEU4CDCDB3RP65D5PG67RRYTYF`. The
[Slice 5 closure](slice-5-acceptance.md#delivery-set-closure) records the exact
command, integrated outcomes and later Markdown-only closure relationship.
The existing vendored crossterm parentheses warning remains unchanged.
