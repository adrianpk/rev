use super::*;
use rev::git::PushIntent;

pub(super) fn tracking(repository: &Repository) {
    repository.git(["fetch", "origin"]);
    repository.git(["branch", "--set-upstream-to=origin/main", "main"]);
}
pub(super) fn prepare(owner: &mut Repositories, refs: Arc<Refs>) -> Arc<PushIntent> {
    let context = refs.observation.context;
    let id = owner.prepare_push(refs, b"refs/heads/main", None).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(id));
    owner.push_intent(context).unwrap().clone()
}
pub(super) fn object(repository: &Repository, remote: &std::path::Path, name: &str) -> Vec<u8> {
    repository.git(["--git-dir", remote.to_str().unwrap(), "rev-parse", name])
}

// Explicit full-object refspecs send only the reviewed branch, regardless of
// matching defaults. A distinct push URL is verified at that actual endpoint.
#[test]
fn exact_upstream() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        repository.commit();
        let origin = remote(&repository, "origin");
        let destination = remote(&repository, "destination");
        tracking(&repository);
        let base = repository.git(["rev-parse", "HEAD"]);
        repository.git([
            "config",
            "remote.origin.pushurl",
            destination.to_str().unwrap(),
        ]);
        repository.git(["config", "push.default", "matching"]);
        repository.commit();
        repository.git(["branch", "unreviewed"]);
        repository.git(["tag", "--annotate", "local-only", "--message=Not published"]);
        let tip = repository.git(["rev-parse", "HEAD"]);
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let refs = observe(&mut owner);
        let intent = prepare(&mut owner, refs);
        assert_eq!(intent.source(), b"refs/heads/main");
        assert_eq!(intent.remote(), b"origin");
        assert_eq!(intent.destination(), b"refs/heads/main");
        assert!(!format!("{intent:?}").contains(destination.to_str().unwrap()));
        let id = owner.push(intent.clone()).unwrap();
        assert!(owner.push(intent).is_err());
        let update = finish(&mut owner);
        assert!(update.published, "{:?}", owner.record(id));
        let record = owner.record(id).unwrap();
        assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
        assert_eq!(record.effects, Effects::Verified);
        assert!(record.exit.unwrap().success());
        assert!(record.stdout.is_empty() && record.stderr.is_empty());
        assert_eq!(object(&repository, &destination, "refs/heads/main"), tip);
        assert_eq!(object(&repository, &origin, "refs/heads/main"), base);
        assert_eq!(
            repository.git([
                "--git-dir",
                destination.to_str().unwrap(),
                "for-each-ref",
                "--format=%(refname)"
            ]),
            b"refs/heads/main\n"
        );
        assert_eq!(repository.git(["rev-parse", "HEAD"]), tip);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
    }
}

// Missing/ambiguous tracking and policies which expand or contradict the
// reviewed request reject before a changing child or any publication.
#[test]
fn push_policy() {
    let repository = Repository::new("sha1");
    repository.commit();
    let origin = remote(&repository, "origin");
    remote(&repository, "other");
    let base = object(&repository, &origin, "main");
    let cases = [
        ("branch.main.merge", "refs/heads/other", true, "ambiguous"),
        ("branch.main.pushremote", "other", false, "contradicts"),
        ("remote.pushdefault", "other", false, "contradicts"),
        ("remote.origin.mirror", "true", false, "mirror"),
        ("push.followtags", "true", false, "followTags"),
        (
            "remote.origin.push",
            "+refs/heads/main:refs/heads/main",
            false,
            "refspecs",
        ),
        (
            "remote.origin.push",
            "refs/heads/*:refs/heads/*",
            false,
            "refspecs",
        ),
        ("push.recursesubmodules", "on-demand", false, "Recursive"),
        (
            "remote.origin.pushurl",
            "second-endpoint",
            true,
            "multiple push URLs",
        ),
    ];
    tracking(&repository);
    for (key, value, append, expected) in cases {
        let saved = repository.git(["config", "--get-all", "branch.main.merge"]);
        if key == "remote.origin.pushurl" {
            repository.git(["config", key, origin.to_str().unwrap()]);
        }
        repository.git([
            "config",
            if append { "--add" } else { "--replace-all" },
            key,
            value,
        ]);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let refs = observe(&mut owner);
        let id = owner.prepare_push(refs, b"refs/heads/main", None).unwrap();
        assert!(!finish(&mut owner).published, "{key}");
        let record = owner.record(id).unwrap();
        assert_eq!(record.effects, Effects::NotStarted, "{key}");
        assert!(
            record.diagnostic.as_ref().unwrap().contains(expected),
            "{key}: {record:?}"
        );
        repository.git(["config", "--unset-all", key]);
        if key == "branch.main.merge" {
            repository.git(["config", key, String::from_utf8_lossy(&saved).trim()]);
        }
        assert_eq!(object(&repository, &origin, "main"), base);
    }
    repository.git(["config", "--unset", "branch.main.remote"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&mut owner);
    let id = owner.prepare_push(refs, b"refs/heads/main", None).unwrap();
    assert!(!finish(&mut owner).published);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
}

// A review is generation/configuration bound, including the exact source
// object. Selecting a different remote never substitutes it for the upstream.
#[test]
fn stale_push() {
    let repository = Repository::new("sha1");
    repository.commit();
    let origin = remote(&repository, "origin");
    remote(&repository, "other");
    tracking(&repository);
    let base = object(&repository, &origin, "main");
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&mut owner);
    let id = owner
        .prepare_push(refs.clone(), b"refs/heads/main", Some(b"other"))
        .unwrap();
    assert!(!finish(&mut owner).published);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    let intent = prepare(&mut owner, refs);
    repository.git(["config", "push.followtags", "true"]);
    let id = owner.push(intent).unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    repository.git(["config", "--unset", "push.followtags"]);
    let refs = observe(&mut owner);
    let intent = prepare(&mut owner, refs);
    repository.commit();
    let id = owner.push(intent).unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert_eq!(object(&repository, &origin, "main"), base);
}

// The installed receiver's fast-forward check rejects an older reviewed
// object; the retained child failure is distinct from verified publication.
#[test]
fn receiver_rejection() {
    let repository = Repository::new("sha1");
    repository.commit();
    let origin = remote(&repository, "origin");
    tracking(&repository);
    let base = repository.git(["rev-parse", "HEAD"]);
    repository.commit();
    let tip = repository.git(["rev-parse", "HEAD"]);
    repository.git(["push", "origin", "main"]);
    repository.git(["reset", "--hard", String::from_utf8_lossy(&base).trim()]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&mut owner);
    let intent = prepare(&mut owner, refs);
    let id = owner.push(intent).unwrap();
    finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome, Outcome::Failed, "{record:?}");
    assert_eq!(record.effects, Effects::Possible);
    assert!(!record.exit.unwrap().success());
    assert_eq!(object(&repository, &origin, "main"), tip);
}
