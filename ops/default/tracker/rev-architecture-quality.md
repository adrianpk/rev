# Architecture Quality Tracker

Status: In progress
Date: 2026-11-09
Delivery set: rev-architecture-quality
Slice strategy: behavior-first
Approved: owner requested one architecture remediation set with strict pre-commit Clippy/linter enforcement
Planning baseline: abbf0c21eeae1680c02c7a477fa0894d7996a393
Specification: [Architecture Quality and Commit Checks](../spec/editor-architecture-quality.md)
Review: [Architecture and Ownership Nit Review](../report/rev-architecture-nit-review.md)
Concern: [Pending Editor Work](../README.md)
Active slice: Slice 10 - Staged enforcement and acceptance
Plan: [Architecture Quality Delivery](../plan/rev-architecture-quality.md)
Execution authorized: one ten-slice Looper run, automatic canonical integration into dev and local hook setup
Progress owner: selected Looper run; no second runtime tracker

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Quality foundation | chore/rev-architecture-quality-01-foundation | chore(lint): establish complete native workspace quality checks | ops/default/report/slices/rev-architecture-quality/slice-1-foundation.md |
| Slice 2 | Plugin and SDK ownership | refactor/rev-architecture-quality-02-plugins | refactor(plugins): clarify request ownership and avoid redundant payload copies | ops/default/report/slices/rev-architecture-quality/slice-2-plugins.md |
| Slice 3 | Core editing and projection | refactor/rev-architecture-quality-03-core | refactor(core): enforce ownership and complexity in native editing | ops/default/report/slices/rev-architecture-quality/slice-3-core.md |
| Slice 4 | Configuration and persistent concerns | refactor/rev-architecture-quality-04-storage | refactor(storage): preserve explicit ownership across native state concerns | ops/default/report/slices/rev-architecture-quality/slice-4-storage.md |
| Slice 5 | Git operation boundaries | refactor/rev-architecture-quality-05-git | refactor(git): separate operation-specific preparation and observation | ops/default/report/slices/rev-architecture-quality/slice-5-git.md |
| Slice 6 | External process workflows | refactor/rev-architecture-quality-06-processes | refactor(process): clarify IO ownership and bounded cleanup paths | ops/default/report/slices/rev-architecture-quality/slice-6-processes.md |
| Slice 7 | Org workflow decomposition | refactor/rev-architecture-quality-07-org | refactor(org): keep native document workflows within explicit owners | ops/default/report/slices/rev-architecture-quality/slice-7-org.md |
| Slice 8 | Session interaction and completion owner | refactor/rev-architecture-quality-08-interaction | refactor(session): own completion transitions and simplify native dispatch | ops/default/report/slices/rev-architecture-quality/slice-8-interaction.md |
| Slice 9 | Fixtures and developer examples | refactor/rev-architecture-quality-09-fixtures | refactor(test): keep acceptance fixtures within native quality contracts | ops/default/report/slices/rev-architecture-quality/slice-9-fixtures.md |
| Slice 10 | Staged enforcement and acceptance | chore/rev-architecture-quality-10-enforcement | chore(lint): enforce strict staged commit checks and document architecture rules | ops/default/report/slices/rev-architecture-quality/slice-10-enforcement.md |

## Slice tasks

## Slice 1: Quality foundation

State: delivered
Dependency: approved preparation and controller preflight
Branch: chore/rev-architecture-quality-01-foundation
PR: #15 - https://forge.adrianpk.com/rev/rev/pulls/15
Implementation: 3c76d32116e739f26c51abe1e171bd39bb69c877
Report: ops/default/report/slices/rev-architecture-quality/slice-1-foundation.md
Integrated: 79d1c50083e83f365036d89013238f9cb431da9a through canonical PR #15
Validation: all five frozen commands passed through Looper on clean published HEAD 79d1c50083e83f365036d89013238f9cb431da9a

- [x] T1.1: Correct Cargo workspace/member/target discovery in the existing Rust source checker; place cohesive discovery/diagnostic helpers under examples/source_check, retain metrics/legacy arguments, and add nonempty scoped checks plus bounded Clippy admission.
- [x] T1.2: Introduce scripts/check-quality.sh with the selected ownership/representation profile, truthful scoped coverage and no Python requirement; repair duplicate macros test support and keep complete baseline all-target Clippy green.
- [x] T1.3: Add real parser/discovery/exception/scope fixtures and an evidence-backed criteria map; cover new tooling itself without presenting unreviewed domains as passing full-profile lint.

## Slice 2: Plugin and SDK ownership

State: delivered
Dependency: Slice 1 integrated
Branch: refactor/rev-architecture-quality-02-plugins
PR: #16 - https://forge.adrianpk.com/rev/rev/pulls/16
Implementation: 6b5abf5e3c61e9d7fc4e6ad99cf4d4b409b6683f
Report: ops/default/report/slices/rev-architecture-quality/slice-2-plugins.md
Progress: all SDK/native/session-plugin owners pass the frozen quality selection
Integrated: 2ff6786409c13793f0e1665ccf153fe72d8a2f42 through canonical PR #16
Validation: all five frozen commands passed through Looper on clean published HEAD 2ff6786409c13793f0e1665ccf153fe72d8a2f42

- [x] T2.1: Resolve F4 by separating encoded request payload ownership from pending correlation while preserving permits, deadlines, captured context, causal propagation and exactly-once completion.
- [x] T2.2: Resolve F5 without duplicating keymap observations on unchanged cache hits; complete SDK error/panic contracts, selected ownership/conversion diagnostics and existing source bounds in SDK/plugin owners.
- [x] T2.3: Verify public protocol, typed client, callbacks, cancellation/retirement, keymap precedence and atomic publication using the existing SDK and native plugin fixtures; retain wire/API/state versions.

## Slice 3: Core editing and projection

State: delivered
Dependency: Slice 2 integrated
Branch: refactor/rev-architecture-quality-03-core
PR: #17 - https://forge.adrianpk.com/rev/rev/pulls/17
Report: ops/default/report/slices/rev-architecture-quality/slice-3-core.md
Implementation: c32cd4fa544f4201811e97c9ea582554f4044c4c
Progress: source bounds and scoped Clippy pass; maintainer-approved fixture corrections resolve the four reproduced baseline display/gutter failures without runtime changes
Integrated: 09e5ea218c54675bdf44c53d2c2443efa243e8a1 through canonical PR #17
Validation: all four frozen commands passed through Looper on clean published HEAD 09e5ea218c54675bdf44c53d2c2443efa243e8a1

- [x] T3.1: Correct selected ownership/representation diagnostics and existing source bounds in buffers, source coordinates, views, windows, display, syntax, gutter/tree and terminal-emulation owners.
- [x] T3.2: Extract coherent decision/preparation/projection functions while preserving source units, revision checks, local history, shared views, caches and existing work/resource ceilings.
- [x] T3.3: Validate checked editing, Unicode/grapheme/visual projection, selection, syntax and view restoration with the actual native owners; record any legitimate invariant exceptions narrowly.

## Slice 4: Configuration and persistent concerns

State: delivered
Dependency: Slice 3 integrated
Branch: refactor/rev-architecture-quality-04-storage
PR: #18 - https://forge.adrianpk.com/rev/rev/pulls/18
Implementation: 9417c9276874476408cc9d698b9287d903995ec7
Report: ops/default/report/slices/rev-architecture-quality/slice-4-storage.md
Progress: scoped Clippy and source bounds pass for 81 files; all 378 native integration cases and documentation checks pass
Integrated: e47e295d4bac66e2a096405f88da7a61c25b486f through canonical PR #18
Validation: all four frozen commands passed through Looper on clean published HEAD e47e295d4bac66e2a096405f88da7a61c25b486f

- [x] T4.1: Correct selected borrowing/conversion/source-policy diagnostics in configuration, file IO/watch, contexts, locations, bookmarks and their native persistence adapters.
- [x] T4.2: Keep configuration/data/state/cache separation, private namespace paths, locks, CAS and uncertain-write evidence; preserve coordinated publication and avoid a generic store framework.
- [x] T4.3: Exercise invalid/stale/readonly sources, partial writes, concurrent records, project/workspace boundaries and existing file-watch retirement through native fixture paths.

## Slice 5: Git operation boundaries

State: delivered
Dependency: Slice 4 integrated
Branch: refactor/rev-architecture-quality-05-git
PR: #19 - https://forge.adrianpk.com/rev/rev/pulls/19
Implementation: 068ddbe672f24c39e9ab59935933333f1dd94fd9
Integrated: 068ddbe672f24c39e9ab59935933333f1dd94fd9 through canonical PR #19
Report: ops/default/report/slices/rev-architecture-quality/slice-5-git.md
Progress: all 112 selected native and session Git sources satisfy source bounds and the selected Clippy profile; all 242 Git integration cases, documentation and diff checks pass; the T5.3 process-owner fixture retains every original assertion; canonical PR #19 is merged into dev
Validation: all four frozen commands passed through Looper on clean published HEAD 068ddbe672f24c39e9ab59935933333f1dd94fd9

- [x] T5.1: Split the verified oversized native Git poll/dispatch/operation handlers into cohesive operation-family and observation owners under the existing modules.
- [x] T5.2: Resolve selected ownership/conversion and five/three/ten diagnostics in native and session Git code; retain immutable observations, process ownership, authority checks, hook/signing policy and mutation preconditions.
- [x] T5.3: Validate status/index/commit/history/remote/sequence/conflict paths through existing disposable repositories, preserving all negative assertions and exact native command behavior.

## Slice 6: External process workflows

State: delivered
Dependency: Slice 5 integrated
Branch: refactor/rev-architecture-quality-06-processes
PR: #20 - https://forge.adrianpk.com/rev/rev/pulls/20
Report: ops/default/report/slices/rev-architecture-quality/slice-6-processes.md
Integrated: 2064a704049fe66c1e44e68a67b2b6e72024649a through canonical PR #20
Progress: all 117 selected process/AI/Pi/LSP/shell/terminal files pass the source guards and selected Clippy profile; blocking Clippy diagnostics decreased from 196 to zero and source findings from 34 to zero; 693 selected-profile diagnostics remain deferred outside this slice
Validation: all four frozen commands passed through Looper on clean published HEAD 2064a704049fe66c1e44e68a67b2b6e72024649a (UMFBE7KRBFCBKKOG2ZRU2BATLG); 213 native cases pass with the same three upstream-acquisition ignores; documentation and diff checks pass
Decision: implemented the maintainer-approved Slice 6 `History { state: Option<PathBuf>, reset: bool }` exception: `Operation::retained` now has five parameters, and only its two existing callers in `src/session/ai/capture.rs` and `tests/pi/sessions.rs` are adapted; behavior, assertions, protocols, resource bounds, frozen checks and the full gate are preserved

- [x] T6.1: Correct selected ownership, narrowing, guard-lifetime and source-policy findings in AI/Pi, LSP, shell, integrated-terminal and native process owners plus their session adapters.
- [x] T6.2: Keep worker IO away from input/rendering, shared mailboxes intentional, owned process cleanup/cancellation finite and generation-bound; preserve existing environment/credential redaction and terminal restoration.
- [x] T6.3: Exercise real owned child fixtures and native request/reload/fallback/terminal relocation boundaries without changing protocol capabilities, external configuration or unrelated host processes.

## Slice 7: Org workflow decomposition

State: delivered
Dependency: Slice 6 integrated
Branch: refactor/rev-architecture-quality-07-org
PR: #21 - https://forge.adrianpk.com/rev/rev/pulls/21
Report: ops/default/report/slices/rev-architecture-quality/slice-7-org.md
Integrated: 954fed70cf96a00ce4711d6a13b0cad1b469630e through canonical PR #21
Decision: maintainer approved the initial-observation correction; the plan records shared stable/unobserved fixture preparation, addressed-view initialization and syntax-policy preservation. The correction is implemented with assertions, in-flight/cancellation cases, guards, deadlines and budgets intact; all four frozen commands passed on the clean published PR head before integration.
Progress: all four source findings are corrected across 431 selected files; native preparation borrows inspected inputs, custom agenda publication retains the supplied view without a second clone, and native numeric widths preserve admitted ranges and resource ceilings; session preparation borrows inspected inputs while prompts/jobs retain their existing owners; table export moves its checked target into the save; selected Clippy diagnostics decrease from 316 to zero, with 377 deferred outside this slice
Validation: all four frozen commands passed through Looper on clean published HEAD 954fed70cf96a00ce4711d6a13b0cad1b469630e (XCFEU34ADA4ANKN2JMAFMFHJCG); scoped quality has zero blockers across 431 files; all 791 native Org cases pass across eight targets with no ignores; documentation, diff and canonical PR contract checks pass
Correction: stable/unobserved fixtures preserve explicit pending and cancellation coverage; addressed agenda and saved-clock views remain stable; native styles honor syntax admission. The approved additional core owner check passes. The report retains earlier failed batches and their corrective checks.

- [x] T7.1: Correct selected lint/source findings in native Org and its session document/agenda/capture/table/export/execution adapters, keeping parsing and publication responsibilities distinct.
- [x] T7.2: Extract cohesive validation, preparation and outcome handlers while preserving Org source text, navigation, folding/indentation, native consent, state ownership and document limits.
- [x] T7.3: Run relevant existing native Org integration cases, preserving public command names, keybindings, timestamps, agenda/capture targets, formulas and code-block behavior.

## Slice 8: Session interaction and completion owner

State: delivered
Dependency: Slice 7 integrated
Branch: refactor/rev-architecture-quality-08-interaction
PR: #22 - https://forge.adrianpk.com/rev/rev/pulls/22
Report: ops/default/report/slices/rev-architecture-quality/slice-8-interaction.md
Validation: all four frozen ordered commands passed at 6edcb6e81c6bf168c8ed525f5c48a258933150ba in ZZ4B65BSUURXCTIABTXRHS6FO5; canonical PR #22 merged that exact HEAD into dev
Progress: F6 and T8.2 are complete. Frozen scoped quality at the published checkpoint passes all 381 selected files with zero blocking Clippy diagnostics and zero source findings; 205 selected-profile diagnostics remain deferred outside this scope. Command metadata retains all 119 Git and 52 macro variant names/descriptions; inherited maps preserve compilation order and public representation. The maintainer-approved fixture correction reconciles the 16 command, two modal and one command-usage failures with existing numeric-prefix outcomes, macro commands, descriptions, catalog ordering and the tree parent row. All 482 frozen native cases and 15 command-usage cases pass, preserving semantic/negative coverage, consent, freshness, source/history, deadlines and budgets. Canonical PR #22 targets dev and records the immutable report-introduction link and frozen commands. Final PR-reference HEAD validation passed and canonical PR #22 is merged; the unchanged delivery-set gate remains controller-owned.

- [x] T8.1: Resolve F6 with a cohesive completion concern owner and explicit admission/retirement transitions; keep public completion observations and native editing/context semantics intact.
- [x] T8.2: Correct remaining command/input/macro/session selected diagnostics and source bounds, including retained macro body identity without raw-pointer-type casts or identity-spelling workarounds.
- [x] T8.3: Verify both interaction profiles, prefix arguments, semantic macro replay/history, stale completion, transient ownership and source/view/context switches using native regression suites.

Decision: the maintainer authorizes diagnosis, bounded technical corrections,
validation and same-run continuation without repeated routine approval pauses.
The report's fixture correction, including its two flow/guard files, is approved
against existing behavior;
negative coverage, consent, source/history, resource bounds and frozen commands
remain required. Both the fixture and final PR-reference commits passed all four frozen checks; canonical integration is verified.

## Slice 9: Fixtures and developer examples

State: delivered
Dependency: Slice 8 integrated
Branch: refactor/rev-architecture-quality-09-fixtures
PR: #23 - https://forge.adrianpk.com/rev/rev/pulls/23
Report: ops/default/report/slices/rev-architecture-quality/slice-9-fixtures.md
Integrated: 1faac03dee6c434fca043fcfb3cd374468038c4d through canonical PR #23
Final validation: all five frozen checks passed in 5YBIGE4TEDA4NXJB4RNFFCAX2A on that clean published HEAD
Validation: all five frozen commands passed in order on clean published implementation HEAD 5a2d56a3d858484a1bc6afa1ae6614b046e72099: scoped quality has zero blocking Clippy diagnostics and zero source findings across 704 selected files and 70 roots; all 400 command/macro/plugin cases, the SDK doc test and documentation catalogs/pages/links pass. All 2889 existing test identities and 19672 assertion sites are preserved. Canonical PR #23 uses the approved title/branch and immutable report link. PR-reference validation 67BIF33WCHYITLZILMJUMWYVWF on 3bd8ac436a933cf0998f3c84d8c2f4159f320256 passed quality but failed signed_movement (203 command cases passed, one failed). The bounded fixture correction now services cooperative counted movement through its existing settle helper; all 28 prefix cases pass with assertions, inputs and deadlines preserved. Fresh unchanged ordered Looper validation is required for the corrected candidate before controller acceptance.
Progress: measured all 704 selected Rust files and 70 Cargo roots. The first ownership cohort reduces blocking Clippy diagnostics from 201 to 99 by making shared clones explicit, moving unused fixture values and preserving eager concurrent startup before joins. Test identities, assertion-site counts, snapshot/consent/resource contracts and deadlines remain unchanged. No selected clone_on_ref_ptr, redundant_clone or needless_collect findings remain; four selected-profile diagnostics stay deferred outside this scope. The second cohort resolves the remaining selected Clippy diagnostics with borrowed configuration/RPC/capture/catalog/key-phase observations and checked numeric fixture conversions. Informational executable-size ratios retain three narrow reasoned precision expectations; all resource assertions and exact byte measurements remain unchanged. All original input literals, case identities and assertion-site counts are preserved across its 62 source files. The publication/probe cohort clears nine source findings through explicit Markdown/output phases and owned native probe timings; both remaining Cargo-root safety declarations are present. Existing publication/catalog unit bodies and wrapping inputs/order are unchanged. All 13 unit cases and five bounded wrapping scenarios pass, and all 279 canonical publication output files match the clean baseline byte for byte. The geometry cohort clears 20 more source findings through independent policy/viewport setup and frame, palette, label and hit-map checks. All 59 existing case identities, 487 assertion sites and text/numeric literal multiplicities remain unchanged in its 12 Rust files; all 260 affected native cases pass with the original bounds. The dispatch cohort clears 12 more source findings through route, chord replay, generated-buffer frame, counted-row reference, click-policy and parent-fault checks. Its 11 Rust files preserve all 115 existing case identities, 668 assertion sites and text/numeric literal multiplicities; all 11 affected native cases pass, including complete direct/named command parity. The admission cohort clears seven more source findings through explicit configuration combinations, competing writes, capacity/staging fixtures, artwork checks and grammar/query catalogs. Its six Rust files preserve all 39 existing case identities, 342 assertion sites and text/numeric literal multiplicities; all seven affected native cases pass with the original resource and retention boundaries. The Git/exit cohort clears 14 more source findings through explicit conflict, captured-review, index/metadata, autosquash, worktree/control and cleanup phases. Its 12 Rust files preserve all 69 existing case identities, 593 assertion sites and text/numeric literal multiplicities; all 13 affected native cases and the final path-borrow recheck pass with original consent, limits and deadlines. The LSP cohort clears five more source findings through borrowed marked documents, local/connection resolve retirement, controlled installer fault preparation and native launch-restoration checks. Its five Rust files preserve all 27 existing case identities, 193 assertion sites and text/numeric literal multiplicities; all 85 LSP/offline installer cases and the final 22-case borrow recheck pass, with the three existing explicit upstream-acquisition ignores unchanged. The plugin cohort clears all ten remaining test-source findings through explicit PTY/workload/idle phases, borrowed keymap setup, oversized store catalogs and separate raw buffer/event/SDK peer owners. Its five Rust files preserve all 15 case identities, 92 assertion sites and text/numeric literal multiplicities; all 86 plugin cases and all 582 selected test/SDK-test source files pass with original wire, consent, callback, deadline and resource boundaries. The tree/gutter example cohort clears 13 more source findings through explicit configuration, workspace/style/hit-map, geometry, copy and PTY phases. All 75 assertion sites and text/numeric literal multiplicities remain unchanged across five example files. Gutter native and both PTY profiles pass before/after; all four file-tree NO_COLOR settings pass with identical output. File-tree native/PTY fixed-row expectation failures already exist at c3428be18edabe81aaa828c883331449ea0ae200 and remain unchanged, with an identical native failed assertion and byte-identical PTY diagnostic. The ranking/hint example cohort clears ten more source findings through explicit promotion, recall, restart/paired dispatch, Org origins, palette geometry and modal-equivalence phases. All 99 assertion sites and text/numeric literal multiplicities remain unchanged across six Rust files. Ranking restart journeys under both profiles and paired publishers pass before/after with original process, idle and time/byte bounds. Ranking native and hint native/PTY expectation failures already exist at cf448f6940a5e906f74c65aace8b5a0b8cdc7731 and remain unchanged; native failed assertions/values match and hint PTY failure diagnostics are byte-identical. The wrapping-navigation cohort clears three more source findings through directory/file/split, visual editing/search, resize/wrap and bounded reader/PTY phases. Both Rust files preserve all 58 assertion sites and text/numeric literal multiplicities. Its real PTY journey passes before/after with original eight-second/1-MiB ownership, 350-ms idle assertion and unchanged inputs; both runs emit 75,608 output bytes. The native directory-buffer expectation already fails at 590ea30976e9abff0b0e1e22b1d055dee29cb0b0 and retains the same assertion and BufferId values. The bookmark example cohort clears four source findings through explicit native record/store/source lifecycle, measured projection/relocation and PTY restart phases. Three Rust files preserve all 80 assertion sites and text/numeric literal multiplicities. Native, 1,024-record resource and real restart PTY journeys pass before/after with original cancellation, session-drop, 512-batch/128-pass and eight-second/1-MiB boundaries. The Projects/Workspaces cohort clears 11 source findings through catalog seeding, owned storage publication, native cancellation/restore/capture and serial PTY startup/membership/conversion/restart phases. Three Rust files preserve all 81 assertion sites and text/numeric literal multiplicities. Native resource/capacity/retirement and non-modal PTY checks pass before/after with identical catalog/output byte totals. The modal PTY retains the existing eight-second deadline failure; final frame diagnostics differ, and no bound or expectation is relaxed. The Pi/interoperability cohort clears the last five source findings through explicit fixture/configuration, native reload/history/reset, bounded PTY and owned source/result lifecycle phases. Five Rust files preserve all 75 assertion sites and text/numeric literal multiplicities. Both native Pi profiles, both Pi PTYs and existing-client interoperability pass before/after with unchanged bounds; manual live mode is not executed. All 704 selected files and 70 roots now pass scoped quality, with all 2889 existing test identities and 19672 assertion sites preserved per file. All five frozen checks pass on the clean implementation HEAD, and canonical PR #23 is open against dev. The first PR-reference run exposed an immediate-position observation in the unchanged signed-movement fixture; five existing settle calls correct that observation, and all 28 prefix cases pass. Fresh final-HEAD validation and controller review/integration follow.

- [x] T9.1: Resolve remaining selected diagnostics and existing source bounds in native test and example code, including every Cargo root and shared fixture ownership.
- [x] T9.2: Keep existing case identities, semantic/resource/PTY assertions and varied Unicode/numeric inputs; extract cohesive setup/check steps instead of hiding branches in macros, closures or unrelated parameter bundles.
- [x] T9.3: Reconcile all-target compilation, native source coverage and developer/publication helper behavior; verify representative macro/plugin/command journeys and SDK documentation before final global adoption.

## Slice 10: Staged enforcement and acceptance

State: reviewing
Dependency: Slice 9 integrated
Branch: chore/rev-architecture-quality-10-enforcement
PR: pending
Report: ops/default/report/slices/rev-architecture-quality/slice-10-enforcement.md
Progress: inherited strict policy, complete source discovery, canonical commands and exact staged-index enforcement are implemented. All 28 real hook fixtures, nine native checker tests, the existing hello workflow, documentation and User Guide checks pass. The actual staged hook passes with all 1836 files and 73 Cargo roots, zero diagnostics and unchanged index bytes. Timings and F1-F7 reconciliation are in the report. Canonical PR reference and final clean-HEAD ordered evidence follow; hook activation and full gate remain controller-owned.
Validation: final published PR-HEAD evidence is recorded through the unchanged five-command Looper sequence

- [x] T10.1: Adopt the final inherited workspace Clippy/source policy, exact method restrictions and reasoned local expectations; add canonical Makefile commands and concise repository AGENTS/local concern guidance.
- [x] T10.2: Implement the nonmutating index-snapshot hook, exclusively owned per-worktree reusable cache and explicit install procedure; verify real Git partial staging, manifests/policy changes, deletes/renames, conflicts, concurrency and unchanged user content.
- [x] T10.3: Recheck F1-F7 and the criteria map against final code, measure cold/warm checks, document use, run focused enforcement acceptance and prepare the unchanged controller gate/Markdown closure.

## Final acceptance

- [ ] F1-F7 and complete selected source/lint findings corrected with evidence.
- [ ] Exact staged-index, partial-staging, conflict and concurrent-check journeys pass without content changes.
- [ ] Final workspace profile covers both packages/all applicable targets; no baselines or broad suppressions.
- [ ] Integrated strict lint/hook checks pass and local hook installation is verified.
- [ ] The unchanged full gate passed on the exact clean integrated candidate.
- [ ] Ten reports delivered and permitted Markdown closure committed/pushed.
