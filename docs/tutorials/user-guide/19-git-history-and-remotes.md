# Explore Repository History

Inspect the two practice commits without checking out another revision, then
create a local tag. Keep the same repository from the previous chapters.

## Inspect an older revision

Open Status with `C-x g`, then press h. Select `Add colors` and press Tab to
inspect its metadata, parents and changed paths. Return in inspection follows
the selected exact parent, revision or historical file. Historical content is
read-only; the current file still contains `gold`.

Return to History with q. Mark an older revision with `*`, select a newer
revision and press Return to inspect their ordered range. These range marks
are separate from Status stage marks and History's ordered cherry-pick/revert
marks. Use f to request another finite ancestry page when one is available;
there is no background history walk.

## Create a local tag

Return to Status and press b for Refs. Select the current local branch and
press t. Enter a new tag name such as `practice-checkpoint`. Lightweight is
the default kind. Follow the prompts and inspect the complete target identity
in the review; accept only the tag you intended. An annotated tag instead
collects a message in the native composer and retains Git-owned tagger/signing
policy. Tag creation and deletion here affect local refs.

Refresh Refs with g and inspect the tag. Nothing has been published. A Git
signing policy incompatible with the selected kind produces a refusal or
actual failure; change that policy explicitly if appropriate to your repository.

## Choose transport separately

This practice repository has no remote. In a repository with a configured
remote, select its remote row in Refs and use `&` / `git-fetch`. Fetch updates
tracking observations. It does not merge, rebase or choose a pull policy.

Inspect the refreshed refs before choosing an explicit merge or rebase. Push
requires one local branch with a consistent configured upstream and a review
of its full source object, remote and destination branch. It does not silently
publish local tags. The [integration task guide](../../how-to/integrate-git-changes.md)
walks through those choices, conflict handling and process inspection.

You can now review saved changes, make a selected commit and inspect its
history. Use the [Git reference](../../reference/git.md) for other command
families and the [ownership explanation](../../explanation/git-ownership.md)
when saved, staged and unsaved text differ.

[Previous](18-git-index-and-commit.md) · [User Guide](git/index.md)
