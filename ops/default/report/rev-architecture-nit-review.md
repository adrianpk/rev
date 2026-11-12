# Architecture and Ownership Nit Review

Status: Completed
Reopened: 2026-11-12; recorded standalone acceptance failures require correction
Completed: 2026-11-12; expanded exact-candidate gate passed
Date: 2026-11-09
Baseline: abbf0c21eeae1680c02c7a477fa0894d7996a393
Scope: first-party editor, Rust SDK, integration tests, examples and developer quality tooling
Specification: [Architecture Quality and Commit Checks](../spec/editor-architecture-quality.md)
Plan: [Architecture Quality Delivery](../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../tracker/rev-architecture-quality.md)

## Findings

### F1 - Source-policy discovery does not cover the Cargo workspace

Resolution: Resolved; see the closure evidence below.

Severity: high
Area: tooling
File: scripts/check-source.py:16

Problem: discovery selects metadata packages[0], now rev-plugin-sdk, while its
recursive source collection covers only the editor's src/tests/examples/benches.
It does not collect SDK child modules and assigns the wrong complete crate-root
set to editor files. The existing source checker therefore cannot establish its
advertised complete first-party policy after the SDK workspace addition.

Why it matters: unsafe/root, parameter, nesting and complexity checks can omit
an entire owner or inspect a real crate root as an ordinary module.

Recommendation: make the existing Rust checker the policy owner, enumerate every
workspace member and Cargo target, exclude dependencies/vendor/build outputs,
reject empty selections, and offer native Bash/Cargo entry points. Retain old
script invocation compatibility through delegation rather than another policy.

### F2 - The complete Clippy target set already fails

Resolution: Resolved; see the closure evidence below.

Severity: medium
Area: tooling
File: tests/macros/pty.rs:16 and tests/macros/acceptance.rs:2

Problem: the two modules independently include the same completion acceptance
support source. Strict workspace/all-target Clippy stops on duplicate_mod.

Why it matters: the previous selected lib/bin check did not cover this target;
turning on a hook without repairing the existing target graph would block commits.

Recommendation: admit one shared test-support owner and reuse it from both
journeys. Preserve real terminal/resource assertions and all existing cases.

### F3 - Existing source contracts have accumulated unchecked violations

Resolution: Resolved; see the closure evidence below.

Severity: high
Area: architecture
File: examples/source-check.rs:80; representative src/git.rs:633 and src/session.rs:1145

Problem: a complete native scan of 1,787 Rust source files, 340,877 lines,
reports 608 diagnostics: 476 cyclomatic, 94 nesting, 34 parameter, three missing
crate-root unsafe prohibitions and one explicit raw-pointer type cast. Some
functions have several diagnostics. These counts are not 608 functional bugs.
Examples and tests are part of the inventory, not merely production code.

Why it matters: the existing five-parameter/three-level/ten-cyclomatic policy is
not enforced by recent delivery gates. Large dispatch/response functions mix
several operation-specific decisions and are difficult to verify in isolation.

Recommendation: preserve the existing metric definitions and thresholds. Extract
cohesive parsing, preparation, observation, dispatch and publication responsibilities
under current owners, rather than renaming branches or hiding logic in closures,
macros or arbitrary parameter bags. Keep all scenarios and resource assertions.

### F4 - Plugin calls copy request payloads and retain them after encoding

Resolution: Resolved; see the closure evidence below.

Severity: medium
Area: architecture
File: src/plugins/lifecycle/worker.rs:302 and src/plugins/lifecycle.rs:140

Problem: dispatch clones arguments and invocation context into CommandInvokeParams,
serializes another owned JSON value, then keeps the original Call with its
arguments in pending. Later completion/cancel handling needs correlation, context,
authority and reply ownership, not another retained copy of the request payload.

Why it matters: each bounded call can hold avoidable full-payload copies. This
is an allocation/ownership improvement, not a demonstrated unbounded-memory bug
or a claimed measured latency reduction.

Recommendation: separate queued request ownership from pending correlation and
serialize borrowed or transferred input where safe. Preserve cancellation,
causal-origin metadata, deadline/permit lifetime and exactly-once completion.

### F5 - Plugin keymap cache hits clone their comparison state

Resolution: Resolved; see the closure evidence below.

Severity: low
Area: architecture
File: src/session/plugins/keymaps.rs:52-86

Problem: refresh collects owned package IDs/generations and then clones the entire
states vector into an Observation before testing for an unchanged cache. This
performs an extra full vector/string clone on the common unchanged path.

Why it matters: cache reuse should avoid duplicating its own observation payload.
The package bound limits the cost; no performance number is inferred.

Recommendation: compare borrowed observations or transfer the already owned vector
once. Retain retirement, buffer selectors, current user precedence and conflict
invalidation semantics; do not replace an observed generation with a bare boolean.

### F6 - Completion lifecycle fields have no cohesive session owner

Resolution: Resolved; see the closure evidence below.

Severity: medium
Area: architecture
File: src/session.rs:139-149 and src/session/completion.rs:34-41

Problem: popup, generation, request, editing state, source, worker, resolver,
recent buffers and policy are separate Session fields. Dismissal and admission
must manually coordinate those fields across completion, context and input paths.
Other asynchronous facilities already have concern-owned Work structs.

Why it matters: lifecycle invariants are spread across the composition root and
handlers. A forgotten reset can keep a stale request or selection authority alive;
existing tests currently protect many such transitions, so this is structural
risk rather than a newly reproduced stale-publication defect.

Recommendation: introduce one completion concern owner with explicit state and
transition methods, preserving native input/view context and worker boundaries.
Do not create a generic job framework or change the public completion behavior.

### F7 - Commit-time enforcement and a shared lint profile are absent

Resolution: Resolved; see the closure evidence below.

Severity: high
Area: tooling
File: Cargo.toml:1; crates/rev-plugin-sdk/Cargo.toml:1; scripts/check-slice.sh:1

Problem: neither package inherits a shared lint profile; no repository-owned
pre-commit hook is installed. Source checks remain behind a Python script and
broad legacy slice checks, while recent focused delivery gates select different
Rust targets. No staged-index check protects partially staged commits.

Why it matters: a clean working-tree check can validate an unstaged correction
while committing an invalid staged version, and individual commands can miss SDK,
example or test targets. The accepted design checklist has no durable mapping to
its mechanical checks, regression protections and manual review obligations.

Recommendation: publish one explicit workspace profile, native source guard and
read-only staged-index hook, with narrow reasoned exceptions and real Git tests.
Keep whole-workspace lint available separately and install the shared hook only
once its final strict profile passes the integrated candidate.

## Mechanical evidence

The baseline workspace/all-target Clippy run used Rust/Clippy 1.98, Cargo.lock,
one build job, no incremental output and no dev/test debug symbols. It failed
on duplicate_mod in the macros integration target. A dependency emits an existing
unused-parentheses warning; dependency source is outside this review's policy.

A separate advisory pedantic/nursery inventory completed successfully as an
inventory, not as a strict acceptance pass. It includes many stylistic and
conditional suggestions. Reviewed examples include unnecessary pass-by-value,
redundant clones, eager collection and narrowing conversions. A diagnostic alone
does not establish a bad API contract or an exploitable conversion. Domain slices
must check ownership intent and existing bounds before changing signatures.

The complete native source scan used the current checker with all workspace
sources and actual Cargo roots supplied explicitly. It exited nonzero with the
608 diagnostics above. No application source was changed during the review.

## Non-issues and preserved decisions

- The terminal engine's Rc<RefCell<Effects>> bridges a single-threaded dependency
  callback into its owner. Presence of Rc/RefCell alone is not a defect.
- Arc<Mutex<_>> around coordinate budgets/caches and cross-thread worker mailboxes
  expresses shared ownership and synchronization. Do not replace it merely to
  make a search result disappear; preserve lock scope and publication invariants.
- Atomic multi-buffer publication already prepares participants, views and history
  before the infallible commit. Keep that boundary and its deliberate invariant
  assertions; a blanket panic ban would weaken rather than improve its contract.
- Concern-specific state stores, package namespaces, source units and opaque IDs
  are deliberate boundaries. This delivery introduces no generic persistence
  framework, source-store replacement, plugin API revision or text normalization.
- Unicode and floating-point test fixtures deliberately exercise unusual inputs.
  Unfiltered pedantic style suggestions are not authority to rewrite test data.

## Review limits and remediation closure

This is a structural review of critical ownership/interaction/process/persistence
boundaries combined with a complete mechanical Rust inventory. It is not a
line-by-line security audit of every test or a claim that all copying is harmful.

All seven findings are resolved by the integrated delivery and final expanded
gate on `422d6f66945d30f7ae640726cf1a63d17586bb1d`, validation
`VSWWUTEOGG2SGZW266X2FB7AMA`. The initial parent gate on
`5625b0dbd5648ca81c888a19ad1faedc5241524d`, validation
`6XRS3YAETTKWUXPPWZ7L323GHI`, remains historical evidence for that candidate.
The findings above retain their baseline descriptions and original file locations.

| Finding | Resolution and evidence |
| --- | --- |
| F1 | [Slice 1](slices/rev-architecture-quality/slice-1-foundation.md) makes the native checker own all Cargo members/target roots and rejects empty scopes. Final strict lint covers 1,838 first-party files, both packages and all 73 roots. |
| F2 | [Slice 1](slices/rev-architecture-quality/slice-1-foundation.md) shares macro acceptance support once. Final all-target/all-feature Clippy and native macro/PTY regressions pass. |
| F3 | [Slices 3-9](../plan/rev-architecture-quality.md#canonical-slice-map) extract concern-owned preparation, observation and publication; final source policy has zero findings with the five/three/ten and safety limits unchanged. Approved compatibility and fixture corrections are recorded in the plan. |
| F4 | [Slice 2](slices/rev-architecture-quality/slice-2-plugins.md) separates queued arguments from pending correlation, retaining permit, deadline, context and exactly-once completion. All 86 plugin tests and independent Go interoperability pass. |
| F5 | [Slice 2](slices/rev-architecture-quality/slice-2-plugins.md) compares keymap observations by reference and moves them into the cache after publication. Cache hits avoid the redundant observation clone; retirement, precedence and conflict regressions pass. |
| F6 | [Slice 8](slices/rev-architecture-quality/slice-8-interaction.md) gives completion its own state and admission/retirement transitions. Native completion, modal, command and macro regressions pass with freshness and cancellation guards retained. |
| F7 | [Slice 10](slices/rev-architecture-quality/slice-10-enforcement.md#delivery-set-closure) supplies inherited strict lints, canonical commands, exact staged-index enforcement and private per-worktree caches. All 28 real hook cases pass; local activation is verified. |

Mechanical checks cover their stated rules. Cohesive ownership, meaningful sharing,
synchronization and captured lifetimes still require the explicit manual review
and native regression obligations in the specification; the hook cannot prove
architecture automatically. No measured allocation or latency improvement is
claimed. The three opt-in upstream-acquisition cases remain ignored. The foreign
vendored compiler warning remains outside first-party policy.

The owner reopened the delivery on 2026-11-12 to correct the standalone failures
retained in [Slice 9](slices/rev-architecture-quality/slice-9-fixtures.md#risks-and-follow-ups).
Canonical PR #28 and bounded fixture repair #29 are integrated. All five complete
standalone entrypoints and all eight unchanged parent stages now pass on the
exact candidate above. The
[correction closure](slices/rev-architecture-quality-acceptance/slice-1-standalone-acceptance.md#delivery-set-closure)
records the fresh immutable receipt/log, runner identity, full native/PTY/resource
results and limits. Both delivery sets are Completed; earlier failures retain
their original outcomes. Only configured Markdown paths differ from the tested
candidate; no requirement, assertion or resource limit changes at closure.
