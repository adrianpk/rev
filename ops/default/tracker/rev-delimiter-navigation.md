# Delimiter Navigation Tracker

Status: Approved
Date: 2026-11-15
Delivery set: rev-delimiter-navigation
Slice strategy: behavior-first
Planning authority: product-owner continuation to the next ordered delivery set, 2026-11-15
Approved: product owner, 2026-11-15; navigation after accepted automatic editing
Execution authority: slices 1-5 and controller-owned predecessor-ordered Rebase + Fast-forward integration into dev; main promotion remains separate
Planning baseline: `0a05c82da5babc23f2fcd58176e35b99823b6c3a`
Spec: [Delimiter Navigation](../spec/editor-delimiter-navigation.md)
Model: [Pair Preferences and Native State](../spec/editor-delimiter-pairs-model.md)
Concern: [Operations index](../README.md)
Concern: [Interaction Parity](../spec/editor-emacs-interaction-parity.md)
Active slice: None — all five slices integrated; awaiting controller gate
Gate state: prerequisites prepared; controller execution pending
Integration target: `dev`
Plan: [Delimiter Navigation Delivery](../plan/rev-delimiter-navigation.md)

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Confirmed mate jump | `feat/rev-delimiter-navigation-01-mate` | `feat(editor): jump to confirmed matching delimiters` | `ops/default/report/slices/rev-delimiter-navigation/slice-1-mate.md` |
| Slice 2 | Forward and backward groups | `feat/rev-delimiter-navigation-02-groups` | `feat(editor): navigate forward and backward across groups` | `ops/default/report/slices/rev-delimiter-navigation/slice-2-groups.md` |
| Slice 3 | Enclosing and nested groups | `feat/rev-delimiter-navigation-03-nesting` | `feat(editor): move out of and into nested groups` | `ops/default/report/slices/rev-delimiter-navigation/slice-3-nesting.md` |
| Slice 4 | Syntax-class expression motions | `feat/rev-delimiter-navigation-04-expressions` | `feat(editor): navigate syntax-aware expression boundaries` | `ops/default/report/slices/rev-delimiter-navigation/slice-4-expressions.md` |
| Slice 5 | Integrated navigation acceptance | `test/rev-delimiter-navigation-05-acceptance` | `test(editor): verify delimiter navigation workflows` | `ops/default/report/slices/rev-delimiter-navigation/slice-5-acceptance.md` |

## Execution boundary

Five new behavior-first slices follow the completed and manually accepted editing
set. No navigation implementation or result is claimed by planning approval.
The controller integrates canonical slice PRs into dev in predecessor order,
then owns one combined gate and exact-candidate closure. Preserve the fresh
run's configuration, conversation, budgets, source/acceptance identities and
historical receipts after startup. Presentation is a separate subsequent set.

## Slice 1: Confirmed mate jump

Requirements: DPR02; shared DPR04 qualification and navigation lifecycle.
Dependency: approved committed preparation and completed editing delivery.
Branch: `feat/rev-delimiter-navigation-01-mate`
Expected PR title: `feat(editor): jump to confirmed matching delimiters`
Report: `ops/default/report/slices/rev-delimiter-navigation/slice-1-mate.md`
State: delivered; PR #62 integrated
PR: [#62](https://forge.adrianpk.com/rev/rev/pulls/62)
Task commits: T1.1/T1.2 `1c3491110d73603a21b0b9a4a792bcc7d4fe7d3a`; T1.3/report introduction `a089ebeac4f55919cff7fae1820c79649e826f16`.
Validation: all three ordered required commands PASS through looper validate at `295aa89a6ac9db401784fbdd972d1624216a392d`, integrated by canonical PR #62. Native invalidation, independent maps/catalog, documentation and both staged-source hooks PASS. Fetched ancestry and clean inactive owned worktree verified; worktree removed without force. The combined gate remains controller-owned.

- [x] T1.1: Add a bounded typed-mate motion through the existing buffer request and session navigation owners. Prefer a delimiter at point, then immediately before; use confirmed structural/quote endpoints and preserve strict type matching. Do not search later in the line, infer raw-string proof or expose group results as typed mates.
- [x] T1.2: Publish matching-delimiter as one unbound native command, with named dispatch, descriptions and truthful hints. It works on read-only source and with automatic pairs or quote editing disabled. Use native unsupported-prefix admission rather than repeating a mate jump. Capture source/revision, view, point/mark and focus; support native cancellation, shared request budgets and safe source-jump publication.
- [x] T1.3: Create tests/pair_navigation.rs with mate cases from the independent approved contract: both endpoints and round trips, immediate-before/adjacent precedence, malformed/literal/stale/limited refusal, quotes, Unicode/CRLF, active linear mark and source/history preservation. Extend the independent catalog expectation from 710 to 711 without changing any factory mapping; report actual focused results.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation mate -- --test-threads=1
git diff --check
```

## Slice 2: Forward and backward groups

Requirements: DPN01; group result, signed counts and admitted syntax classes.
Dependency: Slice 1 integrated and closed.
Branch: `feat/rev-delimiter-navigation-02-groups`
Expected PR title: `feat(editor): navigate forward and backward across groups`
Report: `ops/default/report/slices/rev-delimiter-navigation/slice-2-groups.md`
State: delivered; PR #63 integrated
PR: [#63](https://forge.adrianpk.com/rev/rev/pulls/63)
Task commits: T2.1/T2.2 `1d82c5099ed24fcf35d3670e4ce6c7693fc98894`; T2.3/report introduction `2e4a46f09e466698218fdb3ec8900d31bfd108bf`.
Validation: all three ordered required commands PASS through looper validate at `318a65e4030c857b9457db265d46cc447f3a51aa`, integrated by canonical PR #63. Native navigation (25 tests), invalidation (2), shared qualification (21), modal percent/objects (2), independent maps/catalog, documentation and both staged-source hooks PASS. Fetched ancestry and clean inactive owned worktree verified; worktree removed without force. Combined acceptance remains controller-owned.

- [x] T2.1: Implement forward-pair and backward-pair with the approved mode/group syntax policy, distinct from strict typed mate validation. Traverse complete groups, skip declared comments/strings and preserve the no-next-group case when starting inside an atom without a later group at the current nesting level. Keep mismatched-kind grouping separate from typed proof.
- [x] T2.2: Apply native signed counts once in one captured request; zero is inert, negative reverses direction and all iterations share byte/depth/memory/work admission. Accept normal reference file boundaries while refusing unproved or malformed destinations atomically. Preserve active linear mark and native grapheme boundaries, without source edits or new undo entries.
- [x] T2.3: Add the two canonical commands and C-M-n/C-M-p base-editor defaults, inherited only through existing map rules. Update independent expectations to 713 commands plus exactly these two base leaves. Test reference endpoints, counts, mode lexical exclusions, region/read-only behavior, custom overrides and preserved modal percent/objects; update delivered guidance and the report.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation groups -- --test-threads=1
git diff --check
```

## Slice 3: Enclosing and nested groups

Requirements: DPN01; enclosing/nested movement and reverse directions.
Dependency: Slice 2 integrated and closed.
Branch: `feat/rev-delimiter-navigation-03-nesting`
Expected PR title: `feat(editor): move out of and into nested groups`
Report: `ops/default/report/slices/rev-delimiter-navigation/slice-3-nesting.md`
State: delivered; PR #64 integrated
PR: [#64](https://forge.adrianpk.com/rev/rev/pulls/64)
Task commits: T3.1/T3.2 `b49dd9aa818d5242e9524ee57f3e6ec2eb181884`; T3.3/report introduction `51a95aad55804ebd8f5564a922d9c4f71ce96cc2`.
Validation: all three ordered required commands PASS through looper validate at `3fd22cbbb24f56adecf57b8dade7b242b38f1da4`, integrated by canonical PR #64. Nesting (10 tests), native invalidation (3), across/mate regressions (15), modal percent/objects (2), independent maps/catalog, documentation and both staged-source hooks PASS. Fetched ancestry and clean inactive owned worktree verified; worktree removed without force. Combined acceptance remains controller-owned.

- [x] T3.1: Implement up-pair as backward-up-list and down-pair as down-list under the approved profile, including their signed-count reverse directions. Distinguish leaving an enclosing group, entering the next nested group and top/bottom-level refusal; do not replace them with next/previous delimiter search.
- [x] T3.2: Reuse the qualified shared request, bounded continuation and native cursor/region owner. Preserve enclosing-source authority, readonly use, cancellation, source replacement and two-view ownership. Reveal folded source through the existing view path without modifying source or independent peer folds; rectangular/modal selection remains native.
- [x] T3.3: Add up-pair/down-pair and C-M-u/C-M-d base defaults. Extend independent expectations to 715 commands and the four authorized base leaves. Test nested and mixed groups, opener/closer/before/after positions, both directions, counts, ordinary boundaries and retained point/mark on refusal; provide the report and current command/binding references.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation nesting -- --test-threads=1
git diff --check
```

## Slice 4: Syntax-class expression motions

Requirements: DPN02; EIP04 expression portion, with paragraph scope retained.
Dependency: Slice 3 integrated and closed.
Branch: `feat/rev-delimiter-navigation-04-expressions`
Expected PR title: `feat(editor): navigate syntax-aware expression boundaries`
Report: `ops/default/report/slices/rev-delimiter-navigation/slice-4-expressions.md`
State: delivered; PR #65 integrated
PR: [#65](https://forge.adrianpk.com/rev/rev/pulls/65)
Task commits: T4.1/T4.2 `f918eb682440e27ca4f0dff6b50de5a1429c6045`; T4.3/report introduction `b368dcae92539b730abde92f7099d9383fa575f8`.
Validation: all three ordered required commands PASS through looper validate at `1d078b5599bdb46bb8bd65c507a4ea13f4c436f7`, integrated by canonical PR #65. Expressions (11 tests), native invalidation (4), preserved navigation (25), modal percent/objects (2), independent maps/catalog, documentation and both staged-source hooks PASS. Fetched ancestry and clean inactive owned worktree verified; worktree removed without force. The combined gate remains controller-owned.

- [x] T4.1: Implement forward-sexp/backward-sexp across the approved profile-specific atoms, quoted strings and complete forms. Use admitted syntax/context and reference syntax-class endpoints; a parser node, number token, group-only target or mate position cannot substitute for the declared expression boundary. Preserve word/symbol, punctuation-prefix and language quote distinctions from the independent cases.
- [x] T4.2: Complete mode-specific literal/comment and boundary handling, including ordinary known unterminated-string EOF results where the approved reference admits them. Keep unproved raw/triple/template/interpolation and embedded states conservative. Apply counted cancellation/refusal and native grapheme normalization without publishing stale/intermediate positions, changing source/history or fabricating pair proof.
- [x] T4.3: Add forward-sexp/backward-sexp and C-M-f/C-M-b base defaults. Extend independent expectations to 717 commands and exactly six authorized base leaves. Test exact expression/group/mate differences, strings/escapes/comments, atom classes, signed counts, active mark, Unicode/CRLF and read-only source. Reconcile only EIP04 expression progress, update guide/reference/changelog through its skill and provide the report.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation expressions -- --test-threads=1
git diff --check
```

## Slice 5: Integrated navigation acceptance

Requirements: DPR02, DPN01, DPN02; shared-owner and physical-input acceptance.
Dependency: Slice 4 integrated and closed.
Branch: `test/rev-delimiter-navigation-05-acceptance`
Expected PR title: `test(editor): verify delimiter navigation workflows`
Report: `ops/default/report/slices/rev-delimiter-navigation/slice-5-acceptance.md`
State: delivered; PR #66 integrated
PR: [#66](https://forge.adrianpk.com/rev/rev/pulls/66)
Task commits: T5.1/T5.2 `2c67bdfdbd9baabedaf0d16ee4a698a6744fbe81`; additive pair inventory reconciliation `263bde3ffe17c1dabadd2bbf72f353de1213c117`; T5.3/report introduction `7006dae9907357d3417ef13ce24876614c6651e7`.
Validation: all five ordered required commands PASS through looper validate at clean, pushed `74927143e7ad04b5ec594e4b7d6ddddcae0395d4` in 467.503 seconds, integrated by canonical PR #66. The script covers 180 native cases, one independent complete factory helper and exactly two Linux PTYs; documentation and local guide build pass. Both code commits passed exact staged workspace formatting, strict all-target/all-feature Clippy and native policy over 1,971 files / 80 Cargo roots. Fetched ancestry and clean inactive owned worktree verified; worktree removed without force. The combined gate and navigation closure remain controller-owned.

- [x] T5.1: Create scripts/check-delimiter-navigation.sh with nonempty native filter checks. Batch the complete navigation suite, the shared pair qualifier/editing regressions justified below, focused native modal objects, keymap/override and macro cases. Reuse the current factory acceptance helper once for complete expected maps/catalog; never regenerate historical frozen inputs from runtime output.
- [x] T5.2: Add exactly two representative owned Linux PTY journeys, non-modal and modal Insert, using real C-M input and named mate invocation. Verify endpoints through subsequent saved source effects, mark/read-only/focus behavior and configuration immutability; restore TTY and reap owned children. Measure aggregate budget/depth/retention, cancellation, source/view changes, queued-key release and folded Org/Markdown presentation with finite deadlines.
- [x] T5.3: Finish the user-facing navigation example, reference endpoints/counts/context/normalization, effective bindings, M-x fallback and concise Unreleased entry. Prepare all five reports, navigation-only spec/model/index and EIP04 expression progress before final integration. The controller owns the combined gate and Markdown-only closure; presentation and paragraph parity retain their separate boundaries.

Required commands:

```sh
cargo fmt --all -- --check
bash scripts/check-delimiter-navigation.sh
bash scripts/check-documentation.sh
bash scripts/build-user-guide.sh
git diff --check
```

## Final acceptance and closure

State: prepared; all five slices integrated, controller full gate pending
Required command:

```sh
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash -c 'set -euo pipefail; make lint; make test-hooks; bash scripts/check-delimiter-navigation.sh; bash scripts/check-documentation.sh; bash scripts/build-user-guide.sh; git diff --check'
```

The plan owns the fixed profile/resource contract, proportional regression
selection, expected cost and report/closure rules. The controller owns final
execution and receipt for the exact published dev candidate; preparation and
focused results are not that receipt.

All approved full-gate inputs and commands are present. Required executables are
available, frozen input digests are unchanged, and integrated owned worktrees
are retired. Gate preparation changes only delivery-state Markdown; navigation
remains Approved until the controller records the combined gate and closure.

- [x] All 15 tasks and five canonical PRs delivered in predecessor order.
- [ ] Navigation and shared pair/editing, modal, keymap, macro and projection cases pass.
- [ ] Two representative owned Linux PTYs restore TTY and preserve actual configuration.
- [ ] Bounded-work/cancellation/held-input and source/region/frame contracts pass.
- [ ] User Guide, exact bindings/commands/profile reference and concise Unreleased entry accepted.
- [ ] Unchanged controller full gate passes at the exact candidate; tested hash/receipt recorded.
- [ ] Eleven allowed Markdown records close navigation and only EIP04 expression progress.
- [x] Completed clean/inactive owned worktrees removed without force.
