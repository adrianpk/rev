# Slice 3: Load bounded include graphs

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-03-config-includes`
PR: pending

## Purpose

Resolve user configuration locations and load the explicitly selected source
graph under fixed traversal and read bounds, preserving declaration provenance.

## Delivered Behavior

- Resolves config, data, state and cache roots independently from absolute XDG
  overrides or HOME fallbacks. Empty and relative overrides are ignored;
  unresolved roots identify their variable. Resolution creates no directories.
- Selects `config.toml` under the user config root. A missing optional root
  produces an empty source graph for built-in defaults; a present broken root
  or any broken explicit include rejects the graph.
- Reads includes depth-first in declaration order relative to each declaring
  path, with absolute local paths also supported. Source occurrences retain
  parent, declaration position, selected path and canonical path.
- Enforces root depth zero, at most 16 include edges, 64 source occurrences,
  256 KiB per source and 2 MiB total. Repeated sources count each time.
- Detects active canonical-path and device/inode cycles, including symlink and
  hard-link aliases, while admitting shared leaves after earlier branches finish.

## Implementation Notes

- T3.1: `b55d825bee9bf0d35e638beca57af290f5a2fb27` adds independent XDG
  location resolution and passive location tests with OS-native path bytes.
- T3.2: `6840ad1cf9bdfe8b607ac78751f9060b48e92cbd` adds bounded reads,
  explicit include traversal, occurrence provenance and postorder layering.
- T3.3 adds exact-boundary, alias, permission, literal-path and failure tests
  and this report. All 15 new focused configuration tests pass without skips.
- Regular-file checks precede opening and follow handle acquisition. Reads
  use `Read::take` with the smaller of the source and remaining total budgets;
  metadata size checks precede allocation and follow reading. No sentinel byte
  is read beyond either admitted byte limit.
- The active stack is bounded by include depth, and occurrence admission
  precedes each open. A global visited-file set would incorrectly reject
  ordinary repeated leaves, so it is not used.
- Each successful source is typed and validated before entering the graph.
  Any error discards the owned walk. Graph errors identify the declaring
  include and target path; parse errors retain their original source locations.

## Contracts Added or Changed

- `Roots::from_env` and `Roots::resolve` resolve all four user locations without
  mutating environment or filesystem state. `Roots::config_file` selects the
  sole implicit source.
- `config::load(&Path)` returns `Sources` or a diagnostic. Only an absent
  optional root returns an empty graph; unreadable, dangling and non-regular
  roots remain errors.
- `Sources::sources` records occurrences in preorder; parent indices remain
  stable. `Sources::order` gives declared depth-first postorder, with the root
  last, for subsequent settings resolution. `total_bytes` counts repeated reads.
- Include paths are local literals. Empty, NUL and remote URI values reject;
  variable, tilde and glob-looking names receive no expansion. Unlisted files
  are not discovered, loaded or repaired.

## Files of Interest

- `src/config/xdg.rs`: independent root resolution and configuration filename.
- `src/config/load.rs`: regular-file reads, graph budgets, ordering and identity.
- `tests/configuration/xdg.rs`: overrides, fallback errors and passive lookup.
- `tests/configuration/includes.rs`: ordering, relative children and shared leaves.
- `tests/configuration/include_bounds.rs`: exact budgets, aliases and real IO errors.
- `tests/configuration/fixture.rs`: owned temporary directories and cleanup.

## Validation

Development checks:

- `cargo test --test configuration xdg::`: passed; three tests.
- `cargo test --test configuration includes::`: passed; three tests.
- `cargo test --test configuration include_`: passed; nine boundary tests.
- `cargo clippy --lib -- -D warnings`: passed.

The controller stores final results for the exact clean PR head after its
PR-reference follow-up commit. Required commands, in canonical order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

Layered settings resolution and candidate fallback remain assigned to Slice 4.
Runtime startup activation remains assigned to Slice 20.
