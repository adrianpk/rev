# Write Your First Experimental Plugin

Build a local Rust program, declare one command and run it through rev's native
selector. The command completes without changing text or files. Use a disposable
practice directory and a source checkout of rev with Rust 1.98 or newer.

The API is experimental. Pin the editor checkout and build against its matching
SDK; a package version, SDK version and `rev.plugins/0.1` are separate identities.
Plugins run with your account's ordinary permissions. Review code before running
it. There is no package download or installation command in this workflow.

## Create the program

Arrange a practice directory containing the existing `rev/` checkout and a new
sibling program. Run this from their parent directory:

```sh
cargo new --bin hello-plugin
```

Add the local SDK dependency to `hello-plugin/Cargo.toml`:

```toml
[dependencies]
rev-plugin-sdk = { path = "../rev/crates/rev-plugin-sdk" }
```

Replace `hello-plugin/src/main.rs` with the complete
[hello source](../reference/plugin-hello.rs). Build it:

```sh
cargo build --manifest-path hello-plugin/Cargo.toml
```

The program constructs a public `Client` for `field.hello` version `0.1.0`,
registers the `field.hello.ping` handler for initialization and waits on stdin.
The SDK validates the host's exact API, session, generation and finite profile,
then queues the acknowledgement. The program drains output with partial-write
accounting before reading again. Each accepted callback receives one typed
response. Diagnostics use stderr; stdout contains only protocol frames.

This example issues no host requests and completes callbacks immediately.
Extending it with outgoing requests requires servicing `next_deadline()` and
`expire()` with a monotonic clock while also reading callbacks and cancellation.
Use readiness-based I/O for that event loop. Do not stop reading while waiting
for one nested response. The SDK queues and decodes; it supplies no I/O thread,
runtime, filesystem access or editor internals.

## Declare the command

Save the [example manifest](../reference/plugin-example.toml) as
`plugin-example.toml` in the practice directory, beside `hello-plugin/` and
`rev/`. Its program path resolves against that manifest, and its command matches
the Rust handler exactly:

```toml
[[commands]]
name = "ping"
description = "Run the example command without changing text."
parameters = []
prefix = "none"
recording = "reject"
```

`field.hello.ping` is the qualified command ID. There is no implicit `ping`
alias. The native selector supplies `{}` arguments. This declaration accepts
no prefix and refuses recording; neither feature is inferred from the handler.
An empty activation table keeps the program lazy until command use or explicit
activation. Do not add a shell wrapper or depend on a PATH lookup: program and
literal argument resolution are defined by the manifest contract.

## Run it in the editor

Follow [Try an Experimental Plugin](user-guide/22-experimental-plugins.md),
using the absolute path of this manifest in your startup configuration.
`plugin-list` should first show the declaration without an active process.
Accept `field.hello.ping` in M-x or modal `:`. The command produces a
`Plugin command field.hello.ping [request-id]: completed` notice with its actual
request identity, and your practice text stays
unchanged. Its JSON result is a completion receipt, not text inserted into a
buffer. `plugin-list` now shows the active generation.

If the command fails, inspect its bounded diagnostic with `plugin-list`.
Check the compiled program path, executable permission, package/version/handler
agreement and stdout framing before explicitly restarting. A missing executable
does not prevent passive discovery; an incompatible handshake prevents execution.

## Add editor work deliberately

Start with `buffers.list`, then `buffers.describe` and revision-qualified
`buffers.read`; handles and Unicode scalar ranges are public wire values.
Use `buffers.create` and `buffers.show` for an ordinary unsaved result. Showing
a result requires the display capability and an admissible native input context.
Returned buffers remain native buffers after the package stops.

For edits, retain the original stamps and submit one `edits.apply` batch. On
`stale_revision`, recalculate from a fresh read and make a new explicit proposal;
never silently replace an expected revision. Saving and closing are separate
requests with native file observations and consent checks. Event callbacks and
package-state CAS have their own lifetimes and failure rules.

Use the [protocol reference](../reference/plugin-protocol.md) for every method,
wire shape, error and capability, the [behavior reference](../reference/plugins.md)
for configuration and limits, and [plugin ownership](../explanation/plugin-ownership.md)
for transaction and compatibility boundaries. Return to [Tutorials](index.md).
