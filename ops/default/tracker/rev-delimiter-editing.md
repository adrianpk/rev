# Automatic Delimiter Editing Tracker

Status: Completed
Completed: 2026-11-15
Date: 2026-11-15
Delivery set: rev-delimiter-editing
Slice strategy: layered
Planning authority: product-owner request for first-capability slicing and Looper, 2026-11-15
Approved: product owner, 2026-11-15; automatic editing first, navigation second, presentation third
Execution authority: slices 1-7, predecessor-ordered canonical PRs and controller-owned Rebase + Fast-forward integration into dev; no successor startup
Planning baseline: `fa48f847e9075eafd2272891c29c7dbf04bf7f16`
Spec: [Delimiter Pairs](../spec/editor-delimiter-pairs.md)
Model: [Pair Preferences and Native State](../spec/editor-delimiter-pairs-model.md)
Active slice: None — Slice 7 of 7 completed; delivery set closed
Integration target: `dev`
Plan: [Automatic Delimiter Editing Delivery](../plan/rev-delimiter-editing.md)

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

## Execution boundary

This is a fresh seven-slice editing run. Configuration, qualification and actual
source behavior follow predecessor integration. Navigation and presentation are
subsequent independent sets. No source change, test result or PR is claimed by
planning approval. The controller owns canonical Rebase + Fast-forward merges,
the one combined final gate and candidate-bound closure. Preserve its run,
conversation, commands, budgets and receipts after startup.

## Slice 1: Configuration and language policies

Requirements: DPR04; editing schema in the shared model.
Dependency: approved committed preparation.
Branch: `feat/rev-delimiter-editing-01-config`
Expected PR title: `feat(editor): configure automatic pair policies`
Report: `ops/default/report/slices/rev-delimiter-editing/slice-1-config.md`
State: delivered; PR #55 integrated
PR: [#55](https://forge.adrianpk.com/rev/rev/pulls/55)
Task commits: T1.1/report introduction `1c02645adf98458878588684db44cb78dcd7ecfa`; T1.2 `178c80b447525e60caba98ab84ee2ef2b30942a5`; T1.3 `1fc8533dd8dc95f2220c0a603e863be49d915538`.
Validation: formatting, 11 pair-policy regressions and whitespace PASS through the exact three-command controller validator at `a5507c375908f499de0a369e6e10314c79d70a0a`; canonical PR contract verified. PR #55 integrated that same commit into dev. Five supplemental template/default regressions and all three task commit hooks passed. The combined delivery gate remains controller-owned.

- [x] T1.1: Extend the native parsed, partial and resolved configuration with only the five editing booleans. Add compiled defaults and per-language leaf inheritance, whole-candidate rejection, origin paths and effective snapshots; preserve absent-sibling inheritance and grammar-independent language resolution.
- [x] T1.2: Connect resolved editing policy to the native buffer/language lifecycle without automatic source edits. Preserve explicit user overrides, shared-buffer ownership and unrelated settings. Add only editing leaves to the generated template and configuration reference; do not accept future presentation leaves.
- [x] T1.3: Add parameterized configuration regressions for defaults, every false override, per-language inheritance/provenance, malformed candidates and language changes; report exact coverage and preserve personal configuration bytes.

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
State: delivered; PR #56 integrated
PR: [#56](https://forge.adrianpk.com/rev/rev/pulls/56)
Task commits: T2.1 and native T2.2/report introduction `cf787fbd636b5f80d2f535b6bc8838b7eef56c1e`; T2.2 measurements and T2.3 regressions `8fe4e91dae6acb74a6bd3a015c0ca5d9250f9488`; shared native-observation budget and boundary regression `956fccb21b8b6b480cb6189a92258f5f0b940c7f`.
Validation: exact three-command controller validator PASS at `ed955dac56d22bb623ccec7b62652255af05632f`, including all 19 qualification tests. Canonical PR contract verified; PR #56 integrated the same commit into dev. All three staged-source hooks and two supplemental legacy modal structural/object regressions PASS. All fixed bounds and measured costs are recorded in the report; combined acceptance remains controller-owned.

- [x] T2.1: Implement one cohesive native owner for language/context classification and strict typed-pair qualification. Reuse current syntax when qualified; use the approved conservative lexical policy otherwise. Retain buffer/revision, native spans, delimiter kind, provenance and explicit uncertain/limited outcomes; do not expose navigation commands or reuse loose group-motion results as mate proof.
- [x] T2.2: Enforce the fixed inspected-byte, nesting, retained-memory and cooperative-work bounds, including oversized tokens, counted-request reuse, cancellation and source replacement. Never retain tree nodes across revisions or scan the whole source on each key/frame. Record actual new-service work, latency and retention measurements against the planning baseline.
- [x] T2.3: Introduce tests/pairs.rs with qualification cases for mixed/nested/malformed and multiline punctuation, strings/comments/escapes, approved language profiles, stale/current syntax and disabled syntax colors, UTF-8/CRLF and every resource boundary. Preserve existing modal percent matching and inner/around objects; provide the slice report.

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
State: delivered; PR #57 integrated
PR: [#57](https://forge.adrianpk.com/rev/rev/pulls/57)
Task commits: T3.1, T3.2 and T3.3/report introduction `5efbfa8a3a7ae1038f95ae2bd53dd1f2573e9b13`; native typing/count admission and its command/metadata were committed together to preserve one complete transaction path.
Validation: formatting, all 17 typing tests, 19 shared qualification regressions, four focused count/skip/modal tests, documentation with 22 reference tests, four drift tests and the native 28-map factory helper PASS. Full staged-source hooks PASS for 1,936 Rust files and 77 Cargo roots; frozen catalog/manifest and personal settings preserved. Exact three-command clean-HEAD validation PASS at `f7faa389bcdb785eec47c33408f9add179148ef1`, including all 17 typing tests. Canonical PR contract verified; PR #57 integrated that same commit into dev. Combined acceptance remains controller-owned.

- [x] T3.1: Route eligible physical source typing in non-modal and modal Insert/Replace through one checked native transaction. Insert admitted structural/quote mates, reuse an existing unmatched closer and skip only an admitted closer. Uncertain ordinary typing stays literal; preserve read-only, focus and modal owners.
- [x] T3.2: Honor native numeric prefixes once: positive counts compose one transaction and undo unit, zero is inert and negative typed repetition retains refusal. Preflight expansion and work limits before publication; generated mates are effects rather than new key events. Wire buffer-local auto-pair-mode shared across views and retained across language changes without writing configuration.
- [x] T3.3: Add the unbound auto-pair-mode catalog entry, named dispatch, descriptions and introspection, plus independent additive command-inventory expectations. Test insertion/count/skip/refusal, undo/redo, mode/focus/toggle and source-policy behavior; add concise reader-facing guidance for the delivered feature using the changelog skill.

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
State: delivered; PR #58 integrated
PR: [#58](https://forge.adrianpk.com/rev/rev/pulls/58)
Task commits: T4.1-T4.3/report introduction `a254aae20d567287160d83da3682fe0e006ca8e1`; deletion, conservative quote proof, native regressions and reader guidance committed together as one complete editing path.
Validation: formatting, all 13 required deletion/quote tests, 19 shared qualification and 17 typing regressions, focused current-syntax/history checks and documentation with 22 reference tests PASS. Full staged-source hooks PASS for 1,940 Rust files and 77 Cargo roots. Personal settings and frozen factory catalog/manifest preserved. Exact three-command clean-HEAD validation PASS at `172fcaeea33367a72f1c98ec09c9281b34d12a86`, including all 13 selected deletion/quote tests. Canonical PR contract verified; PR #58 integrated that same commit into dev. Combined acceptance remains controller-owned.

- [x] T4.1: Integrate Backspace with existing deletion and region precedence. Remove both ends only for an adjacent empty admitted pair, in one undo unit; whitespace/content, disabled policy and uncertain context retain ordinary deletion. Preserve grapheme, line-ending, read-only and history behavior.
- [x] T4.2: Complete the approved quote/profile matrix across ordinary code and prose contexts. Keep word apostrophes, escaped quotes, Rust lifetime/character ambiguity, raw/triple/template/interpolation ambiguity and unproved embedded contexts literal; confirm current-language evidence before automatic quote edits.
- [x] T4.3: Add deletion and quote matrices, including nonempty pairs, malformed source, comments, combining characters, split views and toggles. Verify exact source and cursor/mark restoration through undo/redo and update delivered policy documentation and the slice report.

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
State: delivered; PR #59 integrated
PR: [#59](https://forge.adrianpk.com/rev/rev/pulls/59)
Task commits: T5.1-T5.3/report introduction `b37ef0b1e39b4b1da26969dad97b9d7c8e9d1231`; bounded wrapping, native selection/history publication, explicit helpers and independent regressions were committed together as one complete editing path.
Validation: formatting, all 17 required region tests, the complete 64-case pair suite, the final 34-case typing/region batch, focused boundary/history checks, documentation with 22 reference tests and the native 28-map factory helper PASS. Full staged-source hooks PASS for 1,948 Rust files and 77 Cargo roots. Personal settings and frozen catalog/manifest preserved. Exact three-command clean-HEAD controller validation PASS at `419e84ac43fc213078c6cc806e9dee960001525a`, including all 17 selected region cases. Canonical PR contract verified; PR #59 integrated the same commit into dev. Combined acceptance remains controller-owned.

- [x] T5.1: Implement typed opening/closing wrapping for active ordinary linear selections, including both orientations and positive counts. Preserve selected bytes, deactivate the region and use the approved opener/closer point placement. Reject unsafe quote transformations before mutation; leave rectangular/block owners intact.
- [x] T5.2: Add unbound pair-wrap and pair-unwrap through native catalog/dispatch. Use a cancellable native pair-selection prompt for wrapping, with an admitted opening or closing character selecting its pair and opener placement. Unwrap only the innermost confirmed enclosing pair at point, preserving contents/literal meaning and native point/mark mapping; explicit helpers work with automatic editing disabled.
- [x] T5.3: Add independently specified region/helper/count cases with undo/redo and selection orientation. Extend additive inventory expectations by exactly these two commands, retain every old factory mapping and document invocation, cancellation, eligibility and refusals in the existing guide/reference owners.

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
State: delivered; PR #60 integrated into dev
PR: [#60](https://forge.adrianpk.com/rev/rev/pulls/60)
Task commits: T6.1-T6.3/report introduction `40edb1bce7e68b3d537aff9e6a87b69232887241`; native ingress trace, bounded presentation correction and independent provenance/history regressions were committed together as one complete path.
Validation: formatting, 11 required provenance cases, 20 required macro recording/region/runner cases, three offline AI/plugin/semantic-completion ingress cases, 24 presentation-owner regressions, 15 legacy continuity cases, 10 counter/completion cases and documentation with 22 reference tests PASS. Full staged-source hooks PASS for 1,953 Rust files and 77 Cargo roots. Personal settings and frozen catalog/manifest preserved. Exact four-command clean-HEAD controller validation PASS at `62649d8319a0316af03e77d62a3868ec3a773fea`, including 11 provenance and 20 macro cases. Canonical PR #60 integrated that same candidate; combined acceptance remains controller-owned.

- [x] T6.1: Trace and exercise paste, completion, provider/AI edits, external reload, plugins and programmatic replacements through their real native source-edit ingress. They remain literal and never synthesize pair input. Integrate raw macro typing once while accepted literal macro steps replay literally, retaining existing counts, counters, cancellation and history semantics.
- [x] T6.2: Verify shared-buffer/split-view policy, language changes, source replacement and stale-work retirement. Preserve Org/Markdown pretty/indent source-to-display mapping and completed-frame continuity during body typing; pair editing must not introduce transient literal fallback, cursor relocation or stale spans.
- [x] T6.3: Add native provenance and projection regressions plus targeted existing macro cases. Assert exact bytes and source revision/point/mark/undo behavior at these boundaries, without network providers or changes to their business logic; document remaining physical-input acceptance in the report.

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
State: delivered; PR #61 integrated into dev
PR: [#61](https://forge.adrianpk.com/rev/rev/pulls/61)
Task commits: T7.1-T7.3/report introduction `cb3c09ca63e00d43b3f8a0642e7cbe2627874bae`; native/physical acceptance, measured bounds, guidance and editing-only closure preparation form one complete reviewable path.
Validation: formatting, the acceptance script (192 native cases, all fixed bounds, unchanged maps and two owned/restored Linux PTYs), documentation with 22 reference tests and the local User Guide build PASS. Full staged-source hooks PASS for 1,956 Rust files and 78 Cargo roots. Exact five-command clean-HEAD controller validation PASS at `51e97dceef8db07021e9be2f7eef5cf77f61fdc8` in 369.20 seconds; canonical PR contract verified and PR #61 integrated that same candidate. Combined gate and editing-only closure remain controller-owned.

- [x] T7.1: Create scripts/check-delimiter-editing.sh using existing native test/PTY support. Batch the complete pair suite, editing configuration and the focused shared regressions enumerated below; verify a nonempty selected test set. Add representative native and Linux PTY workflows for both profiles, literal paste/macro behavior, one undo/redo cycle and read-only/focus boundaries, with owned fixtures and restored terminal attributes.
- [x] T7.2: Measure new-service bytes/depth/retention/cooperative scheduling on 1 KiB, 64 KiB, 256 KiB and 1 MiB complete/malformed/oversized-token sources, count expansion and split views. Record bounded refusal and continuation, same-line Org/Markdown frame stability and queued-key release behavior. Keep finite native deadlines and disclose cold-build cost; do not tune frozen limits or expected fixtures to observed output.
- [x] T7.3: Finish the User Guide example, how-to controls/language overrides and exact command/config/reference policy. Normalize the existing Unreleased entry to the repository style, without process narrative. Prepare all seven reports and editing-only spec/model/index closure; leave final gate and post-gate closure publication to the controller.

Required commands:

```sh
cargo fmt --all -- --check
bash scripts/check-delimiter-editing.sh
bash scripts/check-documentation.sh
bash scripts/build-user-guide.sh
git diff --check
```

## Final acceptance and closure

State: Completed; all seven slices integrated and full gate PASS
Required command:

```sh
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash -c 'set -euo pipefail; make lint; make test-hooks; bash scripts/check-delimiter-editing.sh; bash scripts/check-documentation.sh; bash scripts/build-user-guide.sh; git diff --check'
```

Gate preparation verified canonical PRs #55-#61 merged into dev with fetched
ancestry, all seven delivered reports and 21 completed tasks. The final owned
worktree was clean and inactive and was removed without force; unrelated
worktrees remain. Required tools, native/PTY fixtures and shell syntax are ready.
No combined gate ran during preparation.

The plan owns scope, fixed bounds, focused selection, cost and report/closure
rules. The controller runs the combined gate only after Slice 7 integration.
Record its tested candidate, exact successful receipt and any distinct subsequent
Markdown-only closure commit. Source/settings/tests/fixtures must not change
between that tested candidate and ordinary closure. Only editing becomes
Completed; navigation/presentation and their model portions remain Draft.

- [x] All 21 tasks and seven canonical slice PRs delivered in predecessor order.
- [x] Integrated editing/configuration and shared-owner regressions pass.
- [x] Two representative owned Linux PTYs and bounded resource/frame workloads pass.
- [x] User Guide, configuration/command reference and concise Unreleased entry accepted.
- [x] Controller-owned unchanged full gate passes at the exact published dev candidate.
- [x] Seven reports, editing-only specification/model/index and tracker reconciled.
- [x] Integrated clean/inactive owned worktrees removed without force.

Tested candidate: `eb65b227dee8b42bf8d0c8a6e0ef12f5675c756b`
Gate result: PASS
Gate receipt: `QD5ALW2WJR64VCCTK4CJAOZPDK`
Closure: subsequent Markdown-only dev commit; exact closure identity retained in Git and Acta.

The unchanged required command above passed through the controller at that
immutable candidate. The [delivery-set closure](../report/slices/rev-delimiter-editing/slice-7-acceptance.md#delivery-set-closure)
records complete results. All 21 tasks, seven PRs/reports, native and Linux PTY
acceptance, guide/reference/changelog and worktree cleanup are closed. Editing
is Completed; the shared model remains Partially approved and its later
navigation/presentation portions remain Draft. The post-gate diff is restricted
to the twelve configured Markdown closure paths and preserves all tested runtime,
settings, fixtures, scripts, frozen identities and acceptance assertions.
