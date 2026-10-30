# Install rev

You need Linux, Rust 1.98 or newer, Cargo and a C compiler.
Use a local checkout of the [rev source](https://forge.adrianpk.com/adrianpk/rev).

From the repository root, run:

```sh
cargo install --path .
```

Cargo builds and installs the executable. With Cargo's default installation
location, make sure `~/.cargo/bin` is on your `PATH`.

You can now start rev from the directory containing your files:

```sh
rev notes.txt
```

Follow [Edit Your First File](../tutorials/user-guide/01-first-file.md) for a
short editing session.

## Check which executable starts

Some Linux systems also provide a utility named `rev` that reverses text.
If the command waits for input without opening an editor, check:

```sh
command -v rev
```

Put Cargo's installation directory before `/usr/bin` in your `PATH` so the
editor is selected.

## Update an existing installation

After updating your source checkout, run the same installation command again.
Cargo rebuilds and replaces the installed editor.

For the accepted arguments, see the [CLI reference](../reference/cli.md).
