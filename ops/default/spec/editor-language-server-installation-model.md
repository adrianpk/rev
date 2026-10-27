# Language Server Installation Model

Status: draft
Date: 2026-10-27
Specification: [Language Server Installation](editor-language-server-installation.md)
Existing ownership: [Editor Configuration Model](editor-configuration-model.md)

## Candidate records

| Record | Candidate evidence | Open representation |
| --- | --- | --- |
| Supported recipe | Server/language identity, version/platform requirements, acquisition and launch recipe. | Catalog format and update policy undecided. |
| Managed installation | Selected server/version/platform, owned payload location, verification and readiness evidence. | Record format, transitions and exact paths undecided. |
| Resolved executable | Managed/external origin, checked launch path and arguments, runtime requirements. | Derived client state; no new persistent settings shape selected. |

Managed server payloads belong to user data when supported; caches are
disposable, settings are user-authored and running process state is owned by
the LSP client. Reusing an external server does not transfer ownership of its
installation to rev. Failed replacement cannot remove a prior usable version.

Freeze the storage/catalog format, consistency rules and lifecycle only after
installation policy is chosen. These are candidate concepts, not implemented
records or authority to change the machine's installed tools.
