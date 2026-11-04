use super::support::{Repository, finish};
use rev::git::{Effects, Outcome, Refs, Repositories};
use std::{fs, path::PathBuf, sync::Arc};

fn observe(owner: &mut Repositories) -> Arc<Refs> {
    let context = owner.contexts()[0].id;
    owner.refs(context).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(update.operation));
    owner
        .context(context)
        .unwrap()
        .refs
        .as_ref()
        .unwrap()
        .clone()
}
fn remote(repository: &Repository, name: &str) -> PathBuf {
    let path = repository.fixture.0.join(format!("{name}.git"));
    repository.git([
        "clone",
        "--bare",
        repository.root.to_str().unwrap(),
        path.to_str().unwrap(),
    ]);
    repository.git(["remote", "add", name, path.to_str().unwrap()]);
    path
}

// Fetch honors the selected remote/refspec and pull policy without integrating
// HEAD, touching the index or replacing either saved or unsaved worktree text.
#[test]
fn selected_fetch() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        fs::write(repository.root.join("source"), "base\n").unwrap();
        repository.commit();
        let base = repository.git(["rev-parse", "HEAD"]);
        let origin = remote(&repository, "origin");
        remote(&repository, "untouched");
        repository.git(["config", "pull.rebase", "true"]);
        repository.git(["config", "branch.main.remote", "origin"]);
        repository.git(["config", "branch.main.merge", "refs/heads/main"]);
        repository.git([
            "config",
            "remote.origin.fetch",
            "+refs/heads/main:refs/remotes/review/main",
        ]);
        fs::write(repository.root.join("source"), "remote change\n").unwrap();
        repository.commit();
        let tip = repository.git(["rev-parse", "HEAD"]);
        repository.git(["push", origin.to_str().unwrap(), "HEAD:refs/heads/main"]);
        repository.git(["reset", "--hard", String::from_utf8_lossy(&base).trim()]);
        fs::write(repository.root.join("source"), "local disk edit\n").unwrap();
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let refs = observe(&mut owner);
        assert!(owner.fetch(refs.clone(), b"missing").is_err());
        let id = owner.fetch(refs.clone(), b"origin").unwrap();
        assert!(owner.fetch(refs, b"origin").is_err());
        let update = finish(&mut owner);
        assert!(update.published, "{:?}", owner.record(id));
        let record = owner.record(id).unwrap();
        assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
        assert_eq!(record.effects, Effects::Verified);
        assert!(record.argv.iter().any(|s| s == "fetch"));
        assert!(record.stdout.is_empty() && record.stderr.is_empty());
        assert_eq!(repository.git(["rev-parse", "HEAD"]), base);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        assert_eq!(
            fs::read(repository.root.join("source")).unwrap(),
            b"local disk edit\n"
        );
        assert_eq!(
            repository.git(["rev-parse", "refs/remotes/review/main"]),
            tip
        );
        assert!(
            repository
                .git(["for-each-ref", "refs/remotes/untouched/"])
                .is_empty()
        );
        let current = owner
            .context(update.context.unwrap())
            .unwrap()
            .refs
            .as_ref()
            .unwrap();
        assert_eq!(
            current
                .reference(b"refs/remotes/review/main")
                .unwrap()
                .object
                .hex(),
            String::from_utf8_lossy(&tip).trim()
        );
        assert!(owner.check_refs(current).is_ok());
    }
}

// A selected name without a configured URL never falls back to a repository
// path; stale/incomplete observations also cannot authorize transport.
#[test]
fn fetch_admission() {
    let repository = Repository::new("sha1");
    repository.commit();
    repository.git([
        "config",
        "remote.path-only.fetch",
        "+refs/heads/*:refs/remotes/path-only/*",
    ]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&mut owner);
    let id = owner.fetch(refs.clone(), b"path-only").unwrap();
    finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome, Outcome::Failed);
    assert_eq!(record.effects, Effects::NotStarted);
    assert!(
        record
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("no transport URL")
    );
    assert!(owner.fetch(refs, b"path-only").is_err());
    let refs = observe(&mut owner);
    repository.git(["branch", "changed"]);
    let id = owner.fetch(refs, b"path-only").unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    remote(&repository, "second");
    let mut limited = repository.owner("max_remotes=1");
    repository.open(&mut limited, &repository.root);
    let refs = observe(&mut limited);
    assert!(limited.fetch(refs, b"path-only").is_err());
}

#[path = "transport_presentation.rs"]
mod presentation;

#[path = "transport_push.rs"]
mod push;

#[path = "transport_acceptance.rs"]
mod acceptance;
