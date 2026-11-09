# Slice 2: Passive declarations and configuration

Status: reviewing
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `feat/rev-plugin-system-02-declarations`
PR: `#2`

## Purpose

Deliver T2.1-T2.3: admit explicit local package declarations and expose their
qualified command metadata through native discovery without executing programs.

## Delivered Behavior

- Startup configuration adds disabled-by-default plugins.enabled, manifests,
  startup and keybindings leaves with existing include/array precedence.
  Disabled configuration opens no manifests or program probes.
- Enabled startup admits the complete passive catalog atomically. Invalid,
  unsupported or duplicate declarations retain complete native defaults and a
  diagnostic; an earlier valid namespace cannot leak from a rejected candidate.
- Manifest paths retain the assigning configuration source. Relative programs
  resolve against their manifest directory, with frozen absolute paths and
  literal argv. No PATH search, shell, execution or directory scan is used.
- M-x and modal colon show each package's qualified commands and descriptions.
  Unavailable calls report the full identity and installed/not-declared state.
- plugin-list opens a bounded read-only inventory. Activate/restart select a
  configured package and report unavailable activation; deactivate reports the
  inactive state. Source buffers and native cancellation retain their ownership.
- Passive binding defaults and user selectors are validated and inspectable.
  They remain ineffective, preserving native maps, transient owners and recovery.

## Implementation Notes

Implementation: `3914b057d19dee39bcaf7dfaed7702d89d0ee90c` implements T2.1-T2.3
and introduces this report. Canonical
[PR #2](https://forge.adrianpk.com/rev/rev/pulls/2) targets dev.

Configuration owns the four startup leaves. A shared immutable registry on the
configuration snapshot retains admitted packages, declarations and resolved
paths. Ordinary snapshot cloning does not reload metadata. Each manifest is
read through an owned nonblocking descriptor, checked as a regular UTF-8 file
and bounded to 256 KiB, including growth during the read.

Native Command remains Copy. PluginName retains up to 130 identity bytes
without changing MacroName or persisted macro formats. The native palette's existing candidate/description path admits these
identities; no parallel selector or dispatch engine is added.

The larger complete command value required a small private Deferred type for
modal grammar results. It contains only sequence-wait or static-message states;
the existing dispatch boundary converts it to the same native outcome. This
keeps error values small without changing count/operator behavior or adding
allocation/lint exceptions.

The built-in inventory now contains 684 commands. Its description guard retains
every approved existing summary and adds the four new descriptions from the
plugin reference. Complete defaults, packaged configuration, command/binding
inventories, the reference index and Unreleased behavior are updated together.

## Contracts Added or Changed

- Strict manifest objects cover package/API/version, literal process launch
  metadata, command argument/prefix/recording policy, activation conditions and
  passive bindings. Unknown fields, invalid identities and duplicate commands
  reject admission before discovery.
- Finite bounds include 64 packages, 64 commands and 128 defaults per package,
  512 total commands/default contributions, eight startup selections and
  512 user overrides. Additional manifest/path/argv bounds are documented.
- Global bindings reject buffer selectors; buffer scope requires filetypes,
  buffer_owner or both, with intersection semantics. Native key parsing is reused.
- Process activation, executable RPC capabilities and native contribution
  reachability/conflict installation remain unavailable at this increment.

## Files of Interest

- `src/plugins/declarations.rs`: finite manifests and atomic passive registry.
- `src/config/plugins.rs`, `src/config/resolve.rs`: startup leaves, provenance
  and shared immutable catalog.
- `src/command/plugins.rs`, `src/input/palette.rs`, `src/session/plugins.rs`:
  native identities, described discovery and unavailable-handler feedback.
- `src/input/grammar.rs`, `src/input/grammar/operator.rs`,
  `src/input/dispatch.rs`: compact deferred modal results.
- `tests/plugins/declarations.rs`, `tests/commands/palette.rs`: real
  config/catalog/profile boundaries and complete description coverage.
- `docs/reference/plugins.md`, `config.toml`, `defaults.md`,
  `commands.md`, `bindings.md`: complete passive contracts and defaults.

## Validation

All seven ordered implementation checks passed locally:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- declarations --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

The SDK selection passed 12 tests and command regressions passed 52 tests.
The declaration selection passed seven tests, including both native profiles,
a real executable that would create a marker, namespace/unknown-field rejection,
source-relative paths, selector admission, Unicode source preservation and
exact/over-limit file and aggregate command bounds. No fixture program is started.

Additional focused modal regression passed 11 tests with:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal -- operators operator_ownership bounded_counts count_conflicts --test-threads=1`

Documentation validation passed 684 commands in each combined inventory,
299 configuration entries, 100 reachable pages and 684 local links/anchors.
These are focused slice checks. The controller validator owns the exact
clean-HEAD evidence for the final PR-reference commit before handoff.

## Risks and Follow-ups

The interface remains experimental. Process activation belongs to Slice 3;
native binding installation and resolution belong to Slice 7. No executable
host API capability, performance measurement, installation or distribution
behavior is claimed by this passive catalog.
