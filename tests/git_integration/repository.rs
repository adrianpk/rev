use super::support::{Repository, finish};
use rev::git::{Completeness, Effects, Outcome, Repositories, StashAction, StashPlan, Stashes};
use std::{ffi::OsString, fs, os::unix::ffi::OsStringExt, sync::Arc};

fn list(repository: &Repository, owner: &mut Repositories) -> Arc<Stashes> {
    if owner.contexts().is_empty() {
        repository.open(owner, &repository.root);
    }
    let context = owner.contexts()[0].id;
    owner.stashes(context).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(update.operation));
    owner.context(context).unwrap().stashes.clone().unwrap()
}
fn prepare(owner: &mut Repositories, stashes: Arc<Stashes>, action: StashAction) -> Arc<StashPlan> {
    let context = stashes.observation.context;
    owner.prepare_stash(stashes, action).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(update.operation));
    assert_eq!(
        owner.record(update.operation).unwrap().effects,
        Effects::NotStarted
    );
    owner.context(context).unwrap().stash_plan.clone().unwrap()
}
fn change(owner: &mut Repositories, plan: Arc<StashPlan>) {
    let id = owner.change_stash(plan).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(id));
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
    assert_eq!(record.effects, Effects::Verified);
}

// Saved tracked/untracked bytes and full stash identities survive native
// create/show/apply/drop in both formats, including raw operating-system paths.
#[test]
fn saved_roundtrip() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        let path = OsString::from_vec(b"-file\n\xff".to_vec());
        fs::write(repository.root.join(&path), b"base\n").unwrap();
        repository.commit();
        let head = repository.git(["rev-parse", "HEAD"]);
        let index = repository.git(["ls-files", "--stage", "-z"]);
        fs::write(repository.root.join(&path), b"saved work\n").unwrap();
        fs::write(repository.root.join("untracked"), b"untracked\0binary").unwrap();
        let mut owner = repository.owner("");
        let stashes = list(&repository, &mut owner);
        assert!(stashes.entries.is_empty());
        let plan = prepare(
            &mut owner,
            stashes,
            StashAction::Create {
                message: "Saved example".into(),
            },
        );
        change(&mut owner, plan);
        let stashes = owner.contexts()[0].stashes.clone().unwrap();
        assert_eq!(stashes.entries.len(), 1);
        let object = stashes.entries[0].object;
        assert_eq!(object.hex().len(), if format == "sha256" { 64 } else { 40 });
        assert_eq!(
            repository.git(["rev-parse", &format!("{}^1", object.hex())]),
            head
        );
        assert_eq!(fs::read(repository.root.join(&path)).unwrap(), b"base\n");
        assert!(!repository.root.join("untracked").exists());
        owner.show_stash(stashes.clone(), object).unwrap();
        let update = finish(&mut owner);
        assert!(update.published);
        let detail = owner.contexts()[0].stash_detail.as_ref().unwrap();
        assert_eq!(detail.object, object);
        assert!(!detail.bytes.is_empty());
        assert_eq!(detail.completeness, Completeness::Complete);
        let plan = prepare(&mut owner, stashes, StashAction::Apply(object));
        change(&mut owner, plan);
        assert_eq!(
            fs::read(repository.root.join(&path)).unwrap(),
            b"saved work\n"
        );
        assert_eq!(
            fs::read(repository.root.join("untracked")).unwrap(),
            b"untracked\0binary"
        );
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        let stashes = owner.contexts()[0].stashes.clone().unwrap();
        assert_eq!(stashes.entries[0].object, object);
        let plan = prepare(&mut owner, stashes, StashAction::Drop(object));
        change(&mut owner, plan);
        assert!(
            owner.contexts()[0]
                .stashes
                .as_ref()
                .unwrap()
                .entries
                .is_empty()
        );
        assert_eq!(
            fs::read(repository.root.join(&path)).unwrap(),
            b"saved work\n"
        );
    }
}

// Pop and branch restore the selected full object and remove only its checked
// entry after success; branch starts at the stash's original parent commit.
#[test]
fn pop_and_branch() {
    for branch in [false, true] {
        let repository = Repository::new("sha256");
        fs::write(repository.root.join("file"), "base\n").unwrap();
        repository.commit();
        let base = repository.git(["rev-parse", "HEAD"]);
        fs::write(repository.root.join("file"), "saved\n").unwrap();
        repository.git(["stash", "push", "--message=selected"]);
        if branch {
            fs::write(repository.root.join("other"), "later\n").unwrap();
            repository.commit();
        }
        let mut owner = repository.owner("");
        let stashes = list(&repository, &mut owner);
        let object = stashes.entries[0].object;
        let action = if branch {
            StashAction::Branch {
                object,
                name: "from-stash".into(),
            }
        } else {
            StashAction::Pop(object)
        };
        let plan = prepare(&mut owner, stashes, action);
        change(&mut owner, plan);
        assert_eq!(fs::read(repository.root.join("file")).unwrap(), b"saved\n");
        assert_eq!(repository.git(["rev-parse", "HEAD"]), base);
        assert_eq!(
            repository.git(["symbolic-ref", "HEAD"]),
            if branch {
                b"refs/heads/from-stash\n".as_slice()
            } else {
                b"refs/heads/main\n"
            }
        );
        assert!(
            owner.contexts()[0]
                .stashes
                .as_ref()
                .unwrap()
                .entries
                .is_empty()
        );
    }
}

// Equal Status labels cannot authorize changed tracked/untracked content,
// index entries or HEAD while a saved-state review is open.
#[test]
fn changed_review() {
    for changed in ["tracked", "untracked", "index", "head"] {
        let repository = Repository::new("sha1");
        fs::write(repository.root.join("file"), "base\n").unwrap();
        repository.commit();
        fs::write(repository.root.join("file"), "saved\n").unwrap();
        repository.git(["stash", "push", "--message=selected"]);
        fs::write(repository.root.join("file"), "first dirty bytes\n").unwrap();
        fs::write(repository.root.join("untracked"), "first\n").unwrap();
        let mut owner = repository.owner("");
        let stashes = list(&repository, &mut owner);
        let plan = prepare(
            &mut owner,
            stashes,
            StashAction::Create {
                message: "reviewed".into(),
            },
        );
        match changed {
            "tracked" => fs::write(repository.root.join("file"), "other dirty bytes\n").unwrap(),
            "untracked" => fs::write(repository.root.join("untracked"), "other\n").unwrap(),
            "index" => {
                repository.git(["add", "--", "file"]);
            }
            "head" => repository.commit(),
            _ => {
                repository.git(["stash", "push", "--include-untracked", "--message=external"]);
            }
        }
        let before = repository.git(["stash", "list", "--format=%H"]);
        let disk = fs::read(repository.root.join("file")).unwrap();
        let id = owner.change_stash(plan).unwrap();
        finish(&mut owner);
        assert_eq!(
            owner.record(id).unwrap().effects,
            Effects::NotStarted,
            "{changed}"
        );
        assert_ne!(owner.record(id).unwrap().outcome, Outcome::Verified);
        assert_eq!(repository.git(["stash", "list", "--format=%H"]), before);
        assert_eq!(fs::read(repository.root.join("file")).unwrap(), disk);
    }
}

// A pop conflict retains its exact stash and exposes real unmerged stages;
// no drop, inferred rollback or synthetic merge/rebase sequence is performed.
#[test]
fn retained_conflict() {
    let repository = Repository::new("sha256");
    fs::write(repository.root.join("file"), "base\n").unwrap();
    repository.commit();
    fs::write(repository.root.join("file"), "stashed\n").unwrap();
    repository.git(["stash", "push"]);
    fs::write(repository.root.join("file"), "current\n").unwrap();
    repository.commit();
    let mut owner = repository.owner("");
    let stashes = list(&repository, &mut owner);
    let object = stashes.entries[0].object;
    let plan = prepare(&mut owner, stashes, StashAction::Pop(object));
    let id = owner.change_stash(plan).unwrap();
    let update = finish(&mut owner);
    assert!(update.published, "{:?}", owner.record(id));
    assert_eq!(owner.record(id).unwrap().outcome, Outcome::InProgress);
    assert_eq!(owner.record(id).unwrap().effects, Effects::Possible);
    assert_eq!(
        owner.contexts()[0].stashes.as_ref().unwrap().entries[0].object,
        object
    );
    let sequence = owner.contexts()[0].sequence.as_ref().unwrap();
    assert!(sequence.family.is_none());
    assert_eq!(sequence.conflicts, [std::path::PathBuf::from("file")]);
    assert!(!repository.git(["ls-files", "--unmerged", "-z"]).is_empty());
    assert!(
        fs::read_to_string(repository.root.join("file"))
            .unwrap()
            .contains("<<<<<<<")
    );
}

// Reached list/detail caps stay observable; partial authority cannot launch a
// changing action and failed dirty restoration preparation has no side effects.
#[test]
fn stash_admission() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("file"), "base\n").unwrap();
    repository.commit();
    for n in 0..2 {
        fs::write(
            repository.root.join("file"),
            format!("{n}{}\n", "x".repeat(20000)),
        )
        .unwrap();
        repository.git(["stash", "push"]);
    }
    let mut owner = repository.owner("max_refs=1\nmax_detail_bytes=16384");
    let stashes = list(&repository, &mut owner);
    assert!(matches!(
        stashes.completeness,
        Completeness::Truncated { .. }
    ));
    assert!(
        owner
            .prepare_stash(
                stashes.clone(),
                StashAction::Drop(stashes.entries[0].object)
            )
            .is_err()
    );
    let mut owner = repository.owner("max_detail_bytes=16384");
    let stashes = list(&repository, &mut owner);
    owner
        .show_stash(stashes.clone(), stashes.entries[0].object)
        .unwrap();
    let update = finish(&mut owner);
    assert!(update.published, "{:?}", owner.record(update.operation));
    assert!(matches!(
        owner.contexts()[0]
            .stash_detail
            .as_ref()
            .unwrap()
            .completeness,
        Completeness::Truncated { .. }
    ));
    fs::write(repository.root.join("file"), "dirty\n").unwrap();
    let id = owner
        .prepare_stash(
            stashes.clone(),
            StashAction::Apply(stashes.entries[0].object),
        )
        .unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert_eq!(fs::read(repository.root.join("file")).unwrap(), b"dirty\n");
}

#[path = "repository_presentation.rs"]
mod presentation;

// A new stash entry shifts the same ordinal without changing saved source
// state. The captured full-object removal cannot target its new occupant.
#[test]
fn stash_ordinal() {
    let repository = Repository::new("sha256");
    fs::write(repository.root.join("file"), "base\n").unwrap();
    repository.commit();
    for value in ["first", "second"] {
        fs::write(repository.root.join("file"), value).unwrap();
        repository.git(["stash", "push"]);
    }
    let mut owner = repository.owner("");
    let stashes = list(&repository, &mut owner);
    let selected = stashes.entries[1].object;
    let plan = prepare(&mut owner, stashes.clone(), StashAction::Drop(selected));
    repository.git(["stash", "store", "--message=external", &selected.hex()]);
    let before = repository.git(["stash", "list", "--format=%H"]);
    assert_eq!(before.iter().filter(|b| **b == b'\n').count(), 3);
    assert_eq!(
        repository.git(["rev-parse", "stash@{1}"]),
        format!("{}\n", stashes.entries[0].object.hex()).as_bytes()
    );
    let id = owner.change_stash(plan).unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert!(
        owner
            .record(id)
            .unwrap()
            .diagnostic
            .as_deref()
            .unwrap()
            .contains("Stash list changed")
    );
    assert_eq!(repository.git(["stash", "list", "--format=%H"]), before);
    assert_eq!(fs::read(repository.root.join("file")).unwrap(), b"base\n");
    let refreshed = list(&repository, &mut owner);
    assert!(
        owner
            .prepare_stash(refreshed, StashAction::Drop(selected))
            .unwrap_err()
            .contains("ambiguous")
    );
    assert_eq!(repository.git(["stash", "list", "--format=%H"]), before);
}
