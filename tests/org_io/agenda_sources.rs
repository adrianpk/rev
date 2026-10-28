use super::Fixture;
use rev::{
    buffer::{Buffers, edit::Replacement},
    config::OrgLimits,
    org::{
        Cancellation,
        agenda::{SourceScan, SourceSelection, SourceVersion},
    },
};
use std::{fs, os::unix::fs::symlink};
fn scan(
    buffers: &Buffers,
    paths: Vec<std::path::PathBuf>,
    limits: &OrgLimits,
) -> rev::org::agenda::SourceSet {
    SourceScan::new(
        buffers,
        SourceSelection {
            paths,
            buffers: Vec::new(),
        },
        limits,
    )
    .unwrap()
    .run(&Cancellation::default())
}
// Explicit roots alone select sources. Immediate .org children are observed;
// unrelated and nested files are never read, and directory aliases terminate.
#[test]
fn agenda_declared_roots() {
    let f = Fixture::new();
    let a = f.write("chosen/a.org", "\u{feff}* TODO A\r\n");
    f.write("chosen/nested/b.org", "* Hidden");
    f.write("chosen/skip.txt", "not org");
    let unrelated = f.write("unrelated.org", "* Unselected");
    symlink(f.0.join("chosen"), f.0.join("alias")).unwrap();
    let buffers = Buffers::new().unwrap();
    let empty = scan(&buffers, Vec::new(), &OrgLimits::default());
    assert!(empty.complete());
    assert!(empty.documents.is_empty());
    let result = scan(
        &buffers,
        vec![f.0.join("chosen"), f.0.join("alias")],
        &OrgLimits::default(),
    );
    assert!(result.complete(), "{}", result.feedback());
    assert_eq!(result.documents.len(), 1);
    assert_eq!(result.documents[0].path.as_ref(), Some(&a));
    assert_eq!(result.documents[0].text().to_string(), "* TODO A\r\n");
    assert_eq!(fs::read_to_string(unrelated).unwrap(), "* Unselected");
}
// Canonical symlink and inode aliases reuse unsaved source. Multiple live
// hard-link buffers are diagnosed instead of arbitrarily choosing one owner.
#[test]
fn agenda_buffer_identity() {
    let f = Fixture::new();
    let path = f.write("task.org", "* Old");
    let alias = f.0.join("alias.org");
    symlink(&path, &alias).unwrap();
    let link = f.0.join("hard.org");
    fs::hard_link(&path, &link).unwrap();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let b = buffers.get_mut(id).unwrap();
    let edit = Replacement {
        source: b.span(0..b.len_chars()).unwrap(),
        text: "* TODO Unsaved 界".into(),
    };
    let prepared = b.prepare(&[edit], &[], None).unwrap();
    b.publish(prepared, &mut []).unwrap();
    let result = scan(
        &buffers,
        vec![alias, link.clone(), path.clone()],
        &OrgLimits {
            agenda_files: 1,
            ..OrgLimits::default()
        },
    );
    assert!(result.complete(), "{}", result.feedback());
    assert_eq!(result.documents.len(), 1);
    assert_eq!(result.documents[0].text().to_string(), "* TODO Unsaved 界");
    assert!(
        matches!(result.documents[0].version,SourceVersion::Buffer(stamp) if stamp==buffers.get(id).unwrap().stamp())
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), "* Old");
    buffers.visit(&link).unwrap();
    let conflict = scan(&buffers, vec![path], &OrgLimits::default());
    assert!(!conflict.complete());
    assert!(conflict.documents.is_empty());
    assert!(conflict.feedback().contains("Multiple open buffers"));
}
// Source failures cannot become a complete empty success. Available whole
// documents survive invalid bytes, absent files, symlink cycles and exact limits.
#[test]
fn agenda_partial_sources() {
    let f = Fixture::new();
    let good = f.write("good.org", "* Good");
    let invalid = f.0.join("invalid.org");
    fs::write(&invalid, [0xff, 0]).unwrap();
    let cycle = f.0.join("cycle.org");
    symlink(&cycle, &cycle).unwrap();
    let buffers = Buffers::new().unwrap();
    for (name, path) in [
        ("missing", f.0.join("absent.org")),
        ("encoding", invalid),
        ("cycle", cycle),
    ] {
        let result = scan(&buffers, vec![path, good.clone()], &OrgLimits::default());
        assert!(!result.complete(), "{name}");
        assert_eq!(result.documents.len(), 1, "{name}");
        assert!(!result.issues.is_empty(), "{name}");
        assert!(result.feedback().contains("incomplete"));
    }
    for (name, limits) in [
        (
            "file",
            OrgLimits {
                agenda_file_bytes: 5,
                ..OrgLimits::default()
            },
        ),
        (
            "total",
            OrgLimits {
                agenda_total_bytes: 5,
                ..OrgLimits::default()
            },
        ),
    ] {
        let result = scan(&buffers, vec![good.clone()], &limits);
        assert!(result.truncated, "{name}");
        assert!(!result.complete(), "{name}");
        assert!(result.documents.is_empty());
    }
    let exact = scan(
        &buffers,
        vec![good.clone()],
        &OrgLimits {
            agenda_file_bytes: 6,
            agenda_total_bytes: 6,
            ..OrgLimits::default()
        },
    );
    assert!(exact.complete(), "{}", exact.feedback());
    let other = f.write("other.org", "* Other");
    let count = scan(
        &buffers,
        vec![good, other],
        &OrgLimits {
            agenda_files: 1,
            ..OrgLimits::default()
        },
    );
    assert!(count.truncated);
    assert_eq!(count.documents.len(), 1);
}
