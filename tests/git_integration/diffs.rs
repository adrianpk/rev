#[path = "diff_presentation.rs"]
mod presentation;

use super::support::{Repository, finish};
use rev::git::{
    Baseline, Completeness, Diff, DiffKind, Group, OperationKind, Outcome, Repositories,
};
use std::{ffi::OsString, fs, os::unix::ffi::OsStringExt, path::Path, sync::Arc};

fn compare(owner: &mut Repositories, group: Group, path: &Path) -> Arc<Diff> {
    let observation = Arc::clone(&owner.contexts()[0].observation);
    let id = owner
        .compare(observation.context, observation.generation, group, path)
        .unwrap();
    let update = finish(owner);
    assert_eq!(update.operation, id);
    assert_eq!(update.kind, OperationKind::Diff);
    assert!(update.published, "{:?}", owner.records());
    Arc::clone(
        owner
            .context(observation.context)
            .unwrap()
            .detail
            .as_ref()
            .unwrap(),
    )
}
fn contains(bytes: &[u8], text: &[u8]) -> bool {
    bytes.windows(text.len()).any(|part| part == text)
}
fn lines() -> Vec<String> {
    (1..=48).map(|line| format!("line {line:02}\n")).collect()
}

// Real index and worktree bytes produce distinct immutable comparisons in both
// object formats. Two distant worktree edits remain two exact semantic hunks.
#[test]
fn layers_and_hunks() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        let helper = repository.fixture.write(
            "external-diff",
            "#!/bin/sh\nprintf invoked > diff-external-invoked\nexit 99\n",
        );
        fs::set_permissions(
            &helper,
            fs::metadata(&repository.program).unwrap().permissions(),
        )
        .unwrap();
        repository.git(["config", "diff.external", helper.to_str().unwrap()]);
        repository.git(["config", "diff.fixture.textconv", helper.to_str().unwrap()]);
        repository.git(["config", "diff.context", "0"]);
        repository.git(["config", "diff.algorithm", "histogram"]);
        fs::write(
            repository.root.join(".gitattributes"),
            "*.txt diff=fixture\n",
        )
        .unwrap();
        let path = repository.root.join("source.txt");
        let mut content = lines();
        fs::write(&path, content.concat()).unwrap();
        repository.commit();
        content[3] = "INDEX CONTENT\n".into();
        fs::write(&path, content.concat()).unwrap();
        repository.git(["add", "source.txt"]);
        let index = repository.git(["rev-parse", ":source.txt"]);
        content[3] = "WORKTREE CONTENT\n".into();
        content[39] = "DISTANT WORKTREE\n".into();
        let source = content.concat();
        fs::write(&path, &source).unwrap();
        let mut owner = repository.owner("");
        assert!(repository.open(&mut owner, &repository.root).published);
        let staged = compare(&mut owner, Group::Staged, Path::new("source.txt"));
        assert!(matches!(staged.baseline, Baseline::Head(_)));
        assert!(staged.supports_hunks());
        assert_eq!(staged.hunks.len(), 1);
        assert!(contains(&staged.evidence, b"+INDEX CONTENT"));
        assert!(!contains(&staged.evidence, b"WORKTREE CONTENT"));
        let worktree = compare(&mut owner, Group::Unstaged, Path::new("source.txt"));
        assert_eq!(worktree.baseline, Baseline::Index);
        assert!(worktree.supports_hunks());
        assert_eq!(worktree.hunks.len(), 2);
        assert!(contains(&worktree.evidence, b"-INDEX CONTENT"));
        assert!(contains(&worktree.evidence, b"+WORKTREE CONTENT"));
        assert!(contains(
            &worktree.evidence[worktree.hunks[1].patch.clone()],
            b"+DISTANT WORKTREE"
        ));
        assert!(worktree.hunk(staged.hunks[0].id).is_none());
        assert!(owner.detail_current(&worktree));
        assert_eq!(repository.git(["rev-parse", ":source.txt"]), index);
        assert_eq!(fs::read_to_string(&path).unwrap(), source);
        assert!(!repository.root.join("diff-external-invoked").exists());
        assert!(!owner.busy());
    }
}

// Unborn staged content uses the empty baseline. Untracked names stay exact OS
// bytes, including pathspec syntax, invalid UTF-8 and leading option spelling.
#[test]
fn unborn_and_path_bytes() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        fs::write(repository.root.join("staged"), "new staged\n").unwrap();
        repository.git(["add", "staged"]);
        let paths = [b":(glob)**".as_slice(), b"--option", b"raw\n\xff"];
        for path in paths {
            fs::write(
                repository.root.join(OsString::from_vec(path.to_vec())),
                b"preview\n",
            )
            .unwrap();
        }
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let staged = compare(&mut owner, Group::Staged, Path::new("staged"));
        assert_eq!(staged.baseline, Baseline::Empty);
        assert!(staged.supports_hunks());
        for path in paths {
            let exact = OsString::from_vec(path.to_vec());
            let detail = compare(&mut owner, Group::Untracked, Path::new(&exact));
            assert_eq!(detail.change.path.as_os_str().as_encoded_bytes(), path);
            assert_eq!(detail.baseline, Baseline::Empty);
            assert_eq!(detail.kind, DiffKind::Text);
            assert!(detail.verified);
            assert!(
                !detail.supports_hunks(),
                "untracked previews have no tracked hunk action"
            );
            assert!(contains(&detail.evidence, b"+preview"));
        }
        repository.git(["add", "--all"]);
        owner.refresh(staged.observation.context).unwrap();
        assert!(finish(&mut owner).published);
        for path in paths {
            let exact = OsString::from_vec(path.to_vec());
            let detail = compare(&mut owner, Group::Staged, Path::new(&exact));
            assert_eq!(detail.change.path.as_os_str().as_encoded_bytes(), path);
            assert!(detail.supports_hunks());
            assert_eq!(
                detail.hunks.len(),
                1,
                "literal pathspec admits exactly one file"
            );
            assert_eq!(
                detail
                    .evidence
                    .windows(11)
                    .filter(|b| *b == b"diff --git ")
                    .count(),
                1
            );
        }
    }
}

// Rename, binary and type-change comparisons preserve their real Git metadata
// without admitting them as ordinary tracked text hunk operations.
#[test]
fn special_files() {
    use std::os::unix::fs::symlink;
    for case in ["rename", "binary", "type", "unmerged"] {
        let repository = Repository::new("sha1");
        fs::write(
            repository.root.join("old"),
            if case == "binary" {
                b"before\0data\n".as_slice()
            } else {
                b"original\n"
            },
        )
        .unwrap();
        repository.commit();
        let (group, path, expected) = match case {
            "rename" => {
                repository.git(["mv", "old", "new"]);
                (Group::Staged, "new", DiffKind::Rename)
            }
            "binary" => {
                fs::write(repository.root.join("old"), b"after\0data\n").unwrap();
                (Group::Unstaged, "old", DiffKind::Binary)
            }
            "unmerged" => {
                repository.git(["branch", "side"]);
                fs::write(repository.root.join("old"), "main\n").unwrap();
                repository.commit();
                repository.git(["switch", "side"]);
                fs::write(repository.root.join("old"), "side\n").unwrap();
                repository.commit();
                repository.git(["switch", "main"]);
                let output = std::process::Command::new(&repository.program)
                    .args(["merge", "--no-edit", "side"])
                    .current_dir(&repository.root)
                    .output()
                    .unwrap();
                assert_eq!(output.status.code(), Some(1));
                (Group::Unmerged, "old", DiffKind::Unmerged)
            }
            _ => {
                fs::remove_file(repository.root.join("old")).unwrap();
                symlink("destination", repository.root.join("old")).unwrap();
                (Group::Unstaged, "old", DiffKind::TypeChange)
            }
        };
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let detail = compare(&mut owner, group, Path::new(path));
        assert_eq!(detail.kind, expected, "{case}: {:?}", detail.evidence);
        assert!(detail.verified);
        assert!(!detail.supports_hunks());
        if case == "rename" {
            assert_eq!(detail.change.old_path.as_deref(), Some(Path::new("old")));
            fs::write(repository.root.join("new"), "work after rename\n").unwrap();
            let context = detail.observation.context;
            owner.refresh(context).unwrap();
            assert!(finish(&mut owner).published);
            let unstaged = compare(&mut owner, Group::Unstaged, Path::new("new"));
            assert_eq!(unstaged.kind, DiffKind::Text);
            assert!(unstaged.supports_hunks());
            assert!(!contains(&unstaged.evidence, b"rename from"));
        }
        if case == "binary" {
            assert!(detail.binary);
        }
    }
    let peer = Repository::new("sha1");
    fs::write(peer.root.join("file"), "base\n").unwrap();
    peer.commit();
    let repository = Repository::new("sha1");
    repository.git([
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "add",
        peer.root.to_str().unwrap(),
        "module",
    ]);
    repository.commit();
    fs::write(repository.root.join("module/file"), "next\n").unwrap();
    repository.git([
        "-C",
        "module",
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "-c",
        "commit.gpgsign=false",
        "commit",
        "-am",
        "Next",
    ]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let detail = compare(&mut owner, Group::Unstaged, Path::new("module"));
    assert_eq!(detail.kind, DiffKind::Submodule);
    assert!(!detail.supports_hunks());
    assert!(contains(&detail.evidence, b"Subproject commit"));
}

// Caps publish an explicitly incomplete inspection, never partial actionable
// hunks. External Arc readers keep old detail charged across replacement.
#[test]
fn bounded_detail() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("file"), "before\n").unwrap();
    repository.commit();
    fs::write(repository.root.join("file"), "after\n").unwrap();
    let mut owner = repository.owner("max_detail_bytes=4096");
    repository.open(&mut owner, &repository.root);
    let retained = compare(&mut owner, Group::Unstaged, Path::new("file"));
    assert!(retained.supports_hunks());
    fs::write(repository.root.join("file"), "data\n".repeat(4096)).unwrap();
    let truncated = compare(&mut owner, Group::Unstaged, Path::new("file"));
    assert!(matches!(
        truncated.completeness,
        Completeness::Truncated { .. }
    ));
    assert!(!truncated.verified);
    assert!(!truncated.supports_hunks());
    assert!(truncated.evidence.len() <= 4096);
    assert_eq!(
        owner.records().last().unwrap().outcome,
        Outcome::OutputLimit
    );
    let before = owner.admitted_bytes();
    drop(retained);
    assert!(owner.admitted_bytes() < before);
    assert!(owner.admitted_bytes() <= rev::git::SESSION_BYTES);
}

// A Status refresh invalidates old comparison authority. A later index change
// rejects a stale selected layer and retains the last inspection for diagnosis.
#[test]
fn stale_generation() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("file"), "base\n").unwrap();
    repository.commit();
    fs::write(repository.root.join("file"), "work\n").unwrap();
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let detail = compare(&mut owner, Group::Unstaged, Path::new("file"));
    let old = Arc::clone(&detail.observation);
    owner.refresh(old.context).unwrap();
    assert!(finish(&mut owner).published);
    assert!(!owner.detail_current(&detail));
    assert!(
        owner
            .compare(
                old.context,
                old.generation,
                Group::Unstaged,
                Path::new("file")
            )
            .is_err()
    );
    let current = Arc::clone(&owner.context(old.context).unwrap().observation);
    assert!(
        owner
            .compare(
                current.context,
                current.generation,
                Group::Staged,
                Path::new("file")
            )
            .is_err()
    );
    repository.git(["add", "file"]);
    owner
        .compare(
            current.context,
            current.generation,
            Group::Unstaged,
            Path::new("file"),
        )
        .unwrap();
    assert!(!finish(&mut owner).published);
    let context = owner.context(current.context).unwrap();
    assert!(context.failure.is_none());
    assert!(context.detail_failure.is_some());
    assert!(Arc::ptr_eq(&detail, context.detail.as_ref().unwrap()));
}

// The wrapper injects only a concurrent real file edit or malformed patch bytes.
// All discovery/index checks remain installed Git, and both faults must prevent
// semantic action authority even when the child exits successfully.
#[test]
fn invalidated_comparison() {
    for case in ["changed bytes", "malformed"] {
        let repository = Repository::new("sha1");
        fs::write(repository.root.join("file"), "base\n").unwrap();
        repository.commit();
        fs::write(repository.root.join("file"), "work\n").unwrap();
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let detail = compare(&mut owner, Group::Unstaged, Path::new("file"));
        let injected = if case == "changed bytes" {
            "/usr/bin/git \"$@\"\nprintf 'newer work\\n' > file\nexit 0"
        } else {
            "printf 'diff --git a/file b/file\\n--- a/file\\n+++ b/file\\n@@ -1,3 +1,3 @@\\n-incomplete\\n'\nexit 0"
        };
        fs::write(&repository.program, format!("#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nif [ \"$3\" = diff ]; then\n{injected}\nfi\nexec /usr/bin/git \"$@\"\n")).unwrap();
        owner
            .compare(
                detail.observation.context,
                detail.observation.generation,
                Group::Unstaged,
                Path::new("file"),
            )
            .unwrap();
        let update = finish(&mut owner);
        let context = &owner.contexts()[0];
        if case == "changed bytes" {
            assert!(!update.published);
            assert!(Arc::ptr_eq(&detail, context.detail.as_ref().unwrap()));
        } else {
            assert!(update.published);
            let invalid = context.detail.as_ref().unwrap();
            assert!(matches!(invalid.kind, DiffKind::Malformed(_)));
            assert!(!invalid.verified);
            assert!(!invalid.supports_hunks());
            assert!(!invalid.lines.is_empty());
        }
        assert_eq!(owner.records().last().unwrap().outcome, Outcome::Failed);
    }
}
