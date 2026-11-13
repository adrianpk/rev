# Markdown Delivery

Status: Approved
Date: 2026-11-13
Delivery set: rev-markdown
Slice strategy: behavior-first
Approved: product owner, 2026-11-13; approved Markdown specification, slicing and established autonomous Looper workflow; this six-slice map is derived under that authority
Planning baseline: `47c8b2210d8e043640f488e1024ea7c7a2389265`
Spec: [Markdown Editing and Presentation](../spec/editor-markdown-mode.md)
Model: [Markdown Policy and Actions](../spec/editor-markdown-mode-model.md)
Tracker: [Markdown Tracker](../tracker/rev-markdown.md)
Active slice: Slice 5 - Table row and column operations
Readiness: parser dependency accepted after license verification; ready for native preflight/startup
Full gate: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test syntax --test configuration --test commands --test display --test pointer --test org_headings --test org_indentation --test org_tables --test external_changes -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-markdown.sh && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh && git diff --check`

## Outcome, strategy and authority

Six vertical slices add native Markdown incrementally: semantic recognition,
heading presentation, lists, basic tables, table structure and joint acceptance.
Each produces usable passing behavior with one exact branch/PR/report. Shared
source geometry, input/profile precedence, canonical transactions and Org remain
protected; an HTML preview, Org task features and formulas are outside this set.

The owner authorizes preparation commits/push on dev, the recorded slice PRs,
controller-owned Rebase + Fast-forward merges into dev, selected integrated
acceptance and documentation closure. Looper is the single execution owner.
Slice PRs do not wait for separate maintainer merge prompts. Main integration,
release, installed-binary changes and mirror publication are separate workflows.

## Parser decision and licensing

Use exactly `pulldown-cmark = { version = "=0.13.6", default-features = false }`
with `Options::ENABLE_TABLES` only. Upstream provides CommonMark events and source
ranges; keeping ranges permits source-backed edits without reconstructing text
from decoded values. Default-off HTML/CLI features avoid an unused renderer and
its optional dependencies. Pin the resolved transitive versions in Cargo.lock.
The native worker may make one bounded contiguous snapshot per source revision;
this is an explicit allocation tradeoff against incremental Tree-sitter parsing.

Tree-sitter's Markdown grammar fits the existing engine but uses block and inline
parsers and warns that its output is unsuitable where correctness is essential.
Native table mutations need reliable structure, not only plausible highlight
captures. A handwritten CommonMark parser would add a larger correctness owner.
References: [pulldown-cmark API](https://docs.rs/pulldown-cmark/0.13.6/pulldown_cmark/)
and [Tree-sitter Markdown goals](https://github.com/tree-sitter-grammars/tree-sitter-markdown#goals).

The owner accepted this dependency subject to appropriate licensing. Inspection
of the checksum-verified 0.13.6 package confirmed MIT terms, compatible with Rev's
GPL-3.0-only license. The minimal selected graph adds unicase 2.10.0 under its MIT
option; bitflags 2.13.2 and memchr 2.8.3 already exist in the project lockfile.
No optional HTML, getopts or serde feature is required. Preserve existing versions,
lock the two additions and retain their exact upstream notices in a shipped
third-party notice document. Do not change Rev's license.

References: [upstream license](https://github.com/pulldown-cmark/pulldown-cmark/blob/main/LICENSE),
[MIT terms](https://opensource.org/license/mit) and
[GPL compatibility](https://www.gnu.org/licenses/license-list.html#Expat).
Keep the parser private so a separately approved owned implementation can replace
it without changes to source, painting or table-editing contracts. That future
implementation is not another slice or successor run in this delivery.

## Worktree and execution hygiene

Keep the primary checkout on dev for integration and retain installed hooks.
Do not reinstall or bypass hooks. Create task worktrees only under the canonical
project worktree root, using its dated naming contract. Each owns private Cargo
and hook caches; never share mutable targets across branches.

After each merge verify that the old slice HEAD is ancestral to fetched dev,
that its worktree is clean/unlocked/inactive and that ignored content is only
classified generated output. Remove that worktree with non-forced Git removal
before proceeding; preserve branches and all pre-existing worktrees. The next
worker turn or final closure performs this cleanup when the controller has just
integrated the prior PR. Never interrupt a live process to enable removal.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Recognition and colors | `feat/rev-markdown-01-recognition` | `feat(markdown): recognize and color native Markdown` | `ops/default/report/slices/rev-markdown/slice-1-recognition.md` |
| Slice 2 | Heading presentation | `feat/rev-markdown-02-headings` | `feat(markdown): present level-colored heading symbols` | `ops/default/report/slices/rev-markdown/slice-2-headings.md` |
| Slice 3 | List presentation and editing | `feat/rev-markdown-03-lists` | `feat(markdown): present and edit nested lists` | `ops/default/report/slices/rev-markdown/slice-3-lists.md` |
| Slice 4 | Table creation and field editing | `feat/rev-markdown-04-tables` | `feat(markdown): create align and navigate GFM tables` | `ops/default/report/slices/rev-markdown/slice-4-tables.md` |
| Slice 5 | Table row and column operations | `feat/rev-markdown-05-table-structure` | `feat(markdown): edit table rows and columns safely` | `ops/default/report/slices/rev-markdown/slice-5-table-structure.md` |
| Slice 6 | Acceptance and guidance | `feat/rev-markdown-06-acceptance` | `test(markdown): verify native editing and document workflows` | `ops/default/report/slices/rev-markdown/slice-6-acceptance.md` |

## Slice 1: Recognition and colors

Requirements: MD01, MD02 semantic colors, MD06.
Dependency: approved preparation published and native preflight passed.

- T1.1: Implement a private, source-range Markdown observation owner and revision-qualified worker integration; add only the accepted pinned parser and its reviewed minimal transitive graph; retain their upstream notices. Preserve literal blocks, CommonMark context, bounded snapshots and cancellation admission.
- T1.2: Add automatic .md/.markdown recognition, markdown-mode/Text-mode transitions, markdown.enabled and the six semantic palette leaves with full parsing, validation, merge/provenance, sample and configuration catalog coverage. Keep LSP optional and existing modes unaffected.
- T1.3: Create tests/markdown.rs with named source/context/offline/invalidation cases, including native palette output and disabled coloring. Update affected references/Unreleased for the usable behavior and record the canonical report.

Ordered slice checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test syntax --test commands -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
4. `git diff --check`

## Slice 2: Heading presentation

Requirements: MD02, MD05, heading-related MD01/MD06.
Dependency: Slice 1 integrated and closed on dev.

- T2.1: Add the six heading palette leaves and markdown.pretty; project only admitted ATX source prefixes with the exact approved marker/color profile. Color Setext titles while retaining underlines; preserve literal source and canonical width/cursor mappings.
- T2.2: Implement markdown-pretty-mode and markdown-reset-presentation with view-local overrides, split inheritance, buffer switches, explicit source view and read-only presentation. Keep structural recognition independent of color policy.
- T2.3: Cover real terminal cells, wide/combining source, clipped/wrapped prefixes, selection/search precedence, copy/save/history, Text-mode suspension and existing Org projection regressions. Complete references, catalogs and the slice report.

Ordered slice checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test commands --test display --test pointer --test org_headings --test org_indentation -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
4. `git diff --check`

## Slice 3: List presentation and editing

Requirements: MD03 and list-related MD05/MD06.
Dependency: Slice 2 integrated and closed on dev.

- T3.1: Decorate unordered source markers by actual nesting depth using the approved three-marker cycle and the shared view pretty flag. Retain ordered source numbers, continuation paragraphs, indentation and literal-code ownership.
- T3.2: Implement continue-item, empty-item level exit and checked indent/outdent with child/continuation retention, ordered increment and one-action undo. Introduce the Markdown editing keymap overlay and approved Return/Meta-Return dispatch without capturing modal Normal/Visual or completion-popup ownership.
- T3.3: Deliver both-profile named commands, effective key hints, override inheritance, read-only/cancellation cases, Unicode and shared-point regressions. Update catalogs, list guidance, Unreleased and the canonical report.

Ordered slice checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test commands --test display --test pointer -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
5. `git diff --check`

## Slice 4: Table creation and field editing

Requirements: MD04 creation/alignment/navigation, MD05, MD06.
Dependency: Slice 3 integrated and closed on dev.

- T4.1: Observe GFM tables from admitted parser context with original source cell spans, delimiter alignments and the fixed finite table profile. Accept optional outer pipes and escaped pipes; refuse structured operations that would discard extra body cells or depend on stale/incomplete context.
- T4.2: Implement explicit table creation, alignment, next/previous field and next-row actions, skipping delimiters and appending body rows when required. Preserve exact cell source, newline identity and existing canonical transaction authority; aligned navigation adds no history.
- T4.3: Wire contextual table routes in both editing profiles and named selectors. Cover header-only tables, all alignments, literal code, ragged/invalid shapes, read-only refusal, limits and exact undo/redo; document behavior and report acceptance.

Ordered slice checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test commands --test org_tables -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
4. `git diff --check`

## Slice 5: Table row and column operations

Requirements: MD04 structure, shared-point MD05 and MD06.
Dependency: Slice 4 integrated and closed on dev.

- T5.1: Implement body-row and whole-column insertion/deletion with the model-defined placement. Retain the header/delimiter, minimum one column, alignment metadata and all surviving cell text; reject unsupported targets before mutation.
- T5.2: Map retained cell source, initiating point and other view marks/points through the existing transaction owner, including Unicode and escaped/code content. Keep read-only, stale revision, canceled work and failure paths atomic with one operation per undo unit.
- T5.3: Finish structural commands/bindings and discoverability in both profiles; protect inherited remaps, modal state boundaries and Org table behavior. Record table geometry/fidelity regressions, documentation and the canonical report.

Ordered slice checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test commands --test pointer --test org_tables -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
5. `git diff --check`

## Slice 6: Acceptance and guidance

Requirements: Joint MD01-MD06 and the complete reproducibility matrix.
Dependency: Slice 5 integrated and closed on dev.

- T6.1: Add scripts/check-markdown.sh using existing owned native/PTY helpers and one finite fixture matrix. Verify actual cells, source/pretty views, both profiles, NO_COLOR, BOM/CRLF, Unicode, copy/save, list/table commands and terminal restoration with owned children and temporary roots.
- T6.2: Complete the Markdown reference and User Guide lesson, exact command/configuration/binding catalogs, packaged sample, learning-path navigation, Spore projection and Unreleased. Keep canonical Markdown as the documentation source and retain all existing guide/reference assertions.
- T6.3: Record native/PTY evidence and complete the report. Prepare the clean published integrated candidate for the controller-owned full gate; after success close only configured Markdown artifacts and remove this set's clean inactive integrated worktrees.

Ordered slice checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test commands -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-markdown.sh`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
6. `git diff --check`

## Integrated gate and cost

Run the exact Full gate above once on the clean published candidate after all
six PRs integrate. The strict lint command protects every owned workspace root;
the selected native batch protects Markdown plus shared syntax, configuration,
commands, geometry, pointers, Org heading/table behavior and external reload.
The modal subset protects contextual input ownership. The owned executable
journey protects terminal behavior; catalogs and guide projection protect docs.
Do not append unrelated Git/plugin/LSP/install or historical exhaustive gates.

Expect several minutes for warm validation and a cold private Cargo compile in
each new worktree; the combined gate may take roughly 5-15 minutes warm. This is
a planning estimate, not a timeout or acceptance waiver. Use one build job,
disabled incremental output and no dev/test debuginfo exactly as frozen above.
Measure actual cost. Do not loosen assertions, skip cases or extend timeouts to
hide failures. The final slice's focused checks do not run the combined gate.

A repository correction after gate failure uses fix/rev-markdown-validation and
one canonical PR, with focused verification followed by the unchanged full gate.
No repair allowance, new dependency, budget expansion or unrelated path grant is
created by a worker; preserve the native run and its explicit authority stops.

## Closure

Only after a successful exact-candidate gate, mark the spec, model, plan and
tracker Completed, record actual PR/commit/receipt evidence in the same reports,
and reconcile this capability's operational-index row. Remove this set's final
clean inactive integrated worktree; preserve unrelated work and installed copies.
Closure may change only the configured exact Markdown artifacts: these four
spec/model/plan/tracker documents, the operational index and six slice reports.
Runtime, tests, manifests, guide inputs and executable drivers retain tested bytes.
Verify clean synchronized dev and record the final closure commit and Acta event.
