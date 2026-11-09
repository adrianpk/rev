# Slice 1: Quality foundation

Status: delivered
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `chore/rev-architecture-quality-01-foundation`
PR: #15 - https://forge.adrianpk.com/rev/rev/pulls/15
Implementation: 3c76d32116e739f26c51abe1e171bd39bb69c877
Integrated: 79d1c50083e83f365036d89013238f9cb431da9a through canonical PR #15

## Purpose

Establish complete first-party Cargo discovery and native, explicitly scoped
quality checks before the remaining approved domain cleanup and hook adoption.

## Delivered Behavior

The Rust source checker discovers both workspace members, their child sources
and every Cargo target root instead of relying on the first metadata package.
Package ownership excludes nested foreign packages and Cargo build output.
Each scope must resolve to owned Rust; empty, missing, foreign and escaped
selections fail. Legacy `--root`/`--source` inputs retain function metrics and
make no workspace coverage claim. Workspace checks support `--metrics`.

The native `scripts/check-quality.sh` entry point checks formatting for the
workspace, runs one all-target/all-feature Clippy scan, admits its JSON
diagnostics and runs the unchanged five/three/ten source rules. Compiler errors,
failed builds and baseline compiler/all-group warnings block every scope.
Selected-profile warnings outside explicitly selected sources are reported as
deferred; the full invocation admits none of that debt. Python is unnecessary
for the native path; the legacy Python wrapper delegates to the Rust owner.

Macro acceptance and PTY journeys now share one completion-support owner.
Allocation, process observation, shutdown restoration and journey phases have
cohesive helpers inside the selected PTY file. Test names, assertions, profiles,
eight-second deadlines, byte ceilings, CPU observations and child reaping remain.

## Implementation Notes

Clippy rules have one list in `examples/source_check/profile.rs`. Warning-level
collection permits complete target discovery during adoption; native admission
rejects every first-party compiler/all-group warning and selected-owner warning.
No capped warnings, historical baseline or broad suppression is introduced.
Malformed/missing/duplicate build results and failed exit statuses cannot pass.
JSON parsing is bounded to 100,000 records and 8 MiB per record.

Tree-sitter checks controlled exceptions, including conditional attributes:
individual local `expect` requires a nonempty reason; `allow`, lint-group and
module/crate exceptions are rejected. Clippy denies unfulfilled expectations.
Existing source metrics and safety checks retain their definitions.

The complete scan exposed two existing baseline warnings after `duplicate_mod`
was repaired. Flattening one nested configuration `format!` in the plugin test
fixture preserves the generated text. Aliases name the existing atomic-edit
snapshot tuple without changing its fields or assertions.

### Criteria evidence

| Criterion | Evidence in this slice | Remaining approved boundary |
| --- | --- | --- |
| Cohesive owners | Separate workspace discovery, diagnostic admission and source inspection; cohesive PTY phases | Plugin payload/cache and completion owners in Slices 2 and 8 |
| Borrow rather than copy | Selected pass-by-value, clone and collection rules; real Clippy fixture | Domain ownership review in Slices 2-9 |
| Meaningful shared ownership | `clone_on_ref_ptr` collected and admitted by source owner | Manual Arc/Rc rationale remains domain work |
| Lifetime design | Existing unsafe/raw-pointer/FFI guard; owned fixture cleanup retained | Exact disallowed leak/forget configuration in Slice 10 |
| Interior mutability | Baseline Clippy plus selected drop rules | Synchronization invariants require manual domain review |
| Async/thread ownership | Unused-async/drop profile; finite native fixture ownership | Process cancellation and retirement in Slice 6 |
| Allocation and copying | Real fixture proves selected warnings defer only outside scope | Payload/cache corrections in Slice 2 |
| Representations | Four selected conversion rules, with no arithmetic/API changes here | Native units/bounds reviewed in Slices 2-9 |
| Explicit API ownership | Existing public and wire contracts preserved | SDK error/panic contracts in Slice 2 |
| Safe failures | `compiler_results` and `diagnostic_admission` reject failed or incomplete evidence | Existing domain negative tests remain authoritative |
| Small functions | Existing parameter <= 5, nesting <= 3, cyclomatic <= 10 rules pass selected sources | Remaining native source findings in Slices 2-9 |
| Complete coverage | Real two-member Cargo fixture includes custom target, build script, bench, tests and SDK children | Full source/profile acceptance after domain cleanup |
| Controlled exceptions | `exceptions` covers local reasons, groups, conditional attributes and quoted text; real Clippy proves unfulfilled expectations block | Final inherited profile in Slice 10 |
| Commit integrity | This slice adds the reusable native check, without claiming index enforcement | Immutable staged-index hook and installation in Slice 10 |

## Contracts Added or Changed

Workspace checking is the source checker's default. Repeat `--scope PATH` for
explicit owner selection. The Bash driver obtains the selected flags from the
same Rust profile used to classify diagnostics. Every actual Cargo root requires
the existing explicit unsafe prohibition. Unresolvable source owners and unknown
arguments fail rather than silently narrowing coverage.

The local scoped result covers 12 of 1,798 first-party Rust files, one selected
root out of 73 discovered Cargo roots, and a complete baseline Clippy scan of
both packages. It reports zero blocking diagnostics and 1,638 selected-profile
diagnostics deferred outside scope, including repeated observations across
targets. These are emitted diagnostics, not a count of functional defects.

## Files of Interest

- `examples/source-check.rs` and `examples/source_check/`: sole source-policy,
  workspace discovery, profile/admission and parser/discovery/exception fixtures.
- `scripts/check-quality.sh` and `scripts/check-source.py`: native entry point
  and compatibility delegation.
- `tests/macros.rs`, `tests/macros/acceptance.rs`, `tests/macros/pty.rs`: shared
  support and preserved native acceptance/resource journeys.
- `tests/plugins/lifecycle.rs` and
  `src/session/source_batch/atomic/tests.rs`: minimal baseline fixture repairs.

## Validation

The five frozen commands passed in canonical order through Looper validation on
clean published HEAD 79d1c50083e83f365036d89013238f9cb431da9a, now integrated
through canonical PR #15. These focused results do not establish the integrated
full gate.

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope examples/source-check.rs --scope examples/source_check --scope tests/macros/acceptance.rs --scope tests/macros/pty.rs`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --example source-check`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros -- capture large_cancellation --test-threads=1`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
5. `git diff --check`

The checker passed eight tests, including real Cargo/Clippy acceptance; macro
selection passed 16 tests. Documentation passed 106 reachable pages, 743 local
links/anchors, 684 commands and 299 configuration entries. An additional local
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros -- pty::terminal_journeys --exact --test-threads=1`
passed the existing PTY journey in both profiles (one test, 4.16 seconds).

## Risks and Follow-ups

Global selected-profile and native source cleanup remain in the later approved
slices. Inherited lint configuration, SDK documentation lints, staged-index
enforcement, local hook activation and the full integrated gate are not claimed
by this foundation. A passing linter does not establish domain ownership or
absence of bugs. The existing vendored dependency warning remains outside
first-party ownership. No dependency or runtime behavior changed.
