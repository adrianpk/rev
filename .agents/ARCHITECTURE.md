# Native Architecture

`src/session/` owns editor orchestration; each concern owns its transitions.
Keep source/history coordinates, persistence, process IO, native projections and
plugin protocol responsibilities distinct. `crates/rev-plugin-sdk` is the public
wire client; it must not depend on private editor types or native callbacks.

Borrow observations for synchronous reads. Move retained values to their owner.
Use explicit `Arc::clone` or `Rc::clone` when shared lifetime is required and
review why that sharing is necessary. Keep cancellation, captured revisions,
exactly-once completion and resource cleanup with the operation that owns them.

Lint success does not prove cohesive boundaries, minimal allocation or correct
synchronization. Review those obligations with the concern's native regressions.
