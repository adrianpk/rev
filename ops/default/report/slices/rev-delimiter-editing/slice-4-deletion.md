# Slice 4: Empty-pair deletion and quote boundaries

Status: delivered
Delivery set: rev-delimiter-editing
Plan: [Automatic Delimiter Editing Delivery](../../../plan/rev-delimiter-editing.md)
Tracker: [Automatic Delimiter Editing Tracker](../../../tracker/rev-delimiter-editing.md)
Branch: `feat/rev-delimiter-editing-04-deletion`
PR: [#58](https://forge.adrianpk.com/rev/rev/pulls/58)

## Purpose

Delete adjacent empty admitted pairs through native Backspace and complete the
approved conservative quote/profile boundary matrix.

## Delivered Behavior

Backspace between confirmed adjacent empty structural or eligible quote pairs
removes both ends in one checked native edit. Whitespace, content, malformed or
excluded context, disabled policy and exhausted qualification retain ordinary
grapheme deletion. Attached combining characters are never split to consume a
closer. Native region, read-only, focus, modal and prefix owners remain in place.

Quote admission follows the current recognized language independently of colors.
The matrix covers ordinary double/single/backtick eligibility, word apostrophes,
escapes, Rust character/lifetime/raw ambiguity, script templates/interpolation,
HTML attributes and embedded script/style, TOML triples and prose code/verbatim.
Existing qualification already supplies these conservative rules; deletion uses
that same proof. Current syntax exclusions and language-change retirement are
verified rather than inferred from highlight labels.

Native undo/redo restores exact source, initiating cursor and mark. Other shared
points retain the native current-position/right-affinity mapping. The shared
runtime toggle and per-language deletion preference govern the operation without
saving settings. The User Guide demonstrates Backspace and undo, the reference
lists the delivered quote profiles, and the existing Unreleased entry includes
the behavior once.

## Implementation Notes

- A fixed two-scalar candidate check avoids scheduling qualification for ordinary
  text. It is only admission: the canonical buffer qualifier must confirm stamped
  opening/closing spans and known depth before deletion can extend its range.
  Both outer grapheme boundaries must also be valid native coordinates.
- One private deletion request captures source point, selected view, mark, mode
  and effective policy. The existing bounded layout/input FIFO polls its buffer
  ticket cooperatively. C-g and changed source retire uncommitted proof; later
  queued typing uses the published cursor once.
- Publication rechecks the capture and uses the native text-entry edit owner,
  including modal repeat/history capture after a cooperative wait. No source
  mutation, parser work or history implementation is duplicated.
- Tests reuse the existing Session fixture and buffer qualification fixture.
  Full Session paths cover editing/history/focus; lower-cost buffer matrices cover
  quote variants. No new fixture service, permanent worker or per-view cache exists.

## Contracts Added or Changed

The resolved delete_empty policy now controls source Backspace. Quotes also
require the quotes preference and the current language profile. No command,
binding, dependency or configuration leaf is added. The 256 KiB request budget,
depth 256, 2 MiB buffer metadata bound, 256-step poll and four-millisecond scheduling
target remain unchanged. Existing literal replacement ingress is unchanged.

## Files of Interest

- `src/session/pairs/deletion.rs`: captured Backspace admission and publication.
- `src/session/layout.rs`, `src/session.rs`: native cooperative routing/cancellation.
- `tests/pairs/deletion.rs`: source, history, policy, ownership and resource cases.
- `tests/pairs/quotes.rs`: profile, ambiguity, current syntax and language retirement.
- `tests/pairs/editor.rs`: shared native physical-input fixture.
- `docs/reference/delimiter-editing.md`: delivered behavior, policies and quote matrix.

## Validation

Successful implementation checks:

- `cargo fmt --all -- --check`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs -- deletion quotes --test-threads=1`: 13 passed in 42.40 seconds with a warm build.
- `git diff --check`: passed.

Supplemental checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs -- deletion::admission deletion::history deletion::pending deletion::focus_modes --test-threads=1`: 4 passed in 18.40 seconds.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs quotes::native_info -- --test-threads=1`: passed in 1.74 seconds.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: passed; 708 commands, 332 configuration entries, 111 reachable pages, 784 links/anchors, 10 command-reference tests and 12 configuration-reference tests.

A broader shared-owner batch selected all 48 pair tests: all 19 qualification and
17 typing regressions passed, with 47 passes overall and one new HTML fixture
provenance failure in 109.53 seconds. The unclosed HTML fixture correctly used
lexical fallback; a balanced fixture now checks current-syntax provenance and
passes both the focused rerun and required selection. The final history case
also exercises exact quote undo/redo, alongside structural deletion.

Development feedback corrected an attempted private repeat-state access by using
the existing owner, and extracted cohesive admission/boundary checks to meet
source complexity limits. Initial test assumptions about BOF feedback, shared-view
history and direct-replacement cursor placement were corrected against their
native contracts. No acceptance limit or native behavior was weakened. A source
rebuild took 75 seconds before the first completed test batch. The vendored
crossterm parentheses warning is pre-existing and non-blocking.

Personal configuration and frozen factory catalog/manifest digests are unchanged.
The implementation commit `a254aae20d567287160d83da3682fe0e006ca8e1` passed
full staged-source formatting, strict workspace Clippy and native policy across
1,940 Rust files and 77 Cargo roots. No blocking/deferred Clippy diagnostic or
source-policy suppression was introduced. The strict check profile took 122 seconds
after the private tool/cache build. T4.1-T4.3 were committed together to keep the
deletion path, its quote proof, native regressions and reader guidance reviewable.
The final structural/quote history matrix also passed its focused test in 1.31
seconds after 3 seconds of linking.
The exact three-command clean-HEAD controller receipt passed at
`172fcaeea33367a72f1c98ec09c9281b34d12a86`, including all 13 selected tests.
Canonical PR #58 integrated that same commit into dev. Combined acceptance
remains controller-owned.

## Risks and Follow-ups

Selection wrapping, explicit helpers, complete literal/macro provenance and
physical-input acceptance remain in their recorded later slices. These native
checks do not claim PTY or completed-frame acceptance. Unproved embedded language
and raw/triple/template context remains conservative.

Delivery-set closure evidence is collected in the
[Slice 7 report](slice-7-acceptance.md). This slice's delivered status records
its own canonical integration and focused checks. The combined exact-candidate
gate and editing-only completion remain pending controller execution after
Slice 7 integration.
