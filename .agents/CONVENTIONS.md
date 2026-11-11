# Native Conventions

Preserve public commands, keys, SDK/wire versions, source coordinate units and
persistence formats. Retain existing timeouts, input bounds, queue capacities,
consent checks, test identities and semantic/resource assertions.

Use safe Rust. Every Cargo crate root declares `#![forbid(unsafe_code)]`.
First-party raw pointer types and FFI are forbidden. Functions have at most five
parameters including `self`, logical nesting three and cyclomatic complexity ten.
Extract cohesive validation, preparation or outcome owners; do not hide branches
in macros, closures or unrelated parameter bundles.

Use checked conversions at representation boundaries. Preserve intentional
internal invariant assertions; handle expected operating failures with `Result`.
A controlled lint exception requires a local `expect` with a specific nonempty
reason. Group, crate and module suppression, historical baselines and blanket
panic or reference-counting bans are forbidden.
