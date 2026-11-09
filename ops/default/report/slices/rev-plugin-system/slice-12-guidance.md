# Slice 12: Authoring and user guidance

Status: reviewing
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `docs/rev-plugin-system-12-guidance`
PR: pending

## Purpose

Make the experimental local-package workflow usable by editor users and Rust
package authors, with complete public contracts and executable examples.

## Delivered Behavior

A separate optional User Guide path introduces explicit admission, qualified
commands in M-x/modal colon, completion feedback, inspection and process control.
A task guide covers startup activation, contextual user overrides, prefixes,
semantic recording and failure recovery. Everyday editor, Git and Org chapter
order remains unchanged.

The authoring tutorial builds one public-SDK command using downloadable Rust,
manifest and startup configuration assets. It explains literal program paths,
protocol-only stdout, partial writes, callback completion and the additional
readiness/deadline work required for outgoing requests. The example completes
without changing source; readers can verify native feedback before adding edits.

The protocol reference catalogs every request and notification, direction,
capability, parameter/result shape, stable error kind and finite-profile field.
The behavior/default/sample references now describe implemented activation and
bindings. An explanation separates native buffers and transactions, local undo,
explicit file consent, generation lifetime, shared state and experimental versions.
The Unreleased entry links the optional lesson and working authoring example.

## Implementation Notes

Markdown remains canonical. Spore navigation adds an independent plugin part;
the projection admits the standalone authoring tutorial and copies its source
and TOML downloads. The existing strict documentation checker now verifies the
optional path in addition to the unchanged editor/Git/Org paths.

Two guidance regressions compile the exact downloadable source against only the
public SDK, admit the unchanged TOML assets, run both native selectors and F8,
and verify deactivation/re-activation/restart without source changes. They also
compare the wire catalog/profile with the public SDK and admit all published
JSON frames. The SDK adds one compiled Unicode-scalar read doctest. Existing
native process cleanup and private fixture roots retain ownership boundaries.

## Contracts Added or Changed

No API, dependency, package version or native runtime behavior changes. The
public docs distinguish schema-only register/unregister/messages methods from
implemented capabilities, root-dependent state from library defaults, atomic
publication from per-buffer undo, and committed work from cancellation. This
remains an experimental API; no release, installer or stability adoption is claimed.

## Files of Interest

- [Optional editor lesson](../../../../../docs/tutorials/user-guide/22-experimental-plugins.md) and its independent entry/navigation.
- [Authoring tutorial](../../../../../docs/tutorials/write-plugin.md) and downloadable hello/configuration assets.
- [Protocol reference](../../../../../docs/reference/plugin-protocol.md) and [ownership explanation](../../../../../docs/explanation/plugin-ownership.md).
- [Guidance regressions](../../../../../tests/plugins/guidance.rs), SDK doctest and the existing documentation projection/check examples.

## Validation

All eight frozen commands passed locally. The final clean PR HEAD receives
separate controller evidence for these exact ordered commands before handoff:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- guidance --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
8. `git diff --check`

Guidance passed two tests covering both editing profiles. SDK validation passed
twelve protocol tests and one compiled doctest; command regression selection
passed 52 tests. The documentation checker passed 106 reachable pages, 743 local
links/anchors, 684 commands and 299 configuration entries. Strict Spore check and
render passed. The first build exposed unquoted angle-bracket placeholders in
the existing plugin reference; code formatting corrected those source examples.
The Rust download also passed standalone rustfmt checking. Rendered entry and
support pages are present, and all three downloads match source bytes. Output
stays in ignored target paths; the guide was built locally without publication.

## Risks and Follow-ups

The hello program deliberately has no outgoing host requests or long-running
callbacks. Its blocking pipe loop is not a general asynchronous plugin runtime;
the tutorial names the required readiness and monotonic-deadline extension.
Experimental consumers must pin compatible editor/SDK sources. Independent Go
interoperability and joint delivery acceptance remain Slice 13; the controller
owns integration, the full gate and successor startup.
