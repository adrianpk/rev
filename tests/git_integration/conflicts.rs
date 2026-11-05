use super::support::{Repository, finish};
use rev::git::{
    Completeness, Conflict, ConflictRole, Effects, Outcome, Repositories, Sequence, SequenceFamily,
};
use std::{ffi::OsString, fs, os::unix::ffi::OsStringExt, path::Path, process::Command, sync::Arc};

fn stopped(repository: &Repository, args: &[&str]) {
    let output = Command::new(&repository.program)
        .args(args)
        .current_dir(&repository.root)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "fixture must stop at an actual conflict"
    );
    assert!(!repository.git(["ls-files", "--unmerged", "-z"]).is_empty());
}
fn observe(owner: &mut Repositories) -> Arc<Sequence> {
    let context = owner.contexts()[0].id;
    owner.sequence(context).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(update.operation));
    owner.context(context).unwrap().sequence.clone().unwrap()
}
fn inspect(owner: &mut Repositories, sequence: Arc<Sequence>, path: &Path) -> Arc<Conflict> {
    let context = sequence.refs.observation.context;
    let id = owner.inspect_conflict(sequence, path).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(id));
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    owner.context(context).unwrap().conflict.clone().unwrap()
}
fn topic(repository: &Repository, path: &Path, base: &[u8], current: &[u8], applied: &[u8]) {
    fs::write(repository.root.join(path), base).unwrap();
    repository.commit();
    repository.git(["switch", "--create", "topic"]);
    fs::write(repository.root.join(path), applied).unwrap();
    repository.commit();
    repository.git(["switch", "main"]);
    fs::write(repository.root.join(path), current).unwrap();
    repository.commit();
}

// Real index stages in four Git-owned families retain full IDs and exact bytes
// in both object formats, including a non-UTF-8 newline/leading-dash path.
#[test]
fn operation_sides() {
    for format in ["sha1", "sha256"] {
        for family in [
            SequenceFamily::Merge,
            SequenceFamily::Rebase,
            SequenceFamily::CherryPick,
            SequenceFamily::Revert,
        ] {
            let repository = Repository::new(format);
            let path = std::path::PathBuf::from(OsString::from_vec(b"-conflict\n\xff".to_vec()));
            topic(&repository, &path, b"base\n", b"current\n", b"applied\n");
            let expected = match family {
                SequenceFamily::Merge => {
                    stopped(&repository, &["merge", "topic"]);
                    [b"base\n".as_slice(), b"current\n", b"applied\n"]
                }
                SequenceFamily::Rebase => {
                    stopped(&repository, &["rebase", "topic"]);
                    [b"base\n".as_slice(), b"applied\n", b"current\n"]
                }
                SequenceFamily::CherryPick => {
                    stopped(&repository, &["cherry-pick", "topic"]);
                    [b"base\n".as_slice(), b"current\n", b"applied\n"]
                }
                _ => {
                    repository.git(["reset", "--hard", "topic"]);
                    fs::write(repository.root.join(&path), b"current\n").unwrap();
                    repository.commit();
                    stopped(&repository, &["revert", "--no-edit", "topic"]);
                    [b"applied\n".as_slice(), b"current\n", b"base\n"]
                }
            };
            let index = repository.git(["ls-files", "--stage", "-z"]);
            let head = repository.git(["rev-parse", "HEAD"]);
            let disk = fs::read(repository.root.join(&path)).unwrap();
            let mut owner = repository.owner("");
            repository.open(&mut owner, &repository.root);
            let sequence = observe(&mut owner);
            assert_eq!(sequence.family, Some(family));
            let conflict = inspect(&mut owner, sequence, &path);
            assert_eq!(conflict.change.path, path);
            assert_eq!(conflict.kind(), "both modified");
            assert_eq!(conflict.completeness, Completeness::Complete);
            for (role, content) in ConflictRole::ALL.into_iter().zip(expected) {
                let side = conflict.side(role).unwrap();
                assert!(side.text);
                assert_eq!(side.mode, 0o100644);
                assert_eq!(side.bytes, content);
                assert_eq!(
                    repository.git(["cat-file", "blob", &side.object.hex()]),
                    content
                );
                assert!(!role.meaning(Some(family)).is_empty());
            }
            assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
            assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
            assert_eq!(fs::read(repository.root.join(&path)).unwrap(), disk);
        }
    }
}

// Deletion sides remain absent, and real binary or capped side data is explicit
// inspection data rather than silently accepted complete editable text.
#[test]
fn partial_and_missing() {
    for variant in ["delete", "binary", "partial", "symlink"] {
        let repository = Repository::new("sha1");
        let path = Path::new("file");
        if variant == "binary" {
            topic(&repository, path, b"a\0base", b"b\0current", b"c\0applied");
        } else if variant == "partial" {
            topic(
                &repository,
                path,
                &vec![b'a'; 20000],
                &vec![b'b'; 20000],
                &vec![b'c'; 20000],
            );
        } else {
            topic(&repository, path, b"base\n", b"current\n", b"applied\n");
        }
        if variant == "delete" {
            repository.git(["rm", "--", "file"]);
            repository.commit();
        }
        if variant == "symlink" {
            fs::remove_file(repository.root.join(path)).unwrap();
            std::os::unix::fs::symlink("target", repository.root.join(path)).unwrap();
            repository.commit();
        }
        stopped(&repository, &["merge", "topic"]);
        let mut owner = repository.owner(if variant == "partial" {
            "max_detail_bytes=16384"
        } else {
            ""
        });
        repository.open(&mut owner, &repository.root);
        let sequence = observe(&mut owner);
        let path = if variant == "symlink" {
            sequence
                .conflicts
                .iter()
                .find(|p| p.as_os_str() == "file")
                .unwrap()
                .clone()
        } else {
            path.into()
        };
        let conflict = inspect(&mut owner, sequence, &path);
        match variant {
            "delete" => {
                assert!(conflict.side(ConflictRole::Current).is_none());
                assert!(conflict.kind().contains("deleted"));
            }
            "binary" => {
                assert!(conflict.sides.iter().flatten().all(|s| s.binary && !s.text));
            }
            "partial" => {
                assert!(matches!(
                    conflict.completeness,
                    Completeness::Truncated { .. }
                ));
                assert!(!conflict.sides.iter().flatten().any(|s| s.text));
                assert!(
                    conflict
                        .sides
                        .iter()
                        .flatten()
                        .map(|s| s.bytes.len())
                        .sum::<usize>()
                        <= 16384
                );
            }
            _ => {
                assert!(
                    conflict
                        .sides
                        .iter()
                        .flatten()
                        .any(|s| s.mode == 0o120000 && !s.text)
                );
            }
        }
    }
}

// A saved edit or stage replacement after capture invalidates the complete
// conflict fingerprint before any side is published or Git mutation begins.
#[test]
fn stale_inspection() {
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
    repository.open(&mut owner, &repository.root);
    let sequence = observe(&mut owner);
    assert!(
        owner
            .inspect_conflict(sequence.clone(), Path::new("other"))
            .is_err()
    );
    fs::write(repository.root.join("file"), "saved resolution\n").unwrap();
    let id = owner.inspect_conflict(sequence, Path::new("file")).unwrap();
    assert!(!finish(&mut owner).published);
    let record = owner.record(id).unwrap();
    assert_eq!(record.effects, Effects::NotStarted);
    assert_eq!(record.outcome, Outcome::Failed);
    assert!(record.diagnostic.as_ref().unwrap().contains("changed"));
    assert!(!repository.git(["ls-files", "--unmerged", "-z"]).is_empty());
}

// Actual unmerged stages also exist without sequence marker files. Inspection
// keeps that absence explicit instead of inventing operation lineage.
#[test]
fn external_index() {
    let repository = Repository::new("sha256");
    topic(
        &repository,
        Path::new("file"),
        b"base\n",
        b"current\n",
        b"applied\n",
    );
    repository.git(["read-tree", "-m", "main^", "main", "topic"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let sequence = observe(&mut owner);
    assert_eq!(sequence.family, None);
    let conflict = inspect(&mut owner, sequence, Path::new("file"));
    assert_eq!(
        conflict.side(ConflictRole::Current).unwrap().bytes,
        b"current\n"
    );
    assert!(ConflictRole::Applied.meaning(None).contains("unavailable"));
}

#[path = "conflicts_presentation.rs"]
mod presentation;
