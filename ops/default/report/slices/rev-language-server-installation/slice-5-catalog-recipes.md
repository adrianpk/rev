# Slice 5: Remaining optional recipes

Status: delivered
Delivery set: rev-language-server-installation
Plan: `ops/default/plan/rev-language-server-installation.md`
Tracker: `ops/default/tracker/rev-language-server-installation.md`
Branch: `feat/rev-language-server-installation-05-catalog-recipes`
PR: `#226`

## Purpose

Complete the initial optional managed recipe catalog while preserving generic
server registration, language associations and ordinary editing.

## Delivered Behavior

- Marksman verifies one exact official native release's bounded length and
  SHA-256, records its version output and completes an empty-root transport probe.
- Ruby installs exact ruby-lsp and Bundler gems into an owned gem home, verifies
  the primary gem against RubyGems API SHA-256 and retains dependency resolution
  plus a frozen local bundle lock. Launch bypasses automatic project bundling.
- Python installs base python-lsp-server without extras in a copied-interpreter
  venv at its final path. Isolated pip retains exact wheel provenance/hashes;
  entrypoint shebangs are never invalidated by moving the environment.
- OLS checks out one immutable Git revision and builds through its pinned upstream
  script with the installed Odin. It retains source/builtin paths and disables
  snippets/fake methods. Missing output fails even if the wrapper returns zero.
- Both native command profiles preserve editing, analysis mode and unrelated
  settings. Ready reuse restores Ruby's owned runtime paths and OLS's builtin
  path/snippet restrictions without running acquisition helpers. Org and arbitrary
  uncataloged identities retain generic custom-server setup and associations.

## Implementation Notes

Rust and Marksman share the bounded GitHub downloader; Rust acquisition regressions
still pass. Ruby acquisition keeps owned temp/cache and build controls despite
launch environment overrides. Native extensions use serial GNU Make with pipe
jobservers (GNU Make 4.4+) to avoid transient FIFOs without weakening admission
of special files. The installed Ruby's concrete Bundler version is retained inside
its managed payload. The frozen launch avoids Ruby LSP's project-local installer.

Git helpers discard inherited Git redirections and global/system configuration
before applying owned paths. OLS retains the checked-out revision, Odin identity
and build provenance. Bounded build diagnostics expose compiler failures hidden
by the upstream wrapper's zero exit status. UTF-8 replacement preserves the
64 KiB diagnostic tail ceiling. Process-group stop/reap, cancellation
and cache-cleanup ownership are shared with existing recipes.

## Contracts Added or Changed

All eight initial recipes have acquisition dispatch. New evidence kinds are
`gem-lock:sha256:<digest>`, `pip-report:sha256:<digest>` and
`git-revision:<40-hex-commit>`, admitted per recipe. Native assets still require
artifact SHA-256. Ready admission checks Ruby's owned frozen bundle paths and
OLS's matching revision evidence, owned builtin path and disabled snippet options.

Limits remain 32 records/2 MiB registry, 256 MiB native asset, 1 GiB payload,
2 GiB cache, 1 MiB helper output, 64 KiB diagnostic tail and 900 seconds per
operation. No compiler/interpreter is installed or upgraded globally. Optional
runtimes are not required by ordinary offline regressions. Inventories remain
414 commands and 219 settings.

## Files of Interest

- `src/lsp/install/{download,marksman,ruby,python,ols}.rs`: acquisition/provenance.
- `src/lsp/install/{runtime,record,evidence,helper}.rs` and
  `src/session/lsp/managed.rs`: ready admission, diagnostics and selection.
- `tests/lsp_installation/{optional,ols,optional-native,live}.rs` and controlled
  shell tools: filesystem/registry/Git/transport failures and opt-in live probes.
- Language-server how-to/reference, bindings and Unreleased: available recipes,
  optional prerequisites and recoverable compatibility failures.

## Validation

Focused development checks passed:

- `cargo check --locked`
- `cargo test --locked --test lsp_installation acquisition:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation optional:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation ols:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation optional_native:: -- --test-threads=1`
- `cargo run --locked --quiet --example source-check -- --source src/lsp/install.rs --source src/lsp/install/download.rs --source src/lsp/install/marksman.rs --source src/lsp/install/ruby.rs --source src/lsp/install/python.rs --source src/lsp/install/runtime.rs --source src/lsp/install/package.rs --source src/lsp/install/evidence.rs --source src/lsp/install/helper.rs --source src/lsp/install/tree.rs`
- `cargo run --locked --quiet --example source-check -- --source src/lsp/install/ols.rs --source src/lsp/install/runtime.rs --source src/lsp/install/helper.rs --source src/lsp/install.rs`
- `cargo run --locked --quiet --example source-check -- --source src/lsp/install/ruby.rs --source src/session/lsp/managed.rs`
- `cargo run --locked --quiet --example documentation-check`
- `cargo test --locked --test commands palette::catalog_reference -- --exact`

Controlled tools exercise missing runtimes, malformed versions/hashes, base-only
pip reports, preserved prior-ready data, Git redirections, mismatched checkouts,
false-success builds, missing builtins, rejected probes and corrupt ready paths.
Real local Git objects and native transport participate. The native driver now
answers Odin's required association prompt and uses distinct compatible current
profiles. Observable assertions and resource limits remain unchanged. These checks
establish acquisition/selection mechanics, not upstream language semantics.

Explicit live probes ran separately in disposable owned roots through the normal
900-second manager:

| Successful command | Actual package/runtime | Observed result |
| --- | --- | --- |
| `REV_LIVE_RECIPE=marksman cargo test --locked --test lsp_installation live::optional_packages -- --ignored --exact --nocapture` | Marksman `2026-02-08` | Native acquisition/version and initialize/shutdown/exit passed in 6,879 ms. |
| `REV_LIVE_RECIPE=ruby-lsp cargo test --locked --test lsp_installation live::optional_packages -- --ignored --exact --nocapture` | ruby-lsp `0.26.11`, Bundler `4.0.23`, Ruby `3.4.10` | Isolated gems/frozen lock and transport passed in 15,746 ms. |
| `REV_LIVE_RECIPE=python-lsp-server cargo test --locked --test lsp_installation live::optional_packages -- --ignored --exact --nocapture` | Base python-lsp-server `1.15.0`, Python `3.14.7` | Copied final-path venv, isolated wheels and transport passed in 8,007 ms. |

Observed native SHA-256:
`be5098e8213219269c47fc0d916a66fa31ce0602ec967475c722260aabf26087`.
Ruby lock SHA-256:
`c2a2eb0d12128b6b275f4e2dfca41c8aaf9d910a9175fd47f93f84ecd904a82f`.
Python report SHA-256:
`73f9de1fb7867a46a470e5713f778ae0214b34b0fe66c3d03fe4de8391feb77c`.

`REV_LIVE_RECIPE=ols cargo test --locked --test lsp_installation live::optional_packages -- --ignored --exact --nocapture`
failed truthfully: revision `9b9711592bed66266311c361dd7a307ecc87025b`
requires `core:odin/tool-format`, absent from installed Odin
`dev-2026-08:8412dc37a`. No executable was produced, so the record stayed
incomplete. No live OLS transport or semantic result is claimed. Earlier Ruby
probes exposed argument ordering, Make FIFO and missing isolated Bundler failures;
the corrected bounded mechanics passed the Ruby live probe. All live fixture
roots were removed after owned workers/probes stopped.

Pre-PR controller validation passed for `5ebf52701be350bd77932324eff56152518828aa`, receipt
`RLQ5RSQHYJ7TOUDA5ET6PZI5M4`, in the configured order:

- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo test --locked --test lsp_installation -- --test-threads=1` (34 passed; three explicit network checks excluded).
- `git diff --check`

Canonical PR: https://forge.adrianpk.com/adrianpk/rev/pulls/226
Final controller receipt `PUURIPZASULVCNLPHKFEBOE66M` passed the same four
commands for `11b6d3abcc24ed250eb400a1578e673bb137fca5` (34 regressions).
Canonical PR #226 merged that exact commit into dev. Slice 5 of 7 completed.
The integrated delivery-set gate remains with the controller.

## Risks and Follow-ups

OLS requires a compatible separately installed Odin; the available compiler
could not build current upstream. Ruby native extensions and Python wheels also
require compatible optional tools. Provenance and transport readiness do not
prove language semantic quality. Lifecycle and combined acceptance remain in
their approved slices.
