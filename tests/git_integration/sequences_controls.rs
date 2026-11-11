use super::*;
use rev::git::{Sequence, SequenceControl};

fn refresh(owner: &mut Repositories) -> Arc<Sequence> {
    let context = owner.contexts()[0].id;
    let id = owner.sequence(context).unwrap();
    assert!(finish(owner).published, "{:?}", owner.record(id));
    owner.context(context).unwrap().sequence.clone().unwrap()
}
pub(super) fn conflict(repository: &Repository, kind: ApplyKind) -> [ObjectId; 3] {
    fs::write(repository.root.join("file"), "base\n").unwrap();
    repository.commit();
    repository.git(["switch", "-c", "topic"]);
    let mut targets = Vec::new();
    for (file, contents) in [
        ("first", "prefix\n"),
        ("file", "topic\n"),
        ("last", "last\n"),
    ] {
        fs::write(repository.root.join(file), contents).unwrap();
        repository.commit();
        targets.push(object(repository, "HEAD"));
    }
    repository.git(["switch", "main"]);
    if kind == ApplyKind::Revert {
        repository.git(["merge", "--ff-only", "topic"]);
    }
    fs::write(repository.root.join("file"), "local\n").unwrap();
    repository.commit();
    [targets[0], targets[1], targets[2]]
}
fn stopped(repository: &Repository, kind: ApplyKind, targets: &[ObjectId]) -> Repositories {
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let h = history(&mut owner, *targets.last().unwrap());
    let p = plan(
        &mut owner,
        h,
        kind,
        &targets
            .iter()
            .map(|object| RevisionChoice {
                object: *object,
                mainline: None,
            })
            .collect::<Vec<_>>(),
    );
    let id = owner.apply_sequence(p).unwrap();
    assert!(finish(&mut owner).published);
    assert_eq!(
        owner.record(id).unwrap().outcome,
        Outcome::InProgress,
        "{:?}",
        owner.record(id)
    );
    owner
}

// Every compatible control operates on an actual stopped Git revision. A
// completed prefix is restored only by explicit abort; later targets keep order.
#[test]
fn ordered_controls() {
    for (kind, format) in [
        (ApplyKind::CherryPick, "sha1"),
        (ApplyKind::Revert, "sha256"),
    ] {
        for action in [
            SequenceControl::Continue,
            SequenceControl::Skip,
            SequenceControl::Abort,
        ] {
            check_ordered_control(kind, format, action);
        }
    }
}

fn check_ordered_control(kind: ApplyKind, format: &str, action: SequenceControl) {
    let repository = Repository::new(format);
    let targets = conflict(&repository, kind);
    let original = object(&repository, "HEAD");
    let original_tree = repository.git(["rev-parse", "HEAD^{tree}"]);
    let mut owner = stopped(&repository, kind, &targets);
    let before = object(&repository, "HEAD");
    let state = refresh(&mut owner);
    assert_eq!(state.step, Some(2));
    assert_eq!(state.total, Some(3));
    assert!(
        owner
            .control_sequence(Arc::clone(&state), SequenceControl::Continue, None)
            .is_err()
    );
    let wrong = if kind == ApplyKind::CherryPick {
        SequenceFamily::Revert
    } else {
        SequenceFamily::CherryPick
    };
    assert!(owner.control_sequence(state, action, Some(wrong)).is_err());
    if action == SequenceControl::Continue {
        fs::write(repository.root.join("file"), "resolved\n").unwrap();
        repository.git(["add", "--", "file"]);
    }
    let state = refresh(&mut owner);
    let id = owner.control_sequence(state, action, None).unwrap();
    let update = finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert!(update.published, "{kind:?} {action:?}: {record:?}");
    assert_eq!(
        record.effects,
        Effects::Verified,
        "{kind:?} {action:?}: {record:?}"
    );
    assert_eq!(
        record.outcome,
        match action {
            SequenceControl::Continue => Outcome::Verified,
            SequenceControl::Skip => Outcome::Skipped,
            SequenceControl::Abort => Outcome::Aborted,
        },
        "{kind:?} {action:?}: {record:?}"
    );
    assert!(
        owner.contexts()[0]
            .sequence
            .as_ref()
            .unwrap()
            .family
            .is_none()
    );
    assert!(repository.git(["status", "--porcelain"]).is_empty());
    assert_eq!(
        repository.git(["symbolic-ref", "HEAD"]),
        b"refs/heads/main\n"
    );
    assert_eq!(object(&repository, "topic"), targets[2]);
    if action == SequenceControl::Abort {
        assert_eq!(object(&repository, "HEAD"), original);
        assert_eq!(repository.git(["rev-parse", "HEAD^{tree}"]), original_tree);
    } else {
        assert_ne!(object(&repository, "HEAD"), before);
        let count = if action == SequenceControl::Continue {
            3
        } else {
            2
        };
        assert_eq!(
            repository.git(["rev-list", "--count", &format!("{}..HEAD", original.hex())]),
            format!("{count}\n").as_bytes()
        );
        assert_eq!(
            fs::read(repository.root.join("file")).unwrap(),
            if action == SequenceControl::Continue {
                b"resolved\n".as_slice()
            } else {
                b"local\n"
            }
        );
        assert_eq!(
            repository.root.join("first").exists(),
            kind == ApplyKind::CherryPick
        );
        assert_eq!(
            repository.root.join("last").exists(),
            kind == ApplyKind::CherryPick
        );
        let middle = if action == SequenceControl::Continue {
            object(&repository, "HEAD~1")
        } else {
            before
        };
        assert_eq!(
            repository.git(["show", "--no-patch", "--format=%P", "HEAD"]),
            format!("{}\n", middle.hex()).as_bytes()
        );
    }
}

// A review binds exact Git metadata, refs and index/disk bytes. External
// completion and missing retained authority cannot be mistaken for permission.
#[test]
fn control_fences() {
    for changed in ["metadata", "target", "family", "index", "HEAD", "missing"] {
        let repository = Repository::new("sha1");
        let targets = conflict(&repository, ApplyKind::CherryPick);
        let mut owner = stopped(&repository, ApplyKind::CherryPick, &targets);
        let state = refresh(&mut owner);
        match changed {
            "family" => {
                fs::write(
                    repository.root.join(".git/REVERT_HEAD"),
                    format!("{}\n", targets[1].hex()),
                )
                .unwrap();
            }
            "target" => {
                fs::write(
                    repository.root.join(".git/CHERRY_PICK_HEAD"),
                    format!("{}\n", targets[0].hex()),
                )
                .unwrap();
            }
            "metadata" => {
                fs::write(repository.root.join(".git/MERGE_MSG"), "external message\n").unwrap();
            }
            "index" => {
                fs::write(repository.root.join("extra"), "extra\n").unwrap();
                repository.git(["add", "extra"]);
            }
            "HEAD" => {
                repository.git(["update-ref", "refs/heads/main", &targets[0].hex()]);
            }
            _ => {
                repository.git(["cherry-pick", "--abort"]);
            }
        }
        let before = object(&repository, "HEAD");
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let id = owner
            .control_sequence(state, SequenceControl::Abort, None)
            .unwrap();
        finish(&mut owner);
        assert_eq!(
            owner.record(id).unwrap().effects,
            Effects::NotStarted,
            "{changed}: {:?}",
            owner.record(id)
        );
        assert_eq!(object(&repository, "HEAD"), before);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
    }
    let repository = Repository::new("sha256");
    let targets = conflict(&repository, ApplyKind::Revert);
    let output = std::process::Command::new(&repository.program)
        .args(["revert", "--no-edit", &targets[1].hex()])
        .current_dir(&repository.root)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let state = refresh(&mut owner);
    assert_eq!(state.family, Some(SequenceFamily::Revert));
    assert_eq!(state.current, Some(targets[1]));
    assert!(
        owner
            .control_sequence(state, SequenceControl::Abort, None)
            .unwrap_err()
            .contains("lineage")
    );
}

// A second real conflict binds the same confirmed request at its next exact
// target after either continuation or skip. Final controls cannot replay a prefix.
#[test]
fn next_conflict() {
    for first_action in [SequenceControl::Continue, SequenceControl::Skip] {
        let repository = Repository::new("sha256");
        let mut targets = conflict(&repository, ApplyKind::CherryPick);
        repository.git(["switch", "topic"]);
        fs::write(repository.root.join("file"), "later topic\n").unwrap();
        repository.commit();
        targets[2] = object(&repository, "HEAD");
        repository.git(["switch", "main"]);
        let original = object(&repository, "HEAD");
        let mut owner = stopped(&repository, ApplyKind::CherryPick, &targets);
        if first_action == SequenceControl::Continue {
            fs::write(repository.root.join("file"), "first resolution\n").unwrap();
            repository.git(["add", "file"]);
        }
        let state = refresh(&mut owner);
        let id = owner
            .control_sequence(state, first_action, Some(SequenceFamily::CherryPick))
            .unwrap();
        assert!(finish(&mut owner).published, "{:?}", owner.record(id));
        assert_eq!(
            owner.record(id).unwrap().outcome,
            Outcome::InProgress,
            "{:?}",
            owner.record(id)
        );
        let state = refresh(&mut owner);
        assert_eq!(state.original, Some(original));
        assert_eq!(state.current, Some(targets[2]));
        assert_eq!(state.step, Some(3));
        let progress = state.application.as_ref().unwrap();
        assert_eq!(progress.next, 2);
        assert_eq!(
            progress.completed.len(),
            if first_action == SequenceControl::Continue {
                2
            } else {
                1
            }
        );
        let before = object(&repository, "HEAD");
        let final_action = if first_action == SequenceControl::Continue {
            SequenceControl::Skip
        } else {
            SequenceControl::Continue
        };
        if final_action == SequenceControl::Continue {
            fs::write(repository.root.join("file"), "final resolution\n").unwrap();
            repository.git(["add", "file"]);
        }
        let state = refresh(&mut owner);
        let id = owner.control_sequence(state, final_action, None).unwrap();
        assert!(finish(&mut owner).published);
        let record = owner.record(id).unwrap();
        assert_eq!(record.effects, Effects::Verified, "{record:?}");
        if final_action == SequenceControl::Skip {
            assert_eq!(record.outcome, Outcome::Skipped, "{record:?}");
            assert_eq!(object(&repository, "HEAD"), before);
            assert_eq!(
                fs::read(repository.root.join("file")).unwrap(),
                b"first resolution\n"
            );
        } else {
            assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
            assert_eq!(
                repository.git(["show", "--no-patch", "--format=%P", "HEAD"]),
                format!("{}\n", before.hex()).as_bytes()
            );
            assert_eq!(
                fs::read(repository.root.join("file")).unwrap(),
                b"final resolution\n"
            );
        }
        assert_eq!(
            repository.git(["rev-list", "--count", &format!("{}..HEAD", original.hex())]),
            b"2\n"
        );
        assert!(
            owner.contexts()[0]
                .sequence
                .as_ref()
                .unwrap()
                .family
                .is_none()
        );
    }
}

// A real Git hook can leave effects while preventing safe state observation.
// The record must retain the changing argv and classify unverified effects.
#[test]
fn control_uncertainty() {
    use std::os::unix::fs::PermissionsExt;
    let repository = Repository::new("sha1");
    let targets = conflict(&repository, ApplyKind::CherryPick);
    let mut owner = stopped(&repository, ApplyKind::CherryPick, &targets[..2]);
    fs::write(repository.root.join("file"), "resolved\n").unwrap();
    repository.git(["add", "file"]);
    let state = refresh(&mut owner);
    let before = object(&repository, "HEAD");
    let hooks = repository.fixture.0.join("hooks");
    fs::create_dir(&hooks).unwrap();
    let hook = hooks.join("post-commit");
    fs::write(&hook, "#!/bin/sh\nln -s /dev/null .git/rebase-merge\n").unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
    repository.git([
        std::ffi::OsStr::new("config"),
        std::ffi::OsStr::new("core.hooksPath"),
        hooks.as_os_str(),
    ]);
    let id = owner
        .control_sequence(state, SequenceControl::Continue, None)
        .unwrap();
    let update = finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert!(!update.published, "{record:?}");
    assert_eq!(record.effects, Effects::Possible, "{record:?}");
    assert_eq!(record.outcome, Outcome::Indeterminate, "{record:?}");
    assert!(record.argv.iter().any(|s| s == "--continue"), "{record:?}");
    assert_ne!(object(&repository, "HEAD"), before);
    assert_eq!(
        repository.git(["show", "--no-patch", "--format=%P", "HEAD"]),
        format!("{}\n", before.hex()).as_bytes()
    );
    assert_eq!(
        fs::read(repository.root.join("file")).unwrap(),
        b"resolved\n"
    );
}
