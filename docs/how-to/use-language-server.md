# Use a language server

Use an installed language server to complete names from unsaved text in the
same popup as native completion. Rust uses `rust-analyzer` by default. Keep the
server executable on `PATH`, or configure its absolute path. Rev does not
install or download a server.

## Enable Rust completion

Open a Rust project containing `Cargo.toml`, then run `M-x lsp-mode`. This enables
LSP for this session and automatically attaches eligible files as you visit
them. Run `M-x lsp-status` to inspect the current connection. A ready connection
can still return no suggestions while the server loads its workspace.

For a persistent choice, change the existing table in
`~/.config/rev/config.toml` and restart rev:

```toml
[lsp]
enabled = true
```

Use `rev/config.toml` beneath `XDG_CONFIG_HOME` when that variable is set. This
excerpt replaces the existing value; do not append a duplicate table. The
[complete sample](../reference/config.toml) includes the default Rust server.

In a Rust file, try a local field:

```rust
struct Widget { unsaved_field: u32 }
fn main() {
    let item = Widget { unsaved_field: 7 };
    item.uns
}
```

Leave the cursor after `uns` and request `M-x completion-at-point`. You can also
wait for the normal typing delay. The popup includes the field:

```text
item.uns
┌──────────────────────┐
│◆ unsaved_field  field│
└──────────────────────┘
```

The indicator and type label use syntax colors. The list can contain other
server candidates, and its width follows the pane.
Edit the field name without saving and request
again: suggestions follow the current buffer. Use arrows or `C-p`/`C-n`, accept
with Tab/Return, and undo with `C-/`.

If a server supplies an import with the selected completion, its checked
same-document edits and the name apply together in one undo operation.
Acceptance can wait briefly for item resolution. Keep editing or press `C-g`
to cancel; a partial import or primary-only fragment is not applied after
resolution fails. Completion never saves the file.

## Exclude or restore a buffer

Run `M-x lsp-disconnect` to use native completion for the current buffer. The
exception covers all views of that buffer and survives visits to other files.
Run `M-x lsp-connect` to restore its eligibility. Restoring does not enable LSP
when the global session mode is disabled. Closing the buffer clears its exception.
Other buffers sharing the same root keep their server.

Run `M-x lsp-mode` again to disable LSP for the session and stop its owned servers.
Native completion remains available within its supported language contexts.

## Recover a failed server

Inspect `M-x lsp-status`. For a missing executable, restore the binary at its configured path. A failed connection remains unavailable across repeated requests and
buffer visits. After restoring the prerequisite, run `M-x lsp-restart` from an
eligible file. It replaces that root's connection and reopens its documents with
current unsaved text. Other roots and buffer exclusions stay unchanged.

Configuration and the process environment are read at startup. Restart rev after
changing `PATH`, executable arguments, settings or server entries. Disabling and re-enabling `lsp-mode` also permits a
new launch attempt with the existing settings. Empty results, timeouts and
unavailable servers use native fallback; ordinary editing continues.

## Connect another installed server

Add an entry for its executable, language ID and project markers. For example,
replace these invented names with those of your installed server:

```toml
[files.associations]
"*.demo" = "demo"

[lsp.servers.demo]
languages = ["demo"]
command = "demo-language-server"
args = ["--stdio"]
root_markers = ["project.demo"]
initialization_options = {}
settings = {}
```

Merge these leaves into existing tables. Arguments are literal strings, without
shell or environment expansion. The server must support the
[admitted protocol profile](../reference/language-servers.md). A bundled grammar
is not required for semantic completion. Syntax coloring and native fallback
still depend on their own language support.
