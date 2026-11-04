use super::*;
use rev::git::CommitMode;

// Amend retains the original author and parent set while replacing only the
// captured HEAD with the checked index and exact accepted subject/body.
#[test]
fn amend_objects() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("source"), "base\n").unwrap();
    repository.commit();
    let parent = repository.git(["rev-parse", "HEAD"]);
    repository.git([
        "commit",
        "--allow-empty",
        "--author=Original Author <original@example.invalid>",
        "-m",
        "Previous subject\n\nPrevious body",
    ]);
    let head = repository.git(["rev-parse", "HEAD"]);
    fs::write(repository.root.join("source"), "staged\n").unwrap();
    repository.git(["add", "source"]);
    fs::write(repository.root.join("source"), "unstaged\n").unwrap();
    let index = repository.git(["ls-files", "--stage", "-z"]);
    let mut owner = repository.owner("");
    let context = repository
        .open(&mut owner, &repository.root)
        .context
        .unwrap();
    owner.prepare_amend(context).unwrap();
    assert!(finish(&mut owner).published);
    let composition = owner.composition(context).unwrap().clone();
    assert!(
        matches!(composition.mode, CommitMode::Amend { head: id } if id.hex().as_bytes() == head.strip_suffix(b"\n").unwrap())
    );
    assert_eq!(composition.message, "Previous subject\n\nPrevious body\n");
    let accepted = "Amended λ\n\nExact replacement body";
    owner.commit_prepared(composition, accepted).unwrap();
    finish(&mut owner);
    assert_eq!(owner.records().last().unwrap().outcome, Outcome::Verified);
    assert_eq!(
        repository.git(["show", "--format=%P", "--no-patch", "HEAD"]),
        parent
    );
    assert_eq!(
        repository.git(["show", "--format=%an <%ae>", "--no-patch", "HEAD"]),
        b"Original Author <original@example.invalid>\n"
    );
    assert_eq!(
        repository.git(["show", "--format=%cn <%ce>", "--no-patch", "HEAD"]),
        b"Example Author <author@example.invalid>\n"
    );
    assert_eq!(repository.git(["show", "HEAD:source"]), b"staged\n");
    assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
    assert_eq!(
        fs::read(repository.root.join("source")).unwrap(),
        b"unstaged\n"
    );
}

// A fixup resolves a symbolic target once. Moving that ref afterward cannot
// change the accepted target or turn the action into an autosquash sequence.
// Installed Git --fixup independently supplies the exact message oracle.
#[test]
fn fixup_objects() {
    let repository = Repository::new("sha256");
    fs::write(repository.root.join("source"), "base\n").unwrap();
    repository.git(["add", "source"]);
    repository.git(["commit", "-m", "Target λ"]);
    let target = String::from_utf8(repository.git(["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    repository.git(["branch", "chosen"]);
    repository.git(["commit", "--allow-empty", "-m", "Current"]);
    let parent = repository.git(["rev-parse", "HEAD"]);
    fs::write(repository.root.join("source"), "staged\n").unwrap();
    repository.git(["add", "source"]);
    let mut owner = repository.owner("");
    let context = repository
        .open(&mut owner, &repository.root)
        .context
        .unwrap();
    let observation = owner.commit_observation(context).unwrap();
    owner.prepare_fixup(observation, "chosen").unwrap();
    assert!(finish(&mut owner).published);
    let composition = owner.composition(context).unwrap().clone();
    assert!(matches!(composition.mode, CommitMode::Fixup { target: id } if id.hex() == target));
    repository.git(["branch", "--force", "chosen", "HEAD"]);
    let accepted = format!("{}fixup body\n", composition.message);
    assert!(
        owner
            .commit_prepared(composition.clone(), "fixup! Wrong target\n")
            .is_err()
    );
    owner.commit_prepared(composition, &accepted).unwrap();
    finish(&mut owner);
    assert_eq!(owner.records().last().unwrap().outcome, Outcome::Verified);
    assert_eq!(
        repository.git(["show", "--format=%P", "--no-patch", "HEAD"]),
        parent
    );
    assert_eq!(repository.git(["rev-list", "--count", "HEAD"]), b"3\n");
    let actual = repository.git(["cat-file", "commit", "HEAD"]);
    let boundary = actual.windows(2).position(|p| p == b"\n\n").unwrap();
    let message = &actual[boundary + 2..];
    repository.git([
        "commit",
        "--allow-empty",
        &format!("--fixup={target}"),
        "-m",
        "fixup body",
    ]);
    let oracle = repository.git(["cat-file", "commit", "HEAD"]);
    let boundary = oracle.windows(2).position(|p| p == b"\n\n").unwrap();
    assert_eq!(message, &oracle[boundary + 2..]);
}

// Preparation is read-only and rejects an unborn amend or missing revision.
// A reviewed amend cannot silently adopt a HEAD moved before submission.
#[test]
fn prepared_preconditions() {
    let repository = Repository::new("sha1");
    let mut owner = repository.owner("");
    let context = repository
        .open(&mut owner, &repository.root)
        .context
        .unwrap();
    assert!(owner.prepare_amend(context).is_err());
    let observation = owner.commit_observation(context).unwrap();
    owner
        .prepare_fixup(observation, "--not-a-revision")
        .unwrap();
    assert!(!finish(&mut owner).published);
    assert_eq!(owner.records().last().unwrap().effects, Effects::NotStarted);
    fs::write(repository.root.join("source"), "base\n").unwrap();
    repository.commit();
    owner.refresh(context).unwrap();
    finish(&mut owner);
    owner.prepare_amend(context).unwrap();
    finish(&mut owner);
    let composition = owner.composition(context).unwrap().clone();
    repository.git(["commit", "--allow-empty", "-m", "External"]);
    let head = repository.git(["rev-parse", "HEAD"]);
    owner
        .commit_prepared(composition, "Retained amended body")
        .unwrap();
    finish(&mut owner);
    assert_eq!(owner.records().last().unwrap().effects, Effects::NotStarted);
    assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
    assert_eq!(owner.commit_draft(context), Some("Retained amended body"));
}
