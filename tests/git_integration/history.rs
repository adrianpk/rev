#[path = "history_inspection.rs"]
mod inspection;
#[path = "history_presentation.rs"]
mod presentation;

use super::support::{Repository, finish};
use rev::git::{Completeness, History, ObjectId, Repositories};
use std::{fs, sync::Arc};

fn history(owner: &mut Repositories) -> Arc<History> {
    let context = owner.contexts()[0].id;
    owner.log(context, None).unwrap();
    assert!(finish(owner).published, "{:?}", owner.records());
    owner
        .context(context)
        .unwrap()
        .history
        .as_ref()
        .unwrap()
        .clone()
}
fn object(repository: &Repository, revision: &str) -> String {
    String::from_utf8(repository.git(["rev-parse", revision]))
        .unwrap()
        .trim()
        .to_owned()
}

// Exact root/count continuation preserves all ancestors without duplicates in
// both hash formats; refresh retains pages, and external HEAD movement rejects.
#[test]
fn ancestry_pages() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        for n in 0..7 {
            fs::write(repository.root.join("file"), format!("version {n}\n")).unwrap();
            repository.commit();
        }
        let before = repository.git(["status", "--porcelain=v2", "-z"]);
        let head = object(&repository, "HEAD");
        let mut owner = repository.owner("history_page_size=2\nmax_history_pages=3\n");
        repository.open(&mut owner, &repository.root);
        let mut h = history(&mut owner);
        assert_eq!(h.revisions().next().unwrap().object.hex(), head);
        assert_eq!(h.revisions().count(), 2);
        for _ in 0..2 {
            owner.continue_history(h.clone(), true).unwrap();
            assert!(finish(&mut owner).published);
            h = owner
                .context(h.observation.context)
                .unwrap()
                .history
                .as_ref()
                .unwrap()
                .clone();
        }
        assert_eq!(h.pages.len(), 3);
        assert!(h.more && h.page_limit);
        assert!(
            owner
                .continue_history(h.clone(), true)
                .unwrap_err()
                .contains("limit")
        );
        let expected = String::from_utf8(repository.git([
            "rev-list",
            "--topo-order",
            "--max-count=6",
            "HEAD",
        ]))
        .unwrap();
        assert_eq!(
            h.revisions().map(|r| r.object.hex()).collect::<Vec<_>>(),
            expected.lines().collect::<Vec<_>>()
        );
        owner.continue_history(h.clone(), false).unwrap();
        assert!(finish(&mut owner).published);
        let refreshed = owner
            .context(h.observation.context)
            .unwrap()
            .history
            .as_ref()
            .unwrap()
            .clone();
        assert_eq!(refreshed.pages.len(), 3);
        assert!(Arc::ptr_eq(&h.pages[0], &refreshed.pages[0]));
        assert_eq!(repository.git(["status", "--porcelain=v2", "-z"]), before);
        assert_eq!(object(&repository, "HEAD"), head);
        repository.commit();
        owner.continue_history(refreshed.clone(), false).unwrap();
        assert!(!finish(&mut owner).published);
        assert!(Arc::ptr_eq(
            owner
                .context(h.observation.context)
                .unwrap()
                .history
                .as_ref()
                .unwrap(),
            &refreshed
        ));
        assert!(owner.admitted_bytes() < rev::git::SESSION_BYTES);
    }
}

// Merge parent comparisons bind either complete parent ID. Historical renames
// keep exact old/new paths without changing source, index or HEAD.
#[test]
fn revisions_and_parents() {
    let repository = Repository::new("sha256");
    fs::write(repository.root.join("old name"), "original\n").unwrap();
    repository.commit();
    repository.git(["switch", "-c", "side"]);
    fs::write(repository.root.join("side"), "side\n").unwrap();
    repository.commit();
    repository.git(["switch", "main"]);
    repository.git(["mv", "old name", "new name"]);
    repository.commit();
    let rename = object(&repository, "HEAD");
    repository.git([
        "merge",
        "--no-ff",
        "side",
        "-m",
        "Merge subject\n\nMerge body",
    ]);
    let before = repository.git(["ls-files", "--stage", "-z"]);
    let head = object(&repository, "HEAD");
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let h = history(&mut owner);
    let merge = h.revisions().next().unwrap();
    assert_eq!(merge.parents.len(), 2);
    for parent in &merge.parents {
        owner
            .show_revision(h.clone(), merge.object, Some(*parent))
            .unwrap();
        assert!(finish(&mut owner).published, "{:?}", owner.records());
        let d = owner
            .context(h.observation.context)
            .unwrap()
            .revision
            .as_ref()
            .unwrap();
        assert_eq!(d.parent, Some(*parent));
        assert_eq!(d.object.hex(), head);
        assert_eq!(d.completeness, Completeness::Complete);
        assert!(d.metadata.windows(10).any(|v| v == b"Merge body"));
        assert!(!d.paths.is_empty());
    }
    let revision = h.revisions().find(|r| r.object.hex() == rename).unwrap();
    owner
        .show_revision(
            h.clone(),
            revision.object,
            revision.parents.first().copied(),
        )
        .unwrap();
    assert!(finish(&mut owner).published);
    let d = owner
        .context(h.observation.context)
        .unwrap()
        .revision
        .as_ref()
        .unwrap();
    assert_eq!(d.paths.len(), 1);
    assert_eq!(d.paths[0].path.to_str(), Some("new name"));
    assert_eq!(
        d.paths[0].old_path.as_ref().unwrap().to_str(),
        Some("old name")
    );
    let foreign = ObjectId::parse(&[b'1'; 64], d.object.format()).unwrap();
    assert!(
        owner
            .show_revision(h.clone(), merge.object, Some(foreign))
            .is_err()
    );
    assert_eq!(object(&repository, "HEAD"), head);
    assert_eq!(repository.git(["ls-files", "--stage", "-z"]), before);
    assert_eq!(
        fs::read(repository.root.join("new name")).unwrap(),
        b"original\n"
    );
}

// Unborn history has an explicit empty state. A detached root resolves exactly;
// a missing object fails without replacing the last accepted observation.
#[test]
fn unborn_and_detached() {
    let repository = Repository::new("sha1");
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let h = history(&mut owner);
    assert!(h.root.is_none());
    assert!(!h.more);
    assert_eq!(h.revisions().count(), 0);
    repository.commit();
    repository.git(["switch", "--detach"]);
    owner.refresh(h.observation.context).unwrap();
    assert!(finish(&mut owner).published);
    owner.log(h.observation.context, Some("HEAD")).unwrap();
    assert!(finish(&mut owner).published);
    let accepted = owner
        .context(h.observation.context)
        .unwrap()
        .history
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(accepted.root.unwrap().hex(), object(&repository, "HEAD"));
    owner
        .log(h.observation.context, Some("missing-revision"))
        .unwrap();
    assert!(!finish(&mut owner).published);
    assert!(Arc::ptr_eq(
        owner
            .context(h.observation.context)
            .unwrap()
            .history
            .as_ref()
            .unwrap(),
        &accepted
    ));
}

// A shallow-boundary change can alter ancestry without moving HEAD. Paging
// rechecks the admitted prefix and refuses to skip into a different traversal.
#[test]
fn changed_ancestry() {
    let repository = Repository::new("sha1");
    for _ in 0..4 {
        repository.commit();
    }
    let mut owner = repository.owner("history_page_size=2\n");
    repository.open(&mut owner, &repository.root);
    let h = history(&mut owner);
    fs::write(
        repository.root.join(".git/shallow"),
        format!("{}\n", h.root.unwrap().hex()),
    )
    .unwrap();
    owner.continue_history(h.clone(), true).unwrap();
    let update = finish(&mut owner);
    assert!(!update.published);
    let record = owner.record(update.operation).unwrap();
    assert!(
        record
            .diagnostic
            .as_deref()
            .is_some_and(|d| d.contains("ancestry changed")),
        "{record:?}"
    );
    assert!(Arc::ptr_eq(
        owner
            .context(h.observation.context)
            .unwrap()
            .history
            .as_ref()
            .unwrap(),
        &h
    ));
}
