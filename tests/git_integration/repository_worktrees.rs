use super::{Repository, finish};
use rev::git::{
    Completeness, Effects, Outcome, Repositories, WorktreeAction, WorktreePlan, Worktrees,
};
use std::{ffi::OsString, fs, os::unix::ffi::OsStringExt, path::PathBuf, sync::Arc};

#[path = "repository_worktrees_presentation.rs"]
mod presentation;

fn list(repository: &Repository, owner: &mut Repositories) -> Arc<Worktrees> {
    if owner.contexts().is_empty() {
        repository.open(owner, &repository.root);
    }
    let context = owner.contexts()[0].id;
    owner.worktrees(context).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(update.operation));
    owner.context(context).unwrap().worktrees.clone().unwrap()
}
fn prepare(
    owner: &mut Repositories,
    list: Arc<Worktrees>,
    action: WorktreeAction,
) -> Arc<WorktreePlan> {
    let context = list.observation.context;
    let id = owner.prepare_worktree(list, action).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(id));
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    owner
        .context(context)
        .unwrap()
        .worktree_plan
        .clone()
        .unwrap()
}
fn change(owner: &mut Repositories, plan: Arc<WorktreePlan>) {
    let id = owner.change_worktree(plan).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(id));
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
    assert_eq!(record.effects, Effects::Verified);
}
pub(super) fn linked(repository: &Repository, name: &str) -> PathBuf {
    let path = repository.fixture.0.join(name);
    repository.git([
        OsString::from("worktree"),
        "add".into(),
        "-b".into(),
        name.into(),
        path.as_os_str().into(),
        "HEAD".into(),
    ]);
    path
}
fn refused(owner: &mut Repositories, list: Arc<Worktrees>, action: WorktreeAction, message: &str) {
    let id = owner.prepare_worktree(list, action).unwrap();
    finish(owner);
    let record = owner.record(id).unwrap();
    assert_eq!(record.effects, Effects::NotStarted, "{record:?}");
    assert!(
        record.diagnostic.as_deref().unwrap().contains(message),
        "{record:?}"
    );
}

// Both object formats retain raw paths, distinct Git identities and full lock
// reasons through create, visit and non-forced removal; source bytes stay saved.
#[test]
fn checked_roundtrip() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        fs::write(repository.root.join("file"), "base\n").unwrap();
        repository.commit();
        fs::write(repository.root.join("file"), "saved source\n").unwrap();
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let mut owner = repository.owner("");
        let observed = list(&repository, &mut owner);
        let source = observed.observation.context;
        let target = observed.entries[0].head.unwrap();
        let path = repository
            .fixture
            .0
            .join(OsString::from_vec(b"-linked\n\xff".to_vec()));
        let plan = prepare(
            &mut owner,
            observed,
            WorktreeAction::Create {
                path: path.clone(),
                branch: "linked".into(),
                target,
            },
        );
        change(&mut owner, plan);
        let observed = owner.context(source).unwrap().worktrees.clone().unwrap();
        assert_eq!(observed.entries.len(), 2);
        let entry = observed.entry(&path).unwrap();
        assert_eq!(entry.head, Some(target));
        assert_eq!(entry.branch, b"refs/heads/linked");
        assert_ne!(
            entry.git_directory,
            observed.observation.identity.git_directory
        );
        assert_eq!(fs::read(path.join("file")).unwrap(), b"base\n");
        let metadata = entry.git_directory.clone();
        let plan = prepare(
            &mut owner,
            observed,
            WorktreeAction::Lock {
                path: path.clone(),
                reason: "offline storage".into(),
            },
        );
        change(&mut owner, plan);
        let observed = owner.context(source).unwrap().worktrees.clone().unwrap();
        assert_eq!(
            observed.entry(&path).unwrap().locked.as_deref(),
            Some(b"offline storage".as_slice())
        );
        refused(
            &mut owner,
            observed,
            WorktreeAction::Remove(path.clone()),
            "locked",
        );
        let observed = list(&repository, &mut owner);
        let plan = prepare(&mut owner, observed, WorktreeAction::Unlock(path.clone()));
        change(&mut owner, plan);
        let observed = owner.context(source).unwrap().worktrees.clone().unwrap();
        owner.visit_worktree(Arc::clone(&observed), &path).unwrap();
        let visited = finish(&mut owner);
        assert!(visited.published, "{:?}", owner.record(visited.operation));
        assert_ne!(visited.context, Some(source));
        let identity = &owner
            .context(visited.context.unwrap())
            .unwrap()
            .observation
            .identity;
        assert_eq!(identity.worktree.as_ref(), Some(&path));
        assert_eq!(identity.git_directory, metadata);
        assert_eq!(
            identity.common_directory,
            observed.observation.identity.common_directory
        );
        let plan = prepare(&mut owner, observed, WorktreeAction::Remove(path.clone()));
        change(&mut owner, plan);
        assert!(!path.exists());
        assert!(!metadata.exists());
        assert_eq!(
            repository.git(["rev-parse", "refs/heads/linked"]),
            format!("{}\n", target.hex()).as_bytes()
        );
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        assert_eq!(
            fs::read(repository.root.join("file")).unwrap(),
            b"saved source\n"
        );
    }
}

// Removal must reject every saved dirty category and active sequence without
// launching Git's changing command, including ignored content Git could delete.
#[test]
fn protected_removal() {
    for variant in [
        "tracked",
        "untracked",
        "ignored",
        "sequence",
        "main",
        "current",
    ] {
        let repository = Repository::new("sha1");
        fs::write(repository.root.join("file"), "base\n").unwrap();
        fs::write(repository.root.join(".gitignore"), "ignored\n").unwrap();
        repository.commit();
        let path = linked(&repository, "linked");
        let mut owner = repository.owner("");
        let observed = list(&repository, &mut owner);
        let metadata = observed.entry(&path).unwrap().git_directory.clone();
        match variant {
            "tracked" => fs::write(path.join("file"), "dirty\n").unwrap(),
            "untracked" => fs::write(path.join("untracked"), "keep\n").unwrap(),
            "ignored" => fs::write(path.join("ignored"), "keep\n").unwrap(),
            "sequence" => {
                fs::write(
                    metadata.join("MERGE_HEAD"),
                    repository.git(["rev-parse", "HEAD"]),
                )
                .unwrap();
            }
            _ => {}
        }
        let observed = list(&repository, &mut owner);
        let (observed, target, expected) = if variant == "main" {
            (observed, repository.root.clone(), "Main or bare")
        } else if variant == "current" {
            repository.open(&mut owner, &path);
            let context = owner
                .contexts()
                .iter()
                .find(|c| c.observation.identity.worktree.as_ref() == Some(&path))
                .unwrap()
                .id;
            owner.worktrees(context).unwrap();
            finish(&mut owner);
            (
                owner.context(context).unwrap().worktrees.clone().unwrap(),
                path.clone(),
                "Current",
            )
        } else {
            (
                observed,
                path.clone(),
                if variant == "sequence" {
                    "active Git sequence"
                } else {
                    "clean saved files"
                },
            )
        };
        refused(
            &mut owner,
            observed,
            WorktreeAction::Remove(target),
            expected,
        );
        assert!(path.join("file").exists());
        assert!(metadata.exists());
    }
}

// A review never authorizes a later lock reason, replacement destination,
// dirty target, source edit or worktree HEAD; each race refuses before launch.
#[test]
fn stale_reviews() {
    for variant in ["lock", "dirty", "head", "source", "destination", "parent"] {
        let repository = Repository::new("sha1");
        fs::write(repository.root.join("file"), "base\n").unwrap();
        repository.commit();
        let path = linked(&repository, "linked");
        let mut owner = repository.owner("");
        let observed = list(&repository, &mut owner);
        let destination = repository.fixture.0.join("new-worktree");
        let action = if matches!(variant, "destination" | "parent") {
            WorktreeAction::Create {
                path: destination.clone(),
                branch: "created".into(),
                target: observed.entries[0].head.unwrap(),
            }
        } else {
            WorktreeAction::Remove(path.clone())
        };
        let plan = prepare(&mut owner, observed, action);
        match variant {
            "lock" => {
                repository.git([
                    OsString::from("worktree"),
                    "lock".into(),
                    "--reason=external".into(),
                    path.as_os_str().into(),
                ]);
            }
            "dirty" => fs::write(path.join("file"), "external\n").unwrap(),
            "head" => {
                repository.git([
                    OsString::from("-C"),
                    path.as_os_str().into(),
                    "commit".into(),
                    "--allow-empty".into(),
                    "--message=external".into(),
                ]);
            }
            "source" => fs::write(repository.root.join("file"), "external source\n").unwrap(),
            "destination" => fs::create_dir(&destination).unwrap(),
            "parent" => fs::write(repository.fixture.0.join("new-sibling"), "external\n").unwrap(),
            _ => unreachable!(),
        }
        let id = owner.change_worktree(plan).unwrap();
        finish(&mut owner);
        assert_eq!(
            owner.record(id).unwrap().effects,
            Effects::NotStarted,
            "{variant}: {:?}",
            owner.record(id)
        );
        assert!(path.join("file").exists());
        assert!(
            !repository
                .git(["branch", "--list", "created"])
                .iter()
                .any(|b| *b != b'\n')
        );
    }
}

// Missing unlocked records alone may be pruned. Locked missing roots survive,
// and a reappeared path invalidates the complete prune review before launch.
#[test]
fn checked_prune() {
    let repository = Repository::new("sha256");
    repository.commit();
    let missing = linked(&repository, "missing");
    let protected = linked(&repository, "protected");
    repository.git([
        OsString::from("worktree"),
        "lock".into(),
        "--reason=offline disk".into(),
        protected.as_os_str().into(),
    ]);
    fs::remove_dir_all(&missing).unwrap();
    fs::remove_dir_all(&protected).unwrap();
    let mut owner = repository.owner("");
    let observed = list(&repository, &mut owner);
    assert!(observed.entry(&missing).unwrap().prunable.is_some());
    let protected_metadata = observed.entry(&protected).unwrap().git_directory.clone();
    let missing_metadata = observed.entry(&missing).unwrap().git_directory.clone();
    let plan = prepare(&mut owner, observed, WorktreeAction::Prune);
    assert_eq!(plan.paths, vec![missing.clone()]);
    fs::create_dir(&missing).unwrap();
    let id = owner.change_worktree(plan).unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert!(missing_metadata.exists());
    fs::remove_dir(&missing).unwrap();
    let observed = list(&repository, &mut owner);
    let plan = prepare(&mut owner, observed, WorktreeAction::Prune);
    change(&mut owner, plan);
    assert!(!missing_metadata.exists());
    assert!(protected_metadata.exists());
    assert!(repository.root.exists());
    let observed = list(&repository, &mut owner);
    assert_eq!(
        observed.entry(&protected).unwrap().locked.as_deref(),
        Some(b"offline disk".as_slice())
    );
    refused(
        &mut owner,
        observed,
        WorktreeAction::Prune,
        "No missing unlocked",
    );
}

// Truncated lists cannot authorize changes, and an active linked context owns
// common-storage work even when a different worktree initiated the request.
#[test]
fn admission_and_ownership() {
    let repository = Repository::new("sha1");
    repository.commit();
    let path = linked(&repository, "linked");
    let mut bounded = repository.owner("max_refs=1");
    let observed = list(&repository, &mut bounded);
    assert!(matches!(
        observed.completeness,
        Completeness::Truncated { .. }
    ));
    assert!(
        bounded
            .prepare_worktree(observed, WorktreeAction::Prune)
            .unwrap_err()
            .contains("complete list")
    );
    let mut owner = repository.owner("");
    let observed = list(&repository, &mut owner);
    repository.open(&mut owner, &path);
    let linked = owner
        .contexts()
        .iter()
        .find(|c| c.observation.identity.worktree.as_ref() == Some(&path))
        .unwrap()
        .id;
    owner.refresh(linked).unwrap();
    assert!(
        owner
            .prepare_worktree(observed, WorktreeAction::Remove(path))
            .unwrap_err()
            .contains("Common repository storage")
    );
    finish(&mut owner);
}
