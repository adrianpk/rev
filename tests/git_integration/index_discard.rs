use super::compare;
use crate::support::{Repository, finish};
use rev::git::{Effects, Group, OperationKind, Outcome};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

// Restore the real index rather than HEAD across text/binary, missing paths,
// type changes and symlinks. Actual bytes/modes and all staged entries decide.
#[test]
fn restore() {
    for case in ["text", "binary", "missing", "symlink", "type"] {
        let repository = Repository::new("sha1");
        let path = repository.root.join("a");
        fs::write(&path, b"HEAD\n").unwrap();
        fs::write(repository.root.join("other"), b"other\n").unwrap();
        repository.commit();
        let expected = if case == "binary" {
            b"index\0\n".as_slice()
        } else {
            b"index\n".as_slice()
        };
        if case == "symlink" {
            fs::remove_file(&path).unwrap();
            std::os::unix::fs::symlink("index-target", &path).unwrap();
        } else {
            fs::write(&path, expected).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        repository.git(["add", "a"]);
        fs::write(repository.root.join("other"), b"other staged\n").unwrap();
        repository.git(["add", "other"]);
        let index = repository.git(["ls-files", "--stage", "-z"]);
        match case {
            "missing" => fs::remove_file(&path).unwrap(),
            "symlink" => {
                fs::remove_file(&path).unwrap();
                std::os::unix::fs::symlink("working-target", &path).unwrap();
            }
            "type" => {
                fs::remove_file(&path).unwrap();
                std::os::unix::fs::symlink("working-target", &path).unwrap();
            }
            _ => fs::write(&path, b"working\n").unwrap(),
        }
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let diff = compare(&mut owner, Group::Unstaged, Path::new("a"));
        let operation = owner.discard(diff).unwrap();
        let update = finish(&mut owner);
        let record = owner.record(operation).unwrap();
        assert!(update.published, "{record:?}");
        assert_eq!(record.kind, OperationKind::Discard);
        assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
        assert_eq!(record.effects, Effects::Verified);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        if case == "symlink" {
            assert_eq!(fs::read_link(&path).unwrap(), Path::new("index-target"));
        } else {
            assert_eq!(fs::read(&path).unwrap(), expected);
            assert_ne!(fs::metadata(&path).unwrap().permissions().mode() & 0o111, 0);
        }
    }
}

// Smudge output is compared byte-for-byte, rather than inferring correctness
// merely from an equivalent clean-filter hash or a zero child exit.
#[test]
fn filters() {
    let repository = Repository::new("sha1");
    repository.git(["config", "filter.fixture.clean", "tr A-Z a-z"]);
    repository.git(["config", "filter.fixture.smudge", "tr a-z A-Z"]);
    fs::write(repository.root.join(".gitattributes"), "a filter=fixture\n").unwrap();
    fs::write(repository.root.join("a"), "INDEX\n").unwrap();
    repository.commit();
    fs::write(repository.root.join("a"), "working\n").unwrap();
    let index = repository.git(["ls-files", "--stage", "-z"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let diff = compare(&mut owner, Group::Unstaged, Path::new("a"));
    let operation = owner.discard(diff).unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(operation).unwrap().effects, Effects::Verified);
    assert_eq!(fs::read(repository.root.join("a")).unwrap(), b"INDEX\n");
    assert_eq!(repository.git(["show", ":a"]), b"index\n");
    assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
}

// Untracked paths never gain deletion authority. A stale comparison cannot
// replace working bytes, and a post-effect failing child is never replayed.
#[test]
fn failures() {
    for case in ["stale", "post-effect"] {
        let repository = Repository::new("sha1");
        let path = repository.root.join("a");
        fs::write(&path, b"index\n").unwrap();
        repository.commit();
        fs::write(&path, b"working\n").unwrap();
        fs::write(repository.root.join("new"), b"untracked\n").unwrap();
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let untracked = compare(&mut owner, Group::Untracked, Path::new("new"));
        assert!(owner.discard(untracked).is_err());
        let diff = compare(&mut owner, Group::Unstaged, Path::new("a"));
        if case == "stale" {
            fs::write(&path, b"different working bytes\n").unwrap();
        } else {
            fs::write(&repository.program, "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nif [ \"$3\" = restore ]; then /usr/bin/git \"$@\"; printf x >> effect-count; echo fixture-discard-failure >&2; exit 70; fi\nexec /usr/bin/git \"$@\"\n").unwrap();
        }
        let operation = owner.discard(diff).unwrap();
        let update = finish(&mut owner);
        let record = owner.record(operation).unwrap();
        if case == "stale" {
            assert_eq!(record.effects, Effects::NotStarted);
            assert!(!update.published);
            assert_eq!(fs::read(&path).unwrap(), b"different working bytes\n");
        } else {
            assert!(update.published);
            assert_eq!(record.effects, Effects::Possible);
            assert_eq!(record.outcome, Outcome::Indeterminate);
            assert_eq!(record.exit.and_then(|s| s.code()), Some(70));
            assert!(
                record
                    .stderr
                    .windows(b"fixture-discard-failure\n".len())
                    .any(|s| s == b"fixture-discard-failure\n")
            );
            assert_eq!(fs::read(&path).unwrap(), b"index\n");
            assert_eq!(
                fs::read(repository.root.join("effect-count")).unwrap(),
                b"x"
            );
        }
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        assert_eq!(
            fs::read(repository.root.join("new")).unwrap(),
            b"untracked\n"
        );
    }
}
