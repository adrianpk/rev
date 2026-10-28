# Slice 16: Package the complete startup configuration template

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-16-bootstrap-template`
PR: pending

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
- `tests/configuration/template.rs`: source bound, exact sample and parser admission.

## Validation

Development `cargo test --test configuration template::` passed packaged-source
admission. `cargo build --quiet --example configuration-template` built the
native fixture. A copied fixture executed in an owned directory without reference
or configuration files emitted exactly 10,013 sample bytes within five seconds
and created no HOME/XDG files or directories. Its exit and stderr were clean.

Final ordered and focused evidence will be recorded through the controller on
the clean candidate after task and PR-reference commits.

## Risks and Follow-ups

Absent-file publication and its race/failure guarantees belong to Slice 17.
