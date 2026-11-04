use super::support::{Repository, finish};
use rev::git::{BranchAction, Completeness, Effects, Outcome, RefKind, Refs, Repositories};
use std::{fs, sync::Arc};

fn observe(repository: &Repository, owner: &mut Repositories) -> Arc<Refs> {
    let id = owner.contexts().first().unwrap().id;
    owner.refs(id).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(update.operation));
    assert_eq!(
        owner.record(update.operation).unwrap().outcome,
        Outcome::Verified
    );
    let refs = owner.context(id).unwrap().refs.as_ref().unwrap().clone();
    assert!(owner.check_refs(&refs).is_ok());
    assert!(
        refs.observation
            .identity
            .worktree
            .as_ref()
            .is_none_or(|p| p == &repository.root)
    );
    refs
}
fn change(owner: &mut Repositories, refs: Arc<Refs>, action: BranchAction, success: bool) {
    let id = owner.branch(refs, action).unwrap();
    finish(owner);
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome == Outcome::Verified, success, "{record:?}");
    assert_eq!(record.effects == Effects::Verified, success, "{record:?}");
}

// Installed Git supplies full branch/tag identities and upstream state in both
// object formats. Credential-bearing remote values never enter public refs.
#[test]
fn exact_refs() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        repository.commit();
        repository.git(["branch", "topic"]);
        repository.git(["tag", "plain"]);
        repository.git(["tag", "--annotate", "annotated", "--message=Annotation"]);
        repository.git([
            "config",
            "remote.origin.url",
            "https://secret:password@example.invalid/private",
        ]);
        repository.git([
            "config",
            "remote.origin.fetch",
            "+refs/heads/*:refs/remotes/origin/*",
        ]);
        repository.git(["update-ref", "refs/remotes/origin/main", "HEAD"]);
        repository.git(["branch", "--set-upstream-to=origin/main", "main"]);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let refs = observe(&repository, &mut owner);
        assert_eq!(refs.completeness, Completeness::Complete);
        assert_eq!(refs.head_ref, b"refs/heads/main");
        assert_eq!(refs.remotes, vec![b"origin".to_vec()]);
        let main = refs.reference(b"refs/heads/main").unwrap();
        assert_eq!(main.upstream, b"refs/remotes/origin/main");
        assert!(main.tracking.is_empty());
        let tracking = refs.reference(b"refs/remotes/origin/main").unwrap();
        assert_eq!(tracking.kind, RefKind::Tracking);
        assert_eq!(tracking.object, main.object);
        let plain = refs.reference(b"refs/tags/plain").unwrap();
        let annotated = refs.reference(b"refs/tags/annotated").unwrap();
        assert_eq!(plain.object, main.object);
        assert_eq!(plain.peeled, None);
        assert_eq!(annotated.object_type, b"tag");
        assert_ne!(annotated.object, main.object);
        assert_eq!(annotated.peeled, Some(main.object));
        assert_eq!(annotated.peeled_type, b"commit");
        assert!(!format!("{refs:?}").contains("password"));
    }
}

// One-shot checked branch requests create, switch, rename and delete the
// accepted full objects without touching unrelated index/worktree content.
#[test]
fn branch_lifecycle() {
    let repository = Repository::new("sha256");
    repository.commit();
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&repository, &mut owner);
    let head = refs.head.unwrap();
    change(
        &mut owner,
        refs,
        BranchAction::Create {
            name: "topic".into(),
            target: head,
        },
        true,
    );
    let refs = observe(&repository, &mut owner);
    change(
        &mut owner,
        refs,
        BranchAction::Switch {
            name: b"refs/heads/topic".to_vec(),
        },
        true,
    );
    let refs = observe(&repository, &mut owner);
    change(
        &mut owner,
        refs,
        BranchAction::Rename {
            name: b"refs/heads/topic".to_vec(),
            new_name: "renamed".into(),
        },
        true,
    );
    assert_eq!(
        repository.git(["symbolic-ref", "HEAD"]),
        b"refs/heads/renamed\n"
    );
    let refs = observe(&repository, &mut owner);
    change(
        &mut owner,
        refs,
        BranchAction::Switch {
            name: b"refs/heads/main".to_vec(),
        },
        true,
    );
    let refs = observe(&repository, &mut owner);
    change(
        &mut owner,
        refs,
        BranchAction::Delete {
            name: b"refs/heads/renamed".to_vec(),
        },
        true,
    );
    assert_eq!(
        repository.git(["rev-parse", "HEAD"]),
        format!("{}\n", head.hex()).as_bytes()
    );
    assert!(repository.git(["status", "--porcelain"]).is_empty());
}

// Stale full-ref observations, current/unmerged deletion and dirty switch
// requests fail before any changing child. Refresh restores useful authority.
#[test]
fn branch_guards() {
    let repository = Repository::new("sha1");
    repository.commit();
    repository.git(["branch", "topic"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&repository, &mut owner);
    repository.git(["branch", "external"]);
    change(
        &mut owner,
        refs,
        BranchAction::Switch {
            name: b"refs/heads/topic".to_vec(),
        },
        false,
    );
    let refs = observe(&repository, &mut owner);
    change(
        &mut owner,
        refs,
        BranchAction::Delete {
            name: b"refs/heads/main".to_vec(),
        },
        false,
    );
    fs::write(repository.root.join("dirty"), "unsaved disk\n").unwrap();
    let refs = observe(&repository, &mut owner);
    change(
        &mut owner,
        refs,
        BranchAction::Switch {
            name: b"refs/heads/topic".to_vec(),
        },
        false,
    );
    fs::remove_file(repository.root.join("dirty")).unwrap();
    repository.git(["switch", "topic"]);
    repository.commit();
    repository.git(["switch", "main"]);
    let refs = observe(&repository, &mut owner);
    change(
        &mut owner,
        refs,
        BranchAction::Delete {
            name: b"refs/heads/topic".to_vec(),
        },
        false,
    );
    assert_eq!(
        repository.git(["symbolic-ref", "HEAD"]),
        b"refs/heads/main\n"
    );
    assert!(!repository.git(["show-ref", "refs/heads/topic"]).is_empty());
}

// Ref/remotes count admission is explicit; incomplete lists cannot authorize
// mutations. Unborn, detached and bare identities remain distinguishable.
#[test]
fn ref_limits_and_heads() {
    let repository = Repository::new("sha1");
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&repository, &mut owner);
    assert_eq!(refs.head, None);
    assert_eq!(refs.head_ref, b"refs/heads/main");
    repository.commit();
    repository.git(["branch", "extra"]);
    let mut owner = repository.owner("max_refs=1");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&repository, &mut owner);
    assert_ne!(refs.completeness, Completeness::Complete);
    assert!(
        owner
            .branch(
                refs.clone(),
                BranchAction::Create {
                    name: "rejected".into(),
                    target: refs.head.unwrap()
                }
            )
            .is_err()
    );
    repository.git(["switch", "--detach"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&repository, &mut owner);
    assert!(refs.head_ref.is_empty());
    assert!(refs.head.is_some());
    let bare = repository.fixture.0.join("bare.git");
    repository.git([
        "clone",
        "--bare",
        repository.root.to_str().unwrap(),
        bare.to_str().unwrap(),
    ]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &bare);
    let refs = observe(&repository, &mut owner);
    assert!(refs.observation.identity.worktree.is_none());
    assert!(refs.head.is_some());
}

#[path = "refs_presentation.rs"]
mod presentation;

// A real external checkout between the Status HEAD check and ref collection
// cannot publish a ref set under the preceding, incompatible Status owner.
#[test]
fn head_between_reads() {
    use std::os::unix::fs::PermissionsExt;
    let repository = Repository::new("sha1");
    repository.commit();
    repository.git(["branch", "other"]);
    fs::write(
        &repository.program,
        r#"#!/bin/sh
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
if [ "$2" = symbolic-ref ]; then
    count=0
    if [ -f "$0.state" ]; then read -r count < "$0.state"; fi
    count=$((count + 1))
    printf '%s\n' "$count" > "$0.state"
    if [ "$count" = 3 ]; then /usr/bin/git switch --quiet other || exit 91; fi
fi
exec /usr/bin/git "$@"
"#,
    )
    .unwrap();
    fs::set_permissions(&repository.program, fs::Permissions::from_mode(0o700)).unwrap();
    let mut owner = repository.owner("");
    let update = repository.open(&mut owner, &repository.root);
    let context = update.context.unwrap();
    let id = owner.refs(context).unwrap();
    let update = finish(&mut owner);
    assert!(
        !update.published,
        "incompatible refs published: {:?}",
        owner.context(context).unwrap().refs
    );
    let record = owner.record(id).unwrap();
    assert_eq!(record.effects, Effects::NotStarted);
    assert_eq!(record.outcome, Outcome::Failed);
    assert_eq!(
        repository.git(["symbolic-ref", "HEAD"]),
        b"refs/heads/other\n"
    );
}

#[path = "refs_tags.rs"]
mod tags;
