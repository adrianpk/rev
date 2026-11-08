---
id: TKT-20261108214514
title: Continue editor development in the rev organization
status: solved
kind: task
severity: unclassified
priority: unclassified
scope: ops
tags: repository, organization, migration
source: chat
reported_at: 2026-11-08T21:45:14Z
ready_at: 2026-11-08T22:01:27Z
started_at: 2026-11-08T22:01:27Z
reviewed_at: 2026-11-08T23:10:54Z
closed_at: 2026-11-08T23:10:54Z
resolution: fixed
branch: dev
pr: https://forge.adrianpk.com/adrianpk/rev/pulls/328
commits: [ed30e6a5986b7d043defd0a0b59a32fc9e9be58a]
---

# Continue editor development in the rev organization

## Requested outcome

The owner selected https://forge.adrianpk.com/rev as the organization for the
project repository family. The owner clarified that adrianpk/rev remains a
historical repository while development continues in the existing rev/rev on
the same forge. Publish the latest accepted Git snapshot there; do not transfer
the old forge repository or its issue/PR records. Associated project repositories
follow rev/{repo-name}; their concrete names are separate decisions.

## Migration scope

- Preserve the accepted Git history, main/dev tips and published version tags.
  Historical slice branches, issues and PRs remain in adrianpk/rev.
- Update the canonical origin configuration, public source/clone links and
  affected repository consumers for future development. Historical evidence links
  continue to identify the old repository and must not be rewritten wholesale.
- Reconcile the existing Codeberg and GitHub mirror configuration and publication
  destinations. Moving the primary repository does not itself authorize renaming
  mirror repositories or associated packages.
- Verify expected ref identities and ordinary fetch/push access at the new
  location. Keep a short README pointer only in the historical repository.

## Observed continuation

- rev/rev main and dev start at dc5142ce71c0d5846e0d278ae912c4c167aa3bf8.
- All six published version tags retain their original object IDs.
- The existing checkout's origin now targets rev/rev. Local drafting changes
  are preserved, and Cargo package/crate identity remains rev.
- Historical README notice commit ed30e6a5 changes only README.md. PR #328 is
  merged into the old repository's main branch, and its notice is readable
  without authentication. The new repository does not contain that commit
  or notice.
- The owner approved public visibility for both the repository and organization.
  Both are now public. Anonymous organization API, repository API and raw README
  requests return HTTP 200, and unauthenticated Git ref discovery succeeds.
- The README identifies rev/rev as the official source, and the installation
  guide points to the same repository. Existing Codeberg and GitHub mirror
  destinations retain their names and their main/dev publication policy.

## Related proposals

- [Plugin System](../../spec/editor-plugin-system.md).
- [Development Workflow Plugin](../../spec/editor-development-workflow.md).

Repository organization is distinct from package IDs, command namespaces, target
projects and runtime state. The continuation preserves those identities.

## Delivery boundary

Git continuation, the historical README notice and public access are verified.
The source links and dependent proposal references are reconciled. The old forge
repository and the existing mirrors retain their own identities. No package
repository names or plugin namespaces are selected by this task.
