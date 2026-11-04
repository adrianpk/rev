#[path = "history_inspection_presentation.rs"]
mod presentation;
use super::*;
use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

fn reflog(owner: &mut Repositories) -> Arc<rev::git::Reflog> {
    let id = owner.contexts()[0].id;
    owner.reflog(id).unwrap();
    let u = finish(owner);
    assert!(u.published, "{:?}", owner.record(u.operation));
    owner.context(id).unwrap().reflog.as_ref().unwrap().clone()
}

// Opposite endpoint order produces opposite real Git changes; range endpoints
// need not have a direct parent relation, and inspection never changes the tree.
#[test]
fn exact_ranges() {
    for format in ["sha1", "sha256"] {
        let r = Repository::new(format);
        for n in 0..4 {
            fs::write(r.root.join("source"), format!("value {n}\n")).unwrap();
            r.commit();
        }
        let head = r.git(["rev-parse", "HEAD"]);
        let index = r.git(["ls-files", "--stage", "-z"]);
        let mut owner = r.owner("");
        r.open(&mut owner, &r.root);
        let h = history(&mut owner);
        let end = h.revisions().next().unwrap().object;
        let base = h.revisions().nth(3).unwrap().object;
        for (base, end, removed, added) in [
            (base, end, b"-value 0".as_slice(), b"+value 3".as_slice()),
            (end, base, b"-value 3".as_slice(), b"+value 0".as_slice()),
        ] {
            owner.show_range(h.clone(), base, end).unwrap();
            let u = finish(&mut owner);
            assert!(u.published, "{:?}", owner.record(u.operation));
            let d = owner
                .context(h.observation.context)
                .unwrap()
                .revision
                .as_ref()
                .unwrap();
            assert!(d.range);
            assert_eq!(d.parent, Some(base));
            assert_eq!(d.object, end);
            assert!(d.patch.windows(removed.len()).any(|p| p == removed));
            assert!(d.patch.windows(added.len()).any(|p| p == added));
        }
        assert_eq!(r.git(["rev-parse", "HEAD"]), head);
        assert_eq!(r.git(["ls-files", "--stage", "-z"]), index);
        assert_eq!(fs::read(r.root.join("source")).unwrap(), b"value 3\n");
    }
}

// Reflog selectors may move while exact objects remain the inspection target.
// A second engine with equal numeric context IDs cannot supply its entry owner.
#[test]
fn reflog_objects() {
    let r = Repository::new("sha256");
    r.commit();
    r.commit();
    let mut owner = r.owner("");
    r.open(&mut owner, &r.root);
    let first = reflog(&mut owner);
    assert_eq!(first.entries.len(), 2);
    let target = first.entries[1].object;
    r.git(["reset", "--soft", "HEAD"]);
    let current = reflog(&mut owner);
    assert_eq!(current.entries.len(), 3);
    assert_eq!(current.entries[2].object, target);
    assert!(owner.show_reflog(first, target).is_err());
    owner.show_reflog(current.clone(), target).unwrap();
    let u = finish(&mut owner);
    assert!(u.published, "{:?}", owner.record(u.operation));
    assert_eq!(
        owner
            .context(current.observation.context)
            .unwrap()
            .revision
            .as_ref()
            .unwrap()
            .object,
        target
    );
    let mut foreign = r.owner("");
    r.open(&mut foreign, &r.root);
    let outside = reflog(&mut foreign);
    assert!(owner.show_reflog(outside, target).is_err());
}

// Installed Git follows a whole-file rename. Each line keeps its originating
// object/path/location, and historical blobs preserve exact bytes independently
// of the current path, unstaged disk bytes and the canonical visiting source.
#[test]
fn blame_renamed_source() {
    for format in ["sha1", "sha256"] {
        let r = Repository::new(format);
        let old = PathBuf::from(OsString::from_vec(b"old\t\"\\\xff\nname".to_vec()));
        let new = PathBuf::from(OsString::from_vec(b"new\t\"\\\xfe\nname".to_vec()));
        fs::write(r.root.join(&old), b"one\ntwo\nthree").unwrap();
        r.commit();
        let initial = object(&r, "HEAD");
        r.git([
            OsString::from("mv"),
            old.as_os_str().to_owned(),
            new.as_os_str().to_owned(),
        ]);
        r.commit();
        fs::write(r.root.join(&new), b"one\nCHANGED\nthree").unwrap();
        r.commit();
        let changed = object(&r, "HEAD");
        fs::write(r.root.join(&new), b"UNSTAGED DISK\n").unwrap();
        let index = r.git(["ls-files", "--stage", "-z"]);
        let mut owner = r.owner("");
        r.open(&mut owner, &r.root);
        let id = owner.contexts()[0].id;
        owner.blame(id, &new).unwrap();
        let u = finish(&mut owner);
        assert!(u.published, "{:?}", owner.record(u.operation));
        let b = owner.context(id).unwrap().blame.as_ref().unwrap().clone();
        assert_eq!(b.lines.len(), 3);
        assert_eq!(b.completeness, Completeness::Complete);
        for (i, line) in b.lines.iter().enumerate() {
            assert_eq!(line.final_line, i + 1);
            assert_eq!(line.original_line, i + 1);
            assert_eq!(&line.object.hex(), if i == 1 { &changed } else { &initial });
            assert_eq!(line.path, if i == 1 { new.clone() } else { old.clone() });
            assert_eq!(line.author, b"Example Author");
        }
        owner.inspect_blame(b.clone(), 3, true).unwrap();
        let u = finish(&mut owner);
        assert!(u.published, "{:?}", owner.record(u.operation));
        let file = owner.context(id).unwrap().revision_file.as_ref().unwrap();
        assert_eq!(file.object.hex(), initial);
        assert_eq!(file.path, old);
        assert_eq!(file.current_path, Some(new.clone()));
        assert_eq!(file.original_line, 3);
        assert_eq!(file.bytes, b"one\ntwo\nthree");
        owner.inspect_blame(b, 2, false).unwrap();
        assert!(finish(&mut owner).published);
        assert_eq!(
            owner
                .context(id)
                .unwrap()
                .revision
                .as_ref()
                .unwrap()
                .object
                .hex(),
            changed
        );
        assert_eq!(r.git(["ls-files", "--stage", "-z"]), index);
        assert_eq!(object(&r, "HEAD"), changed);
        assert_eq!(fs::read(r.root.join(new)).unwrap(), b"UNSTAGED DISK\n");
    }
}
