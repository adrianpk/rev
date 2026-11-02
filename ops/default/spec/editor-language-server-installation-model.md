# Language Server Installation Model

Status: Completed
Date: 2026-10-27
Reviewed: 2026-11-02
Approved: product owner, 2026-11-02; paired implementation approval
Specification: [Language Server Installation](editor-language-server-installation.md)
Implementation evidence: [Slice 7 closure](../report/slices/rev-language-server-installation/slice-7-acceptance.md#delivery-set-closure)
Existing ownership: [Editor Configuration Model](editor-configuration-model.md)

## Records and ownership

| Record | Fields in contract order | Ownership |
| --- | --- | --- |
| Recipe | Stable ID, display name, languages, launch profiles, platform, prerequisites, version resolver, acquisition kind, integrity evidence, probe and limits. | Read-only built-in catalog; not a language allowlist. |
| Managed installation | ID, recipe, concrete version/revision, target, relative payload path, source, digest/package lock evidence, runtime identity, launch profiles, status, creation and verification timestamps. | User data registry, schema 1. |
| Launch profile | Profile ID, languages, executable relative to its owned installation, arguments, environment, root markers, initialization options and settings. | Recipe output; selected settings persist only in the existing user configuration. |
| User server profile | Existing server ID, languages, command, arguments, root markers, initialization options, settings, plus bounded literal environment map. | `lsp.servers.<id>` in the existing TOML graph. |
| Language preference | Language ID and configured server ID, or explicit native-only choice. | `lsp.preferred`; no second selection store. |
| Association | Existing basename/suffix pattern and language ID. | Existing `files.associations`; no grammar/catalog foreign key. |
| Management transaction | Job ID, source fingerprints, intended changes, candidate installation ID, phase, bounded diagnostics and cancellation handle. | Ephemeral session/job owner, never a second editor configuration. |

## Consistency and lifecycle

Recipe IDs are unique in the catalog. Installation IDs are unique within one
data root and contain no path traversal. Ready installations may coexist for a
recipe/version when their runtime/target differs. A profile references a single
checked installation; a web-package installation may supply several profiles.
Language IDs are validated strings, not an enum of bundled parsers or recipes.

The job phases are checking, resolving, acquiring, building, verifying,
publishing and a terminal ready/failed/cancelled result. Persisted installations
are preparing, ready or incomplete. Only ready records may supply new selected
profiles. A process interruption leaves a truthful preparing/incomplete record;
recovery never treats the existence of a file as verification evidence.

Create a candidate directly in its unique final versioned directory, excluded
from selection until verified. This preserves venv shebang paths. Publish ready
registry facts atomically, then compare-and-publish configuration. A config
conflict leaves ready/unselected data, not a falsely completed selection.
Failure before configuration publication leaves prior settings and payloads
intact. Failure after atomic publication reports its actual committed state.

External executables are observations, not managed installations. PATH lookup
does not transfer ownership. Readiness is absent, prerequisite-missing,
unsupported, checking, ready, failed or incomplete, with a bounded explanation.
Management status distinguishes availability, selection and client activation.

Update verifies a new candidate before changing any still-owned profile.
Recheck each profile's captured command/settings and all configuration sources;
do not overwrite a user edit. One web update lists and atomically retargets all
selected launch profiles that still reference the prior installation. Preserve
their user settings. Old versions remain until unreferenced and stopped.
Removal refuses referenced/running installations and only traverses checked
installer-owned directories without following escaping links.

## Lookup and storage boundaries

Resolve every location through the existing XDG root owner:

| Role | Location relative to the corresponding rev root |
| --- | --- |
| Settings | Config `config.toml`, retaining explicit root overrides and includes. |
| Installation facts | Data `lsp/installations.toml`. |
| Versioned payload | Data `lsp/installations/<recipe>/<installation-id>/`. |
| Disposable acquisition/package caches | Cache `lsp/`. |
| Mutation locks and incomplete job diagnostics | State `lsp/`. |

The registry starts with `schema = 1` and an installation sequence in the field
order above. No duplicate ID, absolute payload path, escaping symlink,
unverified ready record, unknown schema or malformed record is silently
accepted. Registry facts never override the user's profile/association choice.
Unknown or invalid records are preserved for diagnosis and block destructive
management; they do not disable external LSP or ordinary editing. No preexisting
registry needs migration. Future schemas require an explicit migration contract.

Lookup queries are by installation ID, recipe/version/target, profile reference
and readiness. A selected configured command is authoritative. Missing explicit
preference retains the existing lexical server-name rule. Native-only avoids
both managed and external LSP for that language without altering its grammar.

## TOML and session delta

The delivered configuration fields are:

```toml
[lsp.preferred]
elixir = "elixir-custom"
markdown = false

[lsp.servers.elixir-custom]
languages = ["elixir"]
command = "elixir-ls"
args = []
root_markers = ["mix.exs"]
env = {}

[files.associations]
"*.ex" = "elixir"
"*.exs" = "elixir"
```

`false` means native-only. Strings must
name an existing compatible profile; `true` and incompatible/unknown profiles
are invalid. Defaults contain no preferences. Existing configurations preserve
their previous selection and disabled-by-default LSP behavior.

`env` defaults to empty and merges by key under existing include precedence.
Values are literal strings, with no command substitution or implicit secret
lookup. Inherit the existing process environment, then apply only that server's
overrides. Do not print environment values in status or diagnostics. Settings,
sample and reference must describe the complete new fields and validation.

The writer retains the selected user root and canonical target provenance,
preserves root symlinks, compares the whole captured configuration graph, and
uses the existing complete candidate loader before atomic publication. It
does not serialize resolved defaults over unrelated user content. A scoped
live application can replace LSP settings and associations only. Preserve
runtime global enablement/exclusions and every other editor setting, including
the startup interaction profile. Changed associations reuse the existing
buffer policy owner; they do not load new grammar libraries at runtime.

## Finite limits and shutdown

| Resource | Default ceiling |
| --- | --- |
| Concurrent mutation jobs per data root | 1, using an OS-owned lock across editor processes. |
| Installation records / registry bytes | 32 / 2 MiB. |
| One downloaded native artifact | 256 MiB. |
| Candidate payload / disposable cache | 1 GiB / 2 GiB. |
| Captured helper output / displayed tail | 1 MiB / 64 KiB. |
| Total management operation | 900 seconds. |
| Initial helper stop grace | 2 seconds, then bounded forced stop/reap. |
| Extra per-server environment | 64 entries, 128-byte names, 8 KiB per value, 64 KiB total. |

Keep existing root configuration/client limits. Enforce transfer and filesystem
budgets before publication, with bounded monitoring for package helpers;
external tools are not a hard OS disk sandbox. Inspect only owned candidates
when accounting. Do not walk projects or start idle polling work.

Cancellation belongs to the job token, helper process group and current UI
owner. Only owned helpers receive signals. Prevent PID/group reuse races and
reap descendants; never use shell process-name matching. Report cleanup or
durability uncertainty distinctly. Reopening rev neither resumes downloads nor
selects incomplete payloads implicitly.
