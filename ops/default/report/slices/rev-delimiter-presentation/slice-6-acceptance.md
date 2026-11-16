# Slice 6: Integrated presentation acceptance

Status: delivered
Delivery set: rev-delimiter-presentation
Plan: [Delimiter Presentation Delivery](../../../plan/rev-delimiter-presentation.md)
Tracker: [Delimiter Presentation Tracker](../../../tracker/rev-delimiter-presentation.md)
Branch: `test/rev-delimiter-presentation-06-acceptance`
PR: [#72](https://forge.adrianpk.com/rev/rev/pulls/72)

## Purpose

Deliver T6.1-T6.3: selected native acceptance, two owned Linux PTYs and complete
reference/User Guide coverage. Canonical integration is verified; the combined
gate and final Markdown-only set/model/spec closure remain controller-owned.

## Delivered Behavior

The acceptance script verifies the frozen contract and all five original input
digests before running selected native batches. It includes the complete
presentation, pairs and pair_navigation suites, relevant configuration/palette,
macro-input and Org/Markdown continuity cases, the native 32-event/4 ms drain
owner and the original completed-update observer regression. Every declared
filter must select a nonzero number of cases. Presentation's existing
catalog_and_maps case is the single independent complete 719-command/28-map
oracle; no historical feature scripts or whole repository suite are invoked.

Exactly two controlling PTYs exercise non-modal and modal editing. Each omits
the startup style to prove depth by default, then checks match, finite pulse,
off and last-active restoration with independent literal RGB inputs. Completed
writer updates are decoded for foreground, background and underline, not only
text. Bounded prompt overflow/cancellation and effective user-binding hints
preserve source, point and viewport. A real edit/save/kill/reopen proves disk
bytes; navigation and configuration files remain byte-exact.

Each PTY holds 96 Down then 12 Up events over the frozen 1000-line source.
Enhanced arrow bytes carry Press, Repeat and Release through the real input
parser. Independent source rows and the writer's final caret placement confirm
line 85, column 11. A positive final pulse precedes expiry observation; an
unmarked frame before asynchronous confirmation cannot count as expiry proof.
After expiry, completed frames retain point/viewport, followed by 350 ms silence
and the existing CPU check. Native cases independently assert zero remaining
pair deadlines. These are isolated acceptance measurements, not physical
workstation latency claims.

## Implementation Notes

Reuse the existing controlling-PTY owner and eight-second/1 MiB boundary,
completed-writer observer, cell decoder and Linux CPU/RSS helpers. Terminal
restoration and owned-child reaping are checked in each journey. Unrelated
asynchronous services are disabled in the fixtures. There is no second PTY or
measurement framework, new dependency, production flag or persistent service.

The automatic-editing pending_cancel fixture now keeps presentation off only
while preparing its large source, then explicitly restores/asserts depth before
replay. Default presentation could otherwise finish the shared qualification
scan during setup, invalidating the fixture's pending-work precondition. The
original Layout-wait, C-g, exact source/stamp/point and empty-deadline assertions
remain intact. The Markdown continuity cancellation case also drains current
independent pair work before its idle assertion, then additionally rechecks
literal source and empty syntax to prove canceled Markdown never restarted.
Its immediate literal/empty-syntax assertions remain intact. Production behavior
is unchanged by this slice.

Reference/default examples document active rendering, palette fallbacks,
overlay priority, finite lifetime and bounded qualification. The existing editing
and preferences chapters provide progressive style/configuration exercises.
The single pending changelog outcome names all four styles without delivery
chronology. Earlier report receipts are reconciled against Acta and canonical
merged PRs; their original observations and immutable introductions remain intact.

Implementation/report introduction: `8f0a2a9741ba48e6230bb92de73dac5a659e46e9`.
Canonical review: [PR #72](https://forge.adrianpk.com/rev/rev/pulls/72).
Canonical integration: `cfa6cdd4e6c86af1cf453af812f4cf63d2f0c47d`; fetched
ancestry, clean state and no owned processes verified. The integrated worktree
was removed without force.

## Contracts Added or Changed

- Nonempty native selection and unchanged frozen-input digest checks.
- Exactly two owned PTYs with completed-cell, source and caret observations.
- Frozen first-progress/release bounds of 250 ms and 350 ms idle observation.
- Original eight-second/1 MiB child/stream limits and owned cleanup.
- Preserve 719 commands, unchanged factory maps and all resource/pulse bounds.

## Files of Interest

- `scripts/check-delimiter-presentation.sh`: selected native and two-PTY batch.
- `examples/delimiter-presentation-acceptance.rs` and `examples/delimiter_presentation_acceptance/`: focused journey and independent observations.
- `tests/pairs/provenance/macros.rs`: cold qualification precondition with depth restored.
- `tests/markdown/continuity.rs`: canceled Markdown stays literal after independent pair work retires.
- `docs/reference/delimiter-editing.md`, `docs/reference/defaults.md`, `docs/reference/config.toml`: active behavior and configuration.
- `docs/tutorials/user-guide/02-edit-text.md`, `docs/tutorials/user-guide/09-preferences.md`: progressive exercises.
- `CHANGELOG.md`: one concise delimiter presentation outcome.

## Validation

All five ordered required commands PASS once through looper validate on clean
published `cfa6cdd4e6c86af1cf453af812f4cf63d2f0c47d`, the canonical PR-reference
commit integrated by PR #72. Acta validation record:
`01M7HBWEMFVRQ18SJXTA04GX8S`. The combined gate is unchanged and remains
controller-owned.

- PASS: `cargo fmt --all -- --check`.
- PASS: `bash scripts/check-delimiter-presentation.sh`: 261 selected native cases, all six frozen digests, one complete catalog/map oracle and exactly two PTYs; no ignored cases or empty filters. The unoptimized checkout batch took 891.10 s, including its builds and journeys.
- PASS: `bash scripts/check-documentation.sh`: 112 reachable pages, 803 local links/anchors and 22 command/configuration reference cases.
- PASS: `bash scripts/build-user-guide.sh`: strict checks, links and rendered guide construction; output stays in the ignored checkout-owned target.
- PASS: `git diff --check`.

| Native batch | Selected cases |
| --- | ---: |
| presentation, including 24 bursts/1536 completed frames and 18 pulse cases | 77 |
| pairs | 78 |
| pair_navigation | 48 |
| configuration | 31 |
| display palette | 5 |
| macro input | 7 |
| Org continuity | 6 |
| Markdown continuity | 7 |
| native input drain owner | 1 |
| original completed-update observer | 1 |

| PTY profile | Journey ms | First progress ms | Reverse/Release to final ms | Output bytes | Observed completed frames | Idle CPU ticks |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| non-modal | 3149 | 14.710 | 45.606 | 397867 | 107 | 0 |
| modal | 3702 | 16.987 | 128.542 | 490749 | 132 | 0 |

The reverse measurement conservatively includes all twelve Up events and the
Release write. Both journeys remain below eight seconds/1 MiB and both input
observations remain below 250 ms. The existing 350 ms CPU helper enforces an
even stricter one-tick ceiling than the frozen three-tick maximum. Source,
caret/viewport, finite pulse, restoration and owned reaping assertions all PASS.
Scoped native source policy PASS covers all seven changed Rust files with the
safe-Rust, five-parameter, three-level nesting and complexity-ten limits intact.
Preserved implementation hook PASS at staged tree
`0a46cc98ee3e00c42c7afe0c00005b535cb1d0e3`: full workspace formatting/Clippy,
zero blocking/deferred diagnostics, and native policy over all 2013 first-party
Rust files and 82 Cargo roots. This PR-reference commit changes operational
documentation only.

## Traceability

| Slice | Canonical PR | Immutable report introduction | Validated integrated HEAD |
| --- | --- | --- | --- |
| 1 | [#67](https://forge.adrianpk.com/rev/rev/pulls/67) | `fdc1aa13dedfcef2c6ee4902669c5caad90dca6d` | `f1515a01cf5434ce163840a99c1729da0bfaea0e` |
| 2 | [#68](https://forge.adrianpk.com/rev/rev/pulls/68) | `f338d0f56f36d6c3e67740e3a597b6922a99f900` | `733061bba1780466675111f86a3704fed71c977c` |
| 3 | [#69](https://forge.adrianpk.com/rev/rev/pulls/69) | `9cedf2c18e2bf90861db5b0b73aff818e5bf5500` | `b21e7523f4eec23194836fae7634be4a9d8e9c76` |
| 4 | [#70](https://forge.adrianpk.com/rev/rev/pulls/70) | `75722e6015ab1bb00ac55cad83ee69f1398d9a4e` | `fd84b2b101479f763bcc015f50a715713c250f1a` |
| 5 | [#71](https://forge.adrianpk.com/rev/rev/pulls/71) | `84572a9c0ac1ae78595683ec290fef374c0c642b` | `67f94244d2982f545be9f363408f62aadced76b5` |
| 6 | [#72](https://forge.adrianpk.com/rev/rev/pulls/72) | `8f0a2a9741ba48e6230bb92de73dac5a659e46e9` | `cfa6cdd4e6c86af1cf453af812f4cf63d2f0c47d` |

## Risks and Follow-ups

The controller must run the unchanged combined gate on its exact clean dev
candidate and record the receipt before Markdown-only delivery closure. No full
delivery-set gate, main promotion, release or publication runs in this slice.
The existing vendored crossterm parentheses warning is unchanged.
