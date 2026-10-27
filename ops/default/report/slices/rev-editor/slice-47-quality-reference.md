# Slice 47: Audit source quality and product reference

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `refactor/rev-47-quality-reference`
PR: `#47`

## Purpose

Check the delivered source against its quality contract and document the real
editor commands, defaults, terminal requirements and recovery behavior.

## Delivered Behavior

- Eight production routines now use concise names, with every Rust caller updated.
- Both public command tables list all 62 live commands exactly once. A native
  catalog test checks the tables against the registry; the dispatch test rejects
  the unhandled-command fallback as well as comparing direct and named outcomes.
- README documents the release build, optional file argument and first editing
  steps. Indexed reference and how-to pages cover all bindings, product scope,
  file/history defaults, guarded saves, buffer/window use and recovery.
- Unreleased changelog entries describe the complete delivered product behavior.
  The plan's Escape deadline now matches the existing 100-ms implementation and
  terminal reference; runtime timing is unchanged.

## Implementation Notes

- T47.1: `95cd1cc506b76c5a26ca71c17b390064e8f6dfe4` performs the source audit, routine renames and catalog guards.
- T47.2: `9c50120253237606b0cfc400839e207702f48290` completes the product reference, how-to material and Unreleased outcomes.
- The production-function inventory contains 772 functions: 641 have at most five
  cyclomatic paths and 131 have six through ten. Maximum complexity is ten;
  maximum nesting is three. The source check enforces at most five parameters,
  including `self`, and first-party safe Rust. Embedded test functions are
  excluded from the production count. No production function name exceeds two
  snake-case words after the renames.
- Semantic inspection covered the command registry and routing, focused input,
  canonical edit/history publication, file publication and cleanup, syntax
  fallback, window geometry and rendering, and runtime ownership. Functions at
  the complexity ceiling retain bounded validation/projection or flat routing;
  no additional dispatcher abstraction was needed.
- Buffer candidates remain privately admitted before publication. Captured view
  membership and parsed-state expectations express synchronous owner invariants;
  ordinary filesystem, input, parser, history and geometry failures retain typed
  errors or their established fallback. Comments explain these boundaries rather
  than narrating statements. No production TODO, stub or unimplemented handler
  was found in the inspected paths.
- The six pinned direct dependencies retain distinct jobs: terminal I/O, rope
  storage, grapheme boundaries, cell widths, incremental parsing and the Rust
  grammar. No dependency was added or removed.

## Contracts Added or Changed

- Routine names change to `is_readonly`, `set_readonly`, `confirm_with`,
  `publish_with`, `parse_with`, `control_suffix`, `suppress_repeat` and `tree_entry`.
  Existing behavior and ownership remain covered by the native and PTY checks.
- The parameterized catalog-reference test requires exact, duplicate-free
  agreement between the registry and both documented command tables. Every live
  command must also avoid the unhandled-command fallback when dispatched.
- Product documentation states actual UTF-8/BOM/LF/CRLF preservation, save consent
  and publication outcomes, bounded per-buffer history, separate linear/block
  clipboards, focused cancellation, and supported terminal behavior.

## Files of Interest

- `src/`: concise routine names and updated production callers.
- `tests/commands/palette.rs`: registry/reference agreement and real dispatch guard.
- `README.md`: supported build, launch and initial editing flow.
- `docs/reference/bindings.md`, `docs/reference/defaults.md`: complete bindings,
  runtime scope and defaults, linked from the reference index.
- `docs/how-to/edit-files.md`, `docs/how-to/recover-edits.md`: file/session use
  and recovery, linked from the how-to index.
- `CHANGELOG.md`: accurate, complete Unreleased product outcomes.

## Validation

All seven ordered checks passed through Looper on task HEAD `9c50120253237606b0cfc400839e207702f48290` under
validation `F3AWJ3PXGKM7M57C57WMLODP7Y`. Final clean PR-head evidence is recorded
after the report and PR-reference commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 455 native tests, zero ignored.
- `python3 scripts/check-source.py`: passed; safe Rust, parameter, nesting and
  cyclomatic limits.
- `cargo build --release --locked`: passed; README build command.
- `python3 scripts/check-pty.py --suite acceptance`: passed; four real terminal
  journeys, zero skipped, with exact saved bytes and terminal restoration.
- `git diff --check`: passed.

Focused catalog tests passed. A documentation link-target check resolved all
62 local targets across README, changelog and the 16 documentation files.

## Risks and Follow-ups

The source checker measures structural limits; semantic findings above apply to
the inspected ownership and behavior paths. Terminal limitations and unsupported
features are stated in the product reference. Complete integrated acceptance
remains the recorded successor slice and controller gate.
