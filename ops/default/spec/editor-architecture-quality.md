# Architecture Quality and Commit Checks

Status: Completed
Completed: 2026-11-12
Date: 2026-11-09
Version: 0.1
Format: OpenSpec capability specification
Approved: owner requested one remediation set including Clippy/linter setup before commits
Baseline: abbf0c21eeae1680c02c7a477fa0894d7996a393
Review: [Architecture and Ownership Nit Review](../report/rev-architecture-nit-review.md)
Plan: [Architecture Quality Delivery](../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../tracker/rev-architecture-quality.md)
Concern: [Pending Editor Work](../README.md)

## Purpose and scope

Resolve the verified architecture/ownership findings and the existing complete
first-party source-policy violations, then prevent their mechanical recurrence
before ordinary commits. Cover editor, Rust SDK, Cargo targets, tests and examples.
Keep native behavior, wire/configuration/state formats, command names/bindings,
resource limits and existing validation assertions unchanged. Scope is rev;
no other product, global agent rule or desktop configuration is changed.

One Looper delivery set owns preparation, ordered remediation, final strict
adoption, exact-candidate acceptance and documentation-only closure. Review and
implementation are authorized together by this request. This is no release,
main promotion, public deployment, mirror sync or global editor installation.

## Selected policy

Use Rust/Clippy 1.98-compatible built-in tools and the existing Tree-sitter Rust
checker. Add no parser crate, Python/Ruby/Go dependency, linter daemon or shared
mutable cross-worktree build directory. Bash, Git, Cargo and existing Linux
utilities suffice. Keep ordinary user build/run instructions unchanged.

### Workspace Clippy profile

Both first-party packages inherit explicit workspace lints. Forbid unsafe code.
Treat compiler warnings and clippy::all as errors. Add these ownership and
representation checks as blocking rules after domain cleanup:

- redundant_clone, needless_pass_by_value, needless_collect;
- trivially_copy_pass_by_ref, large_types_passed_by_value, clone_on_ref_ptr;
- unused_async, significant_drop_in_scrutinee, significant_drop_tightening;
- cast_possible_truncation, cast_possible_wrap, cast_sign_loss, cast_precision_loss.

Existing all-group checks cover lock misuse, redundant allocations, incorrect
reference counting and other correctness/suspicious/performance issues. Check all
workspace targets and supported features. Apply missing_errors_doc and
missing_panics_doc to the public SDK API, with accurate caller-visible contracts.
Reject mem::forget, Box::leak and raw ownership-transfer escapes used to avoid
ordinary lifetime/resource design. List exact disallowed standard-library methods
with reasons in the Clippy configuration; keep legitimate borrowed identity tests.

Do not turn on every restriction/pedantic/nursery rule indiscriminately. The list
above is a deliberate blocking profile for these criteria, not a style baseline.
Do not add file/line baselines, historical-commit exclusions, capped warnings or
crate/module-wide Clippy suppression. A genuine unavoidable individual exception
uses the narrowest expect with a nonempty reason, checked invariant/owner contract
and relevant evidence. Deny unfulfilled lint expectations. Public signatures,
trait contracts and correctly bounded conversions are reviewed before changing
ownership or arithmetic just to satisfy a suggestion.

### Native source guard

Preserve the existing maximum five parameters including self, logical nesting
three and cyclomatic complexity ten, using the current metric definitions.
Preserve the existing first-party unsafe/raw-pointer-type/FFI policy and explicit
unsafe prohibition on every actual Cargo crate root. Resolve the existing address
identity cast without raw-type spelling tricks or a new unchecked ownership API.

Extend examples/source-check.rs as the single source-policy owner. Enumerate all
first-party Cargo workspace members and target roots, including SDK child modules,
tests, examples and benches. Exclude vendored/dependency sources and generated
build trees by ownership, not by arbitrary violation lists. Support an explicit
workspace scope for focused delivery checks and reject empty or invalid scope
success. Retain legacy --root/--source compatibility. Native Bash/Cargo commands
must not require Python; an old Python entry point may delegate to the same owner.

### Commit boundary

Provide make lint for a whole-workspace strict check, make test-hooks for its
real Git integration cases, and make install-hooks for explicit local activation.
Install .githooks/pre-commit through repository-local core.hooksPath; linked
worktrees resolve their own checked-out hook files. Do not overwrite a conflicting
existing hook configuration. Activate the completed hook in this checkout after
integrated strict-lint and hook acceptance checks pass.

Validate the exact index snapshot with staged source, manifests, lockfile and lint
policy. Do not validate an unstaged correction instead. The hook never rewrites,
stages, stashes, resets or discards user content. It rejects conflicts and an index
that changes while being checked. Guard hook/cache source paths against symlink
substitution; use an exclusively owned private per-worktree cache and serial
ownership, preserve unrelated targets, and keep reusable paths stable when safe.

A source/manifest/policy/tooling commit checks formatting, strict Clippy for complete
affected workspace targets and native source policy; ordinary source changes use
the complete applicable crate, not only changed lines. Configuration/shared policy
changes check both packages. Docs-only commits perform applicable fast integrity
and whitespace checks, without recompiling unrelated native code. No Rust file
or package bypass is introduced as an optimization. Keep full-workspace lint
available even when the commit has no code changes. Measure cold and warm cost.

## Criteria and coverage map

| Review criterion | Mechanical enforcement or required evidence |
| --- | --- |
| Cohesive owners and service boundaries | Manual F4-F6 review, concern-owned transition APIs and native regressions. |
| Borrow rather than copy unnecessarily | needless_pass_by_value, redundant_clone, needless_collect and reviewed call ownership. |
| Meaningful shared ownership | clone_on_ref_ptr, all-group allocation/Arc checks and explicit manual Arc/Rc rationale. |
| Lifetime design | disallowed leak/forget escapes, unsafe/source guards and reviewed captured-target lifetimes. |
| Interior mutability | all-group lock checks, drop checks and manual synchronization invariants. |
| Async/thread ownership | lock/RefCell await checks, unused_async, drop checks and existing cancel/retirement tests. |
| Allocation and copying | clone/collection/value-size lints and verified payload/cache corrections; no invented benchmark claims. |
| Appropriate representations | checked conversions, source units, opaque identities and arithmetic boundary tests. |
| Explicit API ownership | reviewed public contracts, SDK error/panic documentation and exact wire compatibility. |
| Safe failures and invariants | typed input errors, preserved deliberate internal invariant assertions and negative tests. |
| Small understandable functions | existing five/three/ten source rules; cohesive extraction instead of hiding branches. |
| Complete code coverage | Cargo member/target discovery and all-target/features Clippy, excluding only foreign/generated code. |
| Controlled exceptions | local reasoned expect, no broad suppressions/baselines, deny unfulfilled expectations. |
| Commit integrity | real staged-index hooks, partial-staging/conflict/change-during-check cases and immutable source/index checks. |

A passing linter is evidence for its rules only. It does not prove business
ownership, design quality, zero allocations or the absence of bugs. Manual review
obligations remain explicit rather than represented by keyword searches.

## Requirements

### Requirement: AQL01 Cover every first-party workspace owner

Checks SHALL discover every workspace member/target and SHALL reject empty scopes.

#### Scenario: SDK child regression
- **GIVEN** a violation in an SDK child module or integration target
- **WHEN** full-workspace lint or an applicable commit check runs
- **THEN** it fails with a source/rule diagnostic instead of checking only packages[0].

### Requirement: AQL02 Enforce the selected quality profile

The completed profile SHALL apply the blocking Rust/Clippy/native rules above,
without broad suppressions, baselines or a newly required language runtime.

#### Scenario: Coherent lint exception
- **GIVEN** a justified exception for one existing public ownership contract
- **WHEN** the narrow expect is no longer needed
- **THEN** unfulfilled-expectation checking fails so that the stale exception is removed.

### Requirement: AQL03 Resolve findings without changing editor contracts

F1-F7 and source-policy violations SHALL be corrected within current owners.
Command, source/history, process, wire and persistence contracts SHALL remain.

#### Scenario: Completion changes context
- **GIVEN** an in-flight completion and a source/view/profile context change
- **WHEN** its concern owner retires that work
- **THEN** late results cannot reopen or edit the new target and ordinary input works.

### Requirement: AQL04 Validate staged content without modifying it

The hook SHALL check the exact staged candidate and SHALL leave source/index
content intact on success, failure and cancellation.

#### Scenario: Unstaged repair masks staged failure
- **GIVEN** invalid Rust staged and an unstaged repair in the same file
- **WHEN** a commit is attempted
- **THEN** the hook rejects the staged source and preserves both versions.

#### Scenario: Unstaged error beside valid staged code
- **GIVEN** valid staged Rust and a later invalid unstaged edit
- **WHEN** the staged candidate is checked
- **THEN** validation uses the valid staged source and does not overwrite either version.

### Requirement: AQL05 Preserve ownership across worktrees and competing checks

Each checkout SHALL own its mutable lint cache and concurrent invocations SHALL
serialize or refuse explicitly without disturbing another build or user content.

#### Scenario: Existing lint owner
- **GIVEN** an active check owns this checkout's export/build slot
- **WHEN** another commit check requests it
- **THEN** ownership is reported or safely awaited within the documented policy,
- **AND** neither invocation deletes the other's files or shares another worktree cache.

### Requirement: AQL06 Keep enforcement usable and documented

Setup SHALL expose simple commands, actionable failures and measured cold/warm
cost; docs-only commits SHALL retain applicable fast checks.

#### Scenario: Installed hook in a linked worktree
- **GIVEN** the repository hook is installed and a linked worktree has current tooling
- **WHEN** a normal commit is attempted there
- **THEN** its own staged code/policy and private cache are used without new manual flags.

## Acceptance and closure

Retain native test names, assertions, publication/cancellation/resource limits and
all meaningful existing journeys. Record actual final lint, complete selected
native workspace tests, hook partial-staging/concurrency cases, SDK docs, Markdown
and generated Spore checks. Recheck the review against final code and identify
remaining limitations truthfully. One successful exact integrated gate permits
only the configured Markdown closure. Main/release/mirror actions stay separate.

## Delivery evidence

AQL01-AQL06 are Completed through all ten integrated slices and three bounded
gate repairs. The unchanged controller gate passed on clean candidate
`5625b0dbd5648ca81c888a19ad1faedc5241524d` in validation `6XRS3YAETTKWUXPPWZ7L323GHI`.
The [review closure](../report/rev-architecture-nit-review.md#review-limits-and-remediation-closure)
resolves F1-F7; the [Slice 10 closure](../report/slices/rev-architecture-quality/slice-10-enforcement.md#delivery-set-closure)
records full-workspace lint, native tests, exact staged-index checks, local hook
installation, SDK docs, Markdown/Spore and independent Go interoperability.
The criteria map and all frozen requirements remain unchanged.
