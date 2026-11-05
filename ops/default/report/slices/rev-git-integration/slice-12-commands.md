# Slice 12: Prepared Repository Command Families

Status: reviewing
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-12-commands`
PR: pending

## Purpose

Deliver GIT14-GIT15 and their GIT16-GIT20 contracts through T12.1-T12.3 in the
[approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

- T12.1 creation routes: Status C C, C s, C b and C m prepare regular, depth-one,
  bare and mirror clones; I I prepares init. Named clone/init commands also
  work from ordinary editing and chosen directories without a Git context.
- The existing bounded native text owner edits a JSON array of literal Git
  arguments. Empty source/destination parameters must be filled. The selected
  family and explicit option/parameter structure remain checked.
- C-Enter or C-c C-c first captures read-only preparation and opens exact
  executable/mode/cwd/argv/destination review. A separate submission executes
  only that canonical plan. Plain Return does not accept review; q/C-g cancels
  it. Ordinary source views and editing state return after cancellation.
- Clone/init require an absent explicit destination and checked canonical
  ordinary parent. Execution rechecks directory/parent identity, source HEAD,
  index, working bytes and untracked hashes. Stale plans refuse before launch.
- Successful creation verifies the actual repository identity and object
  format, full existing HEAD, unborn init, shallow state or bare/mirror kind.
  Empty regular/bare/mirror remotes retain an actual unborn identity rather
  than inventing a commit. Failures retain uncertainty and never delete a
  partially created destination, retry or claim rollback.
- i t and i p visit actual .gitignore and common Git info/exclude files through
  their ordinary native owners, preserving existing unsaved text and inserting
  no pattern or hidden save.
- Status o a/o l/o u/o s/o d prepare actual submodule add, recursive listing,
  indexed-commit checkout/init, URL sync and non-forced deinit. Add names its
  metadata separately from its literal path and source URL in exact review.
  Literal paths, parent sequence/index/saved bytes, configuration, declared mappings
  and child state remain bound to review. Changed evidence refuses launch.
- Update and deinit refuse dirty child state; changing routes protect unsaved
  native repository buffers. Deinit also protects retained visiting aliases,
  file-tree roots and native process/language-server owners. Actual postchecks
  verify gitlinks, checkout objects/bytes, registration or empty deinitialization.
  Listing retains actual bounded output and reports no repository effects.
- Status O a/O m/O f/O p/O s prepare subtree add, merge, pull, push and split.
  Read-only preparation freezes local commits or one explicit remote branch to
  a full object. Exact review includes executable/cwd/argv, literal prefix,
  pinned object and full remote branch. Changed configuration, source or remote
  anchor refuses execution before a changing child.
- Subtree results verify actual clean branch/ref state, prefix objects, saved
  paths outside the prefix, split tree identity and remote publication tree.
  Split output retains its full actual commit. Push names one explicit endpoint
  and branch; implicit followTags, recursive publication, active URL rewrites
  and configured-name ambiguity reject. No force, inferred upstream or retry.
- Actual subtree merge conflicts retain Git-owned sequence lineage and report
  in-progress state rather than verification. Existing operation inspection and
  separately reviewed controls remain available; no automatic continuation or
  abort. Missing optional subtree support is diagnosed before effects.
- T12.2 notes routes: T a/T r/T p prepare add/remove/prune against a full notes
  ref and exact objects. Add checks the stored blob against Git-cleaned literal
  message bytes and reports its full ID. Prune review names every captured
  missing-object target. Other notes, branch refs, HEAD and saved sources remain
  checked; selected symbolic refs, including dangling aliases, refuse before effects.
- T12.2 sparse routes: > e/> d/> r/> s/> a prepare cone-mode enable/disable/
  reapply/set/add. Capture private patterns, effective configuration, literal
  directories and complete index/materialization state. Actual postchecks verify
  selected cones, skip flags and file presence while preserving index objects,
  HEAD, unrelated configuration and distinct linked-worktree state.
- Saved dirty or unsaved native sources, changed patterns/config/index flags and
  manually materialized excluded paths protect sparse actions. Metacharacters
  and supported raw path bytes remain literal. Git cone files cannot represent
  a newline directory, so selecting one refuses before a changing child.
- T12.2 output routes: W c/W s/W r prepare format-patch, saved binary diff and
  request-pull text. The native JSON object separates literal arguments from an
  explicit absent output path. Commit anchors freeze to full IDs; review names
  complete byte count and a Git blob hash without writing an object database.
- Accepted output is regenerated and compared byte for byte before publication.
  Existing file/parent/source/config changes refuse; request-pull also rechecks
  the exact already-published remote branch. Atomic non-overwriting publication
  and actual saved-byte checks protect the chosen output. Changed producer bytes
  with exit zero retain an unverified outcome and publish no file.
- Read-only export preserves unsaved native sources. Both object formats and
  bare contexts work, including an explicitly empty saved diff. Output stays
  within existing detail/scratch and aggregate limits. No mail, push, implicit
  save, overwrite, forced cleanup, retry or rollback is implied.
- T12.2 application routes: w p/w a prepare one complete ordinary local patch
  or mailbox file. Exact input path, fingerprint, byte count and blob hash bind
  the later sealed stdin. Complete clean tracked saved state, unchanged input,
  HEAD/configuration and unsaved native buffer guards protect execution.
- After acceptance, installed Git applies decoded patch bytes to a private index
  to derive independent expected trees. Actual plain application must produce
  that index tree and materialized saved state without changing HEAD. Actual
  mailbox commits must have the predicted trees, exact single-parent lineage,
  decoded authors, author timestamps and cleaned message bytes.
- Changed mailbox payload with unchanged From labels follows its actual patch
  semantics. Real hook changes remain Indeterminate/Possible despite exit zero.
  Later mailbox conflict retains the applied prefix and actual Git-owned progress;
  no automatic continue, abort, rollback or retry is attempted. Git retains its
  configured signing policy; no signing bypass is prepared.
- Owned producer workspaces declare their exact filenames, retain directory FDs
  and admission through active/retained child cleanup, and preserve unexpected
  entries or substituted paths. Existing time/output/cleanup limits remain.
- T12.3 direct routes: ! in all eight Git navigation maps prepares structured
  Git argv; git-run-shell-command separately prepares one explicit shell string.
  Both show exact mode/executable/cwd/input/limits and require separate acceptance.
  Named entry works from ordinary editing and chosen directories; compatible
  hidden Git context remains attached to the source.
- Direct shell selection reuses existing shell-command startup/program/args.
  Execution uses the smaller existing shell and Git-operation time/output bounds,
  closed stdin and the same Git process/cancellation/retained-cleanup owner.
  Direct execution exclusively retains admission across known repositories.
- Arbitrary commands retain unknown Git postconditions even with exit zero.
  Actual argv, exit and bounded output remain inspectable through existing
  escaping/redaction. Compatible Status refreshes; lost source identity retains
  the prior observation with failed freshness. Source/configuration/cwd changes
  and unsaved visiting file buffers protect acceptance. No output replay or
  automatic retry/rollback is performed.
- T12.1-T12.3 implementation is complete. Final ordered slice validation is
  recorded by the controller against the clean PR candidate after its required
  PR-reference follow-up.

## Implementation Notes

Prepared commands use the existing installed-Git worker, process records,
configuration, cancellation, stream limits and retained cleanup owner. One
canonical plan and result stay under the existing Git admission budget.
Preparation waits for active Git work so unknown destination/common identity
cannot race another owned request. Verified results refresh compatible source
observations. When actual compatible facts can still be read after a failure,
they refresh the source while the process record preserves uncertainty.
Publication alone does not verify effects or infer a postcondition. Hidden views do not release the process owner.

The native draft reuses the existing retained source-view and message-buffer
facilities. Commit/tag dispatch remains separate from argument interpretation.
JSON arguments are literal OS arguments and are not passed through a shell.
Output drafts wrap the arguments and explicit path in one JSON object under the
same text/input bounds. Read-only preparation retains the complete producer
bytes; later regeneration must match those bytes before filesystem publication.
The shared execution helper returns bounded complete stdout to output callers
while retaining the same actual argv/exit/cleanup facts for existing mutators.
Output staging reuses the ordinary file owner's Temporary lifecycle through a
captured directory FD; hard-link publication cannot overwrite an entry that
appeared after review. Actual saved bytes are checked before success is claimed.
The existing common saved-state helper also supports an unborn source: exact
index entries plus the working diff from that index replace a nonexistent HEAD
baseline, with the same saved untracked hashes.

Patch input uses the same bounded ordinary-file capture and literal argument
composer. Preparation hashes sealed bytes without writing Git objects; plain
patches also receive a read-only Git apply --check. Private index derivation and
mailbox split/decode start only after separate acceptance. Blob/tree objects may
be written during independent derivation; no source index/file rollback is
inferred if later execution fails. Mailbox envelope count is bounded by
min(max_refs, 256), with all decoded file reads under existing detail/scratch
limits. UTF-8 message encoding and explicit mailbox options are part of review.
Git owns configured signing. A signing-policy override was removed before
commit to comply with the approved repository semantics contract.

The capture owner adds a typed private-index override only for a single owned
prediction child, after clearing ambient repository overrides. Actual mutators
and observations receive no private override. Its existing retained driver
reservation also retains private workspace metadata until child cleanup ends.
The same child owner carries the workspace FD; a dropped live child keeps that
workspace until its reaper finishes. Cleanup admits declared ordinary producer
files only, preserves unexpected entries and refuses a replaced directory.
Mailbox observation now reads its actual original object from Git's ORIG_HEAD;
existing supported continuation/abort admission remains separate.

Direct preparation never executes or classifies arbitrary input. The same
canonical plan binds final argv (including executable/prefix bounds), captured
cwd/source/configuration and limits. Git mode passes OS arguments directly;
Git's own alias/helper semantics remain Git's responsibility. Shell mode reuses
startup selection and argv construction from the existing shell-command owner,
then executes through the Git operation's existing capture and cleanup path.
No second process owner, dependency, queue or timeout override was added.

A central admission flag keeps unknown direct effects exclusive across the
engine, including retained cleanup. The existing child-facts helper now also
accepts complete argv for an explicit shell executable; ordinary Git callers
retain their unchanged prefix, capture and execution facts. Only the accepted
arbitrary child's output is inspectable; configuration/refresh probes remain
private and never replace actual argv, status or output. Failed compatible
refresh marks the retained observation stale. Arbitrary output cannot become a
command, inferred postcondition or automatic document edit.

Bounds remain 32 MiB editor-wide Git admission, 1 MiB scratch, existing process
output/deadline/cleanup ceilings and five-second fixture waits. Prepared input
admits at most 64 arguments, 4096 bytes each, 64 KiB total and an absolute cwd
within 4096 bytes. Native draft text remains bounded to 64 KiB. Transport output
is withheld from public records; diagnostic display keeps existing redaction.
Init, listing, sync and deinit obey the operation timeout; clones, submodule add
and update obey the transport timeout. Subtree add/pull/push preparation and
execution use that same transport timeout; merge/split use the operation timeout.
Request-pull preparation and execution use the transport timeout. Configuration probes stay private, and
Debug omits captured configuration and resolved URL bytes. Relative sync URLs whose resolved
registration cannot be established remain unverified after the actual command.
No runtime/development dependency or validation prerequisite was added.

## Contracts Added or Changed

- Thirty-two working commands: git-clone, git-clone-shallow, git-clone-bare,
  git-clone-mirror, git-init, git-ignore and git-ignore-exclude; five additional
  commands are git-submodule-add/list/update/sync/deinit and five are
  git-subtree-add/merge/pull/push/split. Eight more are git-notes-add/remove/prune
  and git-sparse-checkout-enable/disable/reapply/set/add. Three output commands
  are git-format-patch, git-save-diff and git-request-pull. Two application
  commands are git-apply-patch and git-apply-mailbox. The direct routes are
  git-run-command and git-run-shell-command.
- Status has 72 active defaults; all eight Git navigation maps admit !. The existing keys.git_commit context also
  supplies ordinary editing and explicit submission for prepared argument text.
- Live catalogs contain 593 commands and 271 configuration entries.

## Files of Interest

- `src/git/families.rs`, `src/git/work/families.rs`: typed plans, literal input,
  exact preconditions, installed capability checks and creation verification.
- `src/git/work/families/submodule.rs`: checked submodule configuration, child
  evidence and actual postconditions under the same process/admission owner.
- `src/git/work/families/subtree.rs`: pinned local/remote objects, exact branch
  and tree postconditions, actual conflict retention and publication policy.
- `src/git/work/families/notes.rs`, `sparse.rs`, `state.rs`: exact note/ref maps,
  bounded ordinary-file evidence, cone/index/materialization postconditions and
  compatible actual fact refresh without claiming uncertain effects verified.
- `src/git/work/families/output.rs`, `src/git/work/index.rs`: complete producer
  capture, pinned ranges/remote anchors, unchanged execution facts and atomic
  explicit-file publication with actual byte verification.
- `src/git/work/families/patch.rs`, `src/file/temporary.rs`,
  `src/process/capture.rs`: sealed local input, independent private-index trees,
  decoded mailbox metadata/lineage, retained workspace ownership and cleanup.
- `src/git/work/families/direct.rs`, `src/shell.rs`, `src/session/shell.rs`:
  unknown direct outcomes, compatible refresh, shared configured shell selection
  and exact native argv. `src/git.rs` retains exclusive admission and stale facts.
- `src/session/git/families.rs`, `src/session/git/commit.rs`: native draft,
  two-step submission, exact review and original source restoration.
- `src/git/work/repository.rs`: shared saved-state evidence for unborn sources.
- `tests/git_integration/families.rs`, `families_presentation.rs`: real local
  Git objects/files and both-profile native input, review and visiting owners.
- `tests/git_integration/families_submodule.rs`: SHA-1/SHA-256 real submodule
  round trips, configuration/index/path/child races and dirty child refusal.
- `tests/git_integration/families_subtree.rs`: both-format real subtree workflows,
  changed remote/push policy, actual conflict state and unavailable helpers.

- `tests/git_integration/families_metadata.rs`: both-format note/sparse round
  trips, symbolic-ref and external-state protection, raw paths and linked roots.

- `tests/git_integration/families_output.rs`: real mailbox/diff consumers,
  published request text, bare/empty output, stale evidence, changed producer
  bytes, complete-output limits and transport-deadline evidence.

- `tests/git_integration/families_patch.rs`: both-format real plain/mailbox
  application, changed payload semantics, real hook mismatch, retained prefix
  conflict and input/configuration/source protection.

## Validation

The earlier entries below describe bounded development increments. Delivered
Behavior above records the current complete implementation; the final ordered
receipt is recorded separately against the clean PR head.

Development initially passed six real Git/native cases in 10.29 seconds.
Additional native outside-repository and unborn-source cases passed with the
combined eight-case run in 11.54 seconds. The empty-clone case passed separately
in 0.63 seconds. The final combined nine-case run passed in 16.41 seconds. Both object formats, raw destinations, shallow/bare/mirror
facts, literal shell-looking arguments, stale destination/parent/source/index/
HEAD, ignore-file visiting identity and explicit review are covered.

The unborn-source regression first failed because preflight diffed nonexistent
HEAD. Its correction retains exact staged objects, uses the working diff from
the index only for an unborn source and preserves saved untracked hashes. The
unchanged assertion proves changed saved bytes refuse before creating the
destination. Initial compile errors were a duplicate helper name and incomplete
new-operation/native dispatch wiring; these were corrected before validation.
No observable assertion, deadline or resource limit was weakened.

Three existing native commit cases passed in 8.77 seconds, two native tag cases
in 10.48 seconds and the SHA-1/SHA-256 stash saved-byte round trip in 15.37
seconds. All 21 command/configuration reference cases passed in 14.53 seconds.
Documentation passed at 568 commands, 271 configuration entries, 76 pages and
485 links. Formatter and strict focused Clippy passed during development. After the native
input-bound check, all three native family cases passed in 4.61 seconds. A real
Git wrapper delaying only init/clone verified the separate configured deadlines
in 2.63 seconds: init timed out with possible effects, and clone completed under
its transport limit. The wrapper delegates all Git output and repository work to
installed Git; fixture waits and production limits remain unchanged.

Submodule development first passed all five real variants across SHA-1 and
SHA-256 in 20.29 seconds, then the combined 14-case suite in 68.74 seconds.
After native alias protection, all five presentation cases passed in 16.32
seconds. With final explicit-name preparation, all 15 combined family cases
passed in 79.00 seconds. All 21 command/configuration reference cases passed
(1.87 and 14.74 seconds), documentation passed at 573 commands, 271 configuration
entries, 76 pages and 485 links, and formatter, strict focused Clippy and
whitespace checks passed. These remain development checks before the complete
slice and its exact-HEAD controller receipt.

A new literal-target case caught installed Git returning exit zero from deinit
while leaving the URL registration for a metadata name containing [1]. An
isolated real Git reproduction confirmed the empty selected checkout, retained
registration and untouched sibling; the worker correctly reported unverified
possible effects. Add preparation now explicitly composes the metadata name
separately from URL and literal path. The fixture chooses plain metadata names
for metacharacter and raw newline/non-UTF8 paths. Gitlink, sibling, saved-byte,
URL-removal and checkout assertions remain intact; no automatic metadata cleanup,
extra changing command, forced action, assertion or resource-limit relaxation
was introduced. Existing unusual metadata still requires actual postcondition
verification and can remain unverified. Initial submodule compile wiring needed
an explicit Path import and closure error type, corrected before validation.

Subtree development passed four real engine cases in 25.57 seconds and the
both-profile native case in 8.17 seconds. The final combined 20-case family run
passed in 114.17 seconds. All 21 reference cases passed (1.92 and 14.57 seconds),
strict focused Clippy passed in 18.91 seconds, and formatter/whitespace passed.
Documentation passed at 578 commands, 271 configuration entries, 76 pages and
485 links. This completes T12.1; T12.2-T12.3 remain pending.

The first subtree push correctly remained unverified because Git subtree adds
refs/heads/ itself and the composed full-ref operand therefore targeted a
repeated prefix. Installed helper source confirmed its actual argv construction.
Preparation now retains and reviews the full destination separately and supplies
its exact short operand to that helper. Independent destination branch/tree,
file-byte and local-HEAD assertions are unchanged. The actual failed case was
corrected without automatic cleanup, another changing command or weaker checks.
Strict Clippy also rejected separately retained optional proof payloads as an
oversized publication variant. One mutually exclusive typed family proof now
owns creation, submodule or subtree evidence within the same admission budget;
no lint, observable assertion, fixture wait or resource bound was relaxed.

Notes/sparse development passed all eight selected engine and native cases in
63.81 seconds, including the existing subtree presentation regression. Both
object formats prove add/remove/prune and all five sparse variants. Other-note
and configuration races, existing/dangling symbolic note refs, bare notes,
literal metacharacter/non-UTF8 directories, overlapping cones, changed patterns,
index flags and saved files, linked-worktree isolation and native unsaved guards
are covered. Strict focused Clippy passed in 18.83 seconds, formatter passed,
and all 21 command/configuration reference checks passed (1.96/12.82 seconds).
Documentation passed at 586 commands, 271 configuration entries, 76 pages and
485 links. Whitespace checks passed. These are development checks; remaining
T12.2-T12.3 behavior and the final controller receipt remain pending.

The first new engine run passed five of six cases but found installed Git
rejecting the combined raw sparse directory argument. Direct isolated Git
reproduction confirmed that literal bracket characters require --skip-checks
and that newline directory names still create malformed line-based cone
patterns. Native preparation keeps its ordinary literal-directory checks,
reviews the required flag and refuses newline selections before effects.
Metacharacter/non-UTF8 round trips and unchanged saved bytes after newline
refusal are asserted independently. Cone parent/child redundancy is compared
against Git's actual normalized selection. No fixture deadline, process bound,
forced cleanup or implicit save/stash was added. Initial compilation also
required the expanded command count and explicit notes result error type;
strict Clippy required a byte-string slice. Those errors were corrected before
the successful selected run and final development lint.

Output development first passed four real engine cases in 17.18 seconds. The
expanded nine-case selection passed in 55.00 seconds, including both-profile
native output composition and the existing notes/index round-trip regressions.
Independent installed-Git am/apply consumers in separate owned roots verify
actual tree objects and known text/binary bytes in SHA-1 and SHA-256 repositories.
Request text contains the full local anchors and published branch without
changing either repository. Bare contexts and empty saved diffs are covered.
These consumer checks do not claim native application routes, which are pending.
Formatter and strict focused Clippy pass; the last incremental Clippy check
completed in 2.70 seconds after the 21.45-second check. All 21 command and
configuration reference cases pass (2.29/13.70 seconds).
Documentation passes at 589 commands, 271 configuration entries, 76 pages and
485 links. Whitespace checks pass. These remain development results before
complete native application/direct routes and final exact-HEAD validation.

A real saved attributes change alters producer output while keeping clean source
refs/configuration. The accepted producer exits zero, but exact byte comparison
retains Indeterminate/Possible facts and no destination is created. Parent and
file collisions, source/config/HEAD and remote-branch changes refuse before
publication; complete-output limits reject without a partial file. A real Git
wrapper delays request-pull only, proving both preparation and execution use the
existing transport deadline. No assertions, fixture waits, admission ceiling or
cleanup limit was weakened. No new dependency or process framework was added.

Application development passes five engine cases in 31.06 seconds. After
adding both-profile native preparation/acceptance and unsaved-source guards,
the combined six-case run passes in 32.41 seconds. After retaining configured
signing and adding a real unavailable-signer case, all seven selected engine
and native cases pass in 35.32 seconds. Signing failure preserves the original
HEAD, enabled signing configuration and actual mailbox progress without a
commit or automatic rollback. Both object formats prove
plain staged trees and multi-message commit lineage, authors, non-default author
dates, multi-paragraph/UTF-8 messages, binary bytes, raw paths, additions and
deletions. Ordinary abbreviated patch headers are consumed by installed Git.
A valid changed payload keeps its From commit label but produces its actual
different checked tree. A real successful applypatch-msg hook changes message
bytes and correctly retains Indeterminate/Possible facts with exit zero.
A later actual mailbox conflict retains the first commit, step/total/original
metadata, unchanged conflicting source and Git-owned sequence files.
Input byte/identity, configuration, index and HEAD races refuse before changing
execution; symlinks, oversized input and dirty saved state reject preparation.

The native test build exposed an accidental mechanical rename across existing
presentation cases. Their original code was restored verbatim before adding
only the new case. No assertion, fixture wait or resource limit was relaxed.
All five selected process/workspace unit cases pass in 1.33 seconds, including
existing real cancellation/timeout/leader/descendant cleanup and the new private
index scope/lifetime case. Workspace tests preserve unknown entries, symlinks
and replaced directories, and prove complete bounded reads and owned cleanup.
Formatter and strict focused Clippy pass (22.86 seconds, then 18.06 seconds
after the signing contract correction); all 21 command and
configuration reference cases pass (2.05/15.47 seconds). The documentation
checker first caught the old Status count in defaults.md; correcting it to 71
active bindings preserves the unchanged check. Documentation passes at 591
commands, 271 configuration entries, 76 reachable pages and 485 links.
Whitespace checks pass. These are development checks; T12.3 and final exact-HEAD
controller evidence remain pending.

Direct development passes all seven selected engine/native cases in 32.58
seconds. Real SHA-1/SHA-256 repositories prove literal raw and shell-looking
arguments, empty operands, actual staged/ref effects and retained exit/output.
Configured shell program/args perform real pipelines and saved-file writes.
Outside-repository commands retain unknown results without inventing a context.
Moving actual Git metadata preserves the old observation with failed freshness
while retaining the accepted child's exit-zero output. Source/index/HEAD/config/
cwd and full executable/argv limits refuse before execution. Real Git aliases
and explicit shells prove exclusive cross-context admission, descendant cleanup
and cancellation. Existing shell time/output limits remain distinct after
refresh. Both profiles prove separate preparation/acceptance, Return protection,
cancellation, ! availability in eight maps, hidden compatible Status refresh,
process inspection and protection for an unrelated unsaved visiting file.

Formatter and strict focused Clippy pass (22.03 seconds). Documentation passes
at 593 commands, 271 configuration entries, 76 pages and 485 links. Whitespace
passes. Initial compilation required using the actual retained OperationId
for source failure and measuring OS argument lengths through their borrowed
values. Those corrections preserve assertions, fixture waits, cleanup and
resource limits. The final ordered checks, including the complete families
filter and existing shell-command regression, belong to the controller receipt
for the clean PR candidate after the PR-reference follow-up. No full gate is
claimed here.

Successful development commands:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration families:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration families::empty_clone -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration commits::presentation:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs::tags::presentation:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration repository::saved_roundtrip -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration families::presentation:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration families::configured_deadlines -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration families::subtree:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration families::presentation::native_subtrees -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration -- families::metadata:: families::presentation::native_notes_sparse families::presentation::native_subtrees --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration -- families::output:: families::presentation::native_outputs families::metadata::notes_roundtrip index::path_roundtrip --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration -- families::patch:: families::presentation::native_applications --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib -- process::capture:: file::temporary:: --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration -- families::direct:: families::presentation::native_direct_commands --test-threads=1`
- `git diff --check`

These are focused development results. The controller records final ordered
slice evidence against the clean PR head after its reference follow-up. That
receipt is the exact-candidate evidence; the required full delivery-set gate
remains a later controller phase.

## Risks and Follow-ups

Combined PTY/resource acceptance and learning-path guidance remain in their
recorded later slices. The controller owns this slice's integration and the
unchanged full delivery-set gate.
