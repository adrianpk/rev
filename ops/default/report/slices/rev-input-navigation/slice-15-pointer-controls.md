# Slice 15: Activate native mouse defaults and runtime control

Status: delivered
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-15-pointer-controls`
PR: [#95](https://forge.adrianpk.com/adrianpk/rev/pulls/95)

Integrated: `40d8e252917229ec01e245192819d98f5cd4b71f`

## Purpose

Activate the complete native focus, wheel and canonical drag-selection path with
usable startup defaults and a session-only runtime control.

## Delivered Behavior

Typed `mouse.enabled = true` and `mouse.scroll_lines = 3` enter the existing
schema, include precedence, provenance and immutable startup snapshot. Distances
are integers in 1..=100. `M-x mouse-mode` toggles session admission, including
from tree focus, without rewriting the source. Disabled input rejects pointer
reports and removes capture/deadlines while retaining the canonical region.
Wheel/tree distance uses the configured value; drag-edge steps also clamp to
the current text-area height.

## Implementation Notes

The session owns one effective enabled flag. The frozen snapshot owns the
startup preference and step. The command catalog and context admission use the
same native handler as M-x. The native loop applies that flag through the existing terminal reporting owner
before the initial frame and after each bounded input batch. It acquires only
DEC 1002/1006, rejects pointer input immediately when disabled, and propagates
reporting failures through the ordinary full terminal cleanup path. Repeated
state synchronization emits no redundant mode writes. Documentation describes
the terminal-dependent bypass gesture, native disable path and bounded active
drag behavior without promising a universal physical modifier.

## Contracts Added or Changed

PTR01 and PTR06 supply the native defaults and compiled toggle. CFG05 retains
the startup snapshot and no-write runtime override boundary.

## Files of Interest

- `src/config.rs`, `src/config/{merge,project,validate}.rs`: typed control leaves.
- `src/command.rs`, `src/session/pointer.rs`: fixed command and session admission.
- `src/session/pointer/{scroll,autoscroll}.rs`: configured bounded distances.
- `tests/configuration/mouse.rs`, `tests/pointer/controls.rs`: real startup and handler outcomes.
- `docs/reference/config.toml`, `docs/reference/defaults.md`: exhaustive sample and catalog.
- `src/session.rs`, `src/terminal.rs`: native startup/runtime synchronization and cleanup ownership.
- `scripts/configuration_ui/pointer_controls.py`: actual on/off/restart/capture/fallback journeys.
- `docs/reference/terminal-input.md`, `docs/reference/bindings.md`: gestures, control and bypass guidance.

## Validation

Development `cargo test --test pointer --test configuration` passed 53 pointer
and 85 configuration groups. Coverage includes false/absent layering and source
origins, both integer endpoints and whole-candidate rejection, real M-x in editor
and tree focus, configured wheel/drag steps, disable during live capture,
retained native region and frozen settings after a root-file change.
`python3 scripts/check-config-reference.py` passed sample/model/catalog equality.

`python3 scripts/check-pty.py --suite pointer-lifetime` passed 13 owned PTYs,
including actual factory activation with one reporting pair, unchanged keyboard
editing, decoder coverage, partial acquisition/rollback, disable failure/retry,
normal/error/unwind restoration and passive filesystem behavior. The historical
factory assertion now checks the delivered enabled default and exact restoration
counts/order; its typing, history, prefix and resource assertions are retained.
The exhaustive reference guard passed all 72 catalog leaves and 111 bindings.

Development `cargo test --lib --test pointer` passed 15 library and 54 pointer
groups, including immutable snapshot retention and rejection of held toggle
repeats. `cargo test --test pointer controls::tree_distance` subsequently passed
the configured seven-row loaded-tree behavior with unchanged focus, point and
history. The unchanged safe-Rust/parameter/nesting/complexity guard passed after
activation. Focused native Clippy covers the completed controls and configuration.

`python3 scripts/check-pty.py --suite pointer-controls` passed five owned sessions:
factory enabled behavior; explicit startup off with seven-row runtime enable;
a new process restoring the unchanged false preference; disable during live
outside autoscroll through a mapped native command, retained region and no
revival after enable; and complete enabled/three-row fallback for invalid input.
The cases verify actual reporting pair counts/order, continued raw keyboard
input, pane focus, source/configuration bytes and normal cleanup. Paused/disabled
or retired capture emits no idle output and consumes no process CPU ticks.

The new driver explicitly retires its tested C-x prefix before the separate exit
chord. The invalid-source case uses a 180-column surface to display the complete
source/key diagnostic rather than asserting clipped text. Both corrections
preserve the assertions, original eight-second deadline and 1-MiB output bound.
Every session retains owned HOME/XDG roots, bounded 32-entry/64-KiB snapshots,
terminal restoration, child reaping, descriptor closure, no created preference
files and no implicit OSC 52 clipboard writes.

The first required run at `43e0c23f982d8c70215a9be09ca48abffe45743c` caught
an omitted `mouse-mode` entry in `docs/reference/commands.md`. The independent
catalog equality assertion remained unchanged; adding the delivered command to
that second reference corrected the omission. Final evidence is regenerated for
the corrected clean candidate.

Canonical controller evidence `6T2MWZHM27QLYA6MUUWWGZ6Z5M` passed all five
ordered checks and 336 native cases at the integrated commit above. Focused
`LYCDIYZDCCILOO66PDUJCSHKXG` passed 238 native cases, 26 owned PTYs and the
exhaustive reference guard on the same clean candidate. Required ordered commands:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test configuration --test commands --test display --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused final commands:

- `python3 scripts/check-config-reference.py`
- `cargo test --lib --test coordinates --test windows --test editing --test tree`
- `python3 scripts/check-pty.py --suite pointer-lifetime`
- `python3 scripts/check-pty.py --suite pointer-focus`
- `python3 scripts/check-pty.py --suite pointer-scroll`
- `python3 scripts/check-pty.py --suite pointer-selection`
- `python3 scripts/check-pty.py --suite pointer-autoscroll`
- `python3 scripts/check-pty.py --suite pointer-controls`
- `git diff --check`


## Risks and Follow-ups

All primary pointer slices 10-15 are integrated and the pointer ticket is solved.
