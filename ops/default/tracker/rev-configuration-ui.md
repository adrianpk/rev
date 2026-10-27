# Configuration and Interaction Tracker

Status: Approved
Approved: product owner, 2026-10-27; one combined plan/tracker, all slices and autonomous Looper startup and dev integration authorized.
Delivery set: rev-configuration-ui
Slice strategy: layered
Active slice: Slice 29 - Bound startup and completion work
Full gate: `scripts/check-all.sh`
Spec: [Editor Configuration](../spec/editor-configuration.md)
Model: [Editor Configuration Model](../spec/editor-configuration-model.md)
Concern: [Editor Interaction](../spec/editor-interaction.md)
Plan: [Configuration and Interaction Delivery](../plan/rev-configuration-ui.md)

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Model typed configuration | `feat/rev-config-ui-01-config-types` | `feat(slice-1): model typed configuration` | `ops/default/report/slices/rev-configuration-ui/slice-1-config-types.md` |
| Slice 2 | Validate TOML and diagnostics | `feat/rev-config-ui-02-toml-validation` | `feat(slice-2): validate typed TOML settings` | `ops/default/report/slices/rev-configuration-ui/slice-2-toml-validation.md` |
| Slice 3 | Load bounded include graphs | `feat/rev-config-ui-03-config-includes` | `feat(slice-3): load bounded configuration includes` | `ops/default/report/slices/rev-configuration-ui/slice-3-config-includes.md` |
| Slice 4 | Resolve layered settings | `feat/rev-config-ui-04-config-resolution` | `feat(slice-4): resolve configuration precedence` | `ops/default/report/slices/rev-configuration-ui/slice-4-config-resolution.md` |
| Slice 5 | Identify buffer languages | `feat/rev-config-ui-05-file-associations` | `feat(slice-5): resolve file language associations` | `ops/default/report/slices/rev-configuration-ui/slice-5-file-associations.md` |
| Slice 6 | Project buffer tab policies | `feat/rev-config-ui-06-buffer-display-policy` | `feat(slice-6): apply buffer display policies` | `ops/default/report/slices/rev-configuration-ui/slice-6-buffer-display-policy.md` |
| Slice 7 | Apply indentation and new-file defaults | `feat/rev-config-ui-07-indentation-and-format` | `feat(slice-7): apply indentation and new-file defaults` | `ops/default/report/slices/rev-configuration-ui/slice-7-indentation-and-format.md` |
| Slice 8 | Capture Emacs-style rectangles | `fix/rev-config-ui-08-rectangle-capture` | `fix(slice-8): preserve rectangle tab semantics` | `ops/default/report/slices/rev-configuration-ui/slice-8-rectangle-capture.md` |
| Slice 9 | Keep rectangle and path transitions coherent | `feat/rev-config-ui-09-policy-transitions` | `feat(slice-9): retain buffer policies across path changes` | `ops/default/report/slices/rev-configuration-ui/slice-9-policy-transitions.md` |
| Slice 10 | Define panel geometry contracts | `refactor/rev-config-ui-10-panel-contracts` | `refactor(slice-10): define panel geometry contracts` | `ops/default/report/slices/rev-configuration-ui/slice-10-panel-contracts.md` |
| Slice 11 | Compose editing panels | `refactor/rev-config-ui-11-editor-panels` | `refactor(slice-11): compose editing panes through panels` | `ops/default/report/slices/rev-configuration-ui/slice-11-editor-panels.md` |
| Slice 12 | Compose tree and auxiliary panels | `refactor/rev-config-ui-12-tree-panels` | `refactor(slice-12): compose the tree through panels` | `ops/default/report/slices/rev-configuration-ui/slice-12-tree-panels.md` |
| Slice 13 | Resolve palette and syntax colors | `feat/rev-config-ui-13-palette-and-syntax` | `feat(slice-13): apply semantic RGB palette roles` | `ops/default/report/slices/rev-configuration-ui/slice-13-palette-and-syntax.md` |
| Slice 14 | Render UI palette and restore terminal | `feat/rev-config-ui-14-ui-palette` | `feat(slice-14): render UI palette and restore terminal colors` | `ops/default/report/slices/rev-configuration-ui/slice-14-ui-palette.md` |
| Slice 15 | Apply presentation and syntax preferences | `feat/rev-config-ui-15-presentation-preferences` | `feat(slice-15): apply layout tree and syntax preferences` | `ops/default/report/slices/rev-configuration-ui/slice-15-presentation-preferences.md` |
| Slice 16 | Apply bounded history and prompt preferences | `feat/rev-config-ui-16-bounded-preferences` | `feat(slice-16): apply history ring and prompt limits` | `ops/default/report/slices/rev-configuration-ui/slice-16-bounded-preferences.md` |
| Slice 17 | Validate context keymaps | `feat/rev-config-ui-17-keymap-contracts` | `feat(slice-17): validate configurable context keymaps` | `ops/default/report/slices/rev-configuration-ui/slice-17-keymap-contracts.md` |
| Slice 18 | Dispatch configurable editor bindings | `feat/rev-config-ui-18-editor-keymaps` | `feat(slice-18): dispatch configured editor keybindings` | `ops/default/report/slices/rev-configuration-ui/slice-18-editor-keymaps.md` |
| Slice 19 | Dispatch transient and tree bindings | `feat/rev-config-ui-19-context-keymaps` | `feat(slice-19): dispatch prompt and tree keybindings` | `ops/default/report/slices/rev-configuration-ui/slice-19-context-keymaps.md` |
| Slice 20 | Activate startup configuration | `feat/rev-config-ui-20-startup-configuration` | `feat(slice-20): load user configuration at startup` | `ops/default/report/slices/rev-configuration-ui/slice-20-startup-configuration.md` |
| Slice 21 | Present vertical M-x candidates | `feat/rev-config-ui-21-vertical-command-list` | `feat(slice-21): show vertical command completion` | `ops/default/report/slices/rev-configuration-ui/slice-21-vertical-command-list.md` |
| Slice 22 | Navigate and execute command candidates | `feat/rev-config-ui-22-command-selection` | `feat(slice-22): navigate vertical command candidates` | `ops/default/report/slices/rev-configuration-ui/slice-22-command-selection.md` |
| Slice 23 | Observe bounded filesystem candidates | `feat/rev-config-ui-23-path-candidates` | `feat(slice-23): observe filesystem completion candidates` | `ops/default/report/slices/rev-configuration-ui/slice-23-path-candidates.md` |
| Slice 24 | Complete and navigate file paths | `feat/rev-config-ui-24-path-navigation` | `feat(slice-24): complete and navigate file paths` | `ops/default/report/slices/rev-configuration-ui/slice-24-path-navigation.md` |
| Slice 25 | Open selected and new file paths safely | `feat/rev-config-ui-25-find-file-integration` | `feat(slice-25): open completed file paths safely` | `ops/default/report/slices/rev-configuration-ui/slice-25-find-file-integration.md` |
| Slice 26 | Publish exhaustive configuration reference | `docs/rev-config-ui-26-configuration-reference` | `docs(slice-26): document every supported configuration value` | `ops/default/report/slices/rev-configuration-ui/slice-26-configuration-reference.md` |
| Slice 27 | Exercise combined terminal workflows | `test/rev-config-ui-27-configuration-ui-journeys` | `test(slice-27): exercise configuration and UI workflows` | `ops/default/report/slices/rev-configuration-ui/slice-27-configuration-ui-journeys.md` |
| Slice 28 | Harden cross-feature failure boundaries | `test/rev-config-ui-28-configuration-ui-boundaries` | `test(slice-28): verify configuration and UI failure boundaries` | `ops/default/report/slices/rev-configuration-ui/slice-28-configuration-ui-boundaries.md` |
| Slice 29 | Bound startup and completion work | `perf/rev-config-ui-29-configuration-ui-performance` | `perf(slice-29): bound startup and completion work` | `ops/default/report/slices/rev-configuration-ui/slice-29-configuration-ui-performance.md` |
| Slice 30 | Verify delivery acceptance and references | `test/rev-config-ui-30-configuration-ui-acceptance` | `test(slice-30): verify configuration and interaction acceptance` | `ops/default/report/slices/rev-configuration-ui/slice-30-configuration-ui-acceptance.md` |

## Task state

All tasks start unchecked. Record actual task commit hashes, focused check
results, canonical PR and merge evidence as they complete. Preparatory parsing
or a passing focused check is not delivery-set closure. Advance Active slice
consistently with the plan and controller checkpoint.

### Slice 1: Model typed configuration

Contracts: CFG02-CFG06, CFG20. Tickets: TKT-20261027103708.
Details and checks: [Slice 1](../plan/rev-configuration-ui.md#slice-1-model-typed-configuration).

- [x] T1.1: Establish a small configuration owner with complete defaults and partial input types for the entire accepted model catalog; absence must survive parsing and language/include merging. Document ownership and numeric semantics without introducing a generic framework.
- [x] T1.2: Pin toml 1.1.8 and serde 1.0.229 with only required parser/derive features; commit Cargo.lock, preserve Rust 1.98 and first-party safe Rust. Create the configuration integration test target and verify explicit false, zero and absent values.
- [x] T1.3: Record source/default/catalog invariants and preparatory status. Do not advertise operational configuration before Slice 20 wires all consumers.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T1.1 `64038cccf6332c17410ad4fe85e589adfcccd5d9`;
T1.2 `21bd37e90d8cd48d93e37e4284cf511f4485a524`;
T1.3 `b64a242bc6218d3d28c08aa0a6810118c0533db6`.
Development validation: library/test compilation and focused presence/catalog
checks passed. All five ordered slice checks passed on
`9b2760232ac118abf381e9d043ed312d8e16745d`, controller validation
`VOD75O22JC246SRCLXA6DED5RK`.
Report: [Slice 1](../report/slices/rev-configuration-ui/slice-1-config-types.md).
PR: [#50](https://forge.adrianpk.com/adrianpk/rev/pulls/50);
integrated by Rebase + Fast-forward into dev at
`9b2760232ac118abf381e9d043ed312d8e16745d`. Slice 1 of 30 completed.

### Slice 2: Validate TOML and diagnostics

Contracts: CFG02, CFG04-CFG06, CFG13-CFG14, CFG20. Tickets: TKT-20261027103708.
Details and checks: [Slice 2](../plan/rev-configuration-ui.md#slice-2-validate-toml-and-diagnostics).

- [x] T2.1: Parse UTF-8 TOML 1.1 into typed partial documents, accepting omitted version as 1 and rejecting unknown explicit versions, unknown keys, bad types/enums, invalid RGB and out-of-range/nonfinite numbers.
- [x] T2.2: Validate only accepted language leaves and pattern shapes; retain source/key and spans where available. Reject duplicate keys and non-command binding values without silently defaulting bad input.
- [x] T2.3: Exercise malformed input, Unicode positions, valid empty include arrays and every admitted bound, including soft history limits and zero syntax cutoff. Keep parsing independent of editor mutation.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T2.1 `2b4ff0093157ca30bc540c935f12fd6f18b5ec87`;
T2.2 `4c1512f370a0731c82e97ba30d250c1ab102057d`;
T2.3 `22985c06620b209d1b906b73d5ba7b79a5140a92`.
Development validation: all 15 source-validation tests passed, including every
numeric bound and all 25 RGB roles. All five ordered slice checks passed on
`be0c8abfa16cf3993e52fade6e8aa55dba0a58f5`, controller validation
`W37EBYETIZI3HCIS47HWPLO4UC`, including 20 configuration tests.
Report: [Slice 2](../report/slices/rev-configuration-ui/slice-2-toml-validation.md).
PR: [#51](https://forge.adrianpk.com/adrianpk/rev/pulls/51);
integrated by Rebase + Fast-forward into dev at
`be0c8abfa16cf3993e52fade6e8aa55dba0a58f5`. Slice 2 of 30 completed.

### Slice 3: Load bounded include graphs

Contracts: CFG01, CFG07-CFG08, CFG14, CFG21. Tickets: TKT-20261027103708.
Details and checks: [Slice 3](../plan/rev-configuration-ui.md#slice-3-load-bounded-include-graphs).

- [x] T3.1: Resolve independent XDG roots and optional config.toml without creating files/directories; ignore relative XDG values, use HOME fallbacks, and return precise errors when a required root cannot be resolved.
- [x] T3.2: Read explicit includes depth-first in declaration order relative to the declaring source; enforce root-depth zero, depth 16, 64 occurrences, 256 KiB per source and 2 MiB total before unbounded allocation. Detect active-stack symlink/hard-link cycles by file identity.
- [x] T3.3: Test shared acyclic leaves, duplicates counting toward bounds, permission/missing includes, alias cycles and read errors. A missing optional root alone selects defaults; a broken explicit source rejects the candidate.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T3.1 `b55d825bee9bf0d35e638beca57af290f5a2fb27`;
T3.2 `6840ad1cf9bdfe8b607ac78751f9060b48e92cbd`;
T3.3 `75b0215f37affec202f3ff966ee6bb8737f29ccd`.
Development validation: all 15 new focused configuration tests passed, including
exact byte/depth/occurrence boundaries, real permission errors and alias cycles.
T3.3 follow-up: correct the denied-read fixture's Unix permission literal to
octal after the initial all-target Clippy rejection.
All five ordered checks passed on `035ca7fb9b69fa02424557a00744e6d6c35a6002`,
controller validation `A2VHYFH4JXYZGQG5STDPTSUF6J`, including 35 configuration tests.
Report: [Slice 3](../report/slices/rev-configuration-ui/slice-3-config-includes.md).
PR: [#52](https://forge.adrianpk.com/adrianpk/rev/pulls/52);
integrated by Rebase + Fast-forward into dev at
`035ca7fb9b69fa02424557a00744e6d6c35a6002`. Slice 3 of 30 completed.

### Slice 4: Resolve layered settings

Contracts: CFG03, CFG07, CFG09, CFG14-CFG15. Tickets: TKT-20261027103708.
Details and checks: [Slice 4](../plan/rev-configuration-ui.md#slice-4-resolve-layered-settings).

- [x] T4.1: Resolve built-ins, ordered includes and root leaves with table merging and array replacement. Resolve language partials after global layers so a late global value cannot erase an explicit language leaf.
- [x] T4.2: Validate the complete candidate before publishing one immutable startup snapshot; preserve concise per-leaf provenance where useful. A failed candidate yields complete built-ins and one actionable diagnostic, never mixed accepted/rejected values.
- [x] T4.3: Exercise root/include ordering, false/zero/empty values, per-language inheritance and candidate rollback without reading project-local config or evaluating paths.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T4.1 `88a4185b3e0ce8062c6de09531106f08bf1d389f`;
T4.2 `3a6b77457c047978871d7a01934fb39c5e114792`;
T4.3 `5b7a40e088cf87acc0feaa00e4a66e6e98d5d7da`.
Development validation: all 14 new focused configuration tests passed, covering
partial/source/language precedence, complete projection, leaf origins and whole
candidate fallback. An isolated child verifies explicit user-source selection.
All five ordered checks passed on `100d9a7d939b96a16e8309abba7af2c9705dc6e3`,
controller validation `42OENLRHMME4XSVSRQF6H2QUUO`, including 49 configuration tests.
Report: [Slice 4](../report/slices/rev-configuration-ui/slice-4-config-resolution.md).
PR: [#53](https://forge.adrianpk.com/adrianpk/rev/pulls/53);
integrated by Rebase + Fast-forward into dev at
`100d9a7d939b96a16e8309abba7af2c9705dc6e3`. Slice 4 of 30 completed.

### Slice 5: Identify buffer languages

Contracts: CFG09, CFG18, CFG21. Tickets: TKT-20261027103709.
Details and checks: [Slice 5](../plan/rev-configuration-ui.md#slice-5-identify-buffer-languages).

- [x] T5.1: Implement the exact built-in basename/suffix map from the model, with user rules overriding built-ins, exact names before suffixes and longest suffix specificity. Identical patterns use normal layer precedence; overlapping extensions are valid.
- [x] T5.2: Keep language identity independent of bundled grammar availability; unknown identities inherit global editing preferences and default to plain syntax. Retain the visible visiting basename when canonical file identity uses an alias.
- [x] T5.3: Cover compound suffixes, .h overrides, exact names, unknown extensions and Go policies without a Go grammar. Association resolution must not read directories or open a new file.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T5.1 `35b6faeb78529d279ebbbbd01a53499e38f030d7`;
T5.2 `0cd0ebf2291267ad7bd509f20a7cfaaae16bbbaa`;
T5.3 `4c59dd552f6a0ac3cfcbefe45c7c80b3ccae7d83`.
Development validation: five focused configuration association tests, two file
association tests and the adapted syntax association regression passed. Focused
library/configuration/file Clippy passed. All seven ordered checks passed on
`95923e869370a44de581ed34003fc228549049ef`, controller validation
`YT7IW6CWG4D6XZCCRM2HQUUHXW`, including 54 configuration, 62 file and 21 syntax tests.
Report: [Slice 5](../report/slices/rev-configuration-ui/slice-5-file-associations.md).
PR: [#54](https://forge.adrianpk.com/adrianpk/rev/pulls/54);
integrated by Rebase + Fast-forward into dev at
`95923e869370a44de581ed34003fc228549049ef`. Slice 5 of 30 completed.

### Slice 6: Project buffer tab policies

Contracts: CFG09-CFG11. Tickets: TKT-20261027103709.
Details and checks: [Slice 6](../plan/rev-configuration-ui.md#slice-6-project-buffer-tab-policies).

- [x] T6.1: Give each buffer one effective editing/display policy from a supplied settings snapshot; replace hardcoded tab assumptions in coordinate conversion, viewport projection, vertical goal columns, selection and search geometry.
- [x] T6.2: Use buffer policy coherently across shared views and invalidate display-derived caches without changing canonical text positions, rope revisions, saved-state identity or history.
- [x] T6.3: Test widths 1, 2, 4, 8 and 32, mixed wide/combining/ZWJ clusters, chunk seams, CRLF and clipped views. Default tab width must preserve baseline geometry.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T6.1 `8f8e39e0f2c134d3f8c8c9af014bee1c2538e753`;
T6.2 `fe01eae99816159a732df635830a425e50e88982`;
T6.3 `fd6c061b651ff17ab21ee37dcc23b7de9555583c`.
Development validation: ten new coordinate, display and window cases passed,
including widths 1/2/4/8/32, real rope seams, CRLF, clipped Unicode, marked/search
geometry, shared views and history preservation. Corrected a test assumption
that an active search prompt leaves the terminal cursor in the document;
document position is asserted through the view. Focused Clippy passed.
All eight ordered checks passed on `9db4f5682730cb95dbe33bad260adf972aad8b1d`,
controller validation `QODDPXB5OZODZAJXZJ75KLEGGG`, including 24 coordinate,
20 display, 38 window and 92 editing tests.
Report: [Slice 6](../report/slices/rev-configuration-ui/slice-6-buffer-display-policy.md).
PR: [#55](https://forge.adrianpk.com/adrianpk/rev/pulls/55);
integrated by Rebase + Fast-forward into dev at
`9db4f5682730cb95dbe33bad260adf972aad8b1d`. Slice 6 of 30 completed.

### Slice 7: Apply indentation and new-file defaults

Contracts: CFG10, CFG16. Tickets: TKT-20261027103709.
Details and checks: [Slice 7](../plan/rev-configuration-ui.md#slice-7-apply-indentation-and-new-file-defaults).

- [x] T7.1: Route Tab through the effective policy: literal U+0009 for tabs; enough spaces for the next indent_width stop for spaces. Bracketed paste remains literal and one normal undo operation.
- [x] T7.2: Apply configured LF/CRLF and BOM only when creating a new buffer. Existing files retain recognized BOM, mixed/newline fidelity and final-newline absence through save.
- [x] T7.3: Test nonaligned columns, tabs already in text, wide Unicode, read-only refusal, undo/redo and existing-file save fidelity. Do not implement conversion commands.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T7.1 `8800c20c6a6df0a1f0796dc26607d80faf30b111`;
T7.2 `7c34b501f11ed17174d108dbdfc38419b817e3ee`;
T7.3 `8be9fb6807d7b69038667689cf29f64370729540`.
Development validation: three focused indentation/paste/dispatch tests and five
file-format tests passed. Coverage includes Unicode display columns, opposing
widths, read-only refusal, new-file Return/save/undo, existing format fidelity,
settings replacement and final-buffer recreation. Focused Clippy passed.
All seven ordered checks passed on `e6d781ad529cdb57ca20ede62ac76984c6a77505`,
controller validation `S7M2KNEOYW3XTZSQU4M3ESKR2Y`, including 92 editing,
67 file and 66 command tests.
Report: [Slice 7](../report/slices/rev-configuration-ui/slice-7-indentation-and-format.md).
PR: [#56](https://forge.adrianpk.com/adrianpk/rev/pulls/56);
integrated by Rebase + Fast-forward into dev at
`e6d781ad529cdb57ca20ede62ac76984c6a77505`. Slice 7 of 30 completed.

### Slice 8: Capture Emacs-style rectangles

Contracts: CFG11, modified R20. Tickets: TKT-20261027103709.
Details and checks: [Slice 8](../plan/rev-configuration-ui.md#slice-8-capture-emacs-style-rectangles).

- [x] T8.1: Separate copy and kill rectangle capture semantics: copy expands retained tabs without source mutation; kill preserves whole interior tabs and materializes only partial edges as required.
- [x] T8.2: Use the source buffer policy for display columns, retain whole-grapheme boundaries and baseline short-line/no-right-padding behavior, and keep linear and rectangular stores distinct.
- [x] T8.3: Port the observed Emacs behavior into original fixtures, not source code: a-tab-b and c-tab-d at width 4, partial edges, wide clusters and a one-transaction kill/undo. Update contradictory copy tests deliberately.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T8.1 `8271ccd8fa68ee05af02a412d3832088b5beedfe`;
T8.2 `d858fcd8bbf2e6eff2d6d1bbff02eeda4958aa8a`;
T8.3 `a74eff35b6f54c2b87066d2c15a0c96bc221b787`.
Development validation: three new capture cases, strengthened copy/kill tab-edge
and store-separation checks and the adapted copied-row insertion regression
passed. Coverage includes configured widths 1/2/4/8/32, Unicode, short rows,
CRLF, read-only copy and one-step kill/undo. Focused Clippy passed.
All six ordered checks passed on `7672b1eed5709c05d0221f705603ba12628c203f`,
controller validation `RSTOMMXIYRVKDJNLZI37TEXLBN`, including 38 rectangle
and 24 coordinate tests.
Report: [Slice 8](../report/slices/rev-configuration-ui/slice-8-rectangle-capture.md).
PR: [#57](https://forge.adrianpk.com/adrianpk/rev/pulls/57);
integrated by Rebase + Fast-forward into dev at
`7672b1eed5709c05d0221f705603ba12628c203f`. Slice 8 of 30 completed.

### Slice 9: Keep rectangle and path transitions coherent

Contracts: CFG09, CFG11, CFG16-CFG17. Tickets: TKT-20261027103709.
Details and checks: [Slice 9](../plan/rev-configuration-ui.md#slice-9-keep-rectangle-and-path-transitions-coherent).

- [x] T9.1: Yank retained rectangle rows at one left display column under destination tab policy; preserve retained literal tabs rather than forcing the source right edge. Pad only required left gaps and publish one reversible operation.
- [x] T9.2: Resolve policy after successful save-as/file reassociation and tree rename, including visited descendants of a renamed directory, from the same startup snapshot. Failed/cancelled operations retain prior policy.
- [x] T9.3: Test cross-width copy versus kill/yank, shared views, dirty saved-state truth, undo/redo, renamed directories, and existing file identity/fidelity; preserve all path-change guarantees.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T9.1 `3d7be72f1dc132c4d0adba0d77489deaa66c0f3e`;
T9.2 `ae94322442e1b4b8e77704c8777cd0271206c77d`;
T9.3 `8572b9653894e08f1d084a870af0c52e8bc9b022`.
Development validation: cross-width copy/kill/yank, interior destination edges,
save publication outcomes, file/directory rename policies and shared-view
write-file/history checks passed. Focused Clippy and source quality passed.
All seven ordered checks passed on `a4d9565cf4fcff7487808fe2281f78ec1d1ea24f`,
controller validation `OOM5IPKLFDQUBFEXTAA6PRC44Y`, including 40 rectangle,
58 tree and 70 file tests.
Report: [Slice 9](../report/slices/rev-configuration-ui/slice-9-policy-transitions.md).
PR: [#58](https://forge.adrianpk.com/adrianpk/rev/pulls/58);
integrated by Rebase + Fast-forward into dev at
`a4d9565cf4fcff7487808fe2281f78ec1d1ea24f`. Slice 9 of 30 completed.

### Slice 10: Define panel geometry contracts

Contracts: UI01, baseline R05-R07, R14-R15. Tickets: TKT-20261027103711, TKT-20261027085102.
Details and checks: [Slice 10](../plan/rev-configuration-ui.md#slice-10-define-panel-geometry-contracts).

- [x] T10.1: Introduce the smallest common presentation records for panel outer/content bounds, reserved separator/status cells, clipping and active state; keep Layout, ViewId and buffer ownership authoritative.
- [x] T10.2: Derive separator topology and junctions for nested horizontal/vertical splits, including minimum geometry and future completion-area reservation. Avoid a second window tree or general UI registry.
- [x] T10.3: Create native panel-contract tests for no overlap, bounded coordinates, zero/tiny frames and stable identities. This preparatory layer must compile without changing input semantics.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T10.1 `964812e837b27a4f406d2ae75730bd478b1d48ea`;
T10.2 `407b116d69ce421cd51fa862b9e57f5fcd02b471`;
T10.3 `d73cc5f1dfe467877e40e04c67ed7d8eaafbe5e9`.
Development validation: panel bounds/clipping, native overflow, T/cross
junctions, completion reservation and 4608 small projection cases passed.
Native extremes and active/stable view identities passed. Focused Clippy and
source checks passed. Runtime rendering/input continue through existing owners.
All six ordered checks passed on `e3763fd30b8ab0aeb433d25c44e4f43d7bf1eb13`,
controller validation `HHR6ATRPUOOJEL222DBMUJLWMN`, including 9 panel
and 38 window tests.
Report: [Slice 10](../report/slices/rev-configuration-ui/slice-10-panel-contracts.md).
PR: [#59](https://forge.adrianpk.com/adrianpk/rev/pulls/59);
integrated by Rebase + Fast-forward into dev at
`e3763fd30b8ab0aeb433d25c44e4f43d7bf1eb13`. Slice 10 of 30 completed.

### Slice 11: Compose editing panels

Contracts: UI01, baseline R05-R07, R14-R15. Tickets: TKT-20261027103711, TKT-20261027085102.
Details and checks: [Slice 11](../plan/rev-configuration-ui.md#slice-11-compose-editing-panels).

- [x] T11.1: Migrate editor rendering to panel content bounds and continuous shared Unicode separators/junctions, with no per-pane outer frame. Reserve horizontal border rows and status cells explicitly in layout minima.
- [x] T11.2: Derive text, cursor, syntax, search, selection and status translations from the same geometry owner; remove repeated ASCII-pipe painting as an independent placement convention.
- [x] T11.3: Test nested split directions, active/inactive panes, same-buffer independent points, clipped Unicode and resize. Keep the baseline default split proportion and focus traversal.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T11.1 `d1ae49100a76035ddd9ea98a48484f1d4fd86e8c`;
T11.2 `7b773d08b9dacd93e9da589f54220246c6c35c74`;
T11.3 `ca7ea55c3534c7a894ad0d4dccfa70fe6c19d1f7`.
Development validation: four rendered-panel cases, split admission/geometry,
clipped cells, selected search, shared syntax and all six window resize
regressions passed. Focused Clippy/source checks passed. Existing geometry
fixtures now reserve horizontal separator rows explicitly.
Controller validation initially stopped at the shared-policy fixture's byte
slice across a Unicode separator. Its exact cell-text assertion is retained
with character indexing for the one-cell fixture; focused regression passed.
All ten ordered checks passed on `167a49e33369d48cf5fcb009019ccd9303dde736`,
controller validation `RDUTP3PB6OJIW3ZR5GPRNFFAWB`, including 180 native tests
with zero skips across panels, display, windows, search, syntax and tree.
Report: [Slice 11](../report/slices/rev-configuration-ui/slice-11-editor-panels.md).
PR: [#60](https://forge.adrianpk.com/adrianpk/rev/pulls/60);
integrated by Rebase + Fast-forward into dev at
`167a49e33369d48cf5fcb009019ccd9303dde736`. Slice 11 of 30 completed.

### Slice 12: Compose tree and auxiliary panels

Contracts: UI01, CFG19. Tickets: TKT-20261027103711, TKT-20261027085102.
Details and checks: [Slice 12](../plan/rev-configuration-ui.md#slice-12-compose-tree-and-auxiliary-panels).

- [x] T12.1: Migrate tree content, separator, status and selection to the panel contract; remove string-prefix geometry and duplicated syntax/selection offset arithmetic.
- [x] T12.2: Reserve an auxiliary completion panel through the same composition interface while keeping tree focus and semantic state separate. Hiding a pane must not destroy buffers or pending prompts.
- [x] T12.3: Test tree plus nested splits, narrow hiding, focus repair, reopen/resize and exact cell clipping. No Odra registry/framework or unrelated tree feature is required.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T12.1 `1dd17fc3427736749d8533ce807963f66036628f`;
T12.2 `bb0ded2f9b942e9b9d81ed5575a86227bda1e050`;
T12.3 `7e76ebe38a802314d6b41ba27010be0ad42d39f3`.
Development validation: tree presentation/overlays/lifecycle/nested layout,
auxiliary reservation/limits and native workspace lifecycle/grid cases passed.
Coverage includes 324 workspace projections, Unicode seams, pending draft and
independent point retention, focus repair and undo truth. Focused Clippy/source
checks passed. Final exact-head validation `JATSPFSRYLFSF7NQX5X6YKMPR6`
passed all ten commands on `9ec177b137cc84d566f961fac8422ad741d55fe6`: 185 native
tests, zero skips, formatting, Clippy, source checks and diff whitespace.
Report: [Slice 12](../report/slices/rev-configuration-ui/slice-12-tree-panels.md).
PR: [#61](https://forge.adrianpk.com/adrianpk/rev/pulls/61);
integrated by Rebase + Fast-forward into dev at
`9ec177b137cc84d566f961fac8422ad741d55fe6`. Slice 12 of 30 completed.

### Slice 13: Resolve palette and syntax colors

Contracts: CFG04, CFG18, UI01. Tickets: TKT-20261027103710, TKT-20261027085102.
Details and checks: [Slice 13](../plan/rev-configuration-ui.md#slice-13-resolve-palette-and-syntax-colors).

- [x] T13.1: Implement all 25 selected palette roles using the pinned Tokyo Night Neovim night values, including all seven existing syntax classes. Every present leaf overrides only itself.
- [x] T13.2: Replace hardcoded syntax ANSI indices with configured RGB style resolution through existing captures. Preserve capture mapping, incremental parsing and plain-text fallback; add no grammars.
- [x] T13.3: Test all default values, single-role overrides, invalid RGB rejection, selected text precedence over syntax and Rust/plain policy changes without touching document/history.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T13.1 `bf851d6483286cb6836ac87b159f18c8b4e20d99`;
T13.2 `e86b30c254970d3a06a6635bdf9caeb5327b67c5`;
T13.3 `95717ac8f63029107a11897bf225890e8a644eae`.
Development checks passed for all 25 independent palette leaves, RGB syntax
rendering, partial-token linear/rectangle selection and Rust/plain rename
transitions retaining dirty text, mark, saved state and undo/redo. Focused Clippy
and source checks passed. Final exact-head validation `VVCDW6GTGVC7OTJ2Y44BDNJEYQ`
passed all eight commands on `8921273e463e2d8cdb5999aede301c984893a7b6`: 99 native
tests, three real PTY syntax journeys, zero skips, formatting, Clippy, source
checks and diff whitespace.
Report: [Slice 13](../report/slices/rev-configuration-ui/slice-13-palette-and-syntax.md).
PR: [#62](https://forge.adrianpk.com/adrianpk/rev/pulls/62);
integrated by Rebase + Fast-forward into dev at
`8921273e463e2d8cdb5999aede301c984893a7b6`. Slice 13 of 30 completed.

### Slice 14: Render UI palette and restore terminal

Contracts: CFG04, CFG14, UI01, baseline R03. Tickets: TKT-20261027103710, TKT-20261027085102.
Details and checks: [Slice 14](../plan/rev-configuration-ui.md#slice-14-render-ui-palette-and-restore-terminal).

- [x] T14.1: Apply background/foreground, status, border/focus, tree, region, search, prompt, warning and error roles through composition. Ensure every selected role has a real consumer.
- [x] T14.2: Apply cursor color through supported terminal sequences, with a documented capability fallback; restore cursor/text colors and terminal modes on normal exit and failures through the existing lifecycle owner.
- [x] T14.3: Test safe escape emission, no style leakage between panels, readable selections and restoration after errors. Keep idle rendering event-driven and do not add blinking.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T14.1 `eea4858ae3c2b16e174091f53020867ef0f81c1c`;
T14.2 `7a665f6f4394a487d8344290b866a1e934727f91`;
T14.3 `c57478dedb74bedf16e17e6e2c9fad34289cfff9`.
Focused cell-color checks passed for mixed panels, focus borders, tree selection,
search/region precedence, message severity and escape safety. Real PTYs passed
cursor/text cleanup, partial color writes, failed resets, raw-mode release,
capability/NO_COLOR fallback and executable defaults. Focused Clippy and source
checks passed. Inherited selection assertions now verify effective RGB cells
in display, linear-region, rectangle and search coverage; focused corrections
passed. Final exact-head validation `GPQFHWTX4LUK6AS3RIPAXXV2VY` passed all
thirteen commands on `3ba6bb0ecd825a4f3e75081ec5b056821fa6ff5d`: 87 native
tests and fifteen PTY cases, zero skips, formatting, Clippy, source and whitespace
checks.
Report: [Slice 14](../report/slices/rev-configuration-ui/slice-14-ui-palette.md).
PR: [#63](https://forge.adrianpk.com/adrianpk/rev/pulls/63);
integrated through Rebase + Fast-forward into dev at
`3ba6bb0ecd825a4f3e75081ec5b056821fa6ff5d`. Slice 14 of 30 completed.

### Slice 15: Apply presentation and syntax preferences

Contracts: CFG18-CFG19, UI01. Tickets: TKT-20261027103711.
Details and checks: [Slice 15](../plan/rev-configuration-ui.md#slice-15-apply-presentation-and-syntax-preferences).

- [x] T15.1: Apply page_overlap with clamped forward progress, preferred split_ratio with geometry minima, tree visibility/width/max_fraction/root, hidden entry filtering and directory-first ordering with deterministic filename ties.
- [x] T15.2: Apply syntax enabled/max_bytes and supported per-language rust/plain grammar selection; crossing a cutoff invalidates stale styling without preventing edits or changing text/history.
- [x] T15.3: Test default equivalence, valid extremes, tiny geometry, dotfiles, initial file versus scratch root, retained tree focus and no filesystem reads during drawing. Do not add natural sorting or extra display controls.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T15.1 `44543334610ee49689442241acf984cfe071eadc`;
T15.2 `1c8824ab576504d25c8be3f8bd7698406cb9bb93`;
T15.3 `5affaefc38d8ce4b58826685f06e1b51ba21f1fc`.
Focused preference checks passed for ratio/page extremes, recursive minima,
initial tree visibility/root, hidden/native filename ordering, refresh and
memory-only redraw after observed paths disappear. Grammar/cutoff transitions,
real parser admission and published destination policies preserve text/history.
Focused Clippy and source checks passed after correcting the root conditional.
Report: [Slice 15](../report/slices/rev-configuration-ui/slice-15-presentation-preferences.md).
PR: [#64](https://forge.adrianpk.com/adrianpk/rev/pulls/64);
Rebase + Fast-forward integration verified at
`c946be7f0ee451f2c554b457d5443451eff57797`. Exact-head controller evidence
`C5POK6XVB6OZBUGQ6544ZCYTXU` passed all 10 commands: 192 native tests, zero
skips, formatting, Clippy, source checks and whitespace checks.
Slice 15 of 30 completed.

### Slice 16: Apply bounded history and prompt preferences

Contracts: CFG17, CFG20, CFG22. Tickets: TKT-20261027103712.
Details and checks: [Slice 16](../plan/rev-configuration-ui.md#slice-16-apply-bounded-history-and-prompt-preferences).

- [x] T16.1: Apply whole-group history count and soft payload target through the existing history owner, preserving the newest coherent transaction and truthful saved state.
- [x] T16.2: Apply positive linear-ring capacity and directional coalescing; apply command history/input limits through existing prompt owners. No session persistence or time-based grouping is introduced.
- [x] T16.3: Test retention below one compound rectangle, ring eviction, coalescing disabled, bounded Unicode prompt input and cancellation after an over-limit error. Avoid tests that only mirror field assignment.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T16.1 `c9c6abca1aaf167768bc8810939bd7a3ea0199f7`;
T16.2 `8da79cb8dcfd7cc0dce6e98f8d891b75469f8aff`;
T16.3 `1afef71fbcbe9768b33cb9579169c2fdf46c3e48`.
Focused configured checks passed for whole typing groups, logical byte equality,
deferred pruning/redo branches, saved-state identity, compound rectangles,
capacity-one ring rotation, eviction and directional coalescing. Unicode prompt
limits, bounded completion/recall, command retention, literal search and both
query-replace stages retain cancellation. Focused Clippy and source checks passed.
Report: [Slice 16](../report/slices/rev-configuration-ui/slice-16-bounded-preferences.md).
PR: [#65](https://forge.adrianpk.com/adrianpk/rev/pulls/65);
Rebase + Fast-forward integration verified at
`726dbfbedf3192c25a45188ad143ef96aceb4940`. Exact-head controller evidence
`J2BSWUIQSV6737HEY2XJVFC6MD` passed all eight commands: 240 native tests,
zero skips, formatting, Clippy, source and whitespace checks.
Slice 16 of 30 completed.

### Slice 17: Validate context keymaps

Contracts: CFG06, CFG12-CFG13, modified R22. Tickets: TKT-20261027103713.
Details and checks: [Slice 17](../plan/rev-configuration-ui.md#slice-17-validate-context-keymaps).

- [x] T17.1: Audit the real editor/minibuffer/tree default maps and fixed action identities. Parse configured chord sequences through one normalization vocabulary including terminal aliases and prefix sequences.
- [x] T17.2: Validate inherited-plus-user maps, exact replacement, command availability in context and leaf/prefix conflicts before activation. Existing prompt/tree actions may gain stable catalog names, but configuration never supplies executable bodies.
- [x] T17.3: Keep cancellation reachable: preserve C-g as the transient cancel chord and reject attempts that remove that escape path. Reject non-command/unbinding values and explain terminal-indistinguishable chords. Document every default mapping.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T17.1 `9e7d05767b363d99cf26eb6b649dab21634a2b20`;
T17.2 `921c5038bb8d56c732a7f1b1d2e2e650e53e369b`;
T17.3 `4a758356c05bfbb0aea9e0d482d0d6effc4ae289`.
Focused native/default and configuration checks passed for chord limits, terminal
aliases, normalized replacement, provenance, context availability, inherited and
merged prefix conflicts, cancellation and complete-default fallback. Fixed action
identities and every native default mapping are documented. Focused Clippy and
source checks passed after correcting ordering, alias match and fixed-array forms.
Report: [Slice 17](../report/slices/rev-configuration-ui/slice-17-keymap-contracts.md).
PR: [#66](https://forge.adrianpk.com/adrianpk/rev/pulls/66);
Rebase + Fast-forward integration verified at
`09c26d8951fbbdf63eedaf245288cf9aa1a9fc0f`. Exact-head controller evidence
`CZSYPTGNKC3GWGVR5WHMHZCJSA` passed all eight commands: 233 native tests,
zero skips, formatting, Clippy, source and whitespace checks.
Slice 17 of 30 completed.

### Slice 18: Dispatch configurable editor bindings

Contracts: CFG12-CFG13, modified R22. Tickets: TKT-20261027103713.
Details and checks: [Slice 18](../plan/rev-configuration-ui.md#slice-18-dispatch-configurable-editor-bindings).

- [x] T18.1: Route editor chords and prefixes through the validated effective map and the same Rust handler identities as M-x. Retain built-in aliases, Meta/Escape timing and literal paste semantics.
- [x] T18.2: Apply exact user reassignment without deleting any inherited binding implicitly; commenting an override restores its built-in. Preserve prefix cancellation and reset across context changes.
- [x] T18.3: Test aliases such as M-j to next-line, replacement, multikey sequences, unknown commands, prefix collisions, terminal aliases, read-only refusal and exactly-once dispatch.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T18.1 `a6ecace9ee64235fe8ec324fa63d418d4985efe7`;
T18.2 `2422f5d109f74ebdd0b818854a45368b367c1711`;
T18.3 `3967342f2753d44ec19a5f528df94a852010f2d6`.
Focused configured dispatch checks passed for aliases, exact replacement,
inherited defaults, bounded prefixes, cancellation, context resets, literal paste,
rejected candidates, read-only refusal, phases and M-x parity. A focused regression
caught held Enter aliases bypassing acceptance guards; normalized phase admission
corrects it. Focused Clippy and source checks passed; supplied behavior is documented.
Report: [Slice 18](../report/slices/rev-configuration-ui/slice-18-editor-keymaps.md).
PR: [#67](https://forge.adrianpk.com/adrianpk/rev/pulls/67);
verified controller integration into dev at `acebc81731bb936cb14d5b13f6060546d167ca7c`.
Final evidence `6MGAJQQHEPCP2DO2K2K4H2AR5X` passed all nine ordered checks
on that clean head: 344 native tests, zero skips. Acta merge event:
`01M5YYS776WR2PCKDYPFA5EC46`.

### Slice 19: Dispatch transient and tree bindings

Contracts: CFG12-CFG13, UI02-UI03. Tickets: TKT-20261027103713.
Details and checks: [Slice 19](../plan/rev-configuration-ui.md#slice-19-dispatch-transient-and-tree-bindings).

- [x] T19.1: Wire validated minibuffer/tree maps to their existing action owners without bypassing argument, search, replace, confirmation or tree mutation state machines.
- [x] T19.2: Keep C-g cancellation reachable and destructive confirmations explicit. Reassignment in one context cannot leak into editor text or change unrelated context defaults.
- [x] T19.3: Test context isolation, prefix reset, prompt Unicode editing, tree focus/navigation and cancellation through find-file, M-x, query-replace and pending filesystem decisions.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T19.1 `d025410560191c202f5e59d3e786d82aa3fea936`;
T19.2 `71406c2e227812c86157bffbbaff408e9b34421a`;
T19.3 `6ef075a601e28560fe344ecc3111b6c0134c2121`.
Eighteen configured behavior tests pass for prompt Unicode editing, command
completion/history, find-file, search, replacement, bounded prefixes, tree focus,
filesystem arguments and explicit fresh consent. Focused Clippy/source checks
pass with nesting at most two. Final evidence `ZG7VQKTLCDBWMCIRGAXJPQMUJS`
passed all nine ordered checks at clean head
`c26ba1b857f90c8baf2cb6cf2b3e75c40879a3ec`: 362 tests, zero skips.
Report: [Slice 19](../report/slices/rev-configuration-ui/slice-19-context-keymaps.md).
PR: [#68](https://forge.adrianpk.com/adrianpk/rev/pulls/68);
verified controller integration into dev at that same head.
Acta merge event: `01M5YZMECFWEPVPCMGFBGA550B`.

### Slice 20: Activate startup configuration

Contracts: CFG01-CFG22, modified R01, R17, R20, R22. Tickets: TKT-20261027103708.
Details and checks: [Slice 20](../plan/rev-configuration-ui.md#slice-20-activate-startup-configuration).

- [x] T20.1: Wire XDG discovery, bounded loading, whole-candidate validation and one immutable snapshot into the actual executable/session before buffer creation; connect every accepted consumer from prior slices.
- [x] T20.2: On invalid or unreadable config, enter a usable editor with complete built-ins and a visible source/key diagnostic. Missing config stays silent. Never write config, create empty XDG roots or load project-local files.
- [x] T20.3: Test new startup versus unchanged running session after disk edits, palette/tab/key behavior through session boundaries and rollback of otherwise valid settings when one leaf is invalid. No reload command, polling or watcher.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T20.1 `5360fd648c27305d4feef87a2b1e4955df9f71db`;
T20.2 `190b2e22967c8826692b8e03f85a2ea73ce614c5`;
T20.3 `d9004750b8165e7db2deb580171f49da022fd5c1`.
Seven native startup cases (twelve isolated children) and four executable PTY
sessions pass for XDG discovery, includes, all context maps, palette/tab/format
behavior, complete fallback, diagnostics, passive roots and restart-only effects.
Focused Clippy/source checks pass with nesting at most two. Final evidence
`HDKGXOS4KJGTRHA2HCIOXHQGUW` passed all nine ordered checks at clean head
`2736cc112f04968c2780a496a18020397d373cb9`: 269 native tests and four terminal
sessions, zero skips.
Report: [Slice 20](../report/slices/rev-configuration-ui/slice-20-startup-configuration.md).
PR: [#69](https://forge.adrianpk.com/adrianpk/rev/pulls/69);
verified controller integration into dev at that same head.
Acta merge event: `01M5Z0BTG0TPAP5B7TFEBPQMYG`.

### Slice 21: Present vertical M-x candidates

Contracts: UI02, CFG04, CFG19, CFG22. Tickets: TKT-20261027084156.
Details and checks: [Slice 21](../plan/rev-configuration-ui.md#slice-21-present-vertical-m-x-candidates).

- [x] T21.1: Replace horizontal joined hints with query-plus-vertical candidate rows in the auxiliary panel. Apply configured row limit and matching mode with stable rank/order, safe Unicode clipping and explicit empty results.
- [x] T21.2: Retain selected row and scroll origin independently of editable query; draw selected command clearly using semantic palette roles, without rendering-side filesystem reads or dispatch.
- [x] T21.3: Test many matches, query filtering, zero available candidate rows, tree/nested panes, resize and draft/selection retention. Preserve the underlying editing viewport contract.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T21.1 `5ff7d648583fa5da25a08c6c87b1e9149ec9bea4`;
T21.2 `46cb306acc8409ad2297b23defed8c557c58c4a7`;
T21.3 `e8d7dd186c509f969df483d0963b578a349af94a`.
Nine command-list regressions, one command-region geometry case and two actual
RGB/NO_COLOR terminal sessions pass for configured matching/limits, vertical rows,
explicit empty results, retained selection/origin, Unicode/tiny frames, tree and
nested panes, and unchanged editing viewports. Focused Clippy/source checks pass
with nesting at most two. Final evidence `5OVU5L53D5Q6FFAJFJDAVYIT6O` passed
all nine ordered checks at clean head `f3b62e02f067e64bf515a28c8f0e5aaf4b18ba58`:
226 native tests and two terminal sessions, zero skips.
Report: [Slice 21](../report/slices/rev-configuration-ui/slice-21-vertical-command-list.md).
PR: [#70](https://forge.adrianpk.com/adrianpk/rev/pulls/70);
verified controller integration into dev at that same head.
Acta merge event: `01M5Z18NCM7G3KN36JN0YBMHBY`.

### Slice 22: Navigate and execute command candidates

Contracts: UI02, CFG12-CFG13, baseline R23. Tickets: TKT-20261027084156.
Details and checks: [Slice 22](../plan/rev-configuration-ui.md#slice-22-navigate-and-execute-command-candidates).

- [x] T22.1: Bind Up/Down and C-p/C-n to candidates, M-p/M-n to command history, Tab to complete the selected name and Return to the same named handler exactly once.
- [x] T22.2: Define exact/no-match/empty-query behavior, reset selection coherently after query changes and keep the selected row visible. Continue argument prompts normally; C-g restores the previous editing/tree focus.
- [x] T22.3: Test history versus candidate navigation, no accidental first-command execution, custom context aliases, cancellation and selected handler invocation with arguments.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T22.1 `9e7f3ebb897adc59c32c8a23feb732c187a3ab4a`;
T22.2 `97410b79146cfb26605381ea31099b95116f8609`;
T22.3 `629b4aa7151f531faf25e39e90cf6246eb57294a`.
Eight new native regressions and two RGB/NO_COLOR terminal sessions cover separate
candidate/history navigation, explicit/exact/empty/no-match acceptance, coherent
query resets, custom context aliases, selected file arguments, held/released
acceptance and editor/tree cancellation. Focused Clippy/source checks pass with
nesting at most two. Final evidence `4VLTMTFP4447WUE5MDYDLQATXT` passed all
eight ordered checks at clean head `0bb5aadf82d19849857d0dea63c62dd58f7ee511`:
217 native tests and two terminal sessions, zero skips.
Report: [Slice 22](../report/slices/rev-configuration-ui/slice-22-command-selection.md).
PR: [#71](https://forge.adrianpk.com/adrianpk/rev/pulls/71);
verified controller integration into dev at the same head.
Acta merge event: `01M5Z227PFE13YDWPTRSM0N4RR`.

### Slice 23: Observe bounded filesystem candidates

Contracts: UI03-UI04, baseline R10-R13. Tickets: TKT-20261027085234.
Details and checks: [Slice 23](../plan/rev-configuration-ui.md#slice-23-observe-bounded-filesystem-candidates).

- [x] T23.1: Create a filesystem candidate provider for one current directory component with 4096-entry and 512-KiB-name-byte limits, safe retained path identities, directories-first byte-name ordering and cached filtering.
- [x] T23.2: Handle relative/absolute paths, leading ~/, spaces and Unicode without shell expansion; separate lossless path identity from clipped/control-safe labels. Report non-UTF-8 names explicitly instead of opening lossy paths.
- [x] T23.3: Create provider tests for ambiguous/common/unique prefixes, hidden names, symlinks, permission/disappearance errors and limits. No recursion, watcher or reads during rendering/filtering.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T23.1 `e08a241e09e1bdce280e4619a4839f42ee3f2dc8`;
T23.2 `abb2f921ab3d4e553057ade991ff1347fd31fb34`;
T23.3 `5904932a76fe484dbd3d5d1d3906de6cf9649dca`.
Thirteen native regressions cover cached literal filtering, ordered hidden names,
explicit base/home roots, Unicode/control-safe and unsupported identities,
refresh/error retention, child symlinks and entry/name-byte bounds. Focused Clippy
and source checks pass with nesting at most two. Final evidence
`ZCOJKRCUPP3WQMVGH3Y3TZ2LU2` passed all five ordered checks at clean head
`39cb2a1d1049409ec31ba8fccb7fb86a68cc2904`: 13 native filesystem tests, zero skips.
Report: [Slice 23](../report/slices/rev-configuration-ui/slice-23-path-candidates.md).
PR: [#72](https://forge.adrianpk.com/adrianpk/rev/pulls/72);
verified controller integration into dev at the same head.
Acta merge event: `01M5Z2VGBE4P1KW5MZ2BAHSB3R`.

### Slice 24: Complete and navigate file paths

Contracts: UI03-UI04, CFG12-CFG13. Tickets: TKT-20261027085234.
Details and checks: [Slice 24](../plan/rev-configuration-ui.md#slice-24-complete-and-navigate-file-paths).

- [x] T24.1: Initialize find-file in the current visiting file parent or cwd, and integrate the provider with the common vertical completion panel; show directory markers without modifying the actual path bytes.
- [x] T24.2: Implement Tab unique/shared-prefix completion, directory descent, editable parent navigation and candidate selection. Invalidate cached observations only for directory changes or explicit completion refresh.
- [x] T24.3: Test ./, ../, ~/, absolute names, spaces, Unicode boundaries, trailing separators, changing prefixes, incomplete lists, tiny resize and C-g with draft/focus retained.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T24.1 `324c635437f760dfd2679607e4aec12aff24baf9`;
T24.2 `531930ec9a4115c234df5a11345d97bb9f3c33e5`;
T24.3 `e698d323a802d46d890dafdce40879c02b4c94b5`.
Fourteen new path/navigation tests, one configured candidate-alias regression and
two RGB/NO_COLOR terminal sessions cover native bases, shared/selected completion,
directory/home/Unicode paths, cached redraw, refresh, incomplete lists, bounds,
resize and cancellation. Focused file/tree cursor and panel checks pass, as do
Clippy/source checks with nesting at most two. Controller evidence `ZZKS6TPE3BTHBC74NSVKP536ZR` passed all ten ordered checks
on `9f365b86d4097f6c78a6777f848c651bcbe2bce3`: 246 native tests and two
RGB/NO_COLOR terminal sessions, zero skips. PR #73 merged through Rebase +
Fast-forward into dev at that exact head; Acta merge event
`01M5Z47H2DEAFVSP8SDWCS1HKC` confirms integration. Slice 24 of 30 completed.
Report: [Slice 24](../report/slices/rev-configuration-ui/slice-24-path-navigation.md).
PR: [#73](https://forge.adrianpk.com/adrianpk/rev/pulls/73).

### Slice 25: Open selected and new file paths safely

Contracts: UI03-UI04, CFG09, CFG16. Tickets: TKT-20261027085234.
Details and checks: [Slice 25](../plan/rev-configuration-ui.md#slice-25-open-selected-and-new-file-paths-safely).

- [x] T25.1: Unify C-x C-f and M-x find-file: explicit selected-candidate Return accepts that identity; freely typed Return accepts the draft/new filename; an accepted directory descends without visiting a text buffer.
- [x] T25.2: Open through existing canonical visit/file ownership, reusing dirty buffers and applying startup language/new-file policies. Candidate observation/completion cannot create files or publish saves.
- [x] T25.3: Test revisit through aliases, relative canonical reuse, nonexistent file creation only in memory, failed/unreadable paths retaining prompt and active buffer, symlink destinations and no accidental save. Scope stays find-file.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T25.1 `27b9fd9133d8a018a29a8ac2b47b7a64a98a00c2`;
T25.2 `cb8e0aa1b0bb3bb9d87a1c499dd6cca46789f4ae`;
T25.3 `0f15778c80e7e05e165531a5408dccb397f2c8fb`;
shared-window focus follow-up `5e034847329c880b4cbff876b3d97cc7183067b8`.
Fourteen new path journeys and two configured accept/bound regressions cover
selected/typed paths, directory continuation, dirty canonical reuse, frozen
policies, real permission errors, unsupported names and symlink destinations.
Two RGB/NO_COLOR acceptance PTYs, two navigation PTYs and the focused absent-path
PTY preserve exact fixture bytes and create no files. Focused source checks pass;
added or changed functions have nesting at most two. Controller evidence
`V27U3KLXUNXUICU6XL62WINBHD` passed all eleven ordered checks on
`e86c5451203cac1f64279a4cc6a6e29762ed8989`: 260 native tests and five owned
PTY sessions, zero skips. PR #74 merged through Rebase + Fast-forward into dev
at that exact head; Acta merge event `01M5Z58JW4P9K1H1A796P17QM9` confirms
integration. Slice 25 of 30 completed.
Report: [Slice 25](../report/slices/rev-configuration-ui/slice-25-find-file-integration.md).
PR: [#74](https://forge.adrianpk.com/adrianpk/rev/pulls/74).

### Slice 26: Publish exhaustive configuration reference

Contracts: CFG05-CFG06, CFG20, CFG22, UI01-UI04. Tickets: TKT-20261027103714.
Details and checks: [Slice 26](../plan/rev-configuration-ui.md#slice-26-publish-exhaustive-configuration-reference).

- [x] T26.1: Create docs/reference/config.toml with every accepted option, all 25 palette values active, complete default bindings for all three contexts and commented supported optional include/language/association examples. Keep all comments English.
- [x] T26.2: Document types, bounds, units, scopes, defaults, precedence, invalid-config fallback, startup-only semantics, terminal limitations and the three new UI flows in existing docs. Keep future controls out of the operational sample.
- [x] T26.3: Add deterministic schema/default/sample and command-map consistency checks. Validate actual runtime effects separately; parsing a sample is not UI acceptance. Preserve the concise README product/build style.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T26.1 `f71b106df0511709e9bbfe097fc55257f59b13a1`;
T26.2 `030d2f2ed3d9231293a1a61e6a533944e14184f5`;
T26.3 `b965ba71ecc3223a70be78a5ae3412a53c33e0d2`;
startup query-geometry follow-up `6cfd35d479d313662cbbdbaa514c871d4073aba5`.
The complete sample has 25 active RGB roles and 109 default context bindings.
All 59 catalog entries describe supported types, bounds, defaults and scopes.
Eight native reference tests and seven named drift probes pass, along with
focused Clippy/source and documentation-link checks; added Rust functions have
nesting at most two. The existing startup-consumer check locates the Echo query
above candidates and verifies its cursor. Schema/default evidence remains
separate from UI acceptance.
Controller evidence `Z6U4KHAMZQDKNIIFHVKN5LOHP3` passed all seven ordered
checks on `3a8591e5bee3c38b1abafc3d8a3d1163a045026d`: 78 native tests and
seven named drift probes, zero skips. PR #75 merged through Rebase + Fast-forward
into dev at that exact head; Acta merge event `01M5Z6FFTEJSGBQYA6HPBF1ERC`
confirms integration. Slice 26 of 30 completed.
Report: [Slice 26](../report/slices/rev-configuration-ui/slice-26-configuration-reference.md).
PR: [#75](https://forge.adrianpk.com/adrianpk/rev/pulls/75).

### Slice 27: Exercise combined terminal workflows

Contracts: CFG22, UI01-UI04, baseline R03-R07. Tickets: TKT-20261027103715.
Details and checks: [Slice 27](../plan/rev-configuration-ui.md#slice-27-exercise-combined-terminal-workflows).

- [x] T27.1: Add real isolated PTY journeys for startup palette/tab/key overrides, missing and invalid config, M-x candidate/history selection, find-file directory/Unicode completion and new filename behavior.
- [x] T27.2: Combine tree/nested panels, shared buffer policies, horizontal and vertical boundaries, focus and tiny/growing resize. Assert visible behavior and retained document text, not only escape substring presence.
- [x] T27.3: Use bounded fixtures/timeouts, zero skips and explicit child cleanup after completion. Verify normal/error terminal color/mode restoration and no disk entry created by observation.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T27.1 `1389238803bfea95120cd4632784c3e354209100`;
T27.2 `a9232206e549dad33269bd0740d366c9f2a97385`;
T27.3 `9d431003f671edead10fa8529ad8f741dc6456af`.
Eight focused owned PTY sessions pass: configured and workspace RGB/NO_COLOR,
missing/invalid configuration, output error and unwind. Assertions cover visible
rows/cursors, exact fixture bytes/entries, M-x/path ownership, shared policies,
zero/tiny/growing geometry, restoration order and reaped children/closed PTYs.
Session/output/snapshot bounds remain explicit; no cases are skipped.
All seven ordered checks passed on `27629069653ee043ff0cb5be935fa16f5353f76d`, controller validation
`LKBX5GARCR45ND5OOC5K46TYG5`, including 119 native tests and eight PTY sessions
with zero skips. Canonical PR #76 integrated by Rebase + Fast-forward into dev
at `27629069653ee043ff0cb5be935fa16f5353f76d`. Slice 27 of 30 completed.
Report: [Slice 27](../report/slices/rev-configuration-ui/slice-27-configuration-ui-journeys.md).
PR: [#76](https://forge.adrianpk.com/adrianpk/rev/pulls/76).

### Slice 28: Harden cross-feature failure boundaries

Contracts: CFG01-CFG22, UI01-UI04. Tickets: TKT-20261027103715.
Details and checks: [Slice 28](../plan/rev-configuration-ui.md#slice-28-harden-cross-feature-failure-boundaries).

- [x] T28.1: Complete adversarial coverage of include cycles/limits/read failures, layered key collisions, association overlaps, whole-candidate rollback and candidate-list stale filesystem state.
- [x] T28.2: Exercise cross-width rectangle copy/kill/yank, successful and failed file/directory rename policies, CRLF/BOM fidelity, shared views and undo/saved-state truth with configured limits.
- [x] T28.3: Repair only bounded defects in approved owners revealed by these checks; no new features or duplicated blanket test suite. Map each failure boundary to the requirement it can falsify.
- [x] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T28.1 `a56ad61f17e6092af3c423ad38fce33a1c91c335`; T28.2 `c918b3a0945a07ef26756a1f6ac7e45b3ba0a5d6`;
T28.3 `37ef43b857d9a8a37231855ef236e5850097ecff`.
Focused configuration/include/key/association, stale path, cross-width rectangle
and configured rename/history checks pass. Existing graph fixtures are extended;
four new parameterized native tests cover changed filesystem contents/kinds,
formatted shared transfer and real rename failures/success with bounded history.
The report maps each boundary to the requirement it can falsify. No runtime
repair is required by these checks. All eight ordered checks passed on `dd02eb4fc85630c10565f28a98a5dd16f7a23626`, controller validation
`242PT4VM24GVANC7X5NHVC7YGS`, including 182 native tests with zero failures or
ignored tests. Canonical PR #77 integrated that exact head by Rebase +
Fast-forward into dev. Slice 28 of 30 completed.
Report: [Slice 28](../report/slices/rev-configuration-ui/slice-28-configuration-ui-boundaries.md).
PR: [#77](https://forge.adrianpk.com/adrianpk/rev/pulls/77).

### Slice 29: Bound startup and completion work

Contracts: CFG08, CFG17-CFG19, UI04, baseline R32-R35. Tickets: TKT-20261027103715.
Details and checks: [Slice 29](../plan/rev-configuration-ui.md#slice-29-bound-startup-and-completion-work).

- [x] T29.1: Extend reproducible performance fixtures for maximal admitted include graphs, directory candidate limits/filter reuse, panel redraw with tabbed Unicode and syntax cutoff behavior.
- [x] T29.2: Check event-driven idle behavior, bounded config/candidate allocations and absence of recursive directory work or filesystem reads on draw. Retain the original load/search/edit/scroll/syntax budgets and fixtures.
- [x] T29.3: Record environment, workload sizes, measured values and enforced budgets. Fix demonstrated regressions within scope; do not add speculative tuning or weaken a failing budget.
- [ ] Record successful exact-head checks, report, canonical PR and verified dev integration.

Task commits: T29.1 `651051c5f19f`; T29.2 `f6534d4974f2`.
All six focused performance selections pass: seven native workloads and one
configured PTY with editing/prompt idle. Exact graph/directory limits, zero
cached/recursive fixture open/access events, parser cutoff controls, unchanged
fixture hashes and timing/RSS budgets are enforced. The report records local
hardware/compiler, sample counts, measurements and explicit evidence limits.
Historical performance owners and budgets remain unchanged. Final readiness
requires controller evidence for the clean PR head; dev integration is pending.
Report: [Slice 29](../report/slices/rev-configuration-ui/slice-29-configuration-ui-performance.md).

### Slice 30: Verify delivery acceptance and references

Contracts: CFG01-CFG22, modified R01/R17/R20/R22, UI01-UI04, baseline A01-A20. Tickets: TKT-20261027103715.
Details and checks: [Slice 30](../plan/rev-configuration-ui.md#slice-30-verify-delivery-acceptance-and-references).

- [ ] T30.1: Add a requirement-to-native/PTY/performance evidence map for this delivery and require it from scripts/check-all.sh, retaining all original eight checks and A01-A20 evidence protections. Detect missing cases, skipped suites, stale candidate identity or absent slice reports.
- [ ] T30.2: Reconcile selected baseline statements/default tables and rectangle references with accepted config/interaction contracts, keeping baseline history distinct from the new implementation evidence. Verify sample, docs and all eleven ticket outcomes agree with actual delivered behavior.
- [ ] T30.3: Prepare all 30 reports, resolved ticket paths and acceptance prerequisites; do not run the full gate in the worker. After controller-owned exact-dev full-gate success, close only approved artifacts using that evidence, record Acta outcome and stop without main promotion.
- [ ] Record successful exact-head checks, report, canonical PR and verified dev integration.

## Delivery-set closure

- [ ] All 90 tasks and all 30 slice reports integrated through canonical PRs.
- [ ] All eleven current tickets solved with actual task/PR/check evidence.
- [ ] CFG01-CFG22, four modified baseline requirements and UI01-UI04 mapped to passing acceptance evidence.
- [ ] Exhaustive sample/default/schema/keymap checks agree; unsupported future settings are absent.
- [ ] Controller-owned scripts/check-all.sh passes on the clean exact dev candidate with zero skipped required checks.
- [ ] Record gate identity/candidate, final documentation reconciliation and Acta outcome.
- [ ] Leave the future conversion ticket open; stop before main promotion.
