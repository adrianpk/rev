use super::support::{Repository, finish};
use rev::git::{
    ApplyKind, Effects, History, ObjectId, Outcome, Repositories, RevisionChoice, SequenceFamily,
};
use std::{fs, sync::Arc};

fn object(repository: &Repository, name: &str) -> ObjectId {
    let format = if repository.git(["rev-parse", "--show-object-format"]) == b"sha256\n" {
        rev::git::ObjectFormat::Sha256
    } else {
        rev::git::ObjectFormat::Sha1
    };
    ObjectId::parse(
        repository
            .git(["rev-parse", name])
            .strip_suffix(b"\n")
            .unwrap(),
        format,
    )
    .unwrap()
}
fn history(owner: &mut Repositories, root: ObjectId) -> Arc<History> {
    let context = owner.contexts()[0].id;
    let id = owner.log(context, Some(&root.hex())).unwrap();
    assert!(finish(owner).published, "{:?}", owner.record(id));
    owner.context(context).unwrap().history.clone().unwrap()
}
fn plan(
    owner: &mut Repositories,
    history: Arc<History>,
    kind: ApplyKind,
    choices: &[RevisionChoice],
) -> Arc<rev::git::ApplyPlan> {
    let id = owner.prepare_sequence(history, kind, choices).unwrap();
    assert!(finish(owner).published, "{:?}", owner.record(id));
    owner.contexts()[0].application_plan.clone().unwrap()
}
fn source(repository: &Repository) -> [ObjectId; 2] {
    fs::write(repository.root.join("base"), "base\n").unwrap();
    repository.commit();
    repository.git(["switch", "-c", "topic"]);
    let mut targets = Vec::new();
    for (file, message) in [
        ("first", "First authored change"),
        ("second", "Second authored change"),
    ] {
        fs::write(repository.root.join(file), message).unwrap();
        repository.git(["add", "--all"]);
        repository.git(["commit", "-m", message]);
        targets.push(object(repository, "HEAD"));
    }
    repository.git(["switch", "main"]);
    [targets[0], targets[1]]
}

// The reviewed order is user order, independent of ancestry navigation. Real
// Git outcomes agree in content, authored messages and exact single parents.
#[test]
fn ordered_application() {
    for (format, kind) in [
        ("sha1", ApplyKind::CherryPick),
        ("sha256", ApplyKind::Revert),
    ] {
        let repository = Repository::new(format);
        let [first, second] = source(&repository);
        if kind == ApplyKind::Revert {
            repository.git(["merge", "--ff-only", "topic"]);
        }
        let before = object(&repository, "HEAD");
        repository.git(["branch", "reference"]);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let h = history(&mut owner, second);
        let choices = [
            RevisionChoice {
                object: second,
                mainline: None,
            },
            RevisionChoice {
                object: first,
                mainline: None,
            },
        ];
        let p = plan(&mut owner, h, kind, &choices);
        assert_eq!(
            p.steps.iter().map(|s| s.object).collect::<Vec<_>>(),
            [second, first]
        );
        assert_eq!(object(&repository, "HEAD"), before);
        let id = owner.apply_sequence(p).unwrap();
        let update = finish(&mut owner);
        let record = owner.record(id).unwrap();
        assert!(update.published, "{record:?}");
        assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
        assert_eq!(record.effects, Effects::Verified);
        let tree = repository.git(["rev-parse", "HEAD^{tree}"]);
        let messages = repository.git([
            "log",
            "--reverse",
            "--format=%B%an <%ae>",
            &format!("{}..HEAD", before.hex()),
        ]);
        assert_eq!(
            repository.git(["rev-parse", "HEAD~2"]),
            format!("{}\n", before.hex()).as_bytes()
        );
        if kind == ApplyKind::CherryPick {
            assert_eq!(
                repository.git([
                    "log",
                    "--reverse",
                    "--format=%s",
                    &format!("{}..HEAD", before.hex())
                ]),
                b"Second authored change\nFirst authored change\n"
            );
            assert_eq!(
                fs::read(repository.root.join("first")).unwrap(),
                b"First authored change"
            );
            assert_eq!(
                fs::read(repository.root.join("second")).unwrap(),
                b"Second authored change"
            );
        } else {
            assert!(!repository.root.join("first").exists());
            assert!(!repository.root.join("second").exists());
        }
        repository.git(["switch", "reference"]);
        repository.git([kind.label(), "--no-edit", &second.hex(), &first.hex()]);
        assert_eq!(repository.git(["rev-parse", "HEAD^{tree}"]), tree);
        assert_eq!(
            repository.git([
                "log",
                "--reverse",
                "--format=%B%an <%ae>",
                &format!("{}..HEAD", before.hex())
            ]),
            messages
        );
    }
}

fn merges(repository: &Repository) -> [ObjectId; 2] {
    fs::write(repository.root.join("base"), "base\n").unwrap();
    repository.commit();
    repository.git(["switch", "-c", "side"]);
    fs::write(repository.root.join("side"), "side\n").unwrap();
    repository.commit();
    repository.git(["switch", "-c", "topic", "main"]);
    fs::write(repository.root.join("first"), "first\n").unwrap();
    repository.commit();
    repository.git(["merge", "--no-ff", "--no-edit", "side"]);
    let first = object(repository, "HEAD");
    repository.git(["switch", "-c", "side2"]);
    fs::write(repository.root.join("second"), "second\n").unwrap();
    repository.commit();
    repository.git(["switch", "topic"]);
    fs::write(repository.root.join("third"), "third\n").unwrap();
    repository.commit();
    repository.git(["merge", "--no-ff", "--no-edit", "side2"]);
    let second = object(repository, "HEAD");
    repository.git(["switch", "main"]);
    [first, second]
}

// Different merges can select different exact parents in one ordered request.
// Each actual patch is compared with installed Git using those same mainlines.
#[test]
fn explicit_mainlines() {
    let repository = Repository::new("sha256");
    let [first, second] = merges(&repository);
    repository.git(["branch", "reference"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let h = history(&mut owner, second);
    for mainline in [None, Some(0), Some(3)] {
        assert!(
            owner
                .prepare_sequence(
                    Arc::clone(&h),
                    ApplyKind::CherryPick,
                    &[RevisionChoice {
                        object: first,
                        mainline
                    }]
                )
                .is_err()
        );
    }
    let p = plan(
        &mut owner,
        h,
        ApplyKind::CherryPick,
        &[
            RevisionChoice {
                object: first,
                mainline: Some(2),
            },
            RevisionChoice {
                object: second,
                mainline: Some(1),
            },
        ],
    );
    assert_eq!(
        p.steps[0].parent,
        Some(object(&repository, &format!("{}^2", first.hex())))
    );
    let id = owner.apply_sequence(p).unwrap();
    finish(&mut owner);
    assert_eq!(
        owner.record(id).unwrap().outcome,
        Outcome::Verified,
        "{:?}",
        owner.record(id)
    );
    let tree = repository.git(["rev-parse", "HEAD^{tree}"]);
    assert_eq!(fs::read(repository.root.join("first")).unwrap(), b"first\n");
    assert_eq!(
        fs::read(repository.root.join("second")).unwrap(),
        b"second\n"
    );
    assert!(!repository.root.join("side").exists());
    assert!(!repository.root.join("third").exists());
    repository.git(["switch", "reference"]);
    repository.git(["cherry-pick", "--no-edit", "-m", "2", &first.hex()]);
    repository.git(["cherry-pick", "--no-edit", "-m", "1", &second.hex()]);
    assert_eq!(repository.git(["rev-parse", "HEAD^{tree}"]), tree);
}

// A later conflict retains the verified prefix and exact original context;
// refreshing Git state preserves that authority without running later steps.
#[test]
fn retained_order() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("file"), "base\n").unwrap();
    repository.commit();
    repository.git(["switch", "-c", "topic"]);
    fs::write(repository.root.join("first"), "first\n").unwrap();
    repository.commit();
    let first = object(&repository, "HEAD");
    fs::write(repository.root.join("file"), "topic\n").unwrap();
    repository.commit();
    let second = object(&repository, "HEAD");
    repository.git(["switch", "main"]);
    fs::write(repository.root.join("file"), "local\n").unwrap();
    repository.commit();
    let original = object(&repository, "HEAD");
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let h = history(&mut owner, second);
    let p = plan(
        &mut owner,
        h,
        ApplyKind::CherryPick,
        &[
            RevisionChoice {
                object: first,
                mainline: None,
            },
            RevisionChoice {
                object: second,
                mainline: None,
            },
        ],
    );
    let id = owner.apply_sequence(p).unwrap();
    let update = finish(&mut owner);
    assert_eq!(
        owner.record(id).unwrap().outcome,
        Outcome::InProgress,
        "{:?}",
        owner.record(id)
    );
    let s = owner
        .context(update.context.unwrap())
        .unwrap()
        .sequence
        .clone()
        .unwrap();
    let progress = s.application.as_ref().unwrap();
    assert_eq!(s.family, Some(SequenceFamily::CherryPick));
    assert_eq!(s.current, Some(second));
    assert_eq!(s.original, Some(original));
    assert_eq!(s.step, Some(2));
    assert_eq!(s.total, Some(2));
    assert_eq!(progress.next, 1);
    assert_eq!(progress.completed.len(), 1);
    let head = object(&repository, "HEAD");
    assert_eq!(progress.before, head);
    assert_eq!(fs::read(repository.root.join("first")).unwrap(), b"first\n");
    owner.sequence(update.context.unwrap()).unwrap();
    assert!(finish(&mut owner).published);
    assert_eq!(object(&repository, "HEAD"), head);
    assert!(Arc::ptr_eq(
        owner.contexts()[0]
            .sequence
            .as_ref()
            .unwrap()
            .application
            .as_ref()
            .unwrap(),
        progress
    ));
}

// Duplicate targets, invalid mainlines and stale clean/disk evidence reject
// before changing HEAD; no automatic request is queued after a failed check.
#[test]
fn application_admission() {
    for changed in ["HEAD", "disk"] {
        let repository = Repository::new("sha1");
        let [first, second] = source(&repository);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let h = history(&mut owner, second);
        let choice = RevisionChoice {
            object: first,
            mainline: None,
        };
        assert!(
            owner
                .prepare_sequence(Arc::clone(&h), ApplyKind::CherryPick, &[choice, choice])
                .is_err()
        );
        assert!(
            owner
                .prepare_sequence(
                    Arc::clone(&h),
                    ApplyKind::CherryPick,
                    &[RevisionChoice {
                        object: first,
                        mainline: Some(1)
                    }]
                )
                .is_err()
        );
        let p = plan(&mut owner, h, ApplyKind::CherryPick, &[choice]);
        if changed == "HEAD" {
            repository.commit();
        } else {
            fs::write(repository.root.join("dirty"), "keep\n").unwrap();
        }
        let before = object(&repository, "HEAD");
        let id = owner.apply_sequence(p).unwrap();
        finish(&mut owner);
        assert_eq!(
            owner.record(id).unwrap().effects,
            Effects::NotStarted,
            "{changed}: {:?}",
            owner.record(id)
        );
        assert_eq!(object(&repository, "HEAD"), before);
    }
}

#[path = "sequences_presentation.rs"]
mod presentation;

#[path = "sequences_controls.rs"]
mod controls;

#[path = "sequences_bisect.rs"]
mod bisect;
