# Use a language server

For managed Rust on GNU/Linux x86-64, run `M-x lsp-install-server` (or
`:lsp-install-server` in modal Normal), then choose `rust-analyzer`. Install
`curl`, `gzip` and `sha256sum` separately if they are missing. The operation
runs while editing remains available, verifies an exact release and saves the
server choice. It preserves the current LSP mode: use `lsp-mode` separately
to enable continuous analysis. Missing Rust toolchain or sources require a
separate toolchain installation.

The equivalent `lsp-setup` path is `rust`, then `install:rust-analyzer`.
`lsp-servers` shows progress and ready/incomplete installations. If settings
changed during the operation, the verified payload stays ready and unselected.
Choose its `ready:<id>` entry to reuse it after resolving the configuration
conflict; no second download is needed.

For managed Go, choose `gopls` in `lsp-install-server`, or `go` then
`install:gopls` in `lsp-setup`. Install Go separately first. Acquisition uses
the local compiler and reports an insufficient version without downloading a
replacement toolchain.

For TypeScript/JavaScript/TSX, choose `typescript-language-server`, then the
language to select. For HTML/CSS/JSON, choose `vscode-langservers-extracted`,
then the language. These recipes require installed Node/npm, retain exact package
versions and a lockfile, and disable package scripts. TypeScript remains on major
6. The web package supplies three independently verified profiles; select another
language later through its same `ready:<id>` entry without another installation.
Each choice inherits known associations and current settings. Use `lsp-mode`
separately to enable analysis.

Use an installed language server to complete names from unsaved text in the
same popup as native completion. Rust uses `rust-analyzer` by default. Keep the
server executable on `PATH`, or configure its absolute path. Existing/custom
registration reuses that executable without downloading another one.

## Register or select an installed server

Run `M-x lsp-setup` (or `:lsp-setup` in modal Normal). Enter the language ID,
then an existing compatible server name, an installed recipe preset, a new profile
name, or `native-only`. Language suggestions include known associations, configured
identities and recipe languages; another valid identity can still be typed.
For a new profile, enter the executable separately from its JSON argument array,
then its JSON root-marker array. Empty answers keep the inherited selected
profile's arguments and markers; enter `[]` to clear them. Settings and literal
environment overrides also inherit from that profile.

Known language associations are reused. For a new identity such as `elixir`,
enter a JSON pattern array such as `["*.ex", "*.exs"]` when asked. No grammar or
recipe is required. C-g cancels; modal Escape also cancels. Nothing is saved
until all answers form an admitted complete configuration.

Setup saves and applies the choice without enabling LSP. Use `lsp-mode` to
activate it and `lsp-status` to check its actual transport readiness. A missing
executable remains a recoverable prerequisite failure.

## Inspect recipes and change sources

Run `lsp-servers` to open a read-only snapshot. It shows each language's selected
source, all configured profiles, local executable availability, and the managed
recipe catalog with its optional acquisition prerequisites. It also reads bounded
managed facts to show versions, readiness, current job phase and selected profiles.
It runs no helper and looks up no remote versions. Available external executables
have not necessarily completed LSP initialization; use `lsp-status` for the current
file. Use ordinary buffer switching to return to your source.

For an already installed Go server, choose `go`, then `recipe:gopls` in
`lsp-setup`. The preset observes the executable on the inherited PATH and suggests
its structured launch arguments. Existing root markers, initialization options,
settings and literal environment values inherit from the selected profile.
Empty answers keep those values; `[]` clears arguments or markers. Known `*.go`
and user associations are reused without another patterns prompt. A preset never
overwrites a configured profile with the same name. If the executable is missing,
choose an installed absolute path or inspect the catalog prerequisites.

Use `lsp-select-server` to change an existing language's source without registering
a new executable. Enter its language ID, then a compatible configured profile,
`ready:<id>` installation or `native-only`. It saves and applies the choice without changing global LSP mode.
A language without known associations asks for patterns; a catalog entry or grammar
is not required. C-g or modal Escape cancels without writing settings.

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

Run `lsp-reload-settings` after editing server entries, preferences or file
associations in the selected user root or its includes. It preserves the running
LSP mode, buffer exclusions and startup editing profile. Restart rev after
changing its inherited process environment, such as `PATH`. Disabling and re-enabling `lsp-mode` also permits a
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
