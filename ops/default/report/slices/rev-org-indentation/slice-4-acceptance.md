# Slice 4: Joint acceptance and guidance

Status: delivered
Delivery set: rev-org-indentation
Plan: [Org Indentation Delivery](../../../plan/rev-org-indentation.md)
Tracker: [Org Indentation Tracker](../../../tracker/rev-org-indentation.md)
Branch: `test/rev-org-indentation-04-acceptance`
PR: `#279`
Integrated: `6c010648a49847b54f914063a2261e7403dd8c34`

## Purpose

Complete T4.1-T4.3 by supplying the bounded native/resource/PTY acceptance
entrypoint and Markdown guidance for OIN01-OIN05. The controller owns execution
of the frozen combined gate and delivery-set closure after integration.

## Delivered Behavior

- `scripts/check-org-indentation.sh` builds one Rust acceptance driver and
  reuses the existing owned PTY and native-session helpers. Its two profile
  journeys combine indentation controls, wrapping, virtual-margin pointer
  targeting, source editing and undo, resize, idle silence and terminal restore.
- Native sessions check source-only copy/save and edit/undo/redo. Resource cases
  cover a large outline, an admitted depth-256 heading and over-limit literal
  fallback with Unicode and tabs, repeated toggle/resize reflow, unchanged source and no new idle deadline.
- The existing private cache-retention case uses the runtime buffer override
  across eight policy changes and two widths. It asserts and prints actual
  retained fragment/text peaks under the unchanged 8192-fragment and 1 MiB caps.
- The Org learning path shows enabled/disabled text and all three commands.
  Reference and historical heading/wrapping contracts describe independent
  buffer indentation and per-view pretty choices through one checked mapping.
  Existing historical receipts remain attached to their original candidates.

## Implementation Notes

T4.1-T4.3 are implemented in `da5fc782b23df9d5d360ba306e7fd8629006b7a8`.
The following map connects the four reports to their implementation and integration evidence.

| Slice | Requirements and tasks | Implementation | Integrated PR | Report |
| --- | --- | --- | --- | --- |
| 1 | OIN01, OIN03, OIN05; T1.1-T1.3 | `54395f4feea63a7cc0de34582f4d18c3840c09de` | #276 at `3849381e48c4951a33cba391201fd0bc7cb4f840` | [Projection](slice-1-projection.md) |
| 2 | OIN01-OIN03, OIN05; T2.1-T2.3 | `66e6654eeba004dd997c74144c55253aca5bc9b6` | #277 at `8e839d21cdb029012d6f5c6400c6f5abec6cbff3` | [Presentation](slice-2-presentation.md) |
| 3 | OIN02-OIN05; T3.1-T3.3 | `e6dc1c0f1b07e046309911867b9782d87b24008e` | #278 at `660b97410fd96f6cfb3b664b48dceee7b37ddfc3` | [Controls](slice-3-controls.md) |
| 4 | OIN01-OIN05; T4.1-T4.3 | `da5fc782b23df9d5d360ba306e7fd8629006b7a8` | #279 at `6c010648a49847b54f914063a2261e7403dd8c34` | This report |

No dependency, public inspection API, second PTY harness or production layout owner is introduced.
The existing Markdown-derived Spore publication pipeline consumes the updated
chapter; its strict build remains part of the controller-owned combined gate.

## Contracts Added or Changed

- Script fixtures isolate HOME and XDG stores. Each PTY has the existing
  eight-second lifetime and 1 MiB output ceiling; only owned children are reaped.
- Native/resource invocations have finite deadlines. Resource output records
  source size, reflow/startup duration, idle CPU ticks and peak RSS separately
  from direct private cache-retention assertions.
- Heading decoration preserves source-prefix width inside the structural
  margin. Wrap continuations carry the enabled Org body/title margin; ordinary
  text receives no automatic prefix. Virtual cells never become source scalars.

## Files of Interest

- `examples/org-indentation-acceptance.rs`,
  `examples/org_indentation_acceptance/`, `scripts/check-org-indentation.sh`:
  joint acceptance entrypoint using existing helpers.
- `src/view/wrap/cache.rs`: runtime-policy retention acceptance at its owner.
- `docs/tutorials/user-guide/06-org-notes.md`, `org/index.md` in that guide:
  the focused Org learning path and its existing Spore source.
- `ops/default/spec/editor-org-heading-presentation{,-model}.md`,
  `ops/default/spec/editor-soft-wrap{,-model}.md`: selected baseline reconciliation.

## Validation

The following commands passed locally in the required order:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_indentation --test org_headings -- --test-threads=1`
- `bash -n scripts/check-org-indentation.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The batch passed 32 native tests (16 headings and 16 indentation). Documentation
validated 596 commands, 281 configuration entries, 85 reachable pages and
560 local links/anchors. Observed command times were format 2.79s, Clippy 48.27s,
native tests 24.98s, Bash syntax under 0.01s, documentation 1.07s and diff 0.01s.

All six required commands also passed through Looper on the exact clean
PR-reference HEAD `6c010648a49847b54f914063a2261e7403dd8c34`. The canonical PR
contract was verified, and PR #279 is merged into dev at that commit.
The controller subsequently passed the unchanged combined gate and strict Spore
build after integration; that separate evidence is recorded under Delivery-set closure.

Focused development evidence:

- Eight `view::wrap::` native cases passed in 6.88s after a 41.09s first test build.
  Actual retention peaks were 8192 fragments and 1048576 bytes, including
  eight runtime policy changes at widths 6 and 512. Cooperative-batch cases
  also passed; none of the existing ceilings or work assertions was weakened.
  Rechecking the same eight cases with isolated HOME/XDG stores passed in 6.42s
  (6.34s test runtime), confirming the entrypoint environment. The native command
  was `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib view::wrap:: -- --nocapture --test-threads=1`.
- The scoped Rust source-quality check passed after splitting new fixture
  orchestration at its actual responsibilities; its initial complexity failures
  were confined to new driver functions, not accepted production behavior.
- Development-profile native journeys passed both profiles in 0.47s. They check
  source copy/save, virtual-margin hits, wrap controls and edit/undo/redo.
- Development-profile PTY journeys passed both profiles in 2.12s at
  40x12/64x12/40x12. Startup was 256437us non-modal and 303512us modal;
  each emitted 10149 bytes and consumed zero CPU ticks over a 350ms idle window.
  Peak RSS was 30120/31912 kB. Both restored terminal attributes, cursor,
  alternate screen and bracketed paste, with unchanged source and configuration.
- The first 20-column PTY fixture timed out because its filename was clipped,
  while the shared harness requires that filename in the startup frame. The
  corrected 40-column fixture uses a longer line and retains continuation and
  pointer checks. The eight-second/1-MiB PTY bounds remain unchanged.
- A development-profile resource run hit the existing five-second native
  settle bound. The intended normal-release profile was checked with the
  same workload and deadline; no acceptance limit was raised. The release probe
  passed. The initial deep-margin fixture exceeded the native 256-depth bound;
  it is retained as an explicit 32768-star incomplete-coverage/literal-progress
  case alongside the admitted depth-256 capped-margin case.
- Final normal-release resource probes passed in 1.78s. Large: 779022 bytes,
  729870 scalars, 330309us startup and 5548us for 16 toggle/resize reflows.
  Depth 256: 57607 bytes, 55643us startup, 204038us reflows and a four-cell
  margin at width six. Over-limit: 90119 bytes, 49249us startup, 81524us reflows
  and zero literal margins. Every case retained exact source, empty history,
  no idle deadline and zero CPU ticks over 350ms. Peak RSS was 19200/19184/19184 kB.
- The cold normal-release build took 6m32s. The entrypoint keeps timing probes
  in normal release, with one build job. Private cache/work assertions reuse
  the existing low-debug native test profile to avoid a second optimized
  library-test build; the same eight cases and limits remain enforced.

## Delivery-set closure

The controller passed the unchanged selected full gate at immutable dev candidate
`61e6e6a1b73d94577c7851aae6f200f6101d35c7`, receipt
`YE523W7IGTDKXEUNGCMA3NNJBY`. Its exact successful combined command is recorded
in the tracker and matches the plan's frozen Final validation selection.

All 569 selected integration tests passed: 164 commands, 155 configuration,
39 display, 36 gutter, 16 headings, 16 indentation, 65 pointer, 26 wrapping,
11 Org visibility, 20 modal contexts and 21 external-change cases. Format,
strict all-target Clippy and scoped Rust source quality passed. The eight native
cache/work cases passed with actual retention peaks of 8192 fragments and
1048576 bytes, including eight runtime policy changes at widths 6 and 512.
The normal-release build took 4m19s; the command suite ran in 197.56s and
configuration suite in 91.18s.

Both-profile native journeys passed source-only copy/save, policy controls,
wrap, margin hits, edit/undo/redo and no idle deadline. Normal-release resource
observations were:

| Case | Source bytes | Startup (us) | 16 toggle/resize reflows (us) | Peak RSS (kB) |
| --- | --- | --- | --- | --- |
| Large outline | 779022 | 362757 | 6321 | 19272 |
| Admitted depth 256 | 57607 | 64214 | 241322 | 19192 |
| Over-limit 32768 stars | 90119 | 52583 | 94487 | 19192 |

Every resource case retained exact source, empty history, no idle deadline and
zero CPU ticks over 350ms. At width six, the admitted deep heading retained a
four-cell margin; the over-limit case used zero literal margins with finite
source progress. Bounds and acceptance assertions remain unchanged.

The non-modal/modal PTYs passed at 40x12/64x12/40x12, with startup
69449/93615us, 10149 output bytes each and peak RSS 15148/16408 kB.
Both measured zero idle CPU ticks over 350ms, retained source/configuration
bytes and restored terminal state. The eight-second/1-MiB PTY limits remained
unchanged. Catalog/documentation checks validated 596 commands, 281 settings,
85 reachable pages and 560 local links/anchors. All 85 canonical Markdown pages
projected and built locally through Spore's strict link check and render into
`target/userguide`.

All four canonical PRs are merged and all four reports delivered. The paired
indentation specification/model, plan and tracker are Completed, and the pending
index records the delivered capability. The gate's fixture guards and owned
PTY children completed; inspection found no remaining gate editor/driver
processes or private indentation fixture roots. Only configured closure
documentation differs from the tested candidate. The controller retains final
checkpoint verification.

## Risks and Follow-ups

No implementation follow-up is recorded. The local guide build does not
establish public deployment.
