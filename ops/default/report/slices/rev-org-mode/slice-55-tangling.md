# Slice 55: tangle Org source with bounded noweb references

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-55-tangling`
PR: `#156`

## Purpose

Deliver native tangling under the
[approved plan](../../../plan/rev-org-mode.md#slice-55-tangle-org-source-with-bounded-noweb-references)
and [tracker](../../../tracker/rev-org-mode.md#slice-55-tangle-org-source-with-bounded-noweb-references).

## Delivered Behavior

Explicit whole-buffer or current-block tangling extracts declared source into
separate checked files. Named and grouped noweb references expand recursively,
retaining relative indentation and source-order concatenation. Each existing
output requires replacement confirmation; per-file outcomes remain inspectable
after cancellation, stale targets or partial failure. Canonical source, unsaved
edits, history, BOM and newline format remain unchanged.

## Implementation Notes

A bounded, cancellable source observation produces one immutable plan before
file effects. All expansions and destinations pass preflight together. Ordinary
checked atomic publication retains each file's real outcome and mode. Pinned
parent directories reject symlink replacement. The editor retains exclusive
source input during publication. Literal native headers never evaluate code.

See the [source block reference](../../../../../docs/reference/org-source-blocks.md#tangling)
and the Org manual's [tangling](https://orgmode.org/manual/Extracting-Source-Code.html)
and [noweb syntax](https://orgmode.org/manual/Noweb-Reference-Syntax.html).

Task commits: T55.1 `87343df9eefaf7dd584ce365eeb278d0890cace4`, T55.2 `4480d469780d52215ef2814ee7bda7f918a9f35f`, T55.3 `a66cbda8725e74d4379de994742faa8076f7eb16`.

Canonical PR: [#156](https://forge.adrianpk.com/adrianpk/rev/pulls/156), merged into `dev` at `32e45be6f151015c257f44dd4247178c6c7874a9`.

## Contracts Added or Changed

- ORG28: bounded named expansion and explicit native source tangling.
- ORG18: checked file effects, cancellation and retained independent outcomes.

## Files of Interest

- `src/org/tangle.rs` and `src/org/tangle/`: pure expansion and checked file plans.
- `src/session/tangling.rs`: source ownership, replacement consent and outcomes.
- `tests/org_source/tangling.rs` and `tangle_bounds.rs`: reference, selection,
  header, path, revision and resource contracts.
- `tests/org_io/tangling.rs`: real editor and filesystem workflows.

## Validation

Final required receipt `KN6X63NLXH2KT5RWRNM2RMAURM` passed on
`32e45be6f151015c257f44dd4247178c6c7874a9`:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_source --test files`
- `python3 scripts/check-source.py`
- `git diff --check`

All 307 tests passed: 71 files, 158 I/O and 78 source; zero failures, ignored
or filtered tests. The unchanged source checker retained safe Rust, at most
five parameters, three nesting levels and cyclomatic complexity ten.

Focused development verified exact 64-level, 512-block, 256-file and 8 MiB output
boundaries, the independent 100000-step work ceiling and stale source rejection.
Filesystem cases cover whole-plan preflight, alias/symlink protection, changed
parents, partial success, cancellation, per-file consent, permissions, Unicode,
BOM/CRLF source fidelity and read-only current-block extraction. Nine editor/I/O
and nine native source/expansion tests protect tangling behavior.

Focused receipt `A5ZYSYSFYSM3B5Y2YGNQBMHX6W` passed on the same head:

- `cargo test --test commands --test configuration --test org_editing --test syntax -- --test-threads=4`
- `python3 scripts/check-config-reference.py`
- `git diff --check`

All 538 focused tests passed. An earlier default-parallel focused run exceeded an
existing clock test's five-second wait; four test threads passed every unchanged
assertion and deadline. Configuration references agree on 169 schema/catalog
entries, 54 active color roles and 258 default bindings. Canonical Forgejo and
local/remote `dev` confirm integration at the final validated head.

## Risks and Follow-ups

Source is limited to 8 MiB and 512 blocks. Tangling caps combined outputs at
8 MiB, 256 files, 64 reference levels and 100000 expansion steps. No executable
noweb forms, variable evaluation, comments, shebang or mode transformations are
interpreted. Unsupported requested transformations reject before writing.
