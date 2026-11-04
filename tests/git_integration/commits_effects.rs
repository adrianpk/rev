use super::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

pub(super) fn hook(repository: &Repository, name: &str, text: &str) -> PathBuf {
    let directory = repository.fixture.0.join("hooks");
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join(name);
    fs::write(&path, text).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    repository.git(["config", "core.hooksPath", directory.to_str().unwrap()]);
    path
}

// A zero exit cannot substitute for semantic verification. Actual hook-driven
// message/index changes remain visible and indeterminate; nothing is replayed.
#[test]
fn changed_effects() {
    for message in [false, true] {
        let repository = Repository::new("sha1");
        fs::write(repository.root.join("source"), "base\n").unwrap();
        repository.commit();
        fs::write(repository.root.join("source"), "staged\n").unwrap();
        repository.git(["add", "source"]);
        fs::write(repository.root.join("extra"), "hook content\n").unwrap();
        if message {
            hook(
                &repository,
                "commit-msg",
                "#!/bin/sh\nprintf 'Hook replacement\\n' > \"$1\"\n",
            );
        } else {
            hook(
                &repository,
                "pre-commit",
                "#!/bin/sh\nexec /usr/bin/git add -- extra\n",
            );
        }
        let mut owner = repository.owner("");
        let context = repository
            .open(&mut owner, &repository.root)
            .context
            .unwrap();
        let observation = owner.commit_observation(context).unwrap();
        owner.commit(observation, "Accepted message").unwrap();
        finish(&mut owner);
        let record = owner.records().last().unwrap();
        assert_eq!(
            (record.outcome, record.effects),
            (Outcome::Indeterminate, Effects::Possible),
            "{record:?}"
        );
        assert!(record.exit.unwrap().success());
        assert_eq!(owner.commit_draft(context), Some("Accepted message"));
        if message {
            assert_eq!(
                repository.git(["show", "--format=%B", "--no-patch", "HEAD"]),
                b"Hook replacement\n\n"
            );
        } else {
            assert_eq!(repository.git(["show", "HEAD:extra"]), b"hook content\n");
        }
        assert!(owner.poll().is_empty());
        assert_eq!(repository.git(["rev-list", "--count", "HEAD"]), b"2\n");
    }
}

// The installed Git signing policy invokes the configured signer and reports
// its failure. Neither native submission nor verification disables that policy.
#[test]
fn signing_failure() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("source"), "base\n").unwrap();
    repository.commit();
    fs::write(repository.root.join("source"), "staged\n").unwrap();
    repository.git(["add", "source"]);
    let head = repository.git(["rev-parse", "HEAD"]);
    let called = repository.fixture.0.join("signer-called");
    let signer = repository.fixture.write(
        "signer",
        &format!(
            "#!/bin/sh\nprintf called > '{}'\necho SIGNING_POLICY_SENTINEL >&2\nexit 1\n",
            called.display()
        ),
    );
    fs::set_permissions(&signer, fs::Permissions::from_mode(0o700)).unwrap();
    repository.git(["config", "gpg.program", signer.to_str().unwrap()]);
    repository.git(["config", "commit.gpgsign", "true"]);
    let mut owner = repository.owner("");
    let context = repository
        .open(&mut owner, &repository.root)
        .context
        .unwrap();
    let observation = owner.commit_observation(context).unwrap();
    owner.commit(observation, "Signed request").unwrap();
    finish(&mut owner);
    let record = owner.records().last().unwrap();
    assert_eq!(record.outcome, Outcome::Failed, "{record:?}");
    assert!(!record.exit.unwrap().success());
    assert!(called.exists());
    assert!(String::from_utf8_lossy(&record.stderr).contains("SIGNING_POLICY_SENTINEL"));
    assert_eq!(
        repository.git(["config", "--get", "commit.gpgsign"]),
        b"true\n"
    );
    assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
    assert_eq!(owner.commit_draft(context), Some("Signed request"));
}

// Only the selected owned commit child is cancelled. A waiting pre-commit hook
// is reaped with its process group; effects stay possible and the draft survives.
#[test]
fn cancelled_child() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("source"), "base\n").unwrap();
    repository.commit();
    fs::write(repository.root.join("source"), "staged\n").unwrap();
    repository.git(["add", "source"]);
    let head = repository.git(["rev-parse", "HEAD"]);
    let pid = repository.fixture.0.join("hook-pid");
    hook(
        &repository,
        "pre-commit",
        &format!(
            "#!/bin/sh\necho $$ > '{}'\nwhile :; do sleep 0.02; done\n",
            pid.display()
        ),
    );
    let mut owner = repository.owner("");
    let context = repository
        .open(&mut owner, &repository.root)
        .context
        .unwrap();
    let observation = owner.commit_observation(context).unwrap();
    let operation = owner.commit(observation, "Cancelled request").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let process = loop {
        if let Some(pid) = fs::read_to_string(&pid)
            .ok()
            .and_then(|p| p.trim().parse::<u32>().ok())
        {
            break PathBuf::from(format!("/proc/{pid}"));
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    };
    assert!(process.exists());
    owner.cancel(operation).unwrap();
    finish(&mut owner);
    let record = owner.records().last().unwrap();
    assert_eq!(
        (record.outcome, record.effects),
        (Outcome::Cancelled, Effects::Possible),
        "{record:?}"
    );
    assert!(!process.exists());
    assert!(!owner.busy());
    assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
    assert_eq!(owner.commit_draft(context), Some("Cancelled request"));
}

// A complete changed-path list is insufficient when full index evidence exceeds
// admission. Foreign owners with equal numeric generations also reject before
// a mutator; both cases preserve actual HEAD/index and remain within 32 MiB.
#[test]
fn bounded_authority() {
    let repository = Repository::new("sha1");
    for name in ["source", "unchanged"] {
        fs::write(repository.root.join(name), "base\n").unwrap();
    }
    repository.commit();
    fs::write(repository.root.join("source"), "staged\n").unwrap();
    repository.git(["add", "source"]);
    let head = repository.git(["rev-parse", "HEAD"]);
    let index = repository.git(["ls-files", "--stage", "-z"]);
    let mut owner = repository.owner("max_status_paths=1");
    let context = repository
        .open(&mut owner, &repository.root)
        .context
        .unwrap();
    let observation = owner.commit_observation(context).unwrap();
    owner.commit(observation, "Bounded request").unwrap();
    finish(&mut owner);
    assert_eq!(owner.records().last().unwrap().effects, Effects::NotStarted);
    assert!(owner.admitted_bytes() <= rev::git::SESSION_BYTES);
    assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
    assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
    let mut other = repository.owner("");
    let other_context = repository
        .open(&mut other, &repository.root)
        .context
        .unwrap();
    let mut first = repository.owner("");
    let first_context = repository
        .open(&mut first, &repository.root)
        .context
        .unwrap();
    assert_eq!(
        first.context(first_context).unwrap().observation.generation,
        other.context(other_context).unwrap().observation.generation
    );
    let observation = other.commit_observation(other_context).unwrap();
    assert!(first.commit(observation, "Foreign owner").is_err());
    assert!(!first.busy());
}

// A merge amend preserves every actual parent. Fixed fixture dates make a
// no-op amend retain the same object ID, which is still a verified Git result
// when its message, complete tree and parent set satisfy the accepted request.
#[test]
fn merge_noop_amend() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("source"), "base\n").unwrap();
    repository.commit();
    fs::write(&repository.program, "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nexport GIT_AUTHOR_DATE='2026-01-01T00:00:00Z' GIT_COMMITTER_DATE='2026-01-01T00:00:00Z'\nexec /usr/bin/git \"$@\"\n").unwrap();
    let base = String::from_utf8(repository.git(["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    let tree = String::from_utf8(repository.git(["rev-parse", "HEAD^{tree}"]))
        .unwrap()
        .trim()
        .to_owned();
    let side = String::from_utf8(repository.git(["commit-tree", &tree, "-p", &base, "-m", "Side"]))
        .unwrap()
        .trim()
        .to_owned();
    let merge = String::from_utf8(repository.git([
        "commit-tree",
        &tree,
        "-p",
        &base,
        "-p",
        &side,
        "-m",
        "Merge",
    ]))
    .unwrap()
    .trim()
    .to_owned();
    repository.git(["update-ref", "HEAD", &merge]);
    let parents = repository.git(["show", "--format=%P", "--no-patch", "HEAD"]);
    let mut owner = repository.owner("");
    let context = repository
        .open(&mut owner, &repository.root)
        .context
        .unwrap();
    owner.prepare_amend(context).unwrap();
    finish(&mut owner);
    let composition = owner.composition(context).unwrap().clone();
    let message = composition.message.clone();
    owner.commit_prepared(composition, &message).unwrap();
    finish(&mut owner);
    let record = owner.records().last().unwrap();
    assert_eq!(
        (record.outcome, record.effects),
        (Outcome::Verified, Effects::Verified),
        "{record:?}"
    );
    assert_eq!(
        repository.git(["rev-parse", "HEAD"]),
        format!("{merge}\n").as_bytes()
    );
    assert_eq!(
        repository.git(["show", "--format=%P", "--no-patch", "HEAD"]),
        parents
    );
}
