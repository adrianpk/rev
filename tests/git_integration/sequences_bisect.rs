use super::*;
use rev::git::{BisectAction, BisectPlan, BisectResult, Sequence};

pub(super) fn linear(repository: &Repository, count: usize) -> Vec<ObjectId> {
    let mut ids = Vec::new();
    for n in 0..count {
        fs::write(repository.root.join("value"), format!("{n}\n")).unwrap();
        repository.commit();
        ids.push(object(repository, "HEAD"));
    }
    ids
}
fn state(owner: &mut Repositories) -> Arc<Sequence> {
    let context = owner.contexts()[0].id;
    let id = owner.sequence(context).unwrap();
    assert!(finish(owner).published, "{:?}", owner.record(id));
    owner.context(context).unwrap().sequence.clone().unwrap()
}
fn prepared(owner: &mut Repositories, bad: ObjectId, good: ObjectId) -> Arc<BisectPlan> {
    let s = state(owner);
    let id = owner
        .prepare_bisect(Arc::clone(&s.refs), &bad.hex(), &good.hex())
        .unwrap();
    assert!(finish(owner).published, "{:?}", owner.record(id));
    owner.contexts()[0].bisect_plan.clone().unwrap()
}
fn begin(repository: &Repository, ids: &[ObjectId]) -> Repositories {
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let p = prepared(&mut owner, *ids.last().unwrap(), ids[0]);
    let id = owner.start_bisect(p).unwrap();
    assert!(finish(&mut owner).published, "{:?}", owner.record(id));
    assert_eq!(
        owner.record(id).unwrap().effects,
        Effects::Verified,
        "{:?}",
        owner.record(id)
    );
    owner
}

// Explicit fixture classifications identify the known content regression in
// both object formats. The product never derives a verdict or runs a test.
#[test]
fn manual_bisect() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        let ids = linear(&repository, 9);
        let original = object(&repository, "HEAD");
        let original_tree = repository.git(["rev-parse", "HEAD^{tree}"]);
        let mut owner = begin(&repository, &ids);
        let mut found = false;
        for _ in 0..8 {
            let s = state(&mut owner);
            let b = s.bisect.as_ref().unwrap();
            assert_eq!(s.family, Some(SequenceFamily::Bisect));
            assert_eq!(s.original, Some(original));
            assert_eq!(
                repository.git(["rev-parse", "main"]),
                format!("{}\n", original.hex()).as_bytes()
            );
            if let BisectResult::Found(bad) = b.result {
                assert_eq!(bad, ids[5]);
                found = true;
                break;
            }
            assert_eq!(b.result, BisectResult::Searching);
            let value: usize = String::from_utf8(fs::read(repository.root.join("value")).unwrap())
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            assert_eq!(s.refs.head, Some(ids[value]));
            let verdict = if value >= 5 {
                BisectAction::Bad
            } else {
                BisectAction::Good
            };
            let id = owner.classify_bisect(s, verdict).unwrap();
            assert!(finish(&mut owner).published, "{:?}", owner.record(id));
            assert_eq!(
                owner.record(id).unwrap().effects,
                Effects::Verified,
                "{:?}",
                owner.record(id)
            );
            assert!(
                owner
                    .record(id)
                    .unwrap()
                    .argv
                    .iter()
                    .any(|s| s == verdict.label())
            );
        }
        assert!(
            found,
            "finite manual fixture did not identify the known first bad commit"
        );
        let s = state(&mut owner);
        assert!(
            owner
                .classify_bisect(Arc::clone(&s), BisectAction::Good)
                .is_err()
        );
        let id = owner.classify_bisect(s, BisectAction::Reset).unwrap();
        assert!(finish(&mut owner).published);
        assert_eq!(
            owner.record(id).unwrap().outcome,
            Outcome::Verified,
            "{:?}",
            owner.record(id)
        );
        assert_eq!(
            repository.git(["symbolic-ref", "HEAD"]),
            b"refs/heads/main\n"
        );
        assert_eq!(object(&repository, "HEAD"), original);
        assert_eq!(repository.git(["rev-parse", "HEAD^{tree}"]), original_tree);
        assert!(!repository.root.join(".git/BISECT_START").exists());
        assert!(
            repository
                .git(["for-each-ref", "--format=%(refname)", "refs/bisect/"])
                .is_empty()
        );
    }
}

// Skipping every intermediate candidate cannot establish a unique first bad.
// Git's exit 2 is admitted only with matching actual skipped refs and graph.
#[test]
fn bisect_ambiguity() {
    let repository = Repository::new("sha256");
    let ids = linear(&repository, 4);
    let mut owner = begin(&repository, &ids);
    for n in 0..2 {
        let s = state(&mut owner);
        let id = owner.classify_bisect(s, BisectAction::Skip).unwrap();
        assert!(finish(&mut owner).published, "{:?}", owner.record(id));
        let record = owner.record(id).unwrap();
        assert_eq!(record.effects, Effects::Verified, "{record:?}");
        assert_eq!(record.outcome, Outcome::InProgress, "{record:?}");
        assert_eq!(
            record.exit.unwrap().code(),
            Some(if n == 0 { 0 } else { 2 })
        );
    }
    let s = state(&mut owner);
    let b = s.bisect.as_ref().unwrap();
    assert_eq!(b.result, BisectResult::Ambiguous);
    assert_eq!(b.remaining, 3);
    assert_eq!(b.skipped.len(), 2);
    assert!(
        owner
            .classify_bisect(Arc::clone(&s), BisectAction::Bad)
            .is_err()
    );
    let id = owner.classify_bisect(s, BisectAction::Reset).unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().outcome, Outcome::Verified);
}

// Stale boundaries and candidate metadata/index/HEAD/original refs reject
// before mutation. External bisect remains visible without invented lineage.
#[test]
fn bisect_fences() {
    for changed in ["metadata", "index", "candidate", "original"] {
        let repository = Repository::new("sha1");
        let ids = linear(&repository, 7);
        let mut owner = begin(&repository, &ids);
        let s = state(&mut owner);
        match changed {
            "metadata" => {
                fs::write(repository.root.join(".git/BISECT_LOG"), "external change\n").unwrap();
            }
            "index" => {
                fs::write(repository.root.join("extra"), "extra\n").unwrap();
                repository.git(["add", "extra"]);
            }
            "candidate" => {
                repository.git(["checkout", "--detach", &ids[2].hex()]);
            }
            _ => {
                repository.git(["update-ref", "refs/heads/main", &ids[5].hex()]);
            }
        }
        let before = object(&repository, "HEAD");
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let id = owner.classify_bisect(s, BisectAction::Reset).unwrap();
        finish(&mut owner);
        assert_eq!(
            owner.record(id).unwrap().effects,
            Effects::NotStarted,
            "{changed}: {:?}",
            owner.record(id)
        );
        assert_eq!(object(&repository, "HEAD"), before);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        assert!(repository.root.join(".git/BISECT_START").exists());
    }
    let repository = Repository::new("sha256");
    let ids = linear(&repository, 4);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let s = state(&mut owner);
    let id = owner
        .prepare_bisect(Arc::clone(&s.refs), &ids[0].hex(), &ids[3].hex())
        .unwrap();
    assert!(!finish(&mut owner).published);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    let p = prepared(&mut owner, ids[3], ids[0]);
    repository.commit();
    let before = object(&repository, "HEAD");
    let id = owner.start_bisect(p).unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert_eq!(object(&repository, "HEAD"), before);
    repository.git(["bisect", "start", &ids[3].hex(), &ids[0].hex(), "--"]);
    let s = state(&mut owner);
    assert_eq!(s.family, Some(SequenceFamily::Bisect));
    assert!(s.bisect.as_ref().unwrap().plan.is_none());
    assert!(
        owner
            .classify_bisect(s, BisectAction::Reset)
            .unwrap_err()
            .contains("lineage")
    );
    let limited = Repository::new("sha1");
    let ids = linear(&limited, 4);
    limited.git(["branch", "extra1"]);
    limited.git(["branch", "extra2"]);
    let mut owner = limited.owner("max_refs=2");
    limited.open(&mut owner, &limited.root);
    let s = state(&mut owner);
    assert_ne!(s.refs.completeness, rev::git::Completeness::Complete);
    let id = owner
        .prepare_bisect(Arc::clone(&s.refs), &ids[3].hex(), &ids[0].hex())
        .unwrap();
    assert!(!finish(&mut owner).published);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert!(!limited.root.join(".git/BISECT_START").exists());
    let bounded = Repository::new("sha256");
    let ids = linear(&bounded, 3);
    let mut owner = bounded.owner("max_refs=2");
    bounded.open(&mut owner, &bounded.root);
    let p = prepared(&mut owner, ids[2], ids[0]);
    let id = owner.start_bisect(p).unwrap();
    assert!(finish(&mut owner).published, "{:?}", owner.record(id));
    let s = state(&mut owner);
    let before = object(&bounded, "HEAD");
    let refs = bounded.git([
        "for-each-ref",
        "--format=%(refname) %(objectname)",
        "refs/bisect/",
    ]);
    let id = owner.classify_bisect(s, BisectAction::Good).unwrap();
    assert!(!finish(&mut owner).published);
    assert_eq!(
        owner.record(id).unwrap().effects,
        Effects::NotStarted,
        "{:?}",
        owner.record(id)
    );
    assert_eq!(object(&bounded, "HEAD"), before);
    assert_eq!(
        bounded.git([
            "for-each-ref",
            "--format=%(refname) %(objectname)",
            "refs/bisect/"
        ]),
        refs
    );
}
