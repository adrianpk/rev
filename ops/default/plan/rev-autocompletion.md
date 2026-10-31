# Native Autocompletion Delivery

Status: In progress
Date: 2026-10-31
Delivery set: rev-autocompletion
Slice strategy: behavior-first
Approved: product owner, 2026-10-31; approved capability and explicit commit, push and Looper startup request; this breakdown elaborates that authorized scope
Planning baseline: `6e38bb95fdf0ff81cec6d48a6e8b3bdaa9130fda`
Spec: [Native Autocompletion](../spec/editor-autocompletion.md)
Model: [Native Autocompletion Model](../spec/editor-autocompletion-model.md)
Concern: [Pending Work Index](../README.md)
Tracker: [Native Autocompletion Tracker](../tracker/rev-autocompletion.md)
Full gate: `cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-completion.sh && cargo run --locked --quiet --example documentation-check && git diff --check`
Active slice: Slice 5 — JSON and TOML key completion; in progress

## Outcome and authority

Deliver automatic and manual native completion for Rust, C, Go, HTML, CSS,
JavaScript, TypeScript/TSX, JSON and TOML, native Org contexts and explicitly
registered external grammar assets. Preserve the lightweight terminal editor
and use current/other open buffers rather than a directory index.
The [LSP successor](rev-language-server-support.md) owns semantic source
integration; native closure alone does not claim CMP02 fully delivered.

The owner's approval and startup request authorize planning commits/push to dev,
autonomous task implementation, one canonical PR per slice and Looper-owned
Rebase + Fast-forward merges into dev. Use one ordered Looper sequence: native
completion, exact-candidate gate and closure, then LSP. Do not launch the queued
successor independently. The controller owns task progress, validation receipts,
merge reconciliation and final closure; do not create a parallel tracker.

Main promotion, tags/releases, mirrors, installation, website publication,
server acquisition and unrelated presentation tickets are separate scope.
Do not add Python to required project checks. Use ordinary Conventional Commits
without task identifiers in subjects. Keep repository/configuration English.

## Strategy and prerequisites

8 behavior-first slices keep usable behavior and bounded owner layers
reviewable. Each slice compiles and passes its focused checks; no intentionally
broken contracts, fake working commands or deferred assertions.
Delivered prerequisites: canonical revisions and prepared edits/history,
source/view/coordinate ownership, Emacs command/keymap catalog, startup TOML,
vertical prompts, semantic palette, split panes, soft wrap, pointer input,
native Org and existing incremental Rust Tree-sitter support.
Permitted dependency additions are the measured grammar pins and safe upstream
tree-sitter-loader 0.27.0 with default features disabled. Preserve a single
Tree-sitter 0.27.1 engine and first-party forbid(unsafe_code). Record dependency
features/lockfile and actual executable bytes; do not add language runtimes,
Wasm, a CLI/compiler workflow or other unapproved packages.

Complete sample/default/schema/reference coverage in the slice introducing each
setting/command. Update Unreleased through changelog-normalize once a usable
outcome exists. Use the established User Guide/Diataxis skills for product docs;
keep ordinary editing as the core path and Org as supplementary guidance.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Checked manual completion | `feat/rev-autocompletion-01-session` | `feat(completion): add checked manual Rust completion` | `ops/default/report/slices/rev-autocompletion/slice-1-session.md` |
| Slice 2 | Cursor popup and automatic invocation | `feat/rev-autocompletion-02-interaction` | `feat(completion): show automatic cursor suggestions` | `ops/default/report/slices/rev-autocompletion/slice-2-interaction.md` |
| Slice 3 | Rust, C and Go structural sources | `feat/rev-autocompletion-03-systems` | `feat(completion): cover Rust C and Go declarations` | `ops/default/report/slices/rev-autocompletion/slice-3-systems.md` |
| Slice 4 | HTML, CSS, JavaScript and TypeScript | `feat/rev-autocompletion-04-web` | `feat(completion): cover web languages and TSX` | `ops/default/report/slices/rev-autocompletion/slice-4-web.md` |
| Slice 5 | JSON and TOML key completion | `feat/rev-autocompletion-05-formats` | `feat(completion): complete JSON and TOML keys` | `ops/default/report/slices/rev-autocompletion/slice-5-formats.md` |
| Slice 6 | Native Org context completion | `feat/rev-autocompletion-06-org` | `feat(org): complete native document contexts` | `ops/default/report/slices/rev-autocompletion/slice-6-org.md` |
| Slice 7 | Explicit external grammar admission | `feat/rev-autocompletion-07-external` | `feat(syntax): load registered external grammar assets` | `ops/default/report/slices/rev-autocompletion/slice-7-external.md` |
| Slice 8 | Native acceptance, footprint and guidance | `feat/rev-autocompletion-08-acceptance` | `test(completion): validate native editing and resource bounds` | `ops/default/report/slices/rev-autocompletion/slice-8-acceptance.md` |

## Slice 1: Checked manual completion

Prerequisite: approved preparation committed and pushed to dev.
Acceptance: CMP01, CMP03-CMP04 and the manual part of CMP05.
Owners: canonical buffer edits/history, session/input dispatch and retained Rust syntax.

- T1.1: Implement a revision-qualified completion session and a real manual Rust path using explicit declaration queries on the current buffer. Keep candidate display/insertion identity separate; no fake provider registry or public extension interface.
- T1.2: Add completion-at-point with its compact cursor popup, preselection and manual navigation/accept/cancel; checked literal replacement, deterministic filtering and one-transaction undo. Create tests/completion.rs and cover read-only admission, Unicode/range errors, stale revision/view/session results and source immutability on failure.

A manual Rust invocation produces usable candidates and an accepted edit. Failure/cancel creates no history entry; a stale target is rejected. Do not expose automatic popup controls before they work.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test completion --test syntax --test editing --test commands --test display --test panels`
- `git diff --check`

## Slice 2: Cursor popup and automatic invocation

Prerequisite: Slice 1 integrated.
Acceptance: CMP01, CMP04-CMP06.
Owners: native input contexts/keymaps, terminal frame projection, pane geometry, semantic palette and startup preferences.

- T2.1: Integrate the cursor popup with automatic requests, single-border/full-row styling, above/below placement and source focus. Implement active-popup navigation/accept/cancel while retaining normal Tab/Enter and Org behavior outside that context.
- T2.2: Add debounced automatic identifier/separator triggers, smart-case matching, suppression/cancellation/paste policy, finite settings and runtime controls. Update the complete sample, keymap/command catalogs and actual behavior references in this slice.

Exercise small/split panes, resize, wrapped Unicode rows, no candidates, rapid edits and late results, comments/strings, paste, cancellation and ordinary keys. No whole-buffer work during rendering.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test completion --test commands --test configuration --test display --test panels --test wrapping`
- `git diff --check`

## Slice 3: Rust, C and Go structural sources

Prerequisite: Slice 2 integrated.
Acceptance: CMP03, CMP05-CMP07; systems-language subset.
Owners: syntax admission and retained trees, language associations and bounded open-buffer observations.

- T3.1: Generalize the existing syntax owner without a second parser/index owner. Add the measured C and Go grammars alongside Rust, explicit extraction/highlight queries, language identity and bounded same-language open-buffer reuse.
- T3.2: Implement declaration/scope coverage and deterministic locality/ranking under the specified snapshot/symbol budgets. Invalidate on edits, rename, grammar changes, buffer close and observation eviction; never scan unopened files.

Use representative Rust/C/Go declarations, parameters, scopes, unsaved changes in another open buffer, ambiguous names, incomplete/new syntax and over-limit sources. No macro expansion or inferred member types.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test completion --test syntax --test files --test coordinates`
- `git diff --check`

## Slice 4: HTML, CSS, JavaScript and TypeScript

Prerequisite: Slice 3 integrated.
Acceptance: CMP03, CMP05-CMP07; web-language subset.
Owners: shared language catalog, explicit parser/query adapters, structured triggers and finite HTML/CSS catalogs.

- T4.1: Add the measured HTML, CSS, JavaScript and TypeScript/TSX grammar/query payloads. Distinguish TSX parser selection while preserving existing language preference resolution; include the needed HTML/CSS associations.
- T4.2: Provide the specified declarations, observed tags/attributes/selectors/custom properties and finite standard HTML/CSS catalogs. Admit separator/identifier contexts and preserve graceful plain editing around unsupported syntax.

Cover ordinary TypeScript and TSX separately, declarations and imports, HTML tags/attributes, CSS properties, malformed input and matching-context suppression. No React model, browser tooling, schema engine or injected-language framework.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test completion --test syntax --test configuration --test files`
- `git diff --check`

## Slice 5: JSON and TOML key completion

Prerequisite: Slice 4 integrated.
Acceptance: CMP03, CMP05-CMP07; structured-format subset.
Owners: JSON/TOML syntax adapters, current/open-buffer key observations and shared checked edits.

- T5.1: Add the measured JSON and TOML grammars, explicit highlight/extraction queries and structural key/table contexts to the existing language catalog.
- T5.2: Complete observed keys/tables through the common interaction, including quoted structural-key admission without ordinary value-string suggestions. Cover nested/duplicate keys, escaped names, invalid documents and unsaved edits.

JSON and TOML complete structurally admitted keys and remain quiet in ordinary values. Show only observed names, not invented schema knowledge; errors preserve editing.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test completion --test syntax --test configuration`
- `git diff --check`

## Slice 6: Native Org context completion

Prerequisite: Slice 5 integrated.
Acceptance: CMP08 with CMP01 and CMP04-CMP07.
Owners: existing Rust Org structure, task/tag/property/link observations, source-edit buffers and native keymaps.

- T6.1: Adapt current Org observations and effective configuration for TODO positions, tags, property keys, supported directives/link types and current-document heading targets. Reuse the existing parser and document model.
- T6.2: Integrate structural-prefix triggering, checked replacement and unsaved target invalidation. Preserve literal blocks, ordinary prose, Org Tab actions outside an active popup and declared-language completion in source-edit buffers.

Cover incomplete heading/tag/property/directive/link prefixes, custom TODO states, unsaved heading targets, literal blocks and source-edit publication/undo. No Org grammar, dictionary or LSP dependency.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test completion --test org_editing --test org_source --test configuration --test commands`
- `git diff --check`

## Slice 7: Explicit external grammar admission

Prerequisite: Slice 6 integrated.
Acceptance: CMP07, CMP09 and the R31 delta; preserve R35.
Owners: startup language registration, XDG data resolution, syntax admission and safe upstream loader boundary.

- T7.1: Add the pinned safe tree-sitter-loader API with default features disabled. Implement explicit versioned parser/symbol/highlight/completion registration, lazy one-time bounded admission and bundled/plain fallback. Define and document the finite query capture profile.
- T7.2: Test a real precompiled grammar replacement and an unbundled language through the actual safe loader. Reject missing/wrong-symbol/ABI/query assets without losing fallback; prove startup registration and process-lifetime retention. Reconcile the named R31 baseline restriction while preserving historical gate evidence and R35.

Use owned fixture libraries compiled before editor startup, never by rev at runtime. Retain first-party forbid(unsafe_code); no raw-pointer/custom FFI, Wasm, CLI, project discovery or automatic installer. Document native-code trust and replacement-by-restart.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test completion --test language_assets --test syntax --test configuration --test files`
- `git diff --check`

## Slice 8: Native acceptance, footprint and guidance

Prerequisite: Slice 7 integrated.
Acceptance: CMP01 and CMP03-CMP09 jointly; CMP02 semantic switching reserved for the LSP successor.
Owners: native/Rust PTY acceptance, resource measurements, complete catalogs and focused product guidance.

- T8.1: Add examples/completion-acceptance.rs and scripts/check-completion.sh using the existing Rust/Bash PTY approach. Exercise manual/automatic selection, edit/undo/cancel, all ten parsers, Org contexts, external admission and native fallback. Measure integrated release bytes against the recorded baseline, idle CPU/work, bounded indexing and burst responsiveness.
- T8.2: Complete concise completion tutorial/how-to/reference pages with small visual feedback examples, complete sample/command catalogs and Unreleased notes through the established skills. Record actual capability limits and distinguish syntax candidates from semantic LSP. Keep Org supplementary to ordinary editing.

Use isolated HOME/XDG/fixture sources and finite PTY children; no installed binary or user config changes. The 1.330297 grammar experiment is comparative evidence, not a ceiling or an assertion about final integrated bytes.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test completion --test language_assets --test syntax --test coordinates --test wrapping --test pointer --test org_editing`
- `bash -n scripts/check-completion.sh`
- `bash scripts/check-completion.sh`
- `cargo run --locked --quiet --example documentation-check`
- `git diff --check`

## Final gate and execution cost

The metadata Full gate is the exact immutable controller command. Run it once
after all 8 PRs integrate into clean dev. Shared input dispatch, retained syntax, edit/history publication and pane geometry
justify the all-target native suite here. During slices use only their recorded
focused checks and concrete additional regressions warranted by changed owners.
Do not append all historical Org, export, wrap or bookmark journey scripts.

Expected warm execution is a few minutes, dominated by Cargo compilation and
release/native-grammar/PTY fixtures. First builds may take longer for new dependencies.
Record actual timing/resources instead of treating this estimate as evidence.
Compile owned external-language fixtures ahead of editor startup; rev itself
must never require a runtime compiler for grammar use.
The acceptance wrapper batches the existing source-check example over all production Rust files, marks actual crate roots explicitly and checks introduced example roots. A checker invocation with no source arguments is not evidence. No checks have passed merely because they are listed here.

Use exact-HEAD controller validation and preserve unchanged gate definitions,
checkpoint ownership, budgets and conversation. A failed final gate follows
Looper's explicitly authorized bounded repair/retry path; do not patch primary
dev to bypass it or weaken acceptance. No unsolicited successor set is authorized.

## Closure and successor boundary

Commit the approved spec/model, plan/tracker and index before init/start.
Keep local Looper configuration outside the repository; pin these exact slice
identities and ordered checks before the first checkpoint. Reports are created
by their implementation slices at the canonical map paths, not predeclared as
delivered. Before startup, verify clean canonical dev, installed tool help and
actual detached preflight.

After the full gate passes, closure may change only this plan/tracker,
its 8 configured reports, its specification/model and the pending-work index.
No source, test, script, dependency, product documentation or changelog belongs
to the post-gate closure commit. Preserve exact candidate/receipt and historical
delivery evidence. Record native completion as delivered while explicitly leaving semantic source
switching to the queued LSP set; do not mark that combined acceptance complete.
The controller can hand off only after verified closure; neither set promotes main.
