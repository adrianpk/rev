# Slice 16: Package the complete startup configuration template

Status: delivered
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-16-bootstrap-template`
PR: [#96](https://forge.adrianpk.com/adrianpk/rev/pulls/96)
Integrated: `120101a17b85a7775a6afccda618cd59709428fe`

## Purpose

Supply the complete default-equivalent configuration as a compiled resource for
the native startup owner, independently of runtime reference files or a checkout.

## Delivered Behavior

`config::default_template()` borrows the complete English reference template
from a single compile-time asset. Its size is checked at compilation against
the ordinary 256-KiB configuration source limit. The asset includes the delivered
gutter/mouse controls, explicit palette and default context maps.

## Implementation Notes

The resource is embedded directly from the reference sample, with no second
maintained default source. A native fixture emits only its compiled bytes so
packaging can be verified independently of filesystem loading and terminal UI.
The accessor performs no IO or allocation. Startup publication remains assigned
to the next recorded slice; ordinary loading is unchanged.

## Contracts Added or Changed

CFG23 provides a packaged bounded template source. CFG05 retains pure loading
and immutable startup application.

## Files of Interest

- `src/config/template.rs`: bounded compiled resource and borrowed accessor.
- `examples/configuration-template.rs`: standalone native packaging fixture.
- `tests/configuration/template.rs`: source bounds, complete defaults and native event/frame equivalence.
- `tests/configuration/reference.rs`: packaged typed defaults, context maps and optional examples.
- `tests/configuration/candidate.rs`: pure packaged/resolve/load/prepare behavior.
- `scripts/check-config-reference.py`: actual compiled/reference/schema/catalog comparison.

## Validation

Development `cargo test --test configuration template::` passed packaged-source
admission. `cargo build --quiet --example configuration-template` built the
native fixture. A copied fixture executed in an owned directory without reference
or configuration files emitted the complete sample bytes within five seconds
and created no HOME/XDG files or directories. Its exit and stderr were clean.

Development `cargo test --test configuration` passed 87 groups. The existing
sample/default, complete context-map, optional-example and bounds checks now
consume the packaged accessor. The resolved scalar/format/palette settings equal
compiled defaults, all 111 context bindings equal their compiled targets, all 29
RGB roles are explicit, gutter/mouse fields are complete, and unused includes,
language rules and file associations remain inactive. Independent on-disk bytes
must equal the packaged resource. The English header describes the standalone
file without requiring a neighboring documentation file.

`python3 scripts/check-config-reference.py` passed all 72 schema/catalog entries,
29 RGB roles, 111 default bindings and three optional examples after comparing
actual compiled output with the reference. It builds the native fixture, copies
only that executable into an owned directory and executes it there with isolated
HOME and all four XDG roots. The five-second producer deadline, compile-time
256-KiB output bound, empty stderr and exact directory-entry assertion cover
runtime reference independence and no filesystem creation.

`cargo test --test configuration candidate::missing_candidate` passed with the
original missing-root/default assertions retained. Packaged access and parsing,
location resolution, explicit loading, projection and preparation also leave all
owned destinations absent and the fixture empty.

`cargo test --test configuration template::native_equivalence` passed identical
native actions and complete emitted frames for the packaged and built-in
snapshots through wheel/click/drag/release, cancellation, editing/undo, mouse-mode
on/off and gutter mode changes. Both paths retain truthful point/history and
the owned configuration bytes. This exercises semantics in addition to shared
source provenance. `cargo clippy --test configuration --example configuration-template -- -D warnings`
passed for the completed native checks and fixture.

Canonical evidence `BFCV7FDE46KJARMTSXDNHKRZJF` passed five commands and 175 native
checks on the integrated candidate `120101a17b85a7775a6afccda618cd59709428fe`.
Focused evidence `EE7VWXG2W7Z6PXYSFJLI5IXVPA` passed compiled/reference verification
on that same clean HEAD. Required ordered commands:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration --test gutter --test pointer`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused final commands:

- `python3 scripts/check-config-reference.py`
- `git diff --check`


## Risks and Follow-ups

Absent-file publication and its race/failure guarantees belong to Slice 17.
