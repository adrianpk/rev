# Slice 1: Prefix entry and movement

Status: reviewing
Delivery set: rev-prefix-arguments
Plan: [`ops/default/plan/rev-prefix-arguments.md`](../../../plan/rev-prefix-arguments.md)
Tracker: [`ops/default/tracker/rev-prefix-arguments.md`](../../../tracker/rev-prefix-arguments.md)
Branch: `feat/rev-prefix-arguments-01-entry`
PR: `#314`

## Purpose

Deliver checked transient command arguments and signed native movement. T1.1,
T1.2 and T1.3 are implemented and the selected slice checks pass. Canonical
integration and the delivery-set gate remain controller-owned.

## Delivered Behavior

- C-u, Meta digits and Meta minus construct plain-prefix, explicit integer and
  negative-only forms. Numeric termination, negative zero, checked magnitude
  admission and C-g cancellation retain constant-size entry state.
- The echo area exposes the pending argument. Named digit-argument uses the
  ordinary one-digit prompt. universal-argument shares prefix-argument's
  description, effective shortcut, handler and command-usage identity.
- One typed invocation retains its argument, originating role and native source
  identity through M-x. Query edits, paste, candidate navigation, completion,
  drawing and resize do not consume it. Invalid or canceled input retires it.
- Signed character and word counts use existing grapheme, CRLF and word
  boundaries. Vertical counts use ordinary visual rows when wrapped and logical
  rows otherwise, retaining the desired column. Zero preserves point.
- Large traversal plans a disposable destination in bounded turns through the
  existing deferred-input owner. C-g or source invalidation retires it before
  publication. A fresh physical repeat uses the ordinary distance.
- Existing shell, Org and agenda C-u continuations retain active-map precedence.
  Other commands consume the argument and execute once.
- Wrapped Org block and table operations settle initial presentation through
  the existing layout owner before capturing their checked source/view origin.

## Implementation Notes

Input owns argument construction and retirement. The command registry owns the
four names and checked PrefixArgument value; Invocation carries that value and
its origin without adding serialization, dependencies or preferences.

Counted movement extends the existing layout request/FIFO owner. It advances at
most 256 native steps per turn, also checking the existing 4 ms input budget.
The selected native view is published only after the destination resolves.
Existing source/layout admission and endpoint behavior remain authoritative.
Boxed invocation outcomes keep the existing input error variants small.

Required wrapping checks exposed an initialization race already present at
preparation baseline `c268f429e78a73631eb7fddae69d6722bf6109c7`.
Block/table workers could capture a view before initial Org visibility changed
it. The shared layout preflight now lets the existing Org owner finish that
presentation before capture. The source/view guards and test assertions remain
unchanged; explicit visibility controls retain their existing owner.

T1.1 supplies entry, registry identity, descriptions and effective bindings.
T1.2 supplies invocation transport and native movement.
T1.3 supplies fourteen acceptance cases, current catalogs/default maps,
incremental references, the user-visible Unreleased outcome and this report.

## Contracts Added or Changed

- Public native ArgumentForm and checked PrefixArgument preserve raw form and
  numeric interpretation. Input exposes pending input and typed Invocation.
- The prefix helper is the only keymap leaf allowed to share a key with longer
  continuations. Other command/prefix collisions remain invalid.
- Named usage records a counted movement once and uses prefix-argument as the
  universal-argument alias identity.
- No persistent argument state, general dispatcher repetition or counted edits
  are introduced.

## Files of Interest

- `src/command/argument.rs`, `src/input/argument.rs`: checked values and input.
- `src/session/arguments.rs`, `src/session/layout.rs`: bounded native movement.
- `src/input/keymap/map.rs`, `src/input/hints.rs`: continuations and discovery.
- `tests/commands/prefix_arguments.rs`: fourteen native acceptance cases.
- `docs/reference/{commands,bindings,defaults}.md`,
  `docs/reference/config.toml`, `CHANGELOG.md`: current discovery and behavior.

## Validation

These are local slice checks, not the integrated delivery-set gate. The
controller validation receipt retains the exact clean candidate and ordered
command results outside the repository.

- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- Passed, 22 tests including 14 prefix cases: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments context_isolation transient_isolation prompt_reset workspace_reset tree_up_directory tree_actions tree_transitions tree_open_sequence --test-threads=1`
- Passed, 26 tests: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test wrapping -- --test-threads=1`
- Passed, 628 commands, 291 configuration entries, 91 pages and 629 links:
  `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- Passed, 2 focused wrapped Org regressions:
  `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test wrapping -- org::source_draft org::table_fields --test-threads=1`
- Passed: `git diff --check`

## Risks and Follow-ups

Counted edits/history, remaining contextual policies and complete joint guidance
remain in their approved later slices. No successor has been activated.
