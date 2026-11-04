use super::{change, compare};
use crate::support::{Repository, finish};
use rev::git::{Effects, Group, IndexDirection, IndexTarget, Outcome};
use std::{ffi::OsString, fs, os::unix::ffi::OsStringExt, path::Path};

// One literal NUL manifest changes the complete path group in each direction,
// including unborn HEAD, raw names and binary files; unmarked entries survive.
#[test]
fn paths() {
    for born in [false, true] {
        let repository = Repository::new("sha1");
        let raw = OsString::from_vec(b":(glob)raw\n\xff".to_vec());
        for name in [Path::new("a"), Path::new(&raw), Path::new("other")] {
            if !born || name != Path::new(&raw) {
                fs::write(repository.root.join(name), b"base\0\n").unwrap();
            }
        }
        if born {
            repository.commit();
        }
        for name in [Path::new("a"), Path::new(&raw), Path::new("other")] {
            fs::write(repository.root.join(name), b"changed\0\n").unwrap();
        }
        let initial = repository.git(["ls-files", "--stage", "-z"]);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let targets: Vec<_> = [Path::new("a"), Path::new(&raw)]
            .iter()
            .map(|p| {
                IndexTarget::Path(compare(
                    &mut owner,
                    if born && *p == Path::new("a") {
                        Group::Unstaged
                    } else {
                        Group::Untracked
                    },
                    p,
                ))
            })
            .collect();
        let operation = owner
            .change_index_group(IndexDirection::Stage, &targets)
            .unwrap();
        assert!(finish(&mut owner).published);
        let record = owner.record(operation).unwrap();
        assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
        assert!(record.argv.iter().any(|a| a == "--pathspec-from-file=-"));
        assert_eq!(repository.git(["show", ":a"]), b"changed\0\n");
        if born {
            assert_eq!(repository.git(["show", ":other"]), b"base\0\n");
        }
        let targets: Vec<_> = [Path::new("a"), Path::new(&raw)]
            .iter()
            .map(|p| IndexTarget::Path(compare(&mut owner, Group::Staged, p)))
            .collect();
        let operation = owner
            .change_index_group(IndexDirection::Unstage, &targets)
            .unwrap();
        assert!(finish(&mut owner).published);
        assert_eq!(owner.record(operation).unwrap().effects, Effects::Verified);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), initial);
        for name in [Path::new("a"), Path::new(&raw), Path::new("other")] {
            assert_eq!(
                fs::read(repository.root.join(name)).unwrap(),
                b"changed\0\n"
            );
        }
    }
}

// A complete group rejects incompatible, duplicate and stale members without
// launching effects on its otherwise-valid first member. Binary hash headers
// detect byte changes even when the porcelain flags and binary marker match.
#[test]
fn rejection() {
    let repository = Repository::new("sha1");
    for name in ["a", "b"] {
        fs::write(repository.root.join(name), b"base\0\n").unwrap();
    }
    repository.commit();
    for name in ["a", "b"] {
        fs::write(repository.root.join(name), b"change\0\n").unwrap();
    }
    let initial = repository.git(["ls-files", "--stage", "-z"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let a = IndexTarget::Path(compare(&mut owner, Group::Unstaged, Path::new("a")));
    let b = IndexTarget::Path(compare(&mut owner, Group::Unstaged, Path::new("b")));
    assert!(
        owner
            .change_index_group(IndexDirection::Stage, &[a.clone(), a.clone()])
            .is_err()
    );
    assert!(
        owner
            .change_index_group(IndexDirection::Unstage, &[a.clone(), b.clone()])
            .is_err()
    );
    assert!(
        owner
            .change_index_group(IndexDirection::Stage, &vec![a.clone(); 257])
            .is_err()
    );
    assert!(!owner.busy());
    fs::write(repository.root.join("b"), b"changed-again\0\n").unwrap();
    let operation = owner
        .change_index_group(IndexDirection::Stage, &[a.clone(), b])
        .unwrap();
    let update = finish(&mut owner);
    assert!(!update.published);
    assert_eq!(
        owner.record(operation).unwrap().effects,
        Effects::NotStarted
    );
    assert_eq!(repository.git(["ls-files", "--stage", "-z"]), initial);
    owner.refresh(owner.contexts()[0].id).unwrap();
    finish(&mut owner);
    let fresh = IndexTarget::Path(compare(&mut owner, Group::Unstaged, Path::new("a")));
    assert!(
        owner
            .change_index_group(IndexDirection::Stage, &[fresh, a])
            .is_err()
    );
    assert_eq!(repository.git(["ls-files", "--stage", "-z"]), initial);
}

// Nonadjacent hunks form one cached patch. Insertions/deletions in the first
// hunk must not shift the independently predicted later index range.
#[test]
fn hunks() {
    let repository = Repository::new("sha256");
    let base: Vec<_> = (0..80).map(|n| format!("line {n:02}\n")).collect();
    fs::write(repository.root.join("a"), base.concat()).unwrap();
    repository.commit();
    let mut working = base.clone();
    working.splice(4..5, ["first\n".into(), "extra\n".into()]);
    working[40] = "unmarked\n".into();
    working[70] = "last\n".into();
    fs::write(repository.root.join("a"), working.concat()).unwrap();
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let diff = compare(&mut owner, Group::Unstaged, Path::new("a"));
    assert_eq!(diff.hunks.len(), 3);
    let first = IndexTarget::Hunk {
        diff: diff.clone(),
        hunk: diff.hunks[0].id,
    };
    let last = IndexTarget::Hunk {
        diff: diff.clone(),
        hunk: diff.hunks[2].id,
    };
    assert!(
        owner
            .change_index_group(
                IndexDirection::Stage,
                &[first.clone(), IndexTarget::Path(diff.clone())]
            )
            .is_err()
    );
    assert!(
        owner
            .change_index_group(IndexDirection::Stage, &[first.clone(), first.clone()])
            .is_err()
    );
    let foreign = compare(&mut owner, Group::Unstaged, Path::new("a"));
    let foreign = IndexTarget::Hunk {
        hunk: foreign.hunks[2].id,
        diff: foreign,
    };
    assert!(
        owner
            .change_index_group(IndexDirection::Stage, &[first.clone(), foreign])
            .is_err()
    );
    let operation = owner
        .change_index_group(IndexDirection::Stage, &[last, first])
        .unwrap();
    assert!(finish(&mut owner).published);
    assert_eq!(owner.record(operation).unwrap().outcome, Outcome::Verified);
    let mut expected = working.clone();
    expected[40] = base[39].clone();
    assert_eq!(repository.git(["show", ":a"]), expected.concat().as_bytes());
    let staged = compare(&mut owner, Group::Staged, Path::new("a"));
    let targets: Vec<_> = staged
        .hunks
        .iter()
        .map(|h| IndexTarget::Hunk {
            diff: staged.clone(),
            hunk: h.id,
        })
        .collect();
    let operation = owner
        .change_index_group(IndexDirection::Unstage, &targets)
        .unwrap();
    assert!(finish(&mut owner).published);
    assert_eq!(owner.record(operation).unwrap().outcome, Outcome::Verified);
    assert_eq!(repository.git(["show", ":a"]), base.concat().as_bytes());
    assert_eq!(
        fs::read(repository.root.join("a")).unwrap(),
        working.concat().as_bytes()
    );
    // The same native single-target route still works after group publication.
    let diff = compare(&mut owner, Group::Unstaged, Path::new("a"));
    change(&mut owner, IndexDirection::Stage, IndexTarget::Path(diff));
}
