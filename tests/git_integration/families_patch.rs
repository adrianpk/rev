use super::*;
use std::{ffi::OsStr, os::unix::fs::PermissionsExt, path::PathBuf};

fn arguments(family: Family, input: &Path) -> Vec<OsString> {
    let mut args: Vec<_> = family.seed().into_iter().map(OsString::from).collect();
    *args.last_mut().unwrap() = input.as_os_str().into();
    args
}
fn target(repository: &Repository, base: &str, name: &str) -> PathBuf {
    let path = repository.fixture.0.join(name);
    repository.git([
        OsStr::new("worktree"),
        OsStr::new("add"),
        OsStr::new("--detach"),
        OsStr::new("--"),
        path.as_os_str(),
        OsStr::new(base),
    ]);
    path
}
fn git(repository: &Repository, path: &Path, values: &[&str]) -> Vec<u8> {
    let mut args: Vec<OsString> = vec!["-C".into(), path.as_os_str().into()];
    args.extend(values.iter().map(OsString::from));
    repository.git(args)
}
fn plan(
    repository: &Repository,
    owner: &mut Repositories,
    directory: &Path,
    family: Family,
    input: &Path,
) -> Arc<FamilyPlan> {
    repository.open(owner, directory);
    let context = owner
        .contexts()
        .iter()
        .find(|c| c.observation.identity.worktree.as_deref() == Some(directory))
        .unwrap()
        .id;
    prepare(
        owner,
        directory,
        Some(context),
        family,
        arguments(family, input),
    )
}
fn history(format: &str) -> (Repository, String, PathBuf) {
    let repository = Repository::new(format);
    fs::write(repository.root.join("text"), "base\n").unwrap();
    fs::write(repository.root.join("binary"), [0, 1, 2, 3]).unwrap();
    fs::write(repository.root.join("deleted"), "old\n").unwrap();
    repository.commit();
    let base = String::from_utf8(repository.git(["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    let raw = PathBuf::from(OsString::from_vec(b"-[literal]\n\xff".to_vec()));
    fs::write(repository.root.join(&raw), "new\n").unwrap();
    fs::remove_file(repository.root.join("deleted")).unwrap();
    repository.git(["add", "--all"]);
    repository.git([
        "commit",
        "--message",
        "First subject\n\nFirst body\n\nSecond paragraph",
        "--date=2020-03-04T12:34:56+02:30",
        "--author=Patch Author <patch@example.invalid>",
    ]);
    fs::write(repository.root.join("text"), "delivered\n").unwrap();
    fs::write(repository.root.join("binary"), [0, 8, 9, 255]).unwrap();
    repository.git(["add", "--all"]);
    repository.git([
        "commit",
        "--message",
        "Second café\n\nBody text",
        "--date=2021-05-06T07:08:09-03:00",
    ]);
    (repository, base, raw)
}
fn input(repository: &Repository, base: &str, family: Family) -> PathBuf {
    let bytes = if family == Family::ApplyMailbox {
        repository.git([
            "format-patch",
            "--stdout",
            "--binary",
            "--full-index",
            "--no-signature",
            &format!("{base}..HEAD"),
            "--",
        ])
    } else {
        // Ordinary abbreviated index headers are deliberately accepted too.
        repository.git(["diff", "--binary", base, "HEAD", "--"])
    };
    let path = repository
        .fixture
        .0
        .join(OsString::from_vec(b"input-$(literal)\n\xff.patch".to_vec()));
    fs::write(&path, bytes).unwrap();
    path
}

// Real plain and multi-message applications independently prove trees, staged
// state, materialized bytes and author/message/date/parent metadata in both formats.
#[test]
fn saved_application() {
    for format in ["sha1", "sha256"] {
        let (repository, base, raw) = history(format);
        let expected_tree = repository.git(["rev-parse", "HEAD^{tree}"]);
        let mut owner = repository.owner("");
        for family in [Family::ApplyPatch, Family::ApplyMailbox] {
            let input = input(&repository, &base, family);
            let path = target(&repository, &base, family.label());
            let prepared = plan(&repository, &mut owner, &path, family, &input);
            let (reviewed, bytes, hash) = prepared.input_preview().unwrap();
            assert_eq!(reviewed, input);
            assert_eq!(bytes, fs::read(&input).unwrap().len());
            assert_eq!(hash.hex().len(), if format == "sha256" { 64 } else { 40 });
            assert_eq!(
                git(&repository, &path, &["write-tree"]),
                git(&repository, &path, &["rev-parse", "HEAD^{tree}"])
            );
            execute(&mut owner, prepared);
            assert_eq!(git(&repository, &path, &["write-tree"]), expected_tree);
            assert_eq!(fs::read(path.join("text")).unwrap(), b"delivered\n");
            assert_eq!(fs::read(path.join("binary")).unwrap(), [0, 8, 9, 255]);
            assert_eq!(fs::read(path.join(&raw)).unwrap(), b"new\n");
            assert!(!path.join("deleted").exists());
            if family == Family::ApplyPatch {
                assert_eq!(
                    git(&repository, &path, &["rev-parse", "HEAD"]),
                    format!("{base}\n").as_bytes()
                );
                assert!(!git(&repository, &path, &["diff", "--cached", "--name-only"]).is_empty());
            } else {
                assert_eq!(
                    git(
                        &repository,
                        &path,
                        &["rev-list", "--count", &format!("{base}..HEAD")]
                    ),
                    b"2\n"
                );
                assert_eq!(
                    git(
                        &repository,
                        &path,
                        &["log", "-2", "--format=%T|%an|%ae|%at|%B"]
                    ),
                    repository.git(["log", "-2", "--format=%T|%an|%ae|%at|%B"])
                );
                assert!(git(&repository, &path, &["status", "--porcelain"]).is_empty());
            }
        }
    }
}

// A valid changed payload keeps its original From commit label, but verification
// follows the decoded patch semantics and proves a different actual tree.
#[test]
fn payload_semantics() {
    let (repository, base, _) = history("sha1");
    let path = target(&repository, &base, "changed-mailbox");
    let input = input(&repository, &base, Family::ApplyMailbox);
    let bytes = fs::read(&input).unwrap();
    let bytes = String::from_utf8(bytes)
        .unwrap()
        .replace("+delivered\n", "+altered!!\n");
    fs::write(&input, bytes).unwrap();
    let mut owner = repository.owner("");
    let prepared = plan(&repository, &mut owner, &path, Family::ApplyMailbox, &input);
    execute(&mut owner, prepared);
    assert_eq!(fs::read(path.join("text")).unwrap(), b"altered!!\n");
    assert_ne!(
        git(&repository, &path, &["rev-parse", "HEAD^{tree}"]),
        repository.git(["rev-parse", "HEAD^{tree}"])
    );
}

// A real message hook changes a successful Git commit. Zero exit and correct
// patch trees cannot hide changed metadata or establish a verified result.
#[test]
fn actual_hook_changes() {
    let (repository, base, _) = history("sha1");
    let path = target(&repository, &base, "hook-consumer");
    let input = input(&repository, &base, Family::ApplyMailbox);
    let hooks = repository.fixture.0.join("hooks");
    fs::create_dir(&hooks).unwrap();
    let hook = hooks.join("applypatch-msg");
    fs::write(
        &hook,
        "#!/bin/sh\nprintf '%s\\n' 'Changed by real hook' > \"$1\"\n",
    )
    .unwrap();
    fs::set_permissions(hook, fs::Permissions::from_mode(0o700)).unwrap();
    repository.git([
        OsStr::new("config"),
        OsStr::new("core.hooksPath"),
        hooks.as_os_str(),
    ]);
    let mut owner = repository.owner("");
    let prepared = plan(&repository, &mut owner, &path, Family::ApplyMailbox, &input);
    let id = owner.execute_family(prepared).unwrap();
    let update = finish(&mut owner);
    assert!(update.published, "{:?}", owner.record(id));
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome, Outcome::Indeterminate, "{record:?}");
    assert_eq!(record.effects, Effects::Possible);
    assert_eq!(record.exit.and_then(|s| s.code()), Some(0));
    assert!(
        record
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("independently decoded")
    );
    assert_eq!(
        git(&repository, &path, &["show", "-s", "--format=%s"]),
        b"Changed by real hook\n"
    );
}

// Later patch failure retains the already-applied prefix and Git-owned mailbox
// files. Independent observation reports actual lineage without retry or abort.
#[test]
fn mailbox_conflict() {
    let (repository, base, raw) = history("sha1");
    let input = input(&repository, &base, Family::ApplyMailbox);
    let path = target(&repository, &base, "conflicting-consumer");
    fs::write(path.join("text"), "conflicting\n").unwrap();
    git(&repository, &path, &["add", "--all"]);
    git(
        &repository,
        &path,
        &["commit", "--message=Conflicting saved branch"],
    );
    let original = git(&repository, &path, &["rev-parse", "HEAD"]);
    let mut owner = repository.owner("");
    let prepared = plan(&repository, &mut owner, &path, Family::ApplyMailbox, &input);
    let id = owner.execute_family(prepared).unwrap();
    let update = finish(&mut owner);
    assert!(update.published, "{:?}", owner.record(id));
    assert_eq!(
        owner.record(id).unwrap().outcome,
        Outcome::InProgress,
        "{:?}",
        owner.record(id)
    );
    let actual = owner.contexts()[0].sequence.as_ref().unwrap();
    assert_eq!(actual.family, Some(rev::git::SequenceFamily::Mailbox));
    assert_eq!(
        actual.original.unwrap().hex(),
        String::from_utf8(original).unwrap().trim()
    );
    assert_eq!(actual.step, Some(2));
    assert_eq!(actual.total, Some(2));
    assert_eq!(fs::read(path.join("text")).unwrap(), b"conflicting\n");
    assert_eq!(fs::read(path.join(&raw)).unwrap(), b"new\n");
    assert_eq!(
        git(&repository, &path, &["show", "-s", "--format=%s"]),
        b"First subject\n"
    );
    let state = git(
        &repository,
        &path,
        &["rev-parse", "--git-path", "rebase-apply/applying"],
    );
    let state = PathBuf::from(OsString::from_vec(
        state.strip_suffix(b"\n").unwrap().to_vec(),
    ));
    assert!(path.join(state).is_file());
}

// Captured input bytes/identity, config, index and HEAD must still match before
// any changing child; symlink, oversized input and dirty saved sources reject.
#[test]
fn protected_input() {
    for change in ["bytes", "identity", "config", "index", "head"] {
        let (repository, base, _) = history("sha1");
        let input = input(&repository, &base, Family::ApplyPatch);
        let path = target(&repository, &base, "stale-consumer");
        let mut owner = repository.owner("");
        let prepared = plan(&repository, &mut owner, &path, Family::ApplyPatch, &input);
        change_reviewed_input(&repository, &path, &input, change);
        let before = git(&repository, &path, &["ls-files", "--stage", "-z"]);
        let id = owner.execute_family(prepared).unwrap();
        finish(&mut owner);
        assert_eq!(
            owner.record(id).unwrap().effects,
            Effects::NotStarted,
            "{change}: {:?}",
            owner.record(id)
        );
        assert_eq!(fs::read(path.join("text")).unwrap(), b"base\n");
        assert_eq!(
            git(&repository, &path, &["ls-files", "--stage", "-z"]),
            before
        );
    }
    for variant in ["symlink", "oversized", "dirty"] {
        let (repository, base, _) = history("sha1");
        let input = input(&repository, &base, Family::ApplyPatch);
        let path = target(&repository, &base, "invalid-consumer");
        let selected = match variant {
            "symlink" => {
                let alias = repository.fixture.0.join("input-alias");
                std::os::unix::fs::symlink(&input, &alias).unwrap();
                alias
            }
            "oversized" => {
                fs::write(&input, vec![b'x'; 1048577]).unwrap();
                input
            }
            "dirty" => {
                fs::write(path.join("text"), "unsaved on disk\n").unwrap();
                input
            }
            _ => unreachable!(),
        };
        let mut owner = repository.owner("");
        repository.open(&mut owner, &path);
        let context = owner.contexts()[0].id;
        let id = owner
            .prepare_family(
                &path,
                Some(context),
                Family::ApplyPatch,
                arguments(Family::ApplyPatch, &selected),
            )
            .unwrap();
        assert!(!finish(&mut owner).published, "{variant}");
        assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    }
}

fn change_reviewed_input(repository: &Repository, path: &Path, input: &Path, change: &str) {
    match change {
        "bytes" => fs::write(input, b"changed\n").unwrap(),
        "identity" => {
            let bytes = fs::read(input).unwrap();
            fs::remove_file(input).unwrap();
            fs::write(input, bytes).unwrap();
        }
        "config" => {
            repository.git(["config", "apply.whitespace", "fix"]);
        }
        "index" => {
            fs::write(path.join("new"), "external\n").unwrap();
            git(repository, path, &["add", "--all"]);
        }
        "head" => {
            git(
                repository,
                path,
                &["commit", "--allow-empty", "--message=External"],
            );
        }
        _ => unreachable!(),
    }
}

// Configured signing is not disabled to make mailbox creation succeed. A real
// unavailable signer leaves Git-owned progress and no invented commit success.
#[test]
fn signing_policy() {
    let (repository, base, _) = history("sha1");
    let input = input(&repository, &base, Family::ApplyMailbox);
    let path = target(&repository, &base, "signing-consumer");
    repository.git(["config", "commit.gpgsign", "true"]);
    repository.git(["config", "gpg.program", "/bin/false"]);
    let mut owner = repository.owner("");
    let prepared = plan(&repository, &mut owner, &path, Family::ApplyMailbox, &input);
    assert!(!prepared.args.iter().any(|a| a == "--no-gpg-sign"));
    let id = owner.execute_family(prepared).unwrap();
    let update = finish(&mut owner);
    assert!(update.published, "{:?}", owner.record(id));
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome, Outcome::InProgress, "{record:?}");
    assert_eq!(record.effects, Effects::Possible);
    assert!(record.exit.is_some_and(|s| !s.success()));
    assert_eq!(
        git(&repository, &path, &["rev-parse", "HEAD"]),
        format!("{base}\n").as_bytes()
    );
    assert_eq!(
        repository.git(["config", "--bool", "commit.gpgsign"]),
        b"true\n"
    );
    assert_eq!(
        owner.contexts()[0].sequence.as_ref().unwrap().family,
        Some(rev::git::SequenceFamily::Mailbox)
    );
}
