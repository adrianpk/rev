# Set Up Language Servers

Use a language server to complete a field from unsaved Rust text, then choose
whether that language uses semantic or native completion. The same setup works
through `M-x` in non-modal editing and `:` in modal Normal. The examples below
use `M-x`; replace that prefix with `:` when using the modal profile.

## Open a practice project

Keep your existing Rust compiler and Cargo installation. Create a small project
and open its source:

```sh
cargo new lsp-practice
rev lsp-practice/src/main.rs
```

Ordinary editing and native completion work before a language server is enabled.
Run `M-x lsp-servers`. This read-only buffer shows configured choices, local
availability, optional recipe prerequisites and any managed installations.
Opening it does not download a package or enable analysis. Return to `main.rs`
with the buffer selector, `C-x b`.

## Choose an installed server or acquire one

If `rust-analyzer` is already installed, run `M-x lsp-setup`. Answer `rust`, then
`rust` to select the default configured profile. This reuses the executable on
your PATH. For another installed executable, enter a new profile name and answer
the executable, JSON argument-array and root-marker prompts. Enter `[]` to clear
inherited arguments or markers; an empty answer keeps them.

For a managed Rust server on GNU/Linux x86-64, run `M-x lsp-install-server` and
choose `rust-analyzer`. This path needs installed `curl`, `gzip` and `sha256sum`.
You can continue editing while it resolves, acquires and verifies a release.
The equivalent setup answers are `rust`, then `install:rust-analyzer`.
Missing compiler or Rust source components require a separate toolchain action;
rev does not install or upgrade them.

Setup saves and applies its admitted choice. It keeps your current LSP mode,
editing profile and text. A ready installation and an active connection are
different states: use `M-x lsp-mode` to enable analysis for this session, then
`M-x lsp-status` to inspect this file's connection. A ready server can need more
time to load the project before returning semantic suggestions.

Press `C-g` to cancel an unanswered prompt; modal Escape also cancels it. To
cancel an operation already running, use `lsp-cancel-installation`. Its worker
remains occupied until its owned helpers finish stopping.

## Complete an unsaved field

Replace the practice source with this text. In modal editing, enter Insert
before typing:

```rust
struct Widget { unsaved_field: u32 }
fn main() {
    let item = Widget { unsaved_field: 7 };
    let _ = item.uns;
}
```

Keep these changes unsaved. Place the cursor after `uns`, before the semicolon,
and run `M-x completion-at-point`. With semantic results available, the popup
offers `unsaved_field`. Select it with the arrows or `C-p`/`C-n`, then accept with
Tab or Return. Undo restores `item.uns` in one operation. No completion saves
the file. A checked same-document import, when supplied, shares that operation.

Suggestions use the current buffer, including the new field. Semantic results
replace native results for that request; rev does not append two provider lists.
An empty, failed or unavailable server keeps native fallback where its language
syntax supports it. Cancel the popup with `C-g` or Escape without changing text.

## Choose native-only or another server

Run `M-x lsp-select-server`, enter `rust`, then choose `native-only`. This saves
the per-language choice and retires its semantic connection while preserving
the global LSP mode. Choose a compatible configured profile later to restore
semantic eligibility. To disable analysis for the whole session instead, toggle
`lsp-mode`; `lsp-disconnect` excludes only the current buffer.

The catalog is not a language allowlist. Use `lsp-setup` with another valid
language ID, an installed compatible stdio server and JSON file patterns when
asked. For example, a custom ID can associate with `["*.demo"]` without a
bundled grammar or managed recipe. Semantic completion can work there while
syntax coloring and native fallback retain their independent support limits.
The [configuration example](../../reference/language-server-example.toml) shows
installed Go and custom profiles, literal environment overrides and preferences.

## Maintain a managed installation

Use `lsp-servers` to find its installation ID. `lsp-update-server` asks for that
ID and lists the selected profiles before confirmation. A replacement must pass
verification before those profiles change together; your options and the old
payload remain. If the configuration changed during acquisition, the verified
replacement stays ready and unselected. Resolve the conflict, then choose its
`ready:<id>` entry without another download.

`lsp-remove-server` removes only owned, unreferenced, stopped payloads after
confirmation. Native-only does not delete the old profile table or its payload;
every referring profile must be changed or removed and its connections stopped
before removal. External executables and shared runtimes remain outside managed
removal. See [Use a language server](../../how-to/use-language-server.md) for
recipe prerequisites, conflict recovery, cancellation and cleanup procedures.

Next: [Follow External File Changes](13-file-reload.md). For exact server behavior, use
the [language-server reference](../../reference/language-servers.md).

[Previous](11-modal-editing.md) · [User Guide](index.md)
