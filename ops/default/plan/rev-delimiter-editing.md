# Automatic Delimiter Editing Delivery

Status: Approved
Date: 2026-11-15
Delivery set: rev-delimiter-editing
Slice strategy: layered
Planning authority: product-owner request for first-capability slicing and Looper, 2026-11-15
Approved: product owner, 2026-11-15; automatic editing first, navigation second, presentation third
Execution authority: slices 1-7, predecessor-ordered canonical PRs and controller-owned Rebase + Fast-forward integration into dev; no successor startup
Planning baseline: `fa48f847e9075eafd2272891c29c7dbf04bf7f16`
Spec: [Delimiter Pairs](../spec/editor-delimiter-pairs.md)
Model: [Pair Preferences and Native State](../spec/editor-delimiter-pairs-model.md)
Concern: [Operations index](../README.md)
Active slice: Slice 2 — Bounded context and pair qualification
Integration target: `dev`
Tracker: [Automatic Delimiter Editing Tracker](../tracker/rev-delimiter-editing.md)

## Outcome and strategy

Deliver context-aware automatic pair insertion, closer skipping, empty-pair
Backspace, linear-selection wrapping, explicit wrap/unwrap and a buffer-local
toggle. Keep source/history and literal-input ownership native. No new dependency,
parser, language server, binding assignment or persistent worker is required.

Layered slicing is appropriate because configuration and source-qualified context
are reusable prerequisites for independently valid typing, deletion and region
increments. Each layer compiles and passes meaningful focused checks. Do not add
stubs, deliberately failing tests, skipped acceptance or future command names.
Runtime behavior follows only after its preceding contract layer is integrated.

Navigation, expression motions, depth colors and match/pulse presentation retain
their separate Draft specifications. The shared model's editing portion is
approved; its navigation/presentation portions are not completed by this set.
Preserve the completed factory-shortcut set and its immutable receipts.

## Entry and ownership

The editing specification, language admission, defaults, counts, point placement
and the hard resource bounds below are fixed for this set. Publish this approved
plan and synchronized tracker before creating any formal slice branch. All seven
slices and canonical dev integration are authorized by the continuation.

Keep the primary checkout on dev. Create each slice worktree under the canonical
dated project worktree root, with exclusive ordinary Cargo and hook caches. Use
one build job, disabled incremental builds and no dev/test debuginfo. Preserve
installed hooks without reinstalling or bypassing them. Never mutate the primary
checkout during its build or interrupt another owner. After each integration,
verify fetched ancestry, cleanliness and absence of active processes, then remove
the integrated worktree with non-forced Git removal before starting its successor.
Preserve pre-existing worktrees, doubtful/dirty data and other agents' processes.

No current project ADR index exists. Apply the repository architecture contracts:
source/history mutations belong to native edit owners; syntax provides qualified
evidence; session owns routing; projections consume source coordinates. Keep the
new pair owner cohesive, not a public generic parsing framework.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Configuration and language policies | `feat/rev-delimiter-editing-01-config` | `feat(editor): configure automatic pair policies` | `ops/default/report/slices/rev-delimiter-editing/slice-1-config.md` |
| Slice 2 | Bounded context and pair qualification | `feat/rev-delimiter-editing-02-qualification` | `feat(editor): qualify delimiter pairs with bounded context` | `ops/default/report/slices/rev-delimiter-editing/slice-2-qualification.md` |
| Slice 3 | Typed pairing and buffer toggle | `feat/rev-delimiter-editing-03-typing` | `feat(editor): insert and skip admitted delimiter pairs` | `ops/default/report/slices/rev-delimiter-editing/slice-3-typing.md` |
| Slice 4 | Empty-pair deletion and quote boundaries | `feat/rev-delimiter-editing-04-deletion` | `feat(editor): delete empty pairs and preserve quote contexts` | `ops/default/report/slices/rev-delimiter-editing/slice-4-deletion.md` |
| Slice 5 | Selection wrapping and explicit helpers | `feat/rev-delimiter-editing-05-regions` | `feat(editor): wrap selections and unwrap confirmed pairs` | `ops/default/report/slices/rev-delimiter-editing/slice-5-regions.md` |
| Slice 6 | Literal provenance and source projections | `fix/rev-delimiter-editing-06-provenance` | `fix(editor): preserve literal edits and macro pair provenance` | `ops/default/report/slices/rev-delimiter-editing/slice-6-provenance.md` |
| Slice 7 | Integrated acceptance and guidance | `test/rev-delimiter-editing-07-acceptance` | `test(editor): verify automatic delimiter editing workflows` | `ops/default/report/slices/rev-delimiter-editing/slice-7-acceptance.md` |

## Approved behavior and profiles

The specification owns the full language/profile matrix and transaction rules.
All five global editing booleans default true; each has an optional singular
language override and existing origin/candidate validation. Runtime auto-pair-mode
is buffer-local and shared by its views. It changes no settings file.

Only three unbound catalog entries are added: auto-pair-mode in Slice 3,
pair-wrap and pair-unwrap in Slice 5. The final catalog is the original 707
commands plus these three. Preserve the original catalog, reviewed shortcut
manifest and digests. Extend the current acceptance check with an independently
written additive expectation at each owning slice; do not regenerate expected
maps/catalogs from the runtime or weaken the old equality check. Every existing
factory mapping remains unchanged. Validate the old accepted maps once through
the existing native acceptance helper, without chaining its historical PTYs.

Freeze the independent [editing reference cases](../report/rev-delimiter-editing/reference-editing.json)
and [reference probe](../report/rev-delimiter-editing/reference-editing.el) with preparation.
These 12 GNU Emacs 31.1 observations establish reference effects; Rev's approved
native transaction, literal provenance and uncertainty rules remain authoritative.
They are not generated from Rev, and reference observations are not native or PTY
acceptance. Run reference probes only with a temporary isolated buffer and no init.

## Resource sketch and measured baseline

The existing raw percent-matching primitive was measured in an optimized native
library built from runtime baseline dd1b569d, using Buffers, Coordinates and
Target::Pair. Buffer creation was outside measurement; each row has 41 requests.
This is a local single-host planning baseline, not the future service's result.

| Source bytes | Complete median / p95 microseconds | Unmatched median / p95 microseconds |
| --- | --- | --- |
| 1,024 | 5 / 5 | 4 / 4 |
| 65,536 | 277 / 303 | 271 / 303 |
| 262,144 | 1,057 / 1,148 | 1,023 / 1,139 |
| 1,048,576 | 4,211 / 4,894 | 4,241 / 4,694 |

The current scan has no context/work admission. Limit new fallback inspection to
262,144 bytes total per request, nesting to 256 and retained qualified metadata
to 2,097,152 bytes per buffer. Positive counts share this budget and preflight
output expansion through existing native count/edit limits. Counted refusal is
atomic; uncertain ordinary typing remains literal. Larger documents may admit
local requests when qualification proves them inside the same finite budget.

Use at most 256 tokens per cooperative work slice and check byte progress within
oversized tokens. Reuse the native four-millisecond scheduling target; it is a
scheduling target measured in acceptance, not a universal wall-clock guarantee.
No timer/idle polling is introduced. Cancellation/revision changes retire work.
A depth stack of 256 entries is small relative to the 2 MiB metadata budget;
measure actual retained structures and shared ownership rather than assuming
that cloned per-view caches are free. Byte/depth/retention limits are correctness
assertions; latency distributions are measured evidence with disclosed workload.

Slice 2 verifies the new owner before typing integration. Slice 7 repeats the
representative integrated workloads and reports every early refusal, maximum
work/retention and input/continuity observation. Limits cannot be relaxed to make
a test pass; a material scope/boundary change requires owner review.

## Slice tasks and checks

Required commands below are exact plain shell strings for native Looper config.
The worker may add justified focused commands after this required prefix; include
git diff --check last and list only successful executed commands in PR Validation.
The frozen configuration must contain no Markdown backticks inside command values.

## Slice 1: Configuration and language policies

Requirements: DPR04; editing schema in the shared model.
Dependency: approved committed preparation.
Branch: `feat/rev-delimiter-editing-01-config`
Expected PR title: `feat(editor): configure automatic pair policies`
Report: `ops/default/report/slices/rev-delimiter-editing/slice-1-config.md`

- T1.1: Extend the native parsed, partial and resolved configuration with only the five editing booleans. Add compiled defaults and per-language leaf inheritance, whole-candidate rejection, origin paths and effective snapshots; preserve absent-sibling inheritance and grammar-independent language resolution.
- T1.2: Connect resolved editing policy to the native buffer/language lifecycle without automatic source edits. Preserve explicit user overrides, shared-buffer ownership and unrelated settings. Add only editing leaves to the generated template and configuration reference; do not accept future presentation leaves.
- T1.3: Add parameterized configuration regressions for defaults, every false override, per-language inheritance/provenance, malformed candidates and language changes; report exact coverage and preserve personal configuration bytes.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration pairs -- --test-threads=1
git diff --check
```

## Slice 2: Bounded context and pair qualification

Requirements: DPR04; shared source-qualified typed-pair contract.
Dependency: Slice 1 integrated and closed.
Branch: `feat/rev-delimiter-editing-02-qualification`
Expected PR title: `feat(editor): qualify delimiter pairs with bounded context`
Report: `ops/default/report/slices/rev-delimiter-editing/slice-2-qualification.md`

- T2.1: Implement one cohesive native owner for language/context classification and strict typed-pair qualification. Reuse current syntax when qualified; use the approved conservative lexical policy otherwise. Retain buffer/revision, native spans, delimiter kind, provenance and explicit uncertain/limited outcomes; do not expose navigation commands or reuse loose group-motion results as mate proof.
- T2.2: Enforce the fixed inspected-byte, nesting, retained-memory and cooperative-work bounds, including oversized tokens, counted-request reuse, cancellation and source replacement. Never retain tree nodes across revisions or scan the whole source on each key/frame. Record actual new-service work, latency and retention measurements against the planning baseline.
- T2.3: Introduce tests/pairs.rs with qualification cases for mixed/nested/malformed and multiline punctuation, strings/comments/escapes, approved language profiles, stale/current syntax and disabled syntax colors, UTF-8/CRLF and every resource boundary. Preserve existing modal percent matching and inner/around objects; provide the slice report.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs qualification -- --test-threads=1
git diff --check
```

## Slice 3: Typed pairing and buffer toggle

Requirements: DPR01, DPR04; automatic insertion, skipping and buffer policy.
Dependency: Slice 2 integrated and closed.
Branch: `feat/rev-delimiter-editing-03-typing`
Expected PR title: `feat(editor): insert and skip admitted delimiter pairs`
Report: `ops/default/report/slices/rev-delimiter-editing/slice-3-typing.md`

- T3.1: Route eligible physical source typing in non-modal and modal Insert/Replace through one checked native transaction. Insert admitted structural/quote mates, reuse an existing unmatched closer and skip only an admitted closer. Uncertain ordinary typing stays literal; preserve read-only, focus and modal owners.
- T3.2: Honor native numeric prefixes once: positive counts compose one transaction and undo unit, zero is inert and negative typed repetition retains refusal. Preflight expansion and work limits before publication; generated mates are effects rather than new key events. Wire buffer-local auto-pair-mode shared across views and retained across language changes without writing configuration.
- T3.3: Add the unbound auto-pair-mode catalog entry, named dispatch, descriptions and introspection, plus independent additive command-inventory expectations. Test insertion/count/skip/refusal, undo/redo, mode/focus/toggle and source-policy behavior; add concise reader-facing guidance for the delivered feature using the changelog skill.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs typing -- --test-threads=1
git diff --check
```

## Slice 4: Empty-pair deletion and quote boundaries

Requirements: DPR01, DPR04; empty-pair deletion and language safety.
Dependency: Slice 3 integrated and closed.
Branch: `feat/rev-delimiter-editing-04-deletion`
Expected PR title: `feat(editor): delete empty pairs and preserve quote contexts`
Report: `ops/default/report/slices/rev-delimiter-editing/slice-4-deletion.md`

- T4.1: Integrate Backspace with existing deletion and region precedence. Remove both ends only for an adjacent empty admitted pair, in one undo unit; whitespace/content, disabled policy and uncertain context retain ordinary deletion. Preserve grapheme, line-ending, read-only and history behavior.
- T4.2: Complete the approved quote/profile matrix across ordinary code and prose contexts. Keep word apostrophes, escaped quotes, Rust lifetime/character ambiguity, raw/triple/template/interpolation ambiguity and unproved embedded contexts literal; confirm current-language evidence before automatic quote edits.
- T4.3: Add deletion and quote matrices, including nonempty pairs, malformed source, comments, combining characters, split views and toggles. Verify exact source and cursor/mark restoration through undo/redo and update delivered policy documentation and the slice report.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs -- deletion quotes --test-threads=1
git diff --check
```

## Slice 5: Selection wrapping and explicit helpers

Requirements: DPR01, DPR04; linear-region wrapping and explicit pair helpers.
Dependency: Slice 4 integrated and closed.
Branch: `feat/rev-delimiter-editing-05-regions`
Expected PR title: `feat(editor): wrap selections and unwrap confirmed pairs`
Report: `ops/default/report/slices/rev-delimiter-editing/slice-5-regions.md`

- T5.1: Implement typed opening/closing wrapping for active ordinary linear selections, including both orientations and positive counts. Preserve selected bytes, deactivate the region and use the approved opener/closer point placement. Reject unsafe quote transformations before mutation; leave rectangular/block owners intact.
- T5.2: Add unbound pair-wrap and pair-unwrap through native catalog/dispatch. Use a cancellable native pair-selection prompt for wrapping, with an admitted opening or closing character selecting its pair and opener placement. Unwrap only the innermost confirmed enclosing pair at point, preserving contents/literal meaning and native point/mark mapping; explicit helpers work with automatic editing disabled.
- T5.3: Add independently specified region/helper/count cases with undo/redo and selection orientation. Extend additive inventory expectations by exactly these two commands, retain every old factory mapping and document invocation, cancellation, eligibility and refusals in the existing guide/reference owners.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs regions -- --test-threads=1
git diff --check
```

## Slice 6: Literal provenance and source projections

Requirements: DPR01, DPR04; exactly-once input and native projection integration.
Dependency: Slice 5 integrated and closed.
Branch: `fix/rev-delimiter-editing-06-provenance`
Expected PR title: `fix(editor): preserve literal edits and macro pair provenance`
Report: `ops/default/report/slices/rev-delimiter-editing/slice-6-provenance.md`

- T6.1: Trace and exercise paste, completion, provider/AI edits, external reload, plugins and programmatic replacements through their real native source-edit ingress. They remain literal and never synthesize pair input. Integrate raw macro typing once while accepted literal macro steps replay literally, retaining existing counts, counters, cancellation and history semantics.
- T6.2: Verify shared-buffer/split-view policy, language changes, source replacement and stale-work retirement. Preserve Org/Markdown pretty/indent source-to-display mapping and completed-frame continuity during body typing; pair editing must not introduce transient literal fallback, cursor relocation or stale spans.
- T6.3: Add native provenance and projection regressions plus targeted existing macro cases. Assert exact bytes and source revision/point/mark/undo behavior at these boundaries, without network providers or changes to their business logic; document remaining physical-input acceptance in the report.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs provenance -- --test-threads=1
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- recording regions runner --test-threads=1
git diff --check
```

## Slice 7: Integrated acceptance and guidance

Requirements: DPR01, DPR04; representative acceptance and delivery documentation.
Dependency: Slice 6 integrated and closed.
Branch: `test/rev-delimiter-editing-07-acceptance`
Expected PR title: `test(editor): verify automatic delimiter editing workflows`
Report: `ops/default/report/slices/rev-delimiter-editing/slice-7-acceptance.md`

- T7.1: Create scripts/check-delimiter-editing.sh using existing native test/PTY support. Batch the complete pair suite, editing configuration and the focused shared regressions enumerated below; verify a nonempty selected test set. Add representative native and Linux PTY workflows for both profiles, literal paste/macro behavior, one undo/redo cycle and read-only/focus boundaries, with owned fixtures and restored terminal attributes.
- T7.2: Measure new-service bytes/depth/retention/cooperative scheduling on 1 KiB, 64 KiB, 256 KiB and 1 MiB complete/malformed/oversized-token sources, count expansion and split views. Record bounded refusal and continuation, same-line Org/Markdown frame stability and queued-key release behavior. Keep finite native deadlines and disclose cold-build cost; do not tune frozen limits or expected fixtures to observed output.
- T7.3: Finish the User Guide example, how-to controls/language overrides and exact command/config/reference policy. Normalize the existing Unreleased entry to the repository style, without process narrative. Prepare all seven reports and editing-only spec/model/index closure; leave final gate and post-gate closure publication to the controller.

Required commands:

```sh
cargo fmt --all -- --check
bash scripts/check-delimiter-editing.sh
bash scripts/check-documentation.sh
bash scripts/build-user-guide.sh
git diff --check
```

## Delivery-set acceptance gate

Only the controller runs this combined gate after all seven slices are integrated
into the exact clean, published dev candidate:

```sh
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash -c 'set -euo pipefail; make lint; make test-hooks; bash scripts/check-delimiter-editing.sh; bash scripts/check-documentation.sh; bash scripts/build-user-guide.sh; git diff --check'
```

The new editing script must batch tests/pairs.rs, configuration pairs cases and
focused existing edit/region/count/paste, macro record/replay, modal pair-object
and Org/Markdown projection/continuity regressions. Select exact native filters
in the script with nonzero coverage evidence; do not run catalog_dispatch or
chain all historical acceptance scripts. Reuse the existing factory-shortcut
native helper once to check additive command inventory and unchanged maps.
Use table-driven cases at the lowest sufficient layer. Two representative Linux
PTY journeys cover real physical input, both profiles, saving/reopening exact
source and restoring terminal state. Add further PTYs only for a distinct input
boundary that native tests cannot prove. Isolate HOME/XDG/fixtures, require finite
observable-condition deadlines and preserve all actual user files/settings.

New-service bound cases cover 1 KiB/64 KiB/256 KiB/1 MiB complete/malformed source,
one oversized lexical token, depth 256/257, exhausted retained metadata, stale
revision/cancellation, admitted numeric expansion and multiple views. Assertions
must distinguish limited/uncertain from a confirmed result; never fabricate a
mate or partial pair transaction. Body typing in pretty/indented Org/Markdown
must retain completed-frame presentation continuity and unchanged unrelated bytes.

The prior factory-shortcut gate took about 21 minutes. Budget approximately
15-30 minutes for warm combined acceptance and 25-45 minutes with cold compilation,
subject to the measured report. This is a cost estimate, not an increased retry
allowance. Slice checks avoid that integrated cost; record actual durations and
investigate unexpected multiplication rather than appending historic journeys.
Repository source commits retain the existing full staged-source hook contract.

For a failing candidate, preserve the exact failure and use the native Looper
recovery/repair protocol with candidate-bound authority. Do not alter the frozen
config, counters, fixture expectations, checks or acceptance thresholds, bypass
hooks or start another controller. Correct only this approved editing scope;
material product decisions remain an owner boundary.

## Integration, reports and closure

Every slice has three tracker tasks and one canonical PR/report. Use one
Conventional Commit per task when practical; update the tracker and report with
exact command results and commit/PR identities. Reports follow the required
Purpose, Delivered Behavior, Implementation Notes, Contracts Added or Changed,
Files of Interest, Validation and Risks and Follow-ups sections. PR bodies use
exactly Summary, Changes and Validation; the last Changes item is an immutable
report link. Validation lists executed successful checks only, with required
commands first in canonical order and git diff --check last.

The final tested candidate must already contain all runtime sources, settings,
fixtures, scripts, references, guide/changelog and seven reports. Post-gate
closure may edit only this plan/tracker, the editing specification, the editing
status in the shared model, the operations index and those seven reports, all
Markdown. Set only the editing capability Completed; leave navigation and
presentation Draft. The model remains partially approved, with editing Completed.
Do not change tested runtime bytes or frozen command/requirement identities.
Record the exact tested hash, gate result/receipt and documentation-only closure
relationship. Close all 21 tasks and seven reports only after their real results.

No main promotion, mirror publication, release, global installation or successor
startup is authorized by this delivery. Return the exact run/watch command after
verified controller and worker startup.
