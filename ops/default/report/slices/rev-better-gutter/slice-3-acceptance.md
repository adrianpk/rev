# Slice 3: Joint acceptance and guidance

Status: delivered
Delivery set: rev-better-gutter
Plan: [Better Gutter Delivery](../../../plan/rev-better-gutter.md)
Tracker: [Better Gutter Tracker](../../../tracker/rev-better-gutter.md)
Branch: `test/rev-better-gutter-03-acceptance`
PR: `#284`
Integrated: `09b88ac85eee353dfb9649d43f0e696bb02d7fa9`

## Purpose

Verify BMK03 and BTG01-BTG04 together after Slice 2 integration, complete the
ordinary product guidance and reconcile the existing baseline contracts.

## Delivered Behavior

- A bounded Rust/Bash acceptance entrypoint exercises 240 native presentation
  combinations: three startup configurations, both sides, absolute/relative
  numbers, independent number/marker visibility and five pane widths.
- Factory square/blue, Nerd Fonts bookmark/blue and included legacy
  marker-string/explicit-color input retain the expected cells and source state.
  Additional native cases cover Org fold reveal and resized completion anchors.
- One owned PTY per interaction profile sets, hides and restores a bookmark,
  resizes, changes wrapping, selects/copies source, yanks and undoes the copied
  text, checks idle behavior and exits with terminal attributes restored.
- Bookmark how-to, configuration reference and User Guide describe the current
  presentation, font opt-in and migration from old sample values. Spore derives
  the guide and its configuration download from the same repository sources.
- The gutter, bookmark and configuration specifications/models now agree on
  current geometry, symbols and compatibility. Historical completion metadata,
  candidates and gate receipts remain unchanged. Future folding and debugger
  conventions remain future capabilities.

## Implementation Notes

T3.1 is implemented in `0d1219ed67b779c65d75fddf2d8f0672ac9a8778`.
T3.2 is implemented in `da090457b897ffb0eedd3bc97691baab3aede4f9`.
The preceding documentation commit closes Slice 2 against merged PR #283 and
activates this recorded slice without changing primary dev.

The driver reuses the native Session/Frame API, existing terminal surface
decoder and bounded completion-acceptance PTY helper. It adds no dependency,
provider abstraction or historical resource-gate chain. Configuration variants
stay native, leaving exactly two representative PTYs.

The Bash wrapper builds the local debug editor and example with the approved
low-debug environment and locked offline dependencies. HOME/XDG locations and
source files are temporary owned fixtures. The reused helper retains its
eight-second and 1-MiB limits per PTY; native and combined PTY modes have
20-second outer bounds. Cleanup removes only these fixtures after their checks.

## Contracts Added or Changed

- Checked frame cells, cursor and pointer hits agree with outside marker
  placement and zero folding-column allocation, including narrow omission.
- Presentation-only native cases retain records, source stamp/state, saved
  bytes and empty history. Pointer and whole-source copies exclude gutter
  cells while preserving Unicode and mixed LF/CRLF source.
- Both terminal profiles demonstrate that copied text is exactly the selected
  source substring, with unchanged source/configuration on disk after undo.
- Baseline descriptions distinguish current fixed-symbol configuration from
  ignored legacy string input and from future folding/debugger functionality.

## Files of Interest

- `examples/better-gutter-acceptance.rs`, `examples/better_gutter_acceptance/`:
  native variants, representative profile journeys and small owned helpers.
- `scripts/check-better-gutter.sh`: bounded local build, isolated invocation,
  terminal wrapper and fixture cleanup.
- `docs/how-to/use-bookmarks.md`, `docs/reference/defaults.md`, User Guide
  chapters 04 and 09: current usage, preference and migration guidance.
- Gutter, bookmark and configuration specifications and their models under
  `ops/default/spec/`: current presentation reconciliation with historical
  acceptance evidence preserved.

## Validation

All required commands passed locally, in the approved order:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test gutter --test bookmarks -- --test-threads=1`
- `bash -n scripts/check-better-gutter.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The suites passed 94 tests: bookmarks 58 and gutter 36, with none ignored.
Documentation validation passed 596 commands in each combined inventory,
281 configuration entries, 85 reachable pages and 566 local links and anchors.
Observed times were 3.14s for formatting, 65.30s for all-target Clippy, 23.82s
for the suites, below 0.01s for Bash syntax, 1.08s for documentation and 0.01s
for diff hygiene. The existing vendored crossterm warning remains nonfatal.

Additional focused commands passed:

- `bash scripts/check-better-gutter.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`

The acceptance run passed all native cases. Non-modal PTY elapsed time was
932ms with 11863 output bytes and 29152 kB peak RSS; modal was 1004ms with
11878 output bytes and 30540 kB peak RSS. Both observed 350ms of idle silence
with zero CPU ticks and restored TTY state. The cached acceptance build took
1.17s; initial cold compilation took 1m30s. All owned fixtures finished and
were removed. Spore built the 85-page guide; generated preference links and the
configuration download were checked against their sources.

Looper receipt `KZYA354UGVNE62JZKDAKX5QPLZ` passed all six required commands
against clean PR-reference HEAD `09b88ac85eee353dfb9649d43f0e696bb02d7fa9`.
Canonical PR #284 was verified merged into dev at that same commit.
These slice results do not establish the controller-owned integrated final
gate or delivery-set completion.

### Final-gate repair 1

Repair status: reviewing
Branch: `fix/rev-better-gutter-validation-1`
PR: `#285`
Implementation: `4f5d6da28d93d780d8b4a905f5d53100424f2203`

The controller's unchanged full gate failed at
`10054d1fece9557d37923fa8be0d825727232ac7`, receipt
`VIR36T3PHGW77O2D4MNMV64DGX`. Formatting and Clippy passed; the selected test
command then failed in `org_indentation::gutter_and_bookmark_identity`, which
still expected the former factory dot. The first assertion observed `■` and
expected `●`; its folded-row absence check also searched for the old dot.

The isolated reproduction failed with the same mismatch:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_indentation gutter_and_bookmark_identity -- --exact --test-threads=1`

The repair updates both expectations to the approved factory square. It retains
the checked marker column, both gutter sides, first-fragment/continuation
behavior, Org indentation, cursor placement, folded-marker absence and bookmark
source position assertions. No runtime code, fixture configuration, timeout or
frozen gate requirement changes.

All four required repair commands passed locally, in the injected order:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test gutter --test bookmarks --test configuration -- --test-threads=1`
- `git diff --check`

These suites passed 250 tests: bookmarks 58, configuration 156 and gutter 36,
with none ignored. Observed times were 2.82s for formatting, 63.93s for Clippy,
120.96s for the suites and 0.01s for diff hygiene.

The focused regression command
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_indentation -- --test-threads=1`
passed all 18 tests, including the previously failing identity case. Initial
cold compilation took 1m43s and the suite took 9.31s. Test fixtures completed
and released their owned temporary files.

The exact clean-HEAD batch is recorded through `looper validate` after the
repair PR-reference commit. These focused results do not replace the unchanged
full gate; the controller owns repair integration and that gate's retry.

## Risks and Follow-ups

Native evidence proves emitted code points, geometry and renderer colors, not
the drawing or availability of a physical terminal font. The PTYs exercise the
ordinary factory square; glyph/configuration variants remain native coverage.
The controller owns exact integrated-candidate validation and closure.
No final-gate, main promotion, release or public deployment result is claimed.
