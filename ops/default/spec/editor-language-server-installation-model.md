# Language Server Installation Model

Status: draft
Date: 2026-10-27
Reviewed: 2026-10-31
Specification: [Language Server Installation](editor-language-server-installation.md)
Existing ownership: [Editor Configuration Model](editor-configuration-model.md)

## Candidate records

| Record | Candidate evidence | Open representation |
| --- | --- | --- |
| Supported recipe | Server/language identity, version/platform requirements, acquisition and launch recipe. | Catalog format and update policy undecided. |
| Managed installation | Selected server/version/platform, owned payload location, verification and readiness evidence. | Record format, transitions and exact paths undecided. |
| Resolved executable | Managed/external origin, checked launch path and arguments, runtime requirements. | Derived client state; no new persistent settings shape selected. |

## Consistency and lifecycle

A recipe identifies a supported server/version/platform/runtime combination,
not merely a language name. The eventual catalog must define uniqueness and
how acquisition/verification evidence maps to a usable launch result. Exact
recipe keys, version policy and accepted provenance remain open.

Preparation, acquisition and verification produce a candidate. Only a complete
admitted candidate can become the selected managed installation. Failure or
cancellation preserves a prior usable installation and reports incomplete
work truthfully. Candidate cleanup must stay within installer-owned locations.
Exact state names and atomic publication/rollback mechanism remain open.

An external executable is a resolution observation, not a managed payload.
Its discovery does not authorize rev to replace or delete it. Resolving a server
must distinguish absent, missing prerequisite, unsupported and ready outcomes
under the final policy; installation success and client activation are separate.

## Lookup and storage boundaries

The native client needs lookup by admitted language/server configuration and
a checked launch result. Management may need queries by recipe, installed
version/platform and readiness. Managed/PATH/explicit override precedence is a
product decision, not an accidental filesystem enumeration order.

Managed server payloads belong to user data when supported; caches are
disposable, settings are user-authored and running process state is owned by
the LSP client. Reusing an external server does not transfer ownership of its
installation to rev. Failed replacement cannot remove a prior usable version.

No new configuration, manifest, lock format or migration is selected here.
If an installation record is accepted, specify its version, ordered fields,
uniqueness, consistency checks, recovery and compatibility before delivery.
The resolved executable and running client state need not be persisted merely
because the payload itself lives on disk.

Freeze the storage/catalog format, consistency rules and lifecycle only after
installation policy is chosen. These are candidate concepts, not implemented
records or authority to change the machine's installed tools.
