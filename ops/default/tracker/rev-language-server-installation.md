# Language Server Installation Tracker

Status: Approved
Date: 2026-11-02
Delivery set: rev-language-server-installation
Slice strategy: behavior-first
Approved: product owner, 2026-11-02; capability, preparation and autonomous Looper implementation through dev
Planning baseline: `93b1bd0aa10b35f62d28db5196dd149f2f134bbd`
Spec: [Language Server Installation](../spec/editor-language-server-installation.md)
Model: [Installation Model](../spec/editor-language-server-installation-model.md)
Concern: [Pending Work Index](../README.md)
Full gate: `cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --lib --test configuration --test lsp --test lsp_installation --test completion --test language_assets -- --test-threads=1 && cargo test --locked --test commands palette::catalog_reference -- --exact && cargo test --locked --test modal contexts:: -- --test-threads=1 && bash scripts/check-lsp-installation.sh && cargo run --locked --quiet --example documentation-check && bash scripts/build-user-guide.sh && git diff --check`
Active slice: Slice 5 — Remaining optional recipes
Plan: [Installation Delivery](../plan/rev-language-server-installation.md)

## Execution ownership

The owner approved preparation and autonomous execution on 2026-11-02. Looper
owns worker progress, canonical slice integration, the exact-candidate gate and
closure. Slices 1–4 are delivered through PRs #222–#225. Slice 5 is active; later slices remain pending.

The explicit open-language acceptance is mandatory: the initial recipe catalog
and bundled grammars cannot constrain generic server registration or language
associations. Preserve the existing semantic-or-native source policy.

## Canonical slice map

| Slice | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- |
| Slice 1 | `feat/rev-language-server-installation-01-registration` | `feat(lsp): register and select existing language servers` | `ops/default/report/slices/rev-language-server-installation/slice-1-registration.md` |
| Slice 2 | `feat/rev-language-server-installation-02-catalog` | `feat(lsp): guide server setup and language associations` | `ops/default/report/slices/rev-language-server-installation/slice-2-catalog.md` |
| Slice 3 | `feat/rev-language-server-installation-03-managed-rust` | `feat(lsp): install and verify managed Rust servers` | `ops/default/report/slices/rev-language-server-installation/slice-3-managed-rust.md` |
| Slice 4 | `feat/rev-language-server-installation-04-go-web` | `feat(lsp): add managed Go and web server recipes` | `ops/default/report/slices/rev-language-server-installation/slice-4-go-web.md` |
| Slice 5 | `feat/rev-language-server-installation-05-catalog-recipes` | `feat(lsp): add optional Markdown Ruby Python and Odin recipes` | `ops/default/report/slices/rev-language-server-installation/slice-5-catalog-recipes.md` |
| Slice 6 | `feat/rev-language-server-installation-06-lifecycle` | `feat(lsp): manage server updates cancellation and removal` | `ops/default/report/slices/rev-language-server-installation/slice-6-lifecycle.md` |
| Slice 7 | `feat/rev-language-server-installation-07-acceptance` | `docs(lsp): document and verify guided server management` | `ops/default/report/slices/rev-language-server-installation/slice-7-acceptance.md` |

## Tasks

### Slice 1: Existing server registration

State: delivered.
Ticket: [Existing server registration](../ticket/solved/20261102111448-register-existing-language-servers.md)
Requirements: INS01, INS03, INS04, INS05.
Dependencies: delivered baseline.
Branch: `feat/rev-language-server-installation-01-registration`
PR: `#222`.
Report: `ops/default/report/slices/rev-language-server-installation/slice-1-registration.md`

- [x] T1.1: Add preferred-server/native-only and literal environment fields, whole-candidate validation, root provenance and a format-preserving conflict-checked settings writer.
- [x] T1.2: Expose guided existing/custom registration and scoped live application through the shared command owner; preserve runtime LSP enablement, exclusions, editor profile, includes and unrelated settings.
- [x] T1.3: Cover grammar-independent registration, source switching, configuration conflicts and symlink publication through native integration tests.

T1.1 development checks passed: `cargo check`,
`cargo test --locked --test lsp_installation -- --test-threads=1` (5 cases),
`cargo test --locked --test configuration lsp_ -- --test-threads=1` and
`cargo test --locked --test configuration template:: -- --test-threads=1`.
T1.1 commit: `4c9e7120460c043776cef6bca0470bfeac1e2a9d`.
T1.2 development checks passed: `cargo check --locked`,
`cargo test --locked --test lsp_installation workflows:: -- --test-threads=1`,
`cargo run --locked --quiet --example documentation-check` and
`cargo test --locked --test commands palette::catalog_reference -- --exact`.
The native workflow checks cover both startup profiles and cancellation at each
prompt. T1.2 commit: `b91b6c51b2189f0bd50427555a38c2e36f4c4f81`.
T1.3 development checks passed:
`cargo test --locked --test lsp_installation routing:: -- --test-threads=1`,
`cargo test --locked --test lsp_installation settings::rejected_settings -- --exact`,
`cargo test --locked --test lsp_installation workflows::scoped_reload -- --exact` and
`cargo test --locked --test lsp_installation settings::profile_comments -- --exact`.
T1.3 report-introduction commit: `ae12c9ef7780ea528fc852286ca1dc9e727ba3ec`.
The initial controller batch passed format and Clippy; configuration fallback
comparisons failed because their expectations omitted retained root provenance.
Full snapshot assertions now include the exact selected root. Focused candidate
and agenda configuration checks passed. Correction commit:
`224ad0a48e2b5745658dbed6ef1eb212b3b81499`.
All four configured commands passed on that clean candidate with controller receipt
`FFYT4OFMW4EX5WIW4E6K4XR7ZZ`. Canonical PR:
https://forge.adrianpk.com/adrianpk/rev/pulls/222
Final receipt `LHVTEKV5E7QRCYH3PX4BI5UJLH` passed all four commands for
`7974577e8bdcc0fb4f562553a86acf95ef6201e6`. Canonical PR #222 merged that exact
commit into dev. Slice 1 of 7 completed.

### Slice 2: Catalog and guided setup

State: delivered.
Ticket: [Catalog and guided setup](../ticket/solved/20261102111449-guide-language-server-setup.md)
Requirements: INS01, INS03, INS04.
Dependencies: Slice 1 integrated.
Branch: `feat/rev-language-server-installation-02-catalog`
PR: `#223`.
Report: `ops/default/report/slices/rev-language-server-installation/slice-2-catalog.md`

- [x] T2.1: Add the finite recipe metadata catalog and lsp-setup/lsp-servers/lsp-select-server interactions, including custom languages and existing executables.
- [x] T2.2: Inherit known associations and selected profile settings; ask for patterns only when needed, and display native-only/external/managed readiness without network discovery.
- [x] T2.3: Cover selector cancellation, both interaction profiles, unknown language identities and registry/catalog independence.

T2.1 development checks passed:
`cargo test --locked --test lsp_installation catalog:: -- --test-threads=1`,
`cargo test --locked --test commands palette::catalog_reference -- --exact` and
`cargo run --locked --quiet --example documentation-check`.
T2.1 commit: `92655f9611e6573982b462edf44ecb534663ea61`.
T2.2 development checks passed: `cargo check --locked`,
`cargo test --locked --test lsp_installation selectors:: -- --test-threads=1` and
`cargo run --locked --quiet --example documentation-check`.
The native workflow covers both profiles, inherited settings/associations,
unknown identities, read-only snapshots and corrupt-registry independence.
T2.2 commit: `0a736a305f15bd4c1e641b2b445b186b587ac5e2`.
T2.3 expanded selector and bounded-result checks passed through
`cargo test --locked --test lsp_installation selectors:: -- --test-threads=1`.
T2.3/report-introduction commit: `fa45a2e6040ebefdf5a019c199ee1702eeedde1c`.
All four configured commands passed on that clean candidate with controller receipt
`LNZASM2O7S5S2G2FFW3IP35PUR`. Canonical PR:
https://forge.adrianpk.com/adrianpk/rev/pulls/223
Final receipt `LLIYHTI5G7DR77NSFEFN34MBQ7` passed all four commands for
`2b44e552c8d9db9ed4aeb95636c0fdc5dd2bc5a0`. Canonical PR #223 merged that exact
commit into dev. Slice 2 of 7 completed.

### Slice 3: Managed Rust installation

State: delivered.
Ticket: [Managed Rust installation](../ticket/solved/20261102111450-install-managed-rust-server.md)
Requirements: INS01, INS02, INS03, INS05, INS06.
Dependencies: Slice 2 integrated.
Branch: `feat/rev-language-server-installation-03-managed-rust`
PR: `#224`.
Report: `ops/default/report/slices/rev-language-server-installation/slice-3-managed-rust.md`

- [x] T3.1: Implement versioned XDG payloads, schema-1 installation facts, OS-owned mutation lock and one bounded asynchronous management worker.
- [x] T3.2: Acquire an exact official Rust server release with integrity verification and finite structured helper processes; initialize/shutdown in an owned empty root before publication.
- [x] T3.3: Publish ready inventory before conflict-checked selection; cover missing prerequisites, corrupt downloads, invalid registry, failed probe and installation while LSP is disabled.

T3.1 development checks passed: `cargo check --locked`,
`cargo test --locked --test lsp_installation storage:: -- --test-threads=1` (4 cases),
`cargo test --locked --test lsp_installation settings:: -- --test-threads=1` (6 cases),
`cargo run --locked --quiet --example source-check -- --source src/config/mutation.rs --source src/lsp/install.rs --source src/lsp/install/record.rs --source src/lsp/install/storage.rs --source src/lsp/install/tree.rs --source src/lsp/install/worker.rs`
and `git diff --check`.
Storage coverage includes ready/preparing admission, invalid-registry preservation,
32-record/2 MiB/1 GiB bounds, escaping paths, source conflicts, cross-process data
locking with distinct state roots, shared settings locking and cancellation ownership.
The initial focused source check identified oversized functions; responsibility
boundaries were split and the unchanged quality limits then passed.
T3.1 commit: `377a474ab07a2486c69edeb8ea47268ba6a63941`.
T3.2 development checks passed: `cargo check --locked`,
`cargo test --locked --test lsp_installation acquisition:: -- --test-threads=1` (3 cases),
`cargo run --locked --quiet --example source-check -- --source src/lsp/install/cache.rs --source src/lsp/install/helper.rs --source src/lsp/install/process.rs --source src/lsp/install/probe.rs --source src/lsp/install/rust.rs`
and `git diff --check`.
Acquisition uses the official release tag, asset URL, declared length and SHA-256;
helpers use structured argv, owned working directories, bounded nonblocking sockets,
finite deadlines and process-group cleanup before releasing the unreaped leader.
Controlled acquisition tests use an owned curl fixture and real gzip/SHA-256 tools,
with a protocol peer payload. They cover missing tools, digest/gzip/probe failures,
missing integrity metadata, oversized assets, output overflow, prior-ready preservation,
empty-root initialize/shutdown/exit and forced cancellation of a TERM-ignoring group.
They establish mechanics, not upstream Rust semantic behavior. The first acquisition
run reached its finite deadline because the parent retained Command's writer descriptors;
releasing those copies after spawn corrected EOF ownership. The focused check passed.
New functions were split by responsibility and passed the unchanged source-quality limits.
The approved, already locked rustix 1.1.5 process APIs are now a direct dependency;
existing Org group observation is reused without changing its signaling behavior.
T3.2 commit: `df770c9dfe475b86e4faa935723ff92a4e48ff74`.
T3.3 development checks passed:
`cargo test --locked --test lsp_installation managed:: -- --test-threads=1`,
`cargo test --locked --test commands palette::catalog_reference -- --exact`,
`cargo run --locked --quiet --example documentation-check`,
`cargo run --locked --quiet --example source-check -- --source src/session/lsp/managed.rs --source src/session/lsp/setup.rs --source src/lsp/catalog/inventory.rs --source src/lsp/install/progress.rs --source src/lsp/install/worker.rs`
and `git diff --check`.
The native journey covers both profiles, C-g and modal Escape, ordinary editing
while work is active, disabled analysis, inherited options/associations, preservation
of a custom preset-named profile, an include-source conflict leaving ready data
unselected, ready-record selection without helpers and corrupt-registry isolation.
The command/reference catalogs agree on 414 commands and 219 settings.
T3.3/report-introduction commit: `ee39a681179a7c3ecf6316d20004c900f1b0e529`.
All four configured commands passed on that clean candidate with controller receipt
`WJ6BNUUPPJYWBZTRYTZD2YCSCP`. Canonical PR:
https://forge.adrianpk.com/adrianpk/rev/pulls/224
Final receipt `IKLQJ3752CS64MESVHEZXAIP5A` passed all four commands for
`11bc8142ffe1b0371b050cea91a1ea06df96d19f`. Canonical PR #224 merged that exact
commit into dev. Slice 3 of 7 completed.

### Slice 4: Go and web installation

State: delivered.
Ticket: [Go and web installation](../ticket/solved/20261102111451-install-go-and-web-servers.md)
Requirements: INS01, INS02, INS03, INS04, INS06.
Dependencies: Slice 3 integrated.
Branch: `feat/rev-language-server-installation-04-go-web`
PR: `#225`.
Report: `ops/default/report/slices/rev-language-server-installation/slice-4-go-web.md`

- [x] T4.1: Add isolated gopls acquisition with exact module provenance and local-only compiler selection.
- [x] T4.2: Add exact npm-prefix TypeScript/JavaScript and HTML/CSS/JSON recipes with script execution disabled, package locks and separate launch profiles.
- [x] T4.3: Verify prerequisites, package normalization, inherited associations and one semantic/native source; record actual supported package versions and real Go server evidence.

T4.1 development checks passed: `cargo check --locked`,
`cargo test --locked --test lsp_installation go:: -- --test-threads=1` (2 cases),
`cargo test --locked --test lsp_installation acquisition:: -- --test-threads=1` (3 cases),
`cargo test --locked --test lsp_installation storage:: -- --test-threads=1` (4 cases) and
`cargo run --locked --quiet --example source-check -- --source src/lsp/install/go.rs --source src/lsp/install/evidence.rs --source src/lsp/install/package.rs --source src/lsp/install/cache.rs --source src/lsp/install/progress.rs`.
The Go boundary uses an owned tool fixture to assert exact module resolution,
local-only toolchain selection, isolated writable caches, matching built-module
identity, retained provenance and an empty-root protocol probe. Failure cases
preserve prior ready data. This establishes mechanics, not upstream Go semantic
quality. Package evidence is admitted separately from native release SHA-256;
the native requirement remains intact. New responsibilities pass the unchanged
source-quality limits after splitting build-info admission and phase lookup.
T4.1 commit: `659399f44008122423f8de04eaa40caccc078ef0`. Live package/semantic evidence and native selection
remain assigned to T4.3.

T4.2 development checks passed: `cargo check --locked`,
`cargo test --locked --test lsp_installation npm:: -- --test-threads=1` (2 cases),
`cargo test --locked --test lsp_installation go:: -- --test-threads=1` (2 cases) and
`cargo run --locked --quiet --example source-check -- --source src/lsp/install/npm.rs --source src/lsp/install/npm/tools.rs --source src/lsp/install/npm/lock.rs --source src/lsp/install/npm/version.rs --source src/lsp/install/package.rs --source src/lsp/install/evidence.rs`.
Owned npm/Node fixtures assert exact pins, stable TypeScript 6 normalization,
scripts disabled, isolated configuration/cache/temp roots, lock and installed
package identity, registry HTTPS/integrity and all three web launch probes.
They cover engine, major-version, pin, integrity, source, identity, escaping-link
and probe failures while preserving earlier ready records. Real SHA-256 hashes
the retained lock; actual Node/npm compatibility remains a separate live smoke.
Ready package evidence retains a recipe-specific kind. Payload containment is
admitted before any extracted launch executes. The shared package path and Go
regressions pass the unchanged storage/output/process limits.
T4.2 commit: `50aac5222ff076ba4001bfbeb9b8c7ce674de183`.

T4.3 development checks passed:
`cargo test --locked --test lsp_installation managed::native_packages -- --exact --nocapture`,
`cargo run --locked --quiet --example source-check -- --source src/lsp/install.rs --source src/session/lsp/setup.rs --source src/session/lsp/managed.rs`,
`cargo run --locked --quiet --example documentation-check` and
`cargo test --locked --test commands palette::catalog_reference -- --exact`.
Both native profiles cover Go, shared TypeScript/web selection, association and
settings inheritance, custom commands, editing/point preservation, disabled
analysis, multi-language cancellation and helper-free ready reuse. Catalogs agree
on 414 commands/219 settings; all 67 documentation pages and 402 links passed.
Explicit upstream validation also passed:
`cargo test --locked --test lsp_installation live:: -- --ignored --test-threads=1 --nocapture`.
Actual gopls v0.24.0 was built with local Go 1.27.0-X:nodwarf5 and returned a field
present only in unsaved text, through one semantic source, with one undo and owned
server cleanup. TypeScript Language Server 6.0.3/TypeScript 6.0.3 and web package
4.10.0 passed every native transport probe on Node v26.8.1/npm 12.0.2. These two
explicit network checks took 61.12 seconds and cleaned owned disposable roots;
ordinary deterministic checks do not acquire packages. The Slice 4 report records
exact module/lock evidence and separates web transport from Go semantic evidence.
T4.3/report-introduction commit: `d5a5cea182cb1f6b5869b4482ad3e9ac6feb6a2f`.
All four configured commands passed on that clean candidate with controller receipt
`HKEDMZBY74EBCBV43XPUOV3CKK` (116 deterministic regression cases).
Canonical PR: https://forge.adrianpk.com/adrianpk/rev/pulls/225
Final receipt `7R25H63KMVEPD7TUKPU6WPYQU5` passed all four commands for
`b7e5cc0bb3ee1e3158033aba71144bbfbb97ac44`. Canonical PR #225 merged that exact
commit into dev. Slice 4 of 7 completed.

### Slice 5: Remaining optional recipes

State: in_progress.
Ticket: [Remaining optional recipes](../ticket/in_progress/20261102111452-complete-optional-server-catalog.md)
Requirements: INS01, INS02, INS03, INS06.
Dependencies: Slice 4 integrated.
Branch: `feat/rev-language-server-installation-05-catalog-recipes`
PR: pending.
Report: `ops/default/report/slices/rev-language-server-installation/slice-5-catalog-recipes.md`

- [ ] T5.1: Add verified Marksman native acquisition, isolated Ruby gems and a fixed-path minimal Python virtual environment.
- [ ] T5.2: Add immutable-revision OLS acquisition/build with the installed toolchain and required builtin paths; preserve generic custom setup for Org and all other languages.
- [ ] T5.3: Exercise recipe-specific tool/version failures and available real-server smoke probes; distinguish installed evidence from fixtures and keep all helpers optional.

Validation: pending; use the exact ordered commands in the plan and controller
configuration. Commits and canonical integration evidence: pending.

### Slice 6: Update cancellation and removal

State: ready.
Ticket: [Update cancellation and removal](../ticket/ready/20261102111453-manage-server-installation-lifecycle.md)
Requirements: INS02, INS03, INS04, INS05, INS06.
Dependencies: Slice 5 integrated.
Branch: `feat/rev-language-server-installation-06-lifecycle`
PR: pending.
Report: `ops/default/report/slices/rev-language-server-installation/slice-6-lifecycle.md`

- [ ] T6.1: Add explicit update/remove/cancel commands; prepare and verify new versions before atomically retargeting still-owned profiles.
- [ ] T6.2: Handle shared web-profile references, source conflicts, incomplete jobs and bounded cache cleanup without deleting external or running payloads.
- [ ] T6.3: Verify process-group stop/reap, editor-exit cleanup, concurrent instances, failed replacement and preservation of ordinary editing.

Validation: pending; use the exact ordered commands in the plan and controller
configuration. Commits and canonical integration evidence: pending.

### Slice 7: Acceptance and user guidance

State: ready.
Ticket: [Acceptance and user guidance](../ticket/ready/20261102111454-verify-and-document-server-management.md)
Requirements: INS01-INS06.
Dependencies: Slice 6 integrated.
Branch: `feat/rev-language-server-installation-07-acceptance`
PR: pending.
Report: `ops/default/report/slices/rev-language-server-installation/slice-7-acceptance.md`

- [ ] T7.1: Deliver one representative native/PTY management journey in both startup profiles, real Rust/Go integration, managed acquisition and grammar-independent semantic routing.
- [ ] T7.2: Document ordinary setup, custom languages, source selection, recipes and lifecycle; update complete configuration examples, command/reference catalogs, Unreleased and canonical Markdown-to-Spore guidance.
- [ ] T7.3: Finish slice tickets before the configured integrated gate, record resource/ownership evidence and close only after controller validation of the exact candidate.

Validation: pending; use the exact ordered commands in the plan and controller
configuration. Commits and canonical integration evidence: pending.


## Validation selection

The exact Full gate metadata matches the plan and local controller
configuration. Focused checks protect the active behavior increment. The
combined affected-contract/native/PTY/docs gate runs only after all seven
slices integrate. It is not the exhaustive repository suite and must not be
reported as one.

The final report records concrete package versions and acquisition provenance,
real Rust/Go integration, arbitrary-language routing without a grammar,
both startup profiles, preservation of current text/settings/native fallback,
cancellation/process ownership and observed resources. Distinguish controlled
fixtures, actual package-manager smoke probes and unsupported environments.
Do not require Python or other optional runtimes for ordinary Rust/Bash checks.

## Final gate and closure

Pending. No controller result or implementation evidence exists yet.
Finish ticket transitions and their links before gate admission. Closure is
documentation-only within the preconfigured exact allowlist. Retain the
approval, task identities, canonical reports and exact tested candidate/receipt.
Mark completed only after the configured gate and controller checks pass.
