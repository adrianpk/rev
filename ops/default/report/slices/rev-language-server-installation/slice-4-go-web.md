# Slice 4: Go and web installation

Status: reviewing
Delivery set: rev-language-server-installation
Plan: [Installation Delivery](../../../plan/rev-language-server-installation.md)
Tracker: [Installation Tracker](../../../tracker/rev-language-server-installation.md)
Branch: `feat/rev-language-server-installation-04-go-web`
PR: pending

## Purpose

Acquire isolated Go and npm servers through the existing native manager, retain
exact package evidence and preserve language choices, editing and analysis mode.

## Delivered Behavior

- `lsp-install-server` and compatible `lsp-setup` choices now acquire `gopls`,
  `typescript-language-server` and `vscode-langservers-extracted`. Shared packages
  ask which language to select. Existing/custom setup remains catalog-independent.
- Go resolves one stable exact module, builds with the installed local compiler
  and owned caches/GOBIN, checks the binary's module identity and probes it before
  publication. Insufficient compilers fail without downloading another toolchain.
- npm installs exact saved versions in an owned prefix with scripts disabled,
  strict engines and a retained lock. TypeScript stays on stable major 6. The web
  package supplies independently verified HTML, CSS and JSON stdio profiles.
- Ready facts precede settings publication. Shared-profile choices inherit the
  selected language's options; independent web profiles retain their own values.
  Ready reuse for another language starts no helper and preserves its current
  settings and known associations. Analysis remains disabled until enabled.
- Both native profiles retain text, point, configured custom commands and root
  comments. Multi-language cancellation starts no package helper. Inventory shows
  concrete managed versions without printing environment values.

## Implementation Notes

The existing storage, data/settings locks, one native worker, cancellation token,
process-group supervision and empty-root protocol owner remain authoritative.
The explicit installation dispatcher exposes only four delivered recipes; the
larger metadata catalog does not expose unfinished acquisition operations.

Go uses the official proxy/checksum database, `GOTOOLCHAIN=local`, `GOENV=off`,
`GOWORK=off`, owned module/build/temp caches and final `GOBIN`. Writable module
caches permit owned cleanup; two build workers bound build parallelism. The
resolved module sums, compiler identity and binary build information are retained.
The built binary must match the exact unreplaced module/version/sum.

npm uses owned configuration/cache/temp roots and a private package root. Exact
root pins, installed package names/versions and every dependency's registry HTTPS
source, exact version and SHA-256/512 integrity are admitted. Node's built-in
crypto hashes the retained lock. The complete payload tree is admitted before an
extracted entrypoint executes, including containment of npm `.bin` symlinks.
Every web launch must initialize/shutdown/exit against an owned empty root with
no user document. Helpers retain structured argv and the existing bounded output,
filesystem monitoring, deadlines and stop/reap ownership.

Primary references: [Go toolchain selection](https://go.dev/doc/toolchain),
[Go module installation](https://go.dev/ref/mod#go-install),
[npm installation options](https://docs.npmjs.com/cli/v11/commands/npm-install/),
[TypeScript server](https://github.com/typescript-language-server/typescript-language-server)
and [web servers](https://github.com/hrsh7th/vscode-langservers-extracted).

## Contracts Added or Changed

- Schema 1 retains its existing fields and order. Ready Go evidence is
  `go-module:h1:<module-sum>`; npm evidence is `npm-lock:sha256:<lock-digest>`.
  Native releases still require their artifact SHA-256. Evidence kinds are
  admitted per recipe and never imply semantic quality.
- The worker now reports its actual building phase for Go. Existing ceilings
  remain: 32 records/2 MiB registry, 256 MiB native asset, 1 GiB payload, 2 GiB
  cache, 1 MiB helper output, 64 KiB retained stderr and 900 seconds total.
- One shared npm payload may supply several relative launch profiles. Selection
  changes only the explicitly chosen language preference and keeps the configured
  command authoritative. Existing semantic-or-native completion is reused.
- Shared command/configuration inventories remain 414 commands and 219 settings.
  Tools remain optional and run with user permissions; no compiler/runtime is
  installed globally or upgraded by these recipes.

## Files of Interest

- `src/lsp/install/go.rs`: exact module acquisition and binary provenance.
- `src/lsp/install/npm.rs`, `npm/`: exact npm resolution, package/lock admission
  and isolated helper options.
- `src/lsp/install/package.rs`, `evidence.rs`, `cache.rs`, `progress.rs`:
  shared bounded package publication/probes, evidence kinds and phase ownership.
- `src/session/lsp/setup.rs`, `managed.rs`: recipe/language choices and scoped
  selection with current-language settings inheritance.
- `tests/lsp_installation/go.rs`, `npm.rs`, `managed.rs`, `live.rs`:
  controlled failure boundaries, both-profile interactions and explicit live smoke.
- Language-server how-to/reference, bindings and Unreleased describe the working
  Go/web paths and optional prerequisites.

## Validation

Focused development checks passed:

- `cargo check --locked`
- `cargo test --locked --test lsp_installation go:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation npm:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation acquisition:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation storage:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation managed::native_packages -- --exact --nocapture`
- `cargo run --locked --quiet --example source-check -- --source src/lsp/install/go.rs --source src/lsp/install/evidence.rs --source src/lsp/install/package.rs --source src/lsp/install/cache.rs --source src/lsp/install/progress.rs`
- `cargo run --locked --quiet --example source-check -- --source src/lsp/install/npm.rs --source src/lsp/install/npm/tools.rs --source src/lsp/install/npm/lock.rs --source src/lsp/install/npm/version.rs --source src/lsp/install/package.rs --source src/lsp/install/evidence.rs`
- `cargo run --locked --quiet --example source-check -- --source src/lsp/install.rs --source src/session/lsp/setup.rs --source src/session/lsp/managed.rs`
- `cargo run --locked --quiet --example documentation-check`
- `cargo test --locked --test commands palette::catalog_reference -- --exact`

Controlled Go/npm/Node boundaries cover local-only toolchain and exact pins,
missing runtimes, insufficient engines, invalid checksums/locks/package identity,
unsupported TypeScript majors, escaping launch links, failed probes and preservation
of earlier ready data. They establish acquisition mechanics, not upstream semantics.
The native test runs both startup profiles and checks Go, shared TypeScript/web
selection, association/settings inheritance, custom commands, editing preservation,
disabled analysis, cancellation and helper-free reuse. Its shared-log cancellation
precondition was corrected to compare the captured post-Go baseline; the assertion
still requires zero additional helper calls. Source checks pass the unchanged
safe-Rust, five-parameter, three-nesting and ten-complexity limits.

Explicit network smoke passed separately through
`cargo test --locked --test lsp_installation live:: -- --ignored --test-threads=1 --nocapture`
(two cases, 61.12 seconds):

| Actual managed package | Concrete versions/runtime | Observed behavior |
| --- | --- | --- |
| gopls | `v0.24.0`; local `go1.27.0-X:nodwarf5` | Exact acquisition/probe in 52,240 ms; semantic completion includes only the unsaved field, one semantic source, one undo, disk preserved and owned server reaped. |
| TypeScript Language Server | server `6.0.3`, TypeScript `6.0.3`; Node `v26.8.1`, npm `12.0.2` | Exact acquisition and native empty-root transport probe in 3,100 ms. |
| Web language servers | package `4.10.0`; Node `v26.8.1`, npm `12.0.2` | One acquisition and all three independent native transport probes in 5,052 ms. |

Observed Go module sum: `h1:nPO/LffYa4sP62Ueuema5PPnZPGaZjGl+vZGvbTek4g=`.
Observed npm lock SHA-256 values were
`2adba24a77d7f896252f42f7e850115ace4dc126b7688d9324579ff80a544ba7`
(TypeScript) and
`30f0184f9e380f42bd68637d1b5823bde34799fc73e87a64e337d965bdf661b4`
(web). Each live operation used the normal 900-second manager and owned disposable
roots; fixtures were removed after workers, probes and the Go client stopped.
The explicit network checks are excluded from ordinary deterministic regressions.

The configured ordered format, all-target Clippy, installation/client/completion
and whitespace batch is recorded through the controller on the clean report head
before opening the PR. The PR-reference follow-up requires a fresh exact-HEAD
receipt. The integrated delivery-set gate remains with the controller.

## Risks and Follow-ups

Web transport smoke does not establish comprehensive semantic behavior for each
language. Remaining recipes, update/removal lifecycle and combined acceptance
remain with their approved slices. Cleanup or publication durability uncertainty
is reported explicitly; prior ready payloads and configuration survive failures.
