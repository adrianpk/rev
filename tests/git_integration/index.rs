#[path = "index_groups.rs"]
mod groups;

#[path = "index_presentation.rs"]
mod presentation;

use super::support::{Repository, finish};
use rev::git::{
    Diff, Effects, Group, IndexDirection, IndexTarget, OperationKind, Outcome, Repositories,
};
use std::{ffi::OsString, fs, os::unix::ffi::OsStringExt, path::Path, sync::Arc};

fn compare(owner: &mut Repositories, group: Group, path: &Path) -> Arc<Diff> {
    let observation = owner.contexts()[0].observation.clone();
    owner
        .compare(observation.context, observation.generation, group, path)
        .unwrap();
    assert!(finish(owner).published);
    owner
        .context(observation.context)
        .unwrap()
        .detail
        .as_ref()
        .unwrap()
        .clone()
}
fn change(owner: &mut Repositories, direction: IndexDirection, target: IndexTarget) {
    let operation = owner.change_index(direction, target).unwrap();
    let update = finish(owner);
    let record = owner.record(operation).unwrap();
    assert!(update.published, "{:?}", record);
    assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
    assert_eq!(record.effects, Effects::Verified);
}

// Whole-path staging and unstaging verify actual index blobs/modes, including
// binary bytes, type changes and deletion, while preserving worktree content.
#[test]
fn path_roundtrip() {
    for case in ["text", "binary", "type", "deleted"] {
        let repository = Repository::new("sha1");
        let path = repository.root.join("source");
        fs::write(&path, b"original\n").unwrap();
        repository.commit();
        let original = repository.git(["ls-files", "--stage", "-z"]);
        match case {
            "text" => fs::write(&path, b"changed\n").unwrap(),
            "binary" => fs::write(&path, b"binary\0bytes\n").unwrap(),
            "type" => {
                fs::remove_file(&path).unwrap();
                std::os::unix::fs::symlink("missing-target", &path).unwrap();
            }
            _ => fs::remove_file(&path).unwrap(),
        }
        let contents = if case == "type" {
            fs::read_link(&path)
                .unwrap()
                .as_os_str()
                .as_encoded_bytes()
                .to_vec()
        } else {
            fs::read(&path).unwrap_or_default()
        };
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let diff = compare(&mut owner, Group::Unstaged, Path::new("source"));
        change(&mut owner, IndexDirection::Stage, IndexTarget::Path(diff));
        if case == "deleted" {
            assert!(repository.git(["ls-files", "-z"]).is_empty());
        } else {
            assert_eq!(repository.git(["show", ":source"]), contents);
        }
        let staged = compare(&mut owner, Group::Staged, Path::new("source"));
        change(
            &mut owner,
            IndexDirection::Unstage,
            IndexTarget::Path(staged),
        );
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), original);
        if case == "deleted" {
            assert!(!path.exists());
        } else if case == "type" {
            assert_eq!(
                fs::read_link(&path).unwrap().as_os_str().as_encoded_bytes(),
                contents
            );
        } else {
            assert_eq!(fs::read(&path).unwrap(), contents);
        }
    }
}

// Exact unusual paths stage through real Git literal pathspecs in unborn
// repositories. Unstaging removes only their index entry and retains the file.
#[test]
fn unborn_paths() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        let exact = OsString::from_vec(b":(glob)raw\n\xff".to_vec());
        let path = repository.root.join(&exact);
        fs::write(&path, "source bytes\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        repository.git(["config", "core.filemode", "false"]);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let diff = compare(&mut owner, Group::Untracked, Path::new(&exact));
        change(&mut owner, IndexDirection::Stage, IndexTarget::Path(diff));
        assert_eq!(
            repository.git(["ls-files", "-z"]),
            [exact.clone().into_vec(), vec![0]].concat()
        );
        let staged = compare(&mut owner, Group::Staged, Path::new(&exact));
        change(
            &mut owner,
            IndexDirection::Unstage,
            IndexTarget::Path(staged),
        );
        assert!(repository.git(["ls-files", "-z"]).is_empty());
        assert_eq!(fs::read_to_string(path).unwrap(), "source bytes\n");
    }
}

// Partial index effects are exact semantic hunks in both object formats: the
// other worktree hunk and the source file remain unchanged through both actions.
#[test]
fn hunk_roundtrip() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        let path = repository.root.join("source");
        let base: Vec<_> = (1..=48).map(|n| format!("line {n:02}\n")).collect();
        fs::write(&path, base.concat()).unwrap();
        repository.commit();
        let mut content = base.clone();
        content[3] = "first change\n".into();
        content[39] = "second change\n".into();
        fs::write(&path, content.concat()).unwrap();
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let diff = compare(&mut owner, Group::Unstaged, Path::new("source"));
        assert_eq!(diff.hunks.len(), 2);
        let hunk = diff.hunks[1].id;
        change(
            &mut owner,
            IndexDirection::Stage,
            IndexTarget::Hunk { diff, hunk },
        );
        let mut expected = base.clone();
        expected[39] = "second change\n".into();
        assert_eq!(
            repository.git(["show", ":source"]),
            expected.concat().as_bytes()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), content.concat());
        let staged = compare(&mut owner, Group::Staged, Path::new("source"));
        assert_eq!(staged.hunks.len(), 1);
        let hunk = staged.hunks[0].id;
        change(
            &mut owner,
            IndexDirection::Unstage,
            IndexTarget::Hunk { diff: staged, hunk },
        );
        assert_eq!(
            repository.git(["show", ":source"]),
            base.concat().as_bytes()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), content.concat());
    }
}

// Real empty and missing-final-newline blobs exercise exact patch boundaries;
// binary hunks and foreign-parent hunk IDs cannot acquire mutation authority.
#[test]
fn hunk_boundaries() {
    for (before, after) in [(b"old".as_slice(), b"new".as_slice()), (b"", b"first\n")] {
        let repository = Repository::new("sha1");
        let path = repository.root.join("source");
        fs::write(&path, before).unwrap();
        repository.commit();
        fs::write(&path, after).unwrap();
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let diff = compare(&mut owner, Group::Unstaged, Path::new("source"));
        let hunk = diff.hunks[0].id;
        change(
            &mut owner,
            IndexDirection::Stage,
            IndexTarget::Hunk { diff, hunk },
        );
        assert_eq!(repository.git(["show", ":source"]), after);
        let diff = compare(&mut owner, Group::Staged, Path::new("source"));
        assert!(
            owner
                .change_index(
                    IndexDirection::Unstage,
                    IndexTarget::Hunk {
                        diff: diff.clone(),
                        hunk
                    }
                )
                .is_err()
        );
        let hunk = diff.hunks[0].id;
        change(
            &mut owner,
            IndexDirection::Unstage,
            IndexTarget::Hunk { diff, hunk },
        );
        assert_eq!(repository.git(["show", ":source"]), before);
        assert_eq!(fs::read(path).unwrap(), after);
    }
}

// Worktree edits with unchanged porcelain flags invalidate retained comparison
// bytes before the index-changing child. Stale generations reject admission.
#[test]
fn stale_preflight() {
    for changed_index in [false, true] {
        let repository = Repository::new("sha1");
        let path = repository.root.join("source");
        fs::write(&path, "original\n").unwrap();
        repository.commit();
        fs::write(&path, "captured\n").unwrap();
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let diff = compare(&mut owner, Group::Unstaged, Path::new("source"));
        fs::write(&path, "external change\n").unwrap();
        if changed_index {
            repository.git(["add", "source"]);
            fs::write(&path, "captured\n").unwrap();
        }
        let expected_index = repository.git(["show", ":source"]);
        let expected_source = fs::read(&path).unwrap();
        let id = owner
            .change_index(IndexDirection::Stage, IndexTarget::Path(diff.clone()))
            .unwrap();
        assert!(!finish(&mut owner).published);
        assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
        assert_eq!(owner.record(id).unwrap().outcome, Outcome::Failed);
        assert_eq!(repository.git(["show", ":source"]), expected_index);
        assert_eq!(fs::read(&path).unwrap(), expected_source);
        assert!(
            owner
                .change_index(IndexDirection::Stage, IndexTarget::Path(diff))
                .is_err()
        );
    }
}

// A real successful add followed by a failing wrapper exit is a possible
// effect, never rollback or a replay. Refresh still publishes actual Git facts.
#[test]
fn failed_after_effect() {
    let repository = Repository::new("sha1");
    let path = repository.root.join("source");
    fs::write(&path, "original\n").unwrap();
    repository.commit();
    fs::write(&path, "changed\n").unwrap();
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let diff = compare(&mut owner, Group::Unstaged, Path::new("source"));
    fs::write(&repository.program, "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nif [ \"$3\" = add ]; then /usr/bin/git \"$@\"; printf x >> effect-count; echo 'Fixture post-effect failure' >&2; exit 70; fi\nexec /usr/bin/git \"$@\"\n").unwrap();
    let id = owner
        .change_index(IndexDirection::Stage, IndexTarget::Path(diff))
        .unwrap();
    assert!(finish(&mut owner).published);
    let record = owner.record(id).unwrap();
    assert_eq!(record.kind, OperationKind::Stage);
    assert_eq!(record.outcome, Outcome::Indeterminate);
    assert_eq!(record.effects, Effects::Possible);
    assert_eq!(record.exit.unwrap().code(), Some(70));
    assert!(
        record
            .stderr
            .windows(b"Fixture post-effect failure\n".len())
            .any(|s| s == b"Fixture post-effect failure\n")
    );
    assert_eq!(repository.git(["show", ":source"]), b"changed\n");
    assert_eq!(fs::read(path).unwrap(), b"changed\n");
    assert_eq!(
        fs::read(repository.root.join("effect-count")).unwrap(),
        b"x"
    );
    assert_eq!(
        owner.contexts()[0]
            .observation
            .status
            .as_ref()
            .unwrap()
            .group(Group::Staged)
            .count(),
        1
    );
}

// A staged rename restores both exact HEAD path identities on unstage without
// moving either worktree file or altering an unrelated staged addition.
#[test]
fn renamed_path() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("old"), "original\n").unwrap();
    repository.commit();
    repository.git(["mv", "old", "new"]);
    fs::write(repository.root.join("unrelated"), "retained\n").unwrap();
    repository.git(["add", "unrelated"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let diff = compare(&mut owner, Group::Staged, Path::new("new"));
    change(&mut owner, IndexDirection::Unstage, IndexTarget::Path(diff));
    assert_eq!(repository.git(["show", ":old"]), b"original\n");
    assert_eq!(repository.git(["show", ":unrelated"]), b"retained\n");
    assert!(!repository.root.join("old").exists());
    assert_eq!(
        fs::read(repository.root.join("new")).unwrap(),
        b"original\n"
    );
    assert_eq!(repository.git(["ls-files", "--", "new"]), b"");
}

// Configured clean filters keep installed Git's staging semantics. Index
// verification checks the filtered blob and never rewrites worktree bytes.
#[test]
fn clean_filter() {
    let repository = Repository::new("sha1");
    let path = repository.root.join("source");
    fs::write(&path, "ORIGINAL\n").unwrap();
    fs::write(
        repository.root.join(".gitattributes"),
        "source filter=fixture\n",
    )
    .unwrap();
    repository.git(["config", "filter.fixture.clean", "tr a-z A-Z"]);
    repository.commit();
    fs::write(&path, "mixed Case\n").unwrap();
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let diff = compare(&mut owner, Group::Unstaged, Path::new("source"));
    change(&mut owner, IndexDirection::Stage, IndexTarget::Path(diff));
    assert_eq!(repository.git(["show", ":source"]), b"MIXED CASE\n");
    assert_eq!(fs::read(path).unwrap(), b"mixed Case\n");
}

// Linked worktrees have separate index files but share the same storage owner;
// overlapping mutations reject before launch, then proceed as new requests.
#[test]
fn common_storage() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("source"), "original\n").unwrap();
    repository.commit();
    let linked = repository.fixture.0.join("linked");
    repository.git(["worktree", "add", "-b", "side", linked.to_str().unwrap()]);
    fs::write(repository.root.join("source"), "main change\n").unwrap();
    fs::write(linked.join("source"), "linked change\n").unwrap();
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    repository.open(&mut owner, &linked);
    let contexts: Vec<_> = owner.contexts().iter().map(|c| c.id).collect();
    let mut targets = Vec::new();
    for context in contexts {
        let generation = owner.context(context).unwrap().observation.generation;
        owner
            .compare(context, generation, Group::Unstaged, Path::new("source"))
            .unwrap();
        assert!(finish(&mut owner).published);
        targets.push(
            owner
                .context(context)
                .unwrap()
                .detail
                .as_ref()
                .unwrap()
                .clone(),
        );
    }
    owner
        .change_index(IndexDirection::Stage, IndexTarget::Path(targets[0].clone()))
        .unwrap();
    assert!(
        owner
            .change_index(IndexDirection::Stage, IndexTarget::Path(targets[1].clone()))
            .unwrap_err()
            .contains("common storage")
    );
    assert!(finish(&mut owner).published);
    change(
        &mut owner,
        IndexDirection::Stage,
        IndexTarget::Path(targets[1].clone()),
    );
    assert_eq!(repository.git(["show", ":source"]), b"main change\n");
    assert_eq!(
        repository.git(["-C", linked.to_str().unwrap(), "show", ":source"]),
        b"linked change\n"
    );
}

// Text hunk requests do not include a separate executable-mode change. Both
// directions retain the existing index mode while changing only selected text.
#[test]
fn hunk_mode() {
    use std::os::unix::fs::PermissionsExt;
    for direction in [IndexDirection::Stage, IndexDirection::Unstage] {
        let repository = Repository::new("sha1");
        let path = repository.root.join("source");
        fs::write(&path, "original\n").unwrap();
        repository.commit();
        fs::write(&path, "changed\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        let group = if direction == IndexDirection::Unstage {
            repository.git(["add", "source"]);
            Group::Staged
        } else {
            Group::Unstaged
        };
        let before = repository.git(["ls-files", "--stage", "source"]);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let diff = compare(&mut owner, group, Path::new("source"));
        assert!(diff.supports_hunks());
        let hunk = diff.hunks[0].id;
        change(&mut owner, direction, IndexTarget::Hunk { diff, hunk });
        let after = repository.git(["ls-files", "--stage", "source"]);
        assert_eq!(&after[..6], &before[..6]);
        assert_eq!(
            repository.git(["show", ":source"]),
            if direction == IndexDirection::Stage {
                b"changed\n".as_slice()
            } else {
                b"original\n".as_slice()
            }
        );
        assert_eq!(fs::read(&path).unwrap(), b"changed\n");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }
}

// Whole gitlink staging records the initialized submodule's exact commit, not
// its dirty files or the containing repository's HEAD identity.
#[test]
fn gitlink_path() {
    let nested = Repository::new("sha1");
    fs::write(nested.root.join("source"), "original\n").unwrap();
    nested.commit();
    let repository = Repository::new("sha1");
    repository.git([
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "add",
        nested.root.to_str().unwrap(),
        "module",
    ]);
    repository.commit();
    fs::write(repository.root.join("module/source"), "nested commit\n").unwrap();
    repository.git(["-C", "module", "add", "source"]);
    repository.git([
        "-C",
        "module",
        "-c",
        "user.name=Example Author",
        "-c",
        "user.email=author@example.invalid",
        "-c",
        "commit.gpgsign=false",
        "commit",
        "--message=Nested fixture",
    ]);
    let commit = repository.git(["-C", "module", "rev-parse", "HEAD"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let diff = compare(&mut owner, Group::Unstaged, Path::new("module"));
    change(&mut owner, IndexDirection::Stage, IndexTarget::Path(diff));
    assert_eq!(repository.git(["rev-parse", ":module"]), commit);
    assert_eq!(
        fs::read(repository.root.join("module/source")).unwrap(),
        b"nested commit\n"
    );
}

// Whole-file text hunks keep creation/deletion modes. Reversing a staged
// deletion restores the HEAD index mode while the current file stays absent.
#[test]
fn hunk_file_modes() {
    for deleting in [true, false] {
        let repository = Repository::new("sha1");
        let path = repository.root.join("source");
        fs::write(&path, "source bytes\n").unwrap();
        if deleting {
            repository.commit();
            fs::remove_file(&path).unwrap();
        } else {
            repository.git(["add", "source"]);
        }
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        if deleting {
            let diff = compare(&mut owner, Group::Unstaged, Path::new("source"));
            let hunk = diff.hunks[0].id;
            change(
                &mut owner,
                IndexDirection::Stage,
                IndexTarget::Hunk { diff, hunk },
            );
            assert!(repository.git(["ls-files", "-z"]).is_empty());
        }
        let diff = compare(&mut owner, Group::Staged, Path::new("source"));
        let hunk = diff.hunks[0].id;
        change(
            &mut owner,
            IndexDirection::Unstage,
            IndexTarget::Hunk { diff, hunk },
        );
        if deleting {
            assert_eq!(repository.git(["show", ":source"]), b"source bytes\n");
            assert!(!path.exists());
        } else {
            assert!(repository.git(["ls-files", "-z"]).is_empty());
            assert_eq!(fs::read(path).unwrap(), b"source bytes\n");
        }
    }
}
