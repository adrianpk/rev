# Language Server Installation Delivery

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
Active slice: Slice 7 — Acceptance and user guidance
Tracker: [Installation Tracker](../tracker/rev-language-server-installation.md)

## Outcome and authority

Make optional LSP setup, association, server choice, managed acquisition and
maintenance available through native commands. The existing LSP client is the
baseline, not a new implementation. Any compatible installed server can serve
any admitted language identity, including languages with no bundled grammar or
managed recipe. Initial catalog entries are convenient acquisition recipes,
not the editor's language boundary.

The product owner approved implementation and necessary preparation on
2026-11-02. This set delegates one canonical PR per slice, Rebase + Fast-forward
integration into dev, the configured integrated gate, bounded repair PRs and
documentation closure to Looper. Use one retained controller and its checkpoint
through all seven slices. The tracker and tickets describe work; they do not
operate another controller. Main integration, releases, mirrors, global rev
installation and public guide deployment require separate instructions.

Conventional Commit subjects and PR titles contain no task-number suffix.
Keep all code, configuration, examples and operational records in English.
Preserve unrelated pending tickets, including the quit alias.

## Strategy and delivered prerequisites

Seven behavior-first increments first make existing/custom setup useful, then
add managed installation and lifecycle. Each slice compiles and leaves only
working commands visible. The delivered configuration, file associations,
XDG roots, shared M-x/colon prompts, native completion, stdio LSP client and
both startup interaction profiles are the prerequisites. No other pending
capability is a prerequisite.

Keep the live command inventories, packaged defaults, configuration reference
and sample consistent in the slice that changes their public contract. Slice 7
adds the complete guided narrative and integrated evidence, not deferred fixes
for broken intermediate catalogs.

Read actual owners before extending them: src/config and its source loader,
src/lsp and attachment/process owners, src/session, src/command, src/input,
the native jobs/result-buffer surface and src/buffer/policy.rs. Reuse them
instead of adding a parallel language registry, configuration search path,
completion popup or background service. Explicit file associations already
identify languages independently of grammar availability.

## Approved implementation choices

The owner delegated completing the draft's engineering gaps. These selections
are preparation decisions, not claims that the owner named a particular crate:

- Preserve TOML comments and ordering with the narrowly scoped
  `toml_edit = "=0.25.17"` dependency, parse/display features only. Its TOML 1.1
  support matches the current parser. Keep unbounded parsing disabled.
- Reuse std::process, standard file locks and the already locked rustix 1.1.5
  safe process APIs where process-group supervision needs them. First-party
  unsafe remains forbidden. Do not add an async runtime, HTTP stack or LSP
  framework.
- Use structured invocations of optional installed curl/gzip/SHA-256 and
  language package tools. They belong to explicit recipe execution, not core
  build or normal editing. Do not add Python to mandatory project tooling.
- Resolve exact upstream versions during explicit operations. Store package
  provenance/locks and verify native digests. Use TypeScript 6 compatibility
  until a separately verified upstream migration; never silently upgrade the
  user's compiler, Ruby, Python, Node or Odin.
- Verify managed acquisition on Linux x86-64. Generic installed-server setup
  is independent of that platform catalog. Add no falsely verified Org recipe;
  Org and every other language retain generic registration.

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

## Slice outcomes, tasks and checks

### Slice 1: Existing server registration

Ticket: [Existing server registration](../ticket/solved/20261102111448-register-existing-language-servers.md)
Requirements: INS01, INS03, INS04, INS05.
Dependencies: delivered baseline owners.

- T1.1: Add preferred-server/native-only and literal environment fields, whole-candidate validation, root provenance and a format-preserving conflict-checked settings writer.
- T1.2: Expose guided existing/custom registration and scoped live application through the shared command owner; preserve runtime LSP enablement, exclusions, editor profile, includes and unrelated settings.
- T1.3: Cover grammar-independent registration, source switching, configuration conflicts and symlink publication through native integration tests.

Required command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test configuration --test lsp --test lsp_installation -- --test-threads=1
git diff --check
```

### Slice 2: Catalog and guided setup

Ticket: [Catalog and guided setup](../ticket/solved/20261102111449-guide-language-server-setup.md)
Requirements: INS01, INS03, INS04.
Dependencies: Slice 1 integrated and prior contracts retained.

- T2.1: Add the finite recipe metadata catalog and lsp-setup/lsp-servers/lsp-select-server interactions, including custom languages and existing executables.
- T2.2: Inherit known associations and selected profile settings; ask for patterns only when needed, and display native-only/external/managed readiness without network discovery.
- T2.3: Cover selector cancellation, both interaction profiles, unknown language identities and registry/catalog independence.

Required command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test commands --test lsp_installation -- --test-threads=1
git diff --check
```

### Slice 3: Managed Rust installation

Ticket: [Managed Rust installation](../ticket/solved/20261102111450-install-managed-rust-server.md)
Requirements: INS01, INS02, INS03, INS05, INS06.
Dependencies: Slice 2 integrated and prior contracts retained.

- T3.1: Implement versioned XDG payloads, schema-1 installation facts, OS-owned mutation lock and one bounded asynchronous management worker.
- T3.2: Acquire an exact official Rust server release with integrity verification and finite structured helper processes; initialize/shutdown in an owned empty root before publication.
- T3.3: Publish ready inventory before conflict-checked selection; cover missing prerequisites, corrupt downloads, invalid registry, failed probe and installation while LSP is disabled.

Required command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test lsp_installation -- --test-threads=1
git diff --check
```

### Slice 4: Go and web installation

Ticket: [Go and web installation](../ticket/solved/20261102111451-install-go-and-web-servers.md)
Requirements: INS01, INS02, INS03, INS04, INS06.
Dependencies: Slice 3 integrated and prior contracts retained.

- T4.1: Add isolated gopls acquisition with exact module provenance and local-only compiler selection.
- T4.2: Add exact npm-prefix TypeScript/JavaScript and HTML/CSS/JSON recipes with script execution disabled, package locks and separate launch profiles.
- T4.3: Verify prerequisites, package normalization, inherited associations and one semantic/native source; record actual supported package versions and real Go server evidence.

Required command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test lsp_installation --test lsp --test completion -- --test-threads=1
git diff --check
```

### Slice 5: Remaining optional recipes

Ticket: [Remaining optional recipes](../ticket/solved/20261102111452-complete-optional-server-catalog.md)
Requirements: INS01, INS02, INS03, INS06.
Dependencies: Slice 4 integrated and prior contracts retained.

- T5.1: Add verified Marksman native acquisition, isolated Ruby gems and a fixed-path minimal Python virtual environment.
- T5.2: Add immutable-revision OLS acquisition/build with the installed toolchain and required builtin paths; preserve generic custom setup for Org and all other languages.
- T5.3: Exercise recipe-specific tool/version failures and available real-server smoke probes; distinguish installed evidence from fixtures and keep all helpers optional.

Required command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test lsp_installation -- --test-threads=1
git diff --check
```

### Slice 6: Update cancellation and removal

Ticket: [Update cancellation and removal](../ticket/solved/20261102111453-manage-server-installation-lifecycle.md)
Requirements: INS02, INS03, INS04, INS05, INS06.
Dependencies: Slice 5 integrated and prior contracts retained.

- T6.1: Add explicit update/remove/cancel commands; prepare and verify new versions before atomically retargeting still-owned profiles.
- T6.2: Handle shared web-profile references, source conflicts, incomplete jobs and bounded cache cleanup without deleting external or running payloads.
- T6.3: Verify process-group stop/reap, editor-exit cleanup, concurrent instances, failed replacement and preservation of ordinary editing.

Required command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test lsp_installation -- --test-threads=1
git diff --check
```

### Slice 7: Acceptance and user guidance

Ticket: [Acceptance and user guidance](../ticket/reviewing/20261102111454-verify-and-document-server-management.md)
Requirements: INS01-INS06.
Dependencies: Slice 6 integrated and prior contracts retained.

- T7.1: Deliver one representative native/PTY management journey in both startup profiles, real Rust/Go integration, managed acquisition and grammar-independent semantic routing.
- T7.2: Document ordinary setup, custom languages, source selection, recipes and lifecycle; update complete configuration examples, command/reference catalogs, Unreleased and canonical Markdown-to-Spore guidance.
- T7.3: Finish slice tickets before the configured integrated gate, record resource/ownership evidence and close only after controller validation of the exact candidate.

Required command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test lsp_installation -- --test-threads=1
bash scripts/check-lsp-installation.sh
cargo run --locked --quiet --example documentation-check
bash scripts/build-user-guide.sh
git diff --check
```


## Acceptance ownership and invariants

| Contract | Owner | Required evidence |
| --- | --- | --- |
| INS01 ordinary setup | Slices 1, 2, 3 | Reuse a selected executable; inherit a known association; report a missing prerequisite without losing edits. |
| INS02 optional payloads | Slices 3-6 | Installation with LSP disabled keeps native editing; normal startup/idle does not run helpers or network checks. |
| INS03 open language support | Slices 1, 2, 5, 7 | Configure an arbitrary language ID and suffix with no grammar/catalog entry. Attach and obtain semantic results; no editor rebuild. |
| INS04 source selection | Slices 1, 4, 7 | One selected server per language/root, stale old responses rejected, semantic-or-native completion with no appended cross-provider duplicates. |
| INS05 persistence/live application | Slices 1, 3, 6 | Preserve comments/includes/root symlinks and unrelated settings; reject concurrent source changes; preserve startup profile, global LSP mode and buffer exclusions. |
| INS06 owned lifecycle | Slices 3, 5, 6, 7 | Bound output/storage/time, serialize instances, verify before ready, cancel/reap owned processes, refuse external/referenced removal and preserve prior versions on failure. |

Use real filesystem/process boundaries and compact parameter tables for
policy variants. Controlled local acquisition/protocol fixtures cover offline,
bad digest, blocked helper and stale-response faults. They are not evidence of
a live upstream package or its semantic quality. Record actual managed package
versions and smoke-probe outcomes in reports. Primary acceptance includes real
Rust and Go servers. A real available server for an unbundled language is
preferred in addition to the deterministic grammar-independent routing case.

Preserve current completion edit safety, UTF-8/UTF-16 positions, pane rendering,
language grammars and source fallback. Reassociating a file does not load a new
grammar library at runtime. Client operations remain within the delivered
stdio completion contract; installation does not promise every LSP method.

## Persistence and resource contract

Follow the paired model's field order, schema 1, XDG paths and concrete limits.
The settings writer owns root provenance and full-candidate admission; it
must not flatten defaults, includes or unrelated editor options. A scoped
settings apply preserves the current global LSP toggle/exclusions and applies
only server/preference/association changes. Native-only is explicit per
language. Absence of preferences preserves existing lexical server selection.

A ready payload may be unselected after a source conflict. Report a published
but not durably confirmed write truthfully. Keep candidates in their final
unique versioned paths from creation, especially Python venvs. Removal and
cleanup never follow links outside owned directories. Package tools run as
the user and are not a sandbox; enforce bounded monitoring and report residual
cleanup errors. No shell interpolation of user commands or environment values.

## Integrated gate, cost and recovery

The metadata Full gate is the exact configured command, not an instruction to
run every historical delivery gate. It covers affected configuration, client,
completion, language assets, shared command catalog and modal local contexts,
plus one new representative management script and canonical guide generation.
The new script/example belongs to Slice 7 and must use native Rust/Bash. Build
deterministic recipe fixtures and reuse verified owned caches; the gate must
not install optional Ruby/Python/Odin runtimes or repeatedly fetch all catalog
entries. Record real acquisition separately in the same slice evidence.

Expected cost: several minutes for the integrated Rust tests and one native/
PTY management journey, plus an initial cold compile and explicit package
acquisition. This is an estimate, not a passed timing. Measure actual duration
and idle CPU/memory/disk/process behavior; explain any material expansion.
Per-slice validation stays focused. Do not append historical Org, bookmark,
wrap or modal journey scripts automatically.

Pin bounded-v1 recovery before the first worker: two diagnostics per family,
eight per run, three failed repairs per family, three environmental retries
and at most three final-gate repair PRs. Repair branch prefix:
`fix/rev-language-server-installation-acceptance`; title:
`fix(lsp): repair approved setup acceptance`; reuse Slice 7's report.
Repair authority covers only the installer and affected canonical owners,
tests/fixtures, their guide/reference/config sample/changelog and this set's
operational records. It never covers Looper source, services, unrelated
tickets, releases or weakened acceptance. Preserve the unchanged gate,
conversation and 200-turn/five-stall defaults.

## Ticket, report and closure contract

Move each owning ticket through ready, in_progress, reviewing and solved with
actual timestamps, commits and canonical PR evidence. Update links when paths
move. Complete all ticket resolution before requesting the final gate; no
outstanding implementation is hidden as closure. One report per exact mapped
path uses the global required sections. Never claim an unexecuted check.

After the exact integrated candidate passes, closure may change only the
configured plan, tracker, seven reports, paired spec/model, pending index and
the seven exact solved ticket paths. Review that allowlist before startup.
Retain historical baseline/gate identities. Mark the spec/model and set
Completed only at verified closure. No application change belongs in closure.
