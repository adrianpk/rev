# Terminal dependency source overrides

## alacritty_terminal 0.26.0

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

## Crossterm 0.29.0

The complete published Crossterm 0.29.0 archive is retained under its unchanged
MIT license. Its version, manifest, features and dependency versions are
unchanged. Retained recovery incident 3 authorized this source override for
SIGWINCH and native key readiness losing an edge in the Mio event adapter.

The published archive is:
https://static.crates.io/crates/crossterm/crossterm-0.29.0.crate

Its SHA-256 is:
`d8b9f2e4c67f833b660cdb0a3523065869fb35570177239812ed4c905aeff87b`.
The packaged Git provenance identifies upstream commit
`36d95b26a26e64b0f8c12edfe11f410a6d56a812`.

All 77 archive files match the published bytes except
`src/event/source/unix/mio.rs`. Its complete change is
[crossterm-readiness.patch](crossterm-readiness.patch). The adapter retains its
current readiness batch across event returns, advancing the TTY token only after
its queued bytes are exhausted. A byte-count ioctl prevents a resumed blocking
TTY descriptor from stalling the other tokens. Signal and event-stream wake tokens
advance before returning. One index retains the existing three-token batch;
there is no additional queue, polling timer or input retry.

The real-PTY regression queues SIGWINCH and Alt-x in both orders, and queues a
1,300-byte burst across TTY reads. Every case requires all native keys and the
resize without a further input write. Existing native/PTY journeys retain their
assertions, eight-second/1-MiB bounds and owned cleanup. Registry caches are
unchanged; Cargo selects this source through the root patch table.

The archive's line endings are preserved. Reproduce into an empty directory:

```sh
archive_dir=$(mktemp -d)
archive="$archive_dir/crossterm-0.29.0.crate"
curl --fail --location https://static.crates.io/crates/crossterm/crossterm-0.29.0.crate --output "$archive"
printf '%s  %s\n' d8b9f2e4c67f833b660cdb0a3523065869fb35570177239812ed4c905aeff87b "$archive" | sha256sum --check -
tar -xzf "$archive" -C "$archive_dir"
patch --binary -d "$archive_dir/crossterm-0.29.0" -p1 < vendor/crossterm-readiness.patch
diff -ru "$archive_dir/crossterm-0.29.0" vendor/crossterm-0.29.0
```
