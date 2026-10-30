# Documentation and v0.1.0 Preparation

Status: In progress
Date: 2026-10-30
Delivery set: rev-documentation
Slice strategy: behavior-first
Planning baseline: `0301d34d9f2d43e3ad5247fa0671a7dbaf65fe7b`
Target version: `0.1.0`
Approved: product owner, 2026-10-30; one autonomous Looper cycle for all three slices, including controller-owned Rebase + Fast-forward merges into dev
Active slice: Slice 2 - Independent Org documentation
Tracker: [Documentation Tracker](../tracker/rev-documentation.md)

## Outcome

Readers can learn the everyday editor without passing through Org material.
Org has its own learning path and reference entry points. The documentation
and v0.1.0 changelog describe what this version actually delivers, with simple
examples and complete supporting references.

Use three slices. Each delivers one reader-visible outcome with its navigation,
links and affected checks intact. Do not introduce separate inventory, skeleton,
formatting or controller slices. The inventory below is planning evidence.

The product owner approved this three-slice plan and clarified that Looper
owns the tracker and PR integration. Execute the ordered set in one autonomous
Looper cycle with one branch and PR to `dev` per slice. The controller validates
and merges each PR with Rebase + Fast-forward; do not wait for individual human
PR approvals. Release tagging and publication remain a separate stage.

## Baseline findings

The canonical `dev` contains the initial nine-chapter User Guide and 52 Markdown
pages under `docs/`. All four Diataxis quadrants exist. Their ordinary examples
have already been exercised, so reorganizing content should preserve them.

| Area | Observed state | Required disposition |
| --- | --- | --- |
| Core learning path | Org notes, planning and tables precede preferences in a single sequence. | Complete the editor path with preferences; give Org an independent entry and sequence. |
| General navigation | README and indexes present Org as the next step in learning the editor. | Distinguish everyday editing from additional built-in capabilities. |
| Binding reference | Org rows precede ordinary file/editing keys; the page has 837 lines. | Put ordinary editing first and extract Org-specific controls without losing any command or context. |
| Command reference | Core and Org names share tables and appended task prose. | Separate the catalogs and retain an exhaustive combined registry check. |
| Configuration | One full catalog and packaged sample cover core and Org settings. | Separate reader entry points; keep one complete sample and validate the union of documented settings. |
| Org coverage | Native outlines, tasks, agenda, capture, clocks, links, attachments, tables, source blocks, exports, feeds and previews are delivered. | Preserve detailed coverage and expose it from the Org entry, without reproducing the entire reference in its introduction. |
| Changelog | Only Unreleased exists: 52 Added, 4 Changed and 8 Fixed entries, many with long Org details. | Reconcile and consolidate the delivered product into readable initial-release notes. |
| Version state | Cargo declares 0.1.0; the canonical forge has no tags. | Prepare the first version section without inventing a prior release or changing the package version unnecessarily. |

The plan is based on repository files and implemented interfaces. Pending
tickets and specs do not describe released behavior. In particular, retain
the actual file-only CLI, collapsed initial tree, Rust/Org highlighting scope,
startup-only configuration and current completion behavior. Do not claim
directory launch, soft wrap, Vim mode, LSP, AI commands, integrated terminals,
plugins or system clipboard integration before they are delivered.

## Documentation architecture

Keep the four Diataxis directories. Organize by reader need first, then by
feature inside the appropriate quadrant. A feature is a documentation area,
not a new runtime module, package, plugin or installation requirement.

- `docs/tutorials/user-guide/index.md` remains the User Guide entry. Its core
  path is first file, editing, buffers/panes, search/commands, tree, preferences.
- `docs/tutorials/user-guide/org/index.md` is the independent Org learning entry.
  Reuse the three current Org chapters and link focused tasks in their own
  quadrants. State only the shared editing knowledge needed to begin.
- `docs/how-to/index.md` foregrounds editor tasks. An Org task entry groups
  capture, extraction and export without mixing them into core procedures.
- `docs/reference/index.md` foregrounds editor commands, keys, settings and
  display. `docs/reference/org/index.md` leads to the complete Org reference.
- `docs/reference/org/commands.md`, `bindings.md` and `configuration.md` own
  extracted Org catalogs. Shared keymap semantics and terminal conventions stay
  shared; document the globally available Org entry bindings truthfully.
- `docs/explanation/index.md` distinguishes the editing model from Org concepts.
  The explanation of Org support remains separate from task instructions.

Keep existing chapter and feature-reference URLs where practical. In particular,
do not rename the current numbered chapter files just to renumber the visible
editor path. A six-chapter index can link the existing preferences filename.
When extracting sections, update internal links and preserve published anchors
with concise onward links where needed. Do not duplicate the extracted prose.

Future delivered features can use the same arrangement. Do not add empty module
pages, speculative guides or roadmap claims to the product documentation.

## Editorial and completeness rules

- Core Next/Previous links never require an Org chapter. Org navigation stays
  inside its own path and offers an explicit return to the User Guide index.
- A reader can open a file with `rev notes.txt` and save it without project
  setup, extra flags, an exercise framework or Org knowledge.
- Keep short relative paths and the default successful workflow first. Put
  complete options, limits and exceptional cases in reference pages.
- Keep command/configuration snippets in English. Do not invent controls or
  change application behavior to make a procedure shorter.
- Preserve every documented compiled command, default binding, context and
  accepted configuration leaf. Extracting a feature must not erase its coverage.
- Keep `docs/reference/config.toml` complete and byte-identical to the packaged
  template. Navigation changes do not justify changing defaults or runtime data.
- Audit tutorials, task guides and reference separately. Comprehensive product
  coverage does not require a tutorial for every command or a reference dump in
  the User Guide.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Everyday editor guide | `docs/rev-documentation-01-editor` | `docs(guide): separate the everyday editor learning path` | `ops/default/report/slices/rev-documentation/slice-1-editor-guide.md` |
| Slice 2 | Independent Org documentation | `docs/rev-documentation-02-org` | `docs(org): separate Org guides and references` | `ops/default/report/slices/rev-documentation/slice-2-org-documentation.md` |
| Slice 3 | v0.1.0 documentation closure | `docs/rev-documentation-03-v0.1.0` | `docs(release): prepare v0.1.0 documentation and changelog` | `ops/default/report/slices/rev-documentation/slice-3-v0.1.0-documentation.md` |

## Slice 1: Everyday editor guide

Prerequisite: the planning baseline and an approved, committed plan/tracker.

- T1.1: Finish the core six-chapter learning path. Reconnect tree to preferences,
  remove mandatory Org prerequisites and keep the existing practical examples.
  Create the independent Org learning entry and reconnect its three chapters
  so no existing material is orphaned during the transition.
- T1.2: Align README and quadrant indexes with those two reader paths. Keep
  Org discoverable as a built-in capability and foreground ordinary recovery
  and preferences. Add the native Rust documentation checker and Bash gate
  driver, validate the actual candidate and record report/tracker evidence.

Primary paths: README; documentation indexes; User Guide index and chapter
navigation; the new Org tutorial index; `examples/documentation-check.rs` and
`scripts/check-documentation.sh`. These are development checks, without new
dependencies or application/configuration changes.

Acceptance: a newcomer completes opening, editing, saving, buffers, search,
tree navigation and preferences without entering Org. An Org reader can start
its guide directly, and every existing page remains reachable.

Validation: `cargo run --quiet --example documentation-check` checks local
Markdown targets, heading anchors, reachability and both learning paths; finish
with `git diff --check`. Preserve the native examples already validated at the
baseline; rerun only a workflow whose actual instructions change.

## Slice 2: Independent Org documentation

Prerequisite: Slice 1 integrated into `dev`.

- T2.1: Extract Org commands, bindings and configuration reference into their
  feature pages, with an Org reference index and task grouping. Reconcile the
  existing feature pages and explanation with the independent guide. Keep
  core references focused and retain old destinations through links where needed.
- T2.2: Adapt the existing completeness and link checks to the new ownership.
  Verify the combined catalogs against the live registry and schema, retain
  duplicate/omission rejection, and record the report and tracker evidence.

Primary paths: `docs/reference/commands.md`, `bindings.md`, `defaults.md`,
`org-controls.md`, existing Org feature pages, new `docs/reference/org/` pages,
and task/explanation indexes. The packaged TOML sample stays unchanged.

Affected native guards include `tests/commands/palette.rs` catalog/context
checks, `tests/commands/org_catalog.rs` control-reference checks and
`tests/configuration/reference_bounds.rs`. Extend the native documentation
checker to verify the union of the split command and configuration catalogs.
Retain omission/duplicate rejection, real context inheritance and typed bounds.

The documentation gate and PR validation commands use Rust and Bash. Do not
introduce Python checks or hide Python behind a wrapper. Existing historical
Python guards may need path/input compatibility updates to remain valid after
reference extraction; preserve their assertions without making them the new
delivery's gate. This is not a repository-wide tooling migration.

The native checker covers all Markdown pages under `docs/`, changed entry files,
local anchors, index reachability and the independent learning paths. Reuse the
repository's Rust dependencies, reference assertions and fixtures. Add only
regressions for meaningful omissions, duplicates or broken navigation; do not
create a documentation framework, renderer or generic test infrastructure.

Acceptance: the core command/key references begin with ordinary editing; Org
has complete, independently navigable reference coverage. The union of the
catalogs remains complete and each command has one catalog entry per inventory.
Context tables still describe their own actual inherited/global behavior.

Validation: the focused native and Python reference checks below, local links,
legacy destinations, and manual review of both reference entry points.

## Slice 3: v0.1.0 documentation closure

Prerequisite: Slices 1 and 2 integrated into `dev`.

- T3.1: Reconcile the User Guide, references and CHANGELOG against the delivered
  product. Consolidate repeated development-stage entries into user outcomes;
  cover the editor, configuration, presentation/input improvements and native
  Org support with balanced detail. Prepare `## 0.1.0 - YYYY-MM-DD` using the
  actual preparation date and retain a new empty `## Unreleased` above it.
- T3.2: Complete the cross-quadrant and ergonomic review, run the selected
  documentation gate on the integrated candidate, and record the report and
  tracker closure only after its success. If validation requires a repository
  correction, use the documented validation-repair PR route.

Use Added, Changed and Fixed only when they have meaningful entries. Preserve
release facts, useful compatibility information and necessary public controls.
Remove implementation bookkeeping, repeated safety prose and exhaustive shortcut
lists from release notes. Link details rather than reproducing feature manuals.
Cargo already declares 0.1.0; leave Cargo metadata unchanged if it still agrees.

Acceptance: release notes describe the complete actual initial product without
pending-ticket claims; the editor and Org have independent learning paths;
examples remain short and executable; reference coverage is complete.

This slice closes documentation preparation for 0.1.0. It does not itself
declare the version released. Tagging, main integration, binary distribution
and mirror/site publication need their own authorized release stage.

## Selected delivery gate

The exact controller gate is:

```sh
scripts/check-documentation.sh && git diff --check
```

The Bash driver runs native checks only, in this order:

```sh
cargo run --quiet --example documentation-check
cargo test --test commands reference
cargo test --test configuration reference
```

The native checker includes links, anchors, reachability and learning-path
coverage from Slice 1, then complete split catalogs from Slice 2. Native
reference filters must select real tests and retain registry, context and
parser-bound assertions. The driver must propagate every failure. PRs list
successful Rust/Bash commands and finish with `git diff --check`; no Python
one-liners or wrappers belong in this new validation contract.

This gate protects the affected documentation contracts; it is not the full
release gate. Expect one small Rust checker and two native test binaries,
normally minutes with a warm build cache. Do not add the whole editor/Org
workload or repeat performance runs for prose and link moves. Recheck changed
executable examples in an owned terminal fixture.

For later release publication, require `scripts/check-all.sh` evidence for the
exact final release candidate and the authorized main-integration route. No
scheduled forge workflow is currently present in this repository. Resolve that
release prerequisite explicitly rather than adding or emulating one as part of
this documentation set. Do not reuse an older gate result as validation of a
new release commit.

## Review and execution boundaries

The planning artifacts are ready for review when their three slice maps and
six task IDs agree. No implementation branch, report marked delivered,
controller configuration, release date or tag is created during planning.
Implementation preserves unrelated work and excludes application source,
dependencies, pending feature tickets, screenshots, website content and mirrors.
Native checker/driver creation belongs to Slice 1; split-reference checker/test
adaptation belongs to Slice 2. No optional feature work is bundled with the
documentation.

## Slice 1 integration

Slice 1 integrated through canonical PR #168 at
`01f03f00729f4465d3d853ee68fd9c30aa26c7cf`. Controller receipt
`E5KZ2QKMPB33LEFR44KLTMDIUW` passed the six ordered checks for that exact
candidate. The original task history and report introduction remain intact.
The controller owns subsequent PR integration. Slice 2 is now active.
