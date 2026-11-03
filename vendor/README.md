# Terminal engine source override

This directory contains the runtime sources of `alacritty_terminal` 0.26.0,
under its unchanged Apache-2.0 license. The source override fixes a registered
SIGCHLD pipe sender that survives a failed child spawn. The original spawn
error is preserved. Version, feature selection and terminal behavior are unchanged.

The published archive is:
https://static.crates.io/crates/alacritty_terminal/alacritty_terminal-0.26.0.crate

Its SHA-256 is:
`bda177466b9524d59f1b12f0dd30b68696788e9992a7e959021c4a0ed96fcf59`.
The packaged Git provenance identifies upstream commit
`94e7c8874e526b1e67b349d9ba30ddf81669119e`.

All 23 runtime source files match that archive except `src/tty/unix.rs`, whose
complete change is [terminal-spawn-cleanup.patch](terminal-spawn-cleanup.patch).
The published normalized manifest is retained with its `[[test]]` stanza
removed because the 47 MB upstream reference-fixture corpus is not vendored.
Its original manifest, README, provenance and license are included unchanged.
Repository terminal acceptance continues to use real PTYs and the same
failed-launch descriptor assertion.

To reproduce the source tree into an empty directory, download and verify the
archive, extract its runtime and package metadata, omit its reference-test
target, and apply the patch:

```sh
archive_dir=$(mktemp -d)
archive="$archive_dir/alacritty_terminal-0.26.0.crate"
curl --fail --location https://static.crates.io/crates/alacritty_terminal/alacritty_terminal-0.26.0.crate --output "$archive"
printf '%s  %s\n' bda177466b9524d59f1b12f0dd30b68696788e9992a7e959021c4a0ed96fcf59 "$archive" | sha256sum --check -
tar -xzf "$archive" -C "$archive_dir" --wildcards 'alacritty_terminal-0.26.0/src/*' alacritty_terminal-0.26.0/Cargo.toml alacritty_terminal-0.26.0/Cargo.toml.orig alacritty_terminal-0.26.0/LICENSE-APACHE alacritty_terminal-0.26.0/README.md alacritty_terminal-0.26.0/.cargo_vcs_info.json
sed -i '/^\[\[test\]\]$/,/^$/d' "$archive_dir/alacritty_terminal-0.26.0/Cargo.toml"
patch -d "$archive_dir/alacritty_terminal-0.26.0" -p1 < vendor/terminal-spawn-cleanup.patch
diff -ru "$archive_dir/alacritty_terminal-0.26.0" vendor/alacritty_terminal-0.26.0
```

The repository uses Cargo's `[patch.crates-io]` mechanism. Registry caches are
not modified. The root dependency remains exactly version 0.26.0 with default
features disabled. Retained recovery incident 1 authorized this source override.
