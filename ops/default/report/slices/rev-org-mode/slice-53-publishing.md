# Slice 53: publish native Org document projects

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-53-publishing`
PR: pending

## Purpose

Deliver explicit native document projects under the
[approved plan](../../../plan/rev-org-mode.md#slice-53-publish-native-org-document-projects)
and [tracker](../../../tracker/rev-org-mode.md#slice-53-publish-native-org-document-projects).

## Delivered Behavior

T53.1 adds typed project selection, bounded recursive discovery, native document
exports and binary asset copying. Open sources use canonical unsaved text;
publication does not save, edit or acknowledge source history. Explicit output
paths preserve relative source structure. Every file retains a publication
status and diagnostic through `org-publish-results`. No cleanup or server starts.

T53.2 verifies independent replacement consent, stale observations, source and
output symlinks, hard-link input aliases, duplicated output names and unavailable
sources. Rejected symlink sources remain per-file failures: alias resolution used
only for protection does not admit their content or grant source authority.
Separate upload confirmation uses literal configured argv and the existing bounded
process group, with missing tools, exit failure, timeout and output limits reported
without erasing completed native files. No uploader runs before consent.

T53.3 adds the required `org-export` physical journey in RGB and NO_COLOR,
with actual HTML/Markdown exports, binary copying, partial publication, explicit
replacement and cancellation, numbered result inspection and ordinary undo.
All fixture paths, PTYs and child processes are owned; the existing `org-exports`
journey and 32-entry/64-KiB fixture limits remain intact. Handwritten expected
outputs use explicit file OPTIONS. Filesystem fixtures also exercise Unicode
patterns, recursive excludes, disabled limits, live output aliases, read-only
sources and cancellation of a running upload child.

## Implementation Notes

The ordinary owned worker slot serializes discovery, file effects and configured
tools. Native snapshots retain canonical/disk evidence and include checks. The
existing no-follow directory owner pins output parents, and the native atomic
file publisher retains actual complete, failed or uncertain outcomes. Configuration
is data only; paths resolve against the assigning file without startup IO.

Discovery caps directories at 64 levels and 65536 entries. Source/output counts
share the configured agenda file limit; retained source/rendered/asset bytes share
the total byte limit. Input names cap at 4096 paths and 1 MiB. Rendering, include
expansion and tool output retain their existing finite limits. Formats prepare
and admit one at a time to avoid retaining an unchecked multi-format batch.

## Contracts Added or Changed

- ORG26: typed projects, explicit file publication and retained per-file outcomes.
- ORG25: project exports reuse the immutable native renderers and source evidence.

## Files of Interest

- `src/config/publishing.rs`: closed project and upload argv admission.
- `src/org/publish.rs`: bounded project preparation and per-file ownership.
- `src/org/publish/output.rs`: checked destination publication and asset fidelity.
- `src/session/publishing.rs`: project selection, prompts and serial file workflow.
- `tests/org_io/publishing.rs`: public-session project fixtures.

## Validation

Focused implementation checks are recorded below as tasks complete. Final
exact-candidate controller receipts are recorded in Acta before handoff.

- `cargo test --test org_io project_native_outputs`: passed one owned public-session workflow; zero failures or ignored tests.
- Changed-source quality passed with unchanged limits after discovery/glob/result helpers were separated by responsibility.

- `cargo test --test org_io project_`: four public-session workflows passed, including table-driven replacement, partial-source and upload cases; zero failures or ignored tests.
- Changed-source quality passed with unchanged limits. The partial-source fixture first exposed an alias-observation error that incorrectly prevented an independent valid export; the guarded alias observation was corrected and the original assertion passed.

- `cargo test --test org_io --test configuration publishing`: seven public-session and two configuration workflows passed, zero failures or ignored tests. This includes 19 closed-configuration rejections and owned running-upload cancellation.
- `cargo clippy --lib --test org_io --test configuration --test commands -- -D warnings`: passed before the final fixture additions; the canonical all-target check remains required on the final head.
- Changed-source quality passed with unchanged limits. Excluded directories are pruned before descent, so deliberately deep excluded fixture data cannot consume selected-source discovery bounds.

## Risks and Follow-ups

None recorded.
