use super::*;
use rev::git::SequenceControl;
use std::process::Command;

// A normal commit must not implicitly finish an active merge without the
// checked sequence family's fresh fingerprint and explicit continuation.
#[test]
fn commit_sequence_guard() {
    let repository = Repository::new("sha1");
    let mut owner = stopped(&repository, IntegrationMode::Merge);
    fs::write(repository.root.join("file"), "resolved\n").unwrap();
    repository.git(["add", "--", "file"]);
    repository.open(&mut owner, &repository.root);
    let before = repository.git(["rev-parse", "HEAD"]);
    let observation = owner.commit_observation(owner.contexts()[0].id).unwrap();
    let id = owner
        .commit(observation, "Unreviewed merge bypass\n")
        .unwrap();
    finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert_eq!(record.effects, Effects::NotStarted, "{record:?}");
    assert_eq!(repository.git(["rev-parse", "HEAD"]), before);
    assert!(repository.root.join(".git/MERGE_HEAD").exists());
}

// Installed Git really receives a changed HEAD immediately before rebase. Its
// recorded original object must bind success to the reviewed source, while
// actual resulting content remains visible rather than being rolled back.
#[test]
fn source_binding() {
    let repository = Repository::new("sha1");
    topic(&repository, false);
    fs::write(repository.root.join("local"), "local\n").unwrap();
    repository.commit();
    let before = String::from_utf8(repository.git(["rev-parse", "HEAD"])).unwrap();
    fs::write(repository.root.join("unexpected"), "concurrent source\n").unwrap();
    repository.commit();
    repository.git(["branch", "raced"]);
    repository.git(["reset", "--hard", before.trim()]);
    let marker = repository.fixture.0.join("raced");
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&mut owner);
    fs::write(&repository.program,format!("#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nif [ \"$1\" = --no-optional-locks ] && [ \"$2\" = rebase ] && [ ! -e '{}' ]; then\n /usr/bin/git reset --hard refs/heads/raced >/dev/null || exit 90\n : > '{}'\nfi\nexec /usr/bin/git \"$@\"\n",marker.display(),marker.display())).unwrap();
    let id = owner
        .integrate(refs, b"refs/heads/topic", IntegrationMode::Rebase)
        .unwrap();
    finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert!(marker.exists());
    assert_eq!(record.outcome, Outcome::Indeterminate, "{record:?}");
    assert_eq!(record.effects, Effects::Possible);
    assert_eq!(
        fs::read(repository.root.join("unexpected")).unwrap(),
        b"concurrent source\n"
    );
    assert_eq!(
        repository.git(["symbolic-ref", "HEAD"]),
        b"refs/heads/main\n"
    );
}

// Exact sequence state, not a cached label, controls admission. Independent
// metadata, index, worktree and HEAD changes all reject before effects.
#[test]
fn stale_sequence() {
    for changed in ["metadata", "index", "worktree", "HEAD", "missing"] {
        let repository = Repository::new("sha1");
        let mut owner = stopped(&repository, IntegrationMode::Merge);
        let state = sequence(&mut owner);
        assert!(
            owner
                .control_sequence(
                    state.clone(),
                    SequenceControl::Abort,
                    Some(SequenceFamily::Rebase)
                )
                .is_err()
        );
        assert!(
            owner
                .control_sequence(state.clone(), SequenceControl::Continue, None)
                .is_err()
        );
        match changed {
            "metadata" => {
                fs::write(repository.root.join(".git/MERGE_MSG"), "changed message\n").unwrap()
            }
            "index" => {
                fs::write(repository.root.join("file"), "resolved\n").unwrap();
                repository.git(["add", "--", "file"]);
            }
            "worktree" => fs::write(repository.root.join("file"), "unstaged resolution\n").unwrap(),
            "HEAD" => {
                repository.git(["update-ref", "refs/heads/main", "refs/heads/topic"]);
            }
            "missing" => {
                repository.git(["merge", "--abort"]);
            }
            _ => unreachable!(),
        }
        let head = repository.git(["rev-parse", "HEAD"]);
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let disk = fs::read(repository.root.join("file")).unwrap();
        let id = owner
            .control_sequence(state, SequenceControl::Abort, None)
            .unwrap();
        finish(&mut owner);
        let record = owner.record(id).unwrap();
        assert_eq!(record.effects, Effects::NotStarted, "{changed}: {record:?}");
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head, "{changed}");
        assert_eq!(
            repository.git(["ls-files", "--stage", "-z"]),
            index,
            "{changed}"
        );
        assert_eq!(
            fs::read(repository.root.join("file")).unwrap(),
            disk,
            "{changed}"
        );
    }
}

// Missing lineage, an empty external directory and executable future todo
// content remain observable but cannot construct native mutation authority.
#[test]
fn incompatible_sequence() {
    for changed in ["original", "branch", "onto", "todo", "empty", "autostash"] {
        let repository = Repository::new("sha256");
        let mut owner = stopped(&repository, IntegrationMode::Rebase);
        let directory = repository.root.join(".git/rebase-merge");
        match changed {
            "original" => fs::remove_file(directory.join("orig-head")).unwrap(),
            "branch" => fs::remove_file(directory.join("head-name")).unwrap(),
            "onto" => fs::remove_file(directory.join("onto")).unwrap(),
            "todo" => {
                fs::write(directory.join("git-rebase-todo"), "exec touch EXECUTED\n").unwrap()
            }
            "empty" => {
                fs::remove_dir_all(&directory).unwrap();
                fs::create_dir(&directory).unwrap();
            }
            "autostash" => {
                let head = repository.git(["rev-parse", "HEAD"]);
                fs::write(directory.join("autostash"), head).unwrap();
            }
            _ => unreachable!(),
        }
        let state = sequence(&mut owner);
        assert_eq!(state.family, Some(SequenceFamily::Rebase), "{changed}");
        let head = repository.git(["rev-parse", "HEAD"]);
        let admitted = owner.control_sequence(state, SequenceControl::Abort, None);
        assert!(admitted.is_err(), "{changed}: {admitted:?}");
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        assert!(!repository.root.join("EXECUTED").exists());
    }
    let repository = Repository::new("sha1");
    repository.commit();
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let state = sequence(&mut owner);
    assert!(
        owner
            .control_sequence(state, SequenceControl::Continue, None)
            .is_err()
    );
}

// The apply backend begins outside rev; its exact native original/onto/branch
// evidence admits continuation after a real saved and staged resolution.
#[test]
fn external_rebase() {
    let repository = Repository::new("sha1");
    topic(&repository, true);
    assert!(
        !Command::new(&repository.program)
            .args(["rebase", "--apply", "topic"])
            .current_dir(&repository.root)
            .output()
            .unwrap()
            .status
            .success()
    );
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let state = sequence(&mut owner);
    assert_eq!(state.family, Some(SequenceFamily::Rebase));
    assert_eq!(state.conflicts, [std::path::PathBuf::from("file")]);
    fs::write(repository.root.join("file"), "external resolution\n").unwrap();
    repository.git(["add", "--", "file"]);
    let state = sequence(&mut owner);
    let id = owner
        .control_sequence(state, SequenceControl::Continue, None)
        .unwrap();
    finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
    assert_eq!(
        repository.git(["symbolic-ref", "HEAD"]),
        b"refs/heads/main\n"
    );
    assert_eq!(
        fs::read(repository.root.join("file")).unwrap(),
        b"external resolution\n"
    );
}

// Hook failure and cancellation preserve the actual merge stop and original
// HEAD. Neither authorizes a hidden abort or bypasses configured hooks.
#[test]
fn control_failure() {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant};
    for cancelled in [false, true] {
        let repository = Repository::new("sha1");
        let mut owner = stopped(&repository, IntegrationMode::Merge);
        fs::write(repository.root.join("file"), "resolved\n").unwrap();
        repository.git(["add", "--", "file"]);
        let hooks = repository.fixture.0.join("hooks");
        fs::create_dir(&hooks).unwrap();
        let marker = hooks.join("entered");
        let hook = hooks.join("pre-commit");
        fs::write(
            &hook,
            if cancelled {
                "#!/bin/sh\n: > \"$(dirname \"$0\")/entered\"\nsleep 30\n"
            } else {
                "#!/bin/sh\necho HOOK_REJECTED >&2\nexit 7\n"
            },
        )
        .unwrap();
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
        repository.git(["config", "core.hooksPath", hooks.to_str().unwrap()]);
        let before = repository.git(["rev-parse", "HEAD"]);
        let state = sequence(&mut owner);
        let id = owner
            .control_sequence(state, SequenceControl::Continue, None)
            .unwrap();
        if cancelled {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !marker.exists() {
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(2));
            }
            owner.cancel(id).unwrap();
        }
        finish(&mut owner);
        let record = owner.record(id).unwrap();
        assert_eq!(
            record.outcome,
            if cancelled {
                Outcome::Cancelled
            } else {
                Outcome::InProgress
            },
            "{record:?}"
        );
        assert_eq!(record.effects, Effects::Possible);
        assert!(repository.root.join(".git/MERGE_HEAD").exists());
        assert_eq!(repository.git(["rev-parse", "HEAD"]), before);
        if !cancelled {
            assert!(String::from_utf8_lossy(&record.stderr).contains("HOOK_REJECTED"));
        }
        assert!(!owner.busy());
    }
}

// Prepared autosquash cannot retarget after review, and ambiguous subjects or
// a source range beyond the fixed target bound reject before starting rebase.
#[test]
fn autosquash_admission() {
    for changed in ["HEAD", "disk", "ambiguous", "limit"] {
        let repository = Repository::new("sha1");
        autosquash_source(&repository);
        if changed == "ambiguous" {
            for message in ["Duplicate", "Duplicate", "fixup! Duplicate"] {
                repository.git(["commit", "--allow-empty", "-m", message]);
            }
        } else if changed == "limit" {
            for _ in 0..253 {
                repository.git(["commit", "--allow-empty", "-m", "Extra"]);
            }
        }
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let refs = observe(&mut owner);
        let id = owner.prepare_autosquash(refs, b"refs/heads/topic").unwrap();
        finish(&mut owner);
        if matches!(changed, "ambiguous" | "limit") {
            let record = owner.record(id).unwrap();
            assert_eq!(record.effects, Effects::NotStarted, "{changed}: {record:?}");
            assert_eq!(record.outcome, Outcome::Failed, "{changed}: {record:?}");
        } else {
            let plan = owner.contexts()[0].autosquash.clone().unwrap();
            if changed == "HEAD" {
                repository.commit();
            } else {
                fs::write(repository.root.join("dirty"), "unsaved disk\n").unwrap();
            }
            let head = repository.git(["rev-parse", "HEAD"]);
            let id = owner.autosquash(plan).unwrap();
            finish(&mut owner);
            assert_eq!(
                owner.record(id).unwrap().effects,
                Effects::NotStarted,
                "{changed}"
            );
            assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        }
        assert!(!repository.root.join(".git/rebase-merge").exists());
    }
}
