use super::*;

// Git retains linked-worktree branch protection, while invalid/existing names
// reject before a changing child and leave exact refs/HEAD intact.
#[test]
fn branch_protections() {
    let repository = Repository::new("sha1");
    repository.commit();
    repository.git(["branch", "topic"]);
    let linked = repository.fixture.0.join("linked");
    repository.git(["worktree", "add", linked.to_str().unwrap(), "topic"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    for action in [
        BranchAction::Switch {
            name: b"refs/heads/topic".to_vec(),
        },
        BranchAction::Delete {
            name: b"refs/heads/topic".to_vec(),
        },
    ] {
        let refs = observe(&repository, &mut owner);
        change(&mut owner, refs, action, false);
        assert!(
            String::from_utf8_lossy(&owner.records().last().unwrap().stderr).contains("worktree")
        );
    }
    let before = repository.git(["show-ref"]);
    for name in ["topic", "invalid name", "HEAD", "@{-1}"] {
        let refs = observe(&repository, &mut owner);
        let head = refs.head.unwrap();
        change(
            &mut owner,
            refs,
            BranchAction::Create {
                name: name.into(),
                target: head,
            },
            false,
        );
        assert_eq!(owner.records().last().unwrap().effects, Effects::NotStarted);
        assert_eq!(repository.git(["show-ref"]), before);
    }
    assert_eq!(
        repository.git(["symbolic-ref", "HEAD"]),
        b"refs/heads/main\n"
    );
    assert_eq!(
        repository.git(["-C", linked.to_str().unwrap(), "symbolic-ref", "HEAD"]),
        b"refs/heads/topic\n"
    );
}

// An actual ref move after preflight reaches Git's expected-old-object lock.
// Its rejected changing child is retained once; it is never replayed by name.
#[test]
fn moved_tag_at_lock() {
    use std::os::unix::fs::PermissionsExt;
    let repository = Repository::new("sha256");
    repository.commit();
    repository.git(["tag", "moving"]);
    let tree = repository.git(["rev-parse", "HEAD^{tree}"]);
    let head = repository.git(["rev-parse", "HEAD"]);
    let replacement = repository.git([
        "commit-tree",
        std::str::from_utf8(&tree).unwrap().trim(),
        "-p",
        std::str::from_utf8(&head).unwrap().trim(),
        "-m",
        "Replacement",
    ]);
    let replacement = std::str::from_utf8(&replacement).unwrap().trim();
    fs::write(
        &repository.program,
        format!(
            r#"#!/bin/sh
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
if [ "$2" = update-ref ] && [ "$3" = --no-deref ] && [ "$4" = -d ]; then
    printf 'delete\n' >> "$0.calls"
    /usr/bin/git update-ref "$5" {replacement} || exit 91
fi
exec /usr/bin/git "$@"
"#
        ),
    )
    .unwrap();
    fs::set_permissions(&repository.program, fs::Permissions::from_mode(0o700)).unwrap();
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&repository, &mut owner);
    let id = owner.delete_tag(refs, b"refs/tags/moving").unwrap();
    finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert_ne!(record.outcome, Outcome::Verified);
    assert_eq!(record.effects, Effects::Possible);
    assert!(
        String::from_utf8_lossy(&record.stderr).contains("cannot lock ref"),
        "{record:?}"
    );
    assert_eq!(
        repository.git(["rev-parse", "refs/tags/moving"]),
        format!("{replacement}\n").as_bytes()
    );
    assert_eq!(
        fs::read_to_string(repository.program.with_extension("calls")).unwrap(),
        "delete\n"
    );
}
