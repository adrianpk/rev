use super::*;
use rev::git::{Resolution, ResolutionAction};
use std::os::unix::fs::PermissionsExt;

fn prepare(
    owner: &mut Repositories,
    conflict: Arc<Conflict>,
    action: ResolutionAction,
) -> Arc<Resolution> {
    let context = conflict.sequence.refs.observation.context;
    let id = owner.prepare_resolution(conflict, action).unwrap();
    assert!(finish(owner).published, "{:?}", owner.record(id));
    owner.context(context).unwrap().resolution.clone().unwrap()
}
fn apply(owner: &mut Repositories, plan: Arc<Resolution>) {
    let id = owner.resolve_conflict(plan).unwrap();
    assert!(finish(owner).published, "{:?}", owner.record(id));
    assert_eq!(
        owner.record(id).unwrap().outcome,
        Outcome::Verified,
        "{:?}",
        owner.record(id)
    );
    assert_eq!(owner.record(id).unwrap().effects, Effects::Verified);
}
fn capture(repository: &Repository, owner: &mut Repositories, path: &Path) -> Arc<Conflict> {
    repository.open(owner, &repository.root);
    let sequence = observe(owner);
    inspect(owner, sequence, path)
}

// Each complete side changes only the working file; separately confirmed
// staging proves the real stage-zero object without advancing merge state.
#[test]
fn sides_then_stage() {
    for format in ["sha1", "sha256"] {
        for role in ConflictRole::ALL {
            let repository = Repository::new(format);
            topic(
                &repository,
                Path::new("file"),
                b"base\n",
                b"current\n",
                b"applied\n",
            );
            stopped(&repository, &["merge", "topic"]);
            let index = repository.git(["ls-files", "--stage", "-z"]);
            let head = repository.git(["rev-parse", "HEAD"]);
            let mut owner = repository.owner("");
            let conflict = capture(&repository, &mut owner, Path::new("file"));
            let chosen = conflict.side(role).unwrap().bytes.clone();
            let plan = prepare(&mut owner, conflict, ResolutionAction::Use(role));
            assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
            apply(&mut owner, plan);
            assert_eq!(fs::read(repository.root.join("file")).unwrap(), chosen);
            assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
            assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
            let sequence = owner.contexts()[0].sequence.clone().unwrap();
            let conflict = inspect(&mut owner, sequence, Path::new("file"));
            let plan = prepare(&mut owner, conflict, ResolutionAction::Stage);
            let object = plan.expected_object.unwrap();
            apply(&mut owner, plan);
            assert!(repository.git(["ls-files", "--unmerged", "-z"]).is_empty());
            assert_eq!(repository.git(["show", ":file"]), chosen);
            assert_eq!(repository.git(["cat-file", "blob", &object.hex()]), chosen);
            assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
            assert!(repository.root.join(".git/MERGE_HEAD").exists());
        }
    }
}

// Raw path bytes select only one saved text or exact deletion, including a
// manually saved absence. Other index entries and HEAD remain unchanged.
#[test]
fn saved_and_deleted() {
    for action in [ResolutionAction::Stage, ResolutionAction::Delete] {
        let repository = Repository::new("sha256");
        let path = std::path::PathBuf::from(OsString::from_vec(b"-file\n\xff".to_vec()));
        topic(&repository, &path, b"base\n", b"current\n", b"applied\n");
        fs::write(repository.root.join("keep"), "unchanged\n").unwrap();
        repository.commit();
        stopped(&repository, &["merge", "topic"]);
        let keep = repository.git(["rev-parse", ":keep"]);
        if action == ResolutionAction::Stage {
            fs::write(repository.root.join(&path), "manual saved resolution\n").unwrap();
        }
        let mut owner = repository.owner("");
        let conflict = capture(&repository, &mut owner, &path);
        let plan = prepare(&mut owner, conflict, action);
        apply(&mut owner, plan);
        if action == ResolutionAction::Stage {
            assert_eq!(
                fs::read(repository.root.join(&path)).unwrap(),
                b"manual saved resolution\n"
            );
        } else {
            assert!(!repository.root.join(&path).exists());
        }
        assert_eq!(repository.git(["rev-parse", ":keep"]), keep);
        assert!(repository.git(["ls-files", "--unmerged", "-z"]).is_empty());
    }
    let repository = Repository::new("sha1");
    topic(
        &repository,
        Path::new("file"),
        b"base\n",
        b"current\n",
        b"applied\n",
    );
    stopped(&repository, &["merge", "topic"]);
    fs::remove_file(repository.root.join("file")).unwrap();
    let mut owner = repository.owner("");
    let conflict = capture(&repository, &mut owner, Path::new("file"));
    let plan = prepare(&mut owner, conflict, ResolutionAction::Stage);
    assert!(plan.saved.is_none());
    apply(&mut owner, plan);
    assert!(repository.git(["ls-files", "--stage", "-z"]).is_empty());
}

// Review binds bytes and file identity as well as full index/sequence/HEAD
// facts. Stale confirmation never launches the mutating Git command.
#[test]
fn changed_review() {
    for changed in ["bytes", "inode", "stages", "head", "metadata"] {
        let repository = Repository::new("sha1");
        topic(
            &repository,
            Path::new("file"),
            b"base\n",
            b"current\n",
            b"applied\n",
        );
        stopped(&repository, &["merge", "topic"]);
        let mut owner = repository.owner("");
        let conflict = capture(&repository, &mut owner, Path::new("file"));
        let plan = prepare(&mut owner, conflict, ResolutionAction::Stage);
        let bytes = fs::read(repository.root.join("file")).unwrap();
        match changed {
            "bytes" => fs::write(repository.root.join("file"), "changed\n").unwrap(),
            "inode" => {
                fs::rename(repository.root.join("file"), repository.root.join("old")).unwrap();
                fs::write(repository.root.join("file"), bytes).unwrap();
                fs::remove_file(repository.root.join("old")).unwrap();
            }
            "stages" => {
                repository.git(["add", "--", "file"]);
            }
            "head" => {
                repository.git(["update-ref", "refs/heads/main", "topic"]);
            }
            _ => {
                fs::write(repository.root.join(".git/MERGE_MODE"), "changed").unwrap();
            }
        }
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let id = owner.resolve_conflict(plan).unwrap();
        assert!(!finish(&mut owner).published);
        let record = owner.record(id).unwrap();
        assert_eq!(record.effects, Effects::NotStarted);
        assert_eq!(record.outcome, Outcome::Failed);
        if changed == "inode" {
            assert!(
                record
                    .diagnostic
                    .as_ref()
                    .unwrap()
                    .contains("file identity"),
                "{record:?}"
            );
        }
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
    }
}

// An actual post-index-change hook changes the saved result after Git stages
// it. The owner retains the changing argv and reports possible effects.
#[test]
fn effects_remain_visible() {
    let repository = Repository::new("sha1");
    topic(
        &repository,
        Path::new("file"),
        b"base\n",
        b"current\n",
        b"applied\n",
    );
    stopped(&repository, &["merge", "topic"]);
    fs::write(repository.root.join("file"), "saved\n").unwrap();
    let hooks = repository.fixture.0.join("hooks");
    fs::create_dir(&hooks).unwrap();
    let hook = hooks.join("post-index-change");
    fs::write(&hook, "#!/bin/sh\nprintf 'HOOK_CHANGED\\n' > file\n").unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
    repository.git([
        std::ffi::OsStr::new("config"),
        std::ffi::OsStr::new("core.hooksPath"),
        hooks.as_os_str(),
    ]);
    let mut owner = repository.owner("");
    let conflict = capture(&repository, &mut owner, Path::new("file"));
    let plan = prepare(&mut owner, conflict, ResolutionAction::Stage);
    let id = owner.resolve_conflict(plan).unwrap();
    finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert_eq!(record.effects, Effects::Possible, "{record:?}");
    assert_eq!(record.outcome, Outcome::Indeterminate, "{record:?}");
    assert!(record.argv.iter().any(|a| a == "add"));
    assert!(
        record
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("postconditions")
    );
    assert_eq!(repository.git(["show", ":file"]), b"saved\n");
    assert_eq!(
        fs::read(repository.root.join("file")).unwrap(),
        b"HOOK_CHANGED\n"
    );
}

#[path = "conflicts_resolution_presentation.rs"]
mod presentation;

// Git checkout/clean conversion remains authoritative. Complete selected bytes
// are verified on disk and the separately staged object retains normalized LF.
#[test]
fn converted_text() {
    let repository = Repository::new("sha256");
    topic(
        &repository,
        Path::new("file"),
        b"base\n",
        b"current\n",
        b"applied\n",
    );
    stopped(&repository, &["merge", "topic"]);
    repository.git(["config", "core.autocrlf", "true"]);
    let mut owner = repository.owner("");
    let conflict = capture(&repository, &mut owner, Path::new("file"));
    let plan = prepare(
        &mut owner,
        conflict,
        ResolutionAction::Use(ConflictRole::Applied),
    );
    assert_eq!(plan.expected_bytes, b"applied\r\n");
    apply(&mut owner, plan);
    assert_eq!(
        fs::read(repository.root.join("file")).unwrap(),
        b"applied\r\n"
    );
    let sequence = owner.contexts()[0].sequence.clone().unwrap();
    let conflict = inspect(&mut owner, sequence, Path::new("file"));
    let plan = prepare(&mut owner, conflict, ResolutionAction::Stage);
    apply(&mut owner, plan);
    assert_eq!(repository.git(["show", ":file"]), b"applied\n");
    assert_eq!(
        fs::read(repository.root.join("file")).unwrap(),
        b"applied\r\n"
    );
}

// Resolution keeps native ordered-application lineage alive. The separately
// confirmed family control can then finish the actual stopped revision.
#[test]
fn ordered_continuation() {
    use rev::git::{ApplyKind, ObjectId, RevisionChoice, SequenceControl};
    for kind in [ApplyKind::CherryPick, ApplyKind::Revert] {
        let repository = Repository::new("sha1");
        topic(
            &repository,
            Path::new("file"),
            b"base\n",
            b"current\n",
            b"applied\n",
        );
        if kind == ApplyKind::Revert {
            repository.git(["reset", "--hard", "topic"]);
            fs::write(repository.root.join("file"), "current\n").unwrap();
            repository.commit();
        }
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let context = owner.contexts()[0].id;
        let format = owner.contexts()[0].observation.identity.format;
        let target = repository.git(["rev-parse", "topic"]);
        let target = ObjectId::parse(target.strip_suffix(b"\n").unwrap(), format).unwrap();
        let id = owner.log(context, Some(&target.hex())).unwrap();
        assert!(finish(&mut owner).published, "{:?}", owner.record(id));
        let history = owner.context(context).unwrap().history.clone().unwrap();
        let id = owner
            .prepare_sequence(
                history,
                kind,
                &[RevisionChoice {
                    object: target,
                    mainline: None,
                }],
            )
            .unwrap();
        assert!(finish(&mut owner).published, "{:?}", owner.record(id));
        let application = owner
            .context(context)
            .unwrap()
            .application_plan
            .clone()
            .unwrap();
        let id = owner.apply_sequence(application).unwrap();
        assert!(finish(&mut owner).published);
        assert_eq!(owner.record(id).unwrap().outcome, Outcome::InProgress);
        let before = repository.git(["rev-parse", "HEAD"]);
        let sequence = owner.context(context).unwrap().sequence.clone().unwrap();
        let lineage = sequence.application.clone().unwrap();
        let conflict = inspect(&mut owner, sequence, Path::new("file"));
        let plan = prepare(
            &mut owner,
            conflict,
            ResolutionAction::Use(ConflictRole::Applied),
        );
        apply(&mut owner, plan);
        let sequence = owner.context(context).unwrap().sequence.clone().unwrap();
        let conflict = inspect(&mut owner, sequence, Path::new("file"));
        let plan = prepare(&mut owner, conflict, ResolutionAction::Stage);
        apply(&mut owner, plan);
        let sequence = owner.context(context).unwrap().sequence.clone().unwrap();
        assert!(Arc::ptr_eq(
            sequence.application.as_ref().unwrap(),
            &lineage
        ));
        assert_eq!(repository.git(["rev-parse", "HEAD"]), before);
        let id = owner
            .control_sequence(sequence, SequenceControl::Continue, Some(kind.family()))
            .unwrap();
        assert!(finish(&mut owner).published, "{:?}", owner.record(id));
        assert_eq!(
            owner.record(id).unwrap().outcome,
            Outcome::Verified,
            "{:?}",
            owner.record(id)
        );
        assert_ne!(repository.git(["rev-parse", "HEAD"]), before);
        assert_eq!(
            owner
                .context(context)
                .unwrap()
                .sequence
                .as_ref()
                .unwrap()
                .family,
            None
        );
    }
}

// External unmerged index state has no sequence marker to refresh its index
// implicitly. Saved staging still verifies the full actual stage-zero index.
#[test]
fn external_resolution() {
    let repository = Repository::new("sha256");
    topic(
        &repository,
        Path::new("file"),
        b"base\n",
        b"current\n",
        b"applied\n",
    );
    repository.git(["read-tree", "-m", "main^", "main", "topic"]);
    fs::write(repository.root.join("file"), "external saved resolution\n").unwrap();
    let mut owner = repository.owner("");
    let conflict = capture(&repository, &mut owner, Path::new("file"));
    assert_eq!(conflict.sequence.family, None);
    let plan = prepare(&mut owner, conflict, ResolutionAction::Stage);
    apply(&mut owner, plan);
    assert_eq!(
        repository.git(["show", ":file"]),
        b"external saved resolution\n"
    );
    assert!(repository.git(["ls-files", "--unmerged", "-z"]).is_empty());
}
