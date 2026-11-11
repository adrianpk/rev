use super::support::{Repository, finish};
use rev::git::{Effects, IntegrationMode, Outcome, Refs, Repositories, SequenceFamily};
use std::{fs, sync::Arc};

fn observe(owner: &mut Repositories) -> Arc<Refs> {
    let context = owner.contexts()[0].id;
    owner.refs(context).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(update.operation));
    Arc::clone(owner.context(context).unwrap().refs.as_ref().unwrap())
}
fn topic(repository: &Repository, conflict: bool) {
    fs::write(repository.root.join("file"), "base\n").unwrap();
    repository.commit();
    repository.git(["switch", "--create", "topic"]);
    fs::write(repository.root.join("file"), "topic\n").unwrap();
    repository.commit();
    repository.git(["switch", "main"]);
    if conflict {
        fs::write(repository.root.join("file"), "main\n").unwrap();
        repository.commit();
    }
}

// Explicit policies retain exact ancestry, distinguishing a fast-forward from
// a required merge commit with two full parents in both supported ID formats.
#[test]
fn merge_policies() {
    for (format, mode) in [
        ("sha1", IntegrationMode::FastForward),
        ("sha1", IntegrationMode::Merge),
        ("sha256", IntegrationMode::NoFastForward),
    ] {
        let repository = Repository::new(format);
        topic(&repository, false);
        let before = repository.git(["rev-parse", "HEAD"]);
        let target = repository.git(["rev-parse", "topic"]);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let refs = observe(&mut owner);
        let id = owner.integrate(refs, b"refs/heads/topic", mode).unwrap();
        let update = finish(&mut owner);
        let record = owner.record(id).unwrap();
        assert!(update.published, "{record:?}");
        assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
        assert_eq!(record.effects, Effects::Verified);
        assert_eq!(fs::read(repository.root.join("file")).unwrap(), b"topic\n");
        if mode == IntegrationMode::NoFastForward {
            assert_eq!(
                repository.git(["show", "--no-patch", "--format=%P", "HEAD"]),
                format!(
                    "{} {}\n",
                    String::from_utf8_lossy(&before).trim(),
                    String::from_utf8_lossy(&target).trim()
                )
                .as_bytes()
            );
        } else {
            assert_eq!(repository.git(["rev-parse", "HEAD"]), target);
        }
        assert!(
            owner
                .context(update.context.unwrap())
                .unwrap()
                .sequence
                .as_ref()
                .unwrap()
                .family
                .is_none()
        );
    }
}

// Ordinary rebase replays the checked branch onto the selected exact target,
// preserving authored content and each side's independent file changes.
#[test]
fn ordinary_rebase() {
    let repository = Repository::new("sha256");
    topic(&repository, false);
    fs::write(repository.root.join("local"), "local content\n").unwrap();
    repository.commit();
    let before = repository.git(["rev-parse", "HEAD"]);
    let target = repository.git(["rev-parse", "topic"]);
    let author = repository.git(["show", "--no-patch", "--format=%an <%ae>%n%B", "HEAD"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&mut owner);
    let id = owner
        .integrate(refs, b"refs/heads/topic", IntegrationMode::Rebase)
        .unwrap();
    finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
    assert_ne!(repository.git(["rev-parse", "HEAD"]), before);
    assert_eq!(
        repository.git(["show", "--no-patch", "--format=%P", "HEAD"]),
        target
    );
    assert_eq!(
        repository.git(["show", "--no-patch", "--format=%an <%ae>%n%B", "HEAD"]),
        author
    );
    assert_eq!(fs::read(repository.root.join("file")).unwrap(), b"topic\n");
    assert_eq!(
        fs::read(repository.root.join("local")).unwrap(),
        b"local content\n"
    );
    assert_eq!(
        repository.git(["symbolic-ref", "HEAD"]),
        b"refs/heads/main\n"
    );
}

// Real conflict stops publish exact family, lineage and paths with a distinct
// in-progress result rather than treating a failed child as completed work.
#[test]
fn conflict_stops() {
    for mode in [IntegrationMode::Merge, IntegrationMode::Rebase] {
        let repository = Repository::new("sha1");
        topic(&repository, true);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let refs = observe(&mut owner);
        let original = refs.head;
        let id = owner.integrate(refs, b"refs/heads/topic", mode).unwrap();
        let update = finish(&mut owner);
        let record = owner.record(id).unwrap();
        assert!(update.published, "{record:?}");
        assert_eq!(record.outcome, Outcome::InProgress, "{record:?}");
        assert!(!record.exit.unwrap().success());
        let sequence = owner
            .context(update.context.unwrap())
            .unwrap()
            .sequence
            .as_ref()
            .unwrap();
        assert_eq!(
            sequence.family,
            Some(if mode == IntegrationMode::Merge {
                SequenceFamily::Merge
            } else {
                SequenceFamily::Rebase
            })
        );
        assert_eq!(sequence.original, original);
        assert_eq!(sequence.conflicts, vec![std::path::PathBuf::from("file")]);
        if mode == IntegrationMode::Rebase {
            assert_eq!(sequence.current, original);
            assert_eq!(sequence.step, Some(1));
            assert_eq!(sequence.total, Some(1));
        }
        let status = owner.sequence(update.context.unwrap()).unwrap();
        assert!(finish(&mut owner).published);
        assert_eq!(owner.record(status).unwrap().outcome, Outcome::Verified);
    }
}

// Stale HEAD, dirty disk and autostash policy cannot start changing work.
// Fast-forward-only integration retains Git's actual divergence rejection.
#[test]
fn integration_guards() {
    let repository = Repository::new("sha1");
    topic(&repository, true);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&mut owner);
    let id = owner
        .integrate(refs, b"refs/heads/topic", IntegrationMode::FastForward)
        .unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().outcome, Outcome::Failed);
    let refs = observe(&mut owner);
    repository.commit();
    let id = owner
        .integrate(refs, b"refs/heads/topic", IntegrationMode::Merge)
        .unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    let refs = observe(&mut owner);
    fs::write(repository.root.join("dirty"), "do not overwrite\n").unwrap();
    let id = owner
        .integrate(refs, b"refs/heads/topic", IntegrationMode::Merge)
        .unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    fs::remove_file(repository.root.join("dirty")).unwrap();
    repository.git(["config", "rebase.autostash", "true"]);
    let refs = observe(&mut owner);
    let before = repository.git(["rev-parse", "HEAD"]);
    let id = owner
        .integrate(refs, b"refs/heads/topic", IntegrationMode::Rebase)
        .unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert_eq!(repository.git(["stash", "list"]), b"");
    assert_eq!(repository.git(["rev-parse", "HEAD"]), before);
}

#[path = "integration_presentation.rs"]
mod presentation;

fn sequence(owner: &mut Repositories) -> Arc<rev::git::Sequence> {
    let context = owner.contexts()[0].id;
    owner.sequence(context).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(update.operation));
    owner.context(context).unwrap().sequence.clone().unwrap()
}
fn stopped(repository: &Repository, mode: IntegrationMode) -> Repositories {
    topic(repository, true);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&mut owner);
    let id = owner.integrate(refs, b"refs/heads/topic", mode).unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().outcome, Outcome::InProgress);
    owner
}

// Checked family controls complete staged resolutions, explicitly discard a
// conflicted rebase step, or restore the original context on a real abort.
#[test]
fn sequence_controls() {
    use rev::git::SequenceControl;
    for (mode, action) in [
        (IntegrationMode::Merge, SequenceControl::Continue),
        (IntegrationMode::Merge, SequenceControl::Abort),
        (IntegrationMode::Rebase, SequenceControl::Continue),
        (IntegrationMode::Rebase, SequenceControl::Skip),
        (IntegrationMode::Rebase, SequenceControl::Abort),
    ] {
        let repository = Repository::new("sha256");
        let mut owner = stopped(&repository, mode);
        let original = repository.git(["rev-parse", "main"]);
        if action == SequenceControl::Continue {
            fs::write(repository.root.join("file"), "resolved\n").unwrap();
            repository.git(["add", "--", "file"]);
        }
        let state = sequence(&mut owner);
        let id = owner
            .control_sequence(Arc::clone(&state), action, state.family)
            .unwrap();
        let update = finish(&mut owner);
        let record = owner.record(id).unwrap();
        assert!(update.published, "{record:?}");
        assert_eq!(
            record.outcome,
            match action {
                SequenceControl::Continue => Outcome::Verified,
                SequenceControl::Skip => Outcome::Skipped,
                SequenceControl::Abort => Outcome::Aborted,
            },
            "{record:?}"
        );
        assert_eq!(record.effects, Effects::Verified, "{record:?}");
        assert!(
            owner
                .context(update.context.unwrap())
                .unwrap()
                .sequence
                .as_ref()
                .unwrap()
                .family
                .is_none()
        );
        assert_eq!(
            repository.git(["symbolic-ref", "HEAD"]),
            b"refs/heads/main\n"
        );
        if action == SequenceControl::Abort {
            assert_eq!(repository.git(["rev-parse", "HEAD"]), original);
        }
        if action == SequenceControl::Continue {
            assert_eq!(
                fs::read(repository.root.join("file")).unwrap(),
                b"resolved\n"
            );
        }
        if action == SequenceControl::Skip {
            assert_eq!(
                repository.git(["rev-parse", "HEAD"]),
                repository.git(["rev-parse", "topic"])
            );
        }
    }
}

fn autosquash_source(repository: &Repository) {
    fs::write(repository.root.join("base"), "base\n").unwrap();
    repository.commit();
    repository.git(["switch", "--create", "topic"]);
    fs::write(repository.root.join("remote"), "remote\n").unwrap();
    repository.commit();
    repository.git(["switch", "main"]);
    for (file, content, message) in [
        ("first", "first\n", "First subject"),
        ("second", "second\n", "Second subject"),
        ("first", "fixed\n", "fixup! First subject"),
        ("second", "squashed\n", "squash! Second subject"),
    ] {
        fs::write(repository.root.join(file), content).unwrap();
        repository.git(["add", "--all"]);
        repository.git(["commit", "-m", message]);
    }
}

// Compare actual trees, ordered messages and authors against Git's own
// autosquash sequencer; plan provenance alone cannot prove those semantics.
#[test]
fn reviewed_autosquash() {
    use std::process::Command;
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        autosquash_source(&repository);
        fs::write(repository.root.join("first"), "amended\n").unwrap();
        repository.git(["add", "--", "first"]);
        repository.git([
            "commit",
            "-m",
            "amend! First subject",
            "-m",
            "Replacement subject\n\nReplacement body",
        ]);
        repository.git(["branch", "reference"]);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let refs = observe(&mut owner);
        let id = owner.prepare_autosquash(refs, b"refs/heads/topic").unwrap();
        finish(&mut owner);
        assert_eq!(owner.record(id).unwrap().outcome, Outcome::Verified);
        let plan = owner.contexts()[0].autosquash.clone().unwrap();
        assert_eq!(
            plan.steps
                .iter()
                .map(|s| s.action.label())
                .collect::<Vec<_>>(),
            ["pick", "fixup", "fixup -C", "pick", "squash"]
        );
        assert!(!repository.root.join(".git/rebase-merge").exists());
        let id = owner.autosquash(plan).unwrap();
        let update = finish(&mut owner);
        let record = owner.record(id).unwrap();
        assert!(update.published, "{record:?}");
        assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
        let tree = repository.git(["rev-parse", "HEAD^{tree}"]);
        let history = repository.git([
            "log",
            "--reverse",
            "--format=%s%n%b%an <%ae>",
            "topic..HEAD",
        ]);
        repository.git(["switch", "reference"]);
        let output = Command::new(&repository.program)
            .args([
                "rebase",
                "--interactive",
                "--autosquash",
                "--no-autostash",
                "--no-update-refs",
                "--no-rebase-merges",
                "--no-fork-point",
                "topic",
            ])
            .current_dir(&repository.root)
            .env("GIT_SEQUENCE_EDITOR", "/bin/true")
            .env("GIT_EDITOR", "/bin/true")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(repository.git(["rev-parse", "HEAD^{tree}"]), tree);
        assert_eq!(
            repository.git([
                "log",
                "--reverse",
                "--format=%s%n%b%an <%ae>",
                "topic..HEAD"
            ]),
            history
        );
        assert_eq!(
            fs::read(repository.root.join("first")).unwrap(),
            b"amended\n"
        );
        assert_eq!(
            fs::read(repository.root.join("second")).unwrap(),
            b"squashed\n"
        );
    }
}

#[path = "integration_guards.rs"]
mod guards;
