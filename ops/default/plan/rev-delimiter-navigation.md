# Delimiter Navigation Delivery

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
Active slice: Slice 5 — Integrated navigation acceptance
Integration target: `dev`
Tracker: [Delimiter Navigation Tracker](../tracker/rev-delimiter-navigation.md)

## Outcome and strategy

Deliver an explicit confirmed-mate jump, four structural group motions and two
expression motions. The approved specification fixes the seven names, six
factory chords, entry/endpoints, mode admission, counts and source/region bounds.
Behavior-first slicing exposes each complete native operation with its own
catalog, routing, validation and guidance rather than committing public stubs.
The final slice supplies representative joint acceptance and documentation.

Automatic editing is Completed and manually accepted. Its existing source,
history, literal/macro, quotation and resource contracts remain protected.
Navigation does not depend on automatic pairing being enabled or source being
writable. Presentation/depth colors and EIP04 paragraph motion retain separate
approval and delivery scope; this set discharges only the expression portion.
No dependency, grammar package or language server is added.

## Entry, integration and worktree ownership

Publish the approved specification/model portion, this plan, matching tracker
and independent reference cases before a formal runtime branch. Each slice
starts only after its predecessor integrates and closes. Looper owns canonical
slice PR integration into dev after the frozen checks pass, as clarified by the
owner. This is not authority to promote main or start the presentation successor.
Do not reuse the completed editing run, conversation, budgets or receipts.

Keep the primary checkout on dev. Task worktrees use the canonical dated project
worktree root; every checkout owns exclusive Cargo and hook caches. Use one build
job with incremental builds and dev/test debuginfo disabled. Preserve installed
hooks; do not reinstall, bypass or weaken them. Do not interrupt an existing
editor, build, gate or controller. After each integration, verify fetched ancestry,
cleanliness and absence of active processes; remove the integrated worktree
without force in the same task before its successor. Preserve unrelated worktrees
and doubtful/dirty data.

No project ADR index exists. Apply the native architecture contracts: canonical
buffer/revision and lexical proof belong to the pair/context owner; session
routes captured input; coordinates/view own source-to-grapheme/fold projection.
Use native group/expression outcomes distinct from typed Confirmed pair facts.
Do not introduce parallel mutable caches, a broad parser framework or new timers.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Confirmed mate jump | `feat/rev-delimiter-navigation-01-mate` | `feat(editor): jump to confirmed matching delimiters` | `ops/default/report/slices/rev-delimiter-navigation/slice-1-mate.md` |
| Slice 2 | Forward and backward groups | `feat/rev-delimiter-navigation-02-groups` | `feat(editor): navigate forward and backward across groups` | `ops/default/report/slices/rev-delimiter-navigation/slice-2-groups.md` |
| Slice 3 | Enclosing and nested groups | `feat/rev-delimiter-navigation-03-nesting` | `feat(editor): move out of and into nested groups` | `ops/default/report/slices/rev-delimiter-navigation/slice-3-nesting.md` |
| Slice 4 | Syntax-class expression motions | `feat/rev-delimiter-navigation-04-expressions` | `feat(editor): navigate syntax-aware expression boundaries` | `ops/default/report/slices/rev-delimiter-navigation/slice-4-expressions.md` |
| Slice 5 | Integrated navigation acceptance | `test/rev-delimiter-navigation-05-acceptance` | `test(editor): verify delimiter navigation workflows` | `ops/default/report/slices/rev-delimiter-navigation/slice-5-acceptance.md` |

## Frozen behavior, reference and keymaps

The [specification](../spec/editor-delimiter-navigation.md) owns the approved
profile and context matrix. Its defaults add C-M-n/p, C-M-u/d and C-M-f/b to the
base editor table in their behavior-owning slices. Existing default inheritance
propagates them to Insert/Replace and source overlays; Normal/Visual filtering,
pending modal operators and percent matching retain their native rules.
matching-delimiter remains unbound and has native non-counted argument admission.
Explicit user maps retain their existing precedence; effective M-x hints describe
the selected profile/context rather than merely the base table.

At preparation there are 710 commands: the original frozen 707 plus three
editing helpers. Expected totals after Slices 1-4 are 711, 713, 715 and 717.
The independent factory acceptance helper must preserve all original names,
actions and maps, plus the delivered editing commands. Add only each slice's
literal expected commands and authorized base-editor leaves to its independent
expected builder; do not derive expectations from Command::ALL or live defaults.
Preserve catalog.json, mapping-proposal.json, their hashes and old receipts.
The six new base entries are the only approved shortcut changes in this set.

The [82-case reference matrix](../report/rev-delimiter-navigation/reference-navigation.json)
and [batch probe](../report/rev-delimiter-navigation/reference-navigation.el) record GNU Emacs 31.1
without personal init. They cover modes, forms/atoms/strings/comments, mixed and
malformed groups, counts, ordinary boundaries, Unicode/CRLF, active/empty mark
and readonly source. They are independent observations, not Rev acceptance.
The [native mate cases](../report/rev-delimiter-navigation/mate-contract.json) freeze Rev's distinct
direct-entry policy. Stock mode-specific observations are not universal rules:
use the approved profile mapping, including declared conservative native fallback
where a corresponding stock grammar/package is absent. Preserve observation
bytes; add a separately identified case only for a justified uncovered contract.

## Measured starting work and fixed limits

The completed buffer pair qualifier was measured through its public native API
in an optimized library at the planning baseline. Each request used a fresh
buffer, with allocation outside timing; each size has 11 samples of a complete
outer parenthesis and one long atom. This measures existing qualification,
not the future group/sexp engine or a physical-input latency guarantee.

| Source bytes | Median / maximum request microseconds | Maximum poll microseconds | Inspected bytes | Maximum retained bytes | Outcome |
| --- | --- | --- | --- | --- | --- |
| 1,024 | 61 / 72 | 22 | 1,024 | 27,632 | confirmed |
| 65,536 | 3,738 / 4,175 | 32 | 65,536 | 27,632 | confirmed |
| 262,144 | 15,211 / 16,964 | 35 | 262,144 | 27,632 | confirmed |
| 1,048,576 | 15,182 / 16,547 | 32 | 262,144 | 19,440 | limited: byte cap |

Navigation retains the existing 262,144-byte cumulative inspection, depth 256,
2,097,152-byte retained-metadata/buffer and 256-step cooperative-poll caps, with
the native four-millisecond scheduling target. Native count magnitude remains at
most 1,000,000; an admitted count shares one work budget and may exhaust earlier.
Count is not permission to rescan the full file per iteration or allocate a
per-count result vector. Keep one bounded destination/state and qualified facts
under the buffer owner, not copies per view. Cancellation, source/view/focus/mark
changes retire stale work before publication. No permanent idle worker is added.

Latency, work and retained-state measurements for the new engine are required
in its owning slices and integrated acceptance. Assert hard byte/depth/retention
limits and absence of partial/stale publication; report actual poll/total/input
latency rather than changing a bound or extending retries to fit a test.
Use 1 KiB/64 KiB/256 KiB/1 MiB complete and malformed sources, oversized atoms/
strings, depth 256/257, large counts and split views. The 1 MiB byte-cap result
is intentional safe refusal, not a fabricated mate or cursor target.

## Slice tasks and focused checks

Commands below are exact plain shell strings for native Looper configuration.
PR Validation lists exactly these required commands in order, with
git diff --check last. Supplemental successful focused checks belong in the
slice report, not inserted into this frozen PR command list. Config command
values contain no Markdown backticks.

## Slice 1: Confirmed mate jump

Requirements: DPR02; shared DPR04 qualification and navigation lifecycle.
Dependency: approved committed preparation and completed editing delivery.
Branch: `feat/rev-delimiter-navigation-01-mate`
Expected PR title: `feat(editor): jump to confirmed matching delimiters`
Report: `ops/default/report/slices/rev-delimiter-navigation/slice-1-mate.md`

- T1.1: Add a bounded typed-mate motion through the existing buffer request and session navigation owners. Prefer a delimiter at point, then immediately before; use confirmed structural/quote endpoints and preserve strict type matching. Do not search later in the line, infer raw-string proof or expose group results as typed mates.
- T1.2: Publish matching-delimiter as one unbound native command, with named dispatch, descriptions and truthful hints. It works on read-only source and with automatic pairs or quote editing disabled. Use native unsupported-prefix admission rather than repeating a mate jump. Capture source/revision, view, point/mark and focus; support native cancellation, shared request budgets and safe source-jump publication.
- T1.3: Create tests/pair_navigation.rs with mate cases from the independent approved contract: both endpoints and round trips, immediate-before/adjacent precedence, malformed/literal/stale/limited refusal, quotes, Unicode/CRLF, active linear mark and source/history preservation. Extend the independent catalog expectation from 710 to 711 without changing any factory mapping; report actual focused results.

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

- T2.1: Implement forward-pair and backward-pair with the approved mode/group syntax policy, distinct from strict typed mate validation. Traverse complete groups, skip declared comments/strings and preserve the no-next-group case when starting inside an atom without a later group at the current nesting level. Keep mismatched-kind grouping separate from typed proof.
- T2.2: Apply native signed counts once in one captured request; zero is inert, negative reverses direction and all iterations share byte/depth/memory/work admission. Accept normal reference file boundaries while refusing unproved or malformed destinations atomically. Preserve active linear mark and native grapheme boundaries, without source edits or new undo entries.
- T2.3: Add the two canonical commands and C-M-n/C-M-p base-editor defaults, inherited only through existing map rules. Update independent expectations to 713 commands plus exactly these two base leaves. Test reference endpoints, counts, mode lexical exclusions, region/read-only behavior, custom overrides and preserved modal percent/objects; update delivered guidance and the report.

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

- T3.1: Implement up-pair as backward-up-list and down-pair as down-list under the approved profile, including their signed-count reverse directions. Distinguish leaving an enclosing group, entering the next nested group and top/bottom-level refusal; do not replace them with next/previous delimiter search.
- T3.2: Reuse the qualified shared request, bounded continuation and native cursor/region owner. Preserve enclosing-source authority, readonly use, cancellation, source replacement and two-view ownership. Reveal folded source through the existing view path without modifying source or independent peer folds; rectangular/modal selection remains native.
- T3.3: Add up-pair/down-pair and C-M-u/C-M-d base defaults. Extend independent expectations to 715 commands and the four authorized base leaves. Test nested and mixed groups, opener/closer/before/after positions, both directions, counts, ordinary boundaries and retained point/mark on refusal; provide the report and current command/binding references.

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

- T4.1: Implement forward-sexp/backward-sexp across the approved profile-specific atoms, quoted strings and complete forms. Use admitted syntax/context and reference syntax-class endpoints; a parser node, number token, group-only target or mate position cannot substitute for the declared expression boundary. Preserve word/symbol, punctuation-prefix and language quote distinctions from the independent cases.
- T4.2: Complete mode-specific literal/comment and boundary handling, including ordinary known unterminated-string EOF results where the approved reference admits them. Keep unproved raw/triple/template/interpolation and embedded states conservative. Apply counted cancellation/refusal and native grapheme normalization without publishing stale/intermediate positions, changing source/history or fabricating pair proof.
- T4.3: Add forward-sexp/backward-sexp and C-M-f/C-M-b base defaults. Extend independent expectations to 717 commands and exactly six authorized base leaves. Test exact expression/group/mate differences, strings/escapes/comments, atom classes, signed counts, active mark, Unicode/CRLF and read-only source. Reconcile only EIP04 expression progress, update guide/reference/changelog through its skill and provide the report.

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

- T5.1: Create scripts/check-delimiter-navigation.sh with nonempty native filter checks. Batch the complete navigation suite, the shared pair qualifier/editing regressions justified below, focused native modal objects, keymap/override and macro cases. Reuse the current factory acceptance helper once for complete expected maps/catalog; never regenerate historical frozen inputs from runtime output.
- T5.2: Add exactly two representative owned Linux PTY journeys, non-modal and modal Insert, using real C-M input and named mate invocation. Verify endpoints through subsequent saved source effects, mark/read-only/focus behavior and configuration immutability; restore TTY and reap owned children. Measure aggregate budget/depth/retention, cancellation, source/view changes, queued-key release and folded Org/Markdown presentation with finite deadlines.
- T5.3: Finish the user-facing navigation example, reference endpoints/counts/context/normalization, effective bindings, M-x fallback and concise Unreleased entry. Prepare all five reports, navigation-only spec/model/index and EIP04 expression progress before final integration. The controller owns the combined gate and Markdown-only closure; presentation and paragraph parity retain their separate boundaries.

Required commands:

```sh
cargo fmt --all -- --check
bash scripts/check-delimiter-navigation.sh
bash scripts/check-documentation.sh
bash scripts/build-user-guide.sh
git diff --check
```

## Controller-owned delivery gate

After all five slices integrate, run the unchanged combined gate once against
the exact clean, published dev candidate:

```sh
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash -c 'set -euo pipefail; make lint; make test-hooks; bash scripts/check-delimiter-navigation.sh; bash scripts/check-documentation.sh; bash scripts/build-user-guide.sh; git diff --check'
```

The navigation script batches tests/pair_navigation.rs and the complete existing
pair suite, because shared lexical/request changes can affect editing admission,
quotation, counts and undo/literal provenance. Add focused configuration keymap,
native modal structural/object and macro motion/count/region regressions, plus
Org/Markdown folded-source/reveal and frame continuity cases. Select nonzero
filters explicitly. Do not execute all command/catalog-dispatch tests, chain
historical feature scripts or multiply every native case into a PTY.

Reuse the native factory helper once for complete accepted maps/catalog after
the explicit additive updates. Two owned Linux PTYs cover real C-M control/meta
bytes, source-relative endpoint effects, named invocation, effective profile
hints, focus, save/reopen and restored terminal state. Native cases cover the
larger mode/count/region/error matrix at lower cost. Support enhanced decoded
key variants where the existing parser admits them; do not replace terminal
input protocol or hide a platform interception behind a misleading hint.

Verify navigation with automatic editing disabled and read-only source. Check
rapid/held input, bounded cancellation, source/view replacement, active linear
regions, normal file boundaries, real malformed refusal, resource limits and
source/undo immutability. Org/Markdown source jumps reveal folded targets through
the native view without changing peer folds, source bytes or completed-frame
heading presentation. Rectangular/modal grammar and external UI focus retain
current owners.

The prior editing combined gate passed in 604 seconds. Expect roughly 10-20
minutes warm and 20-35 minutes with cold compilation for this selected gate,
with one build job; record actual costs. Per-slice focused tests avoid repeating
combined native/PTY/docs acceptance. Existing complete staged-source hooks still
apply to source commits. A long unexpected wait is investigated from its actual
owner/tool phase; it is not grounds to interrupt work or expand retries.

On failure preserve the exact candidate, diagnostics and frozen gate. Follow
native candidate-bound repair/retry authority for this set; no worker changes
config, budgets, assertions or limits, bypasses hooks, starts another controller
or invents an extra numbered slice. Material behavior changes return for review.

## Reports, integration and closure

Each slice has three tasks, one exact branch, one PR into dev and one concise
canonical report. Prefer one Conventional Commit per task. Reports use Purpose,
Delivered Behavior, Implementation Notes, Contracts Added or Changed, Files of
Interest, Validation and Risks and Follow-ups, with actual command outcomes,
commit/PR identities and measured limits. PR bodies have exactly Summary, Changes
and Validation; the last Changes item links the immutable report-introduction
commit. List executed successful validation commands only in their required order.

All runtime sources, scripts, fixtures, mappings, guide/changelog and five reports
must exist before the full-gate candidate. Ordinary post-gate closure may change
only this plan/tracker, the navigation spec, navigation status in the shared model,
the operations index, EIP04 expression progress in Interaction Parity, and the five
reports: eleven Markdown files. Editing stays Completed, navigation becomes
Completed, presentation stays Draft and the model stays Partially approved.
The broad parity spec and paragraph portion are not marked Completed.

Record the tested hash/receipt and the separate Markdown-only closure relationship.
No tested runtime/fixture/input bytes, required command strings or requirement
identities change during closure. Close all 15 tasks and five reports only after
their actual integration/evidence. No main promotion, mirror publication, release,
global installation or successor startup is part of this run.
