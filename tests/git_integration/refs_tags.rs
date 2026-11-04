use super::*;
use rev::git::{TagIntent, TagKind};

fn prepare(owner: &mut Repositories, refs: Arc<Refs>, name: &str, kind: TagKind) -> Arc<TagIntent> {
    let context = refs.observation.context;
    let head = refs.head.unwrap();
    let id = owner.prepare_tag(refs, name, kind, head).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(id));
    owner.tag_intent(context).unwrap().clone()
}
fn create(owner: &mut Repositories, intent: Arc<TagIntent>, text: &str, success: bool) {
    let id = owner.create_tag(intent, text).unwrap();
    finish(owner);
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome == Outcome::Verified, success, "{record:?}");
}

// Both creation kinds retain their exact target; annotations contain Git's
// tagger and accepted message. Tag deletion uses the actual tag object.
#[test]
fn tag_kinds() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        repository.commit();
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let refs = observe(&repository, &mut owner);
        let head = refs.head.unwrap();
        let intent = prepare(&mut owner, refs, "plain", TagKind::Lightweight);
        create(&mut owner, intent, "", true);
        assert_eq!(
            repository.git(["rev-parse", "refs/tags/plain"]),
            format!("{}\n", head.hex()).as_bytes()
        );
        let refs = observe(&repository, &mut owner);
        let intent = prepare(&mut owner, refs, "annotation", TagKind::Annotated);
        create(&mut owner, intent, "Subject\n\nBody with literal c", true);
        let raw = repository.git(["cat-file", "tag", "refs/tags/annotation"]);
        let text = String::from_utf8(raw).unwrap();
        assert!(text.starts_with(&format!("object {}\ntype commit\ntag annotation\ntagger Example Author <author@example.invalid>",head.hex())));
        assert!(text.ends_with("Subject\n\nBody with literal c"), "{text:?}");
        let refs = observe(&repository, &mut owner);
        let tag = refs.reference(b"refs/tags/annotation").unwrap();
        assert_ne!(tag.object, head);
        assert_eq!(tag.peeled, Some(head));
        for name in [b"refs/tags/plain".as_slice(), b"refs/tags/annotation"] {
            let refs = observe(&repository, &mut owner);
            let id = owner.delete_tag(refs, name).unwrap();
            finish(&mut owner);
            assert_eq!(owner.record(id).unwrap().outcome, Outcome::Verified);
        }
        assert!(repository.git(["tag", "--list"]).is_empty());
    }
}

// Configured signing cannot turn the selected lightweight kind into an
// annotation. Actual signer failure keeps the draft and changing diagnostic.
#[test]
fn signing_and_draft() {
    let repository = Repository::new("sha1");
    repository.commit();
    repository.git(["config", "tag.gpgsign", "true"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&repository, &mut owner);
    let head = refs.head.unwrap();
    let id = owner
        .prepare_tag(refs, "blocked", TagKind::Lightweight, head)
        .unwrap();
    assert!(!finish(&mut owner).published);
    let record = owner.record(id).unwrap();
    assert_eq!(record.effects, Effects::NotStarted);
    assert!(
        record
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("Lightweight tag conflicts")
    );
    assert!(repository.git(["tag", "--list"]).is_empty());
    let signer = repository.fixture.write(
        "signer",
        "#!/bin/sh\nprintf 'Fixture signer rejected annotation\\n' >&2\nexit 7\n",
    );
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&signer, fs::Permissions::from_mode(0o700)).unwrap();
    repository.git(["config", "gpg.program", signer.to_str().unwrap()]);
    let refs = observe(&repository, &mut owner);
    let intent = prepare(&mut owner, refs, "failed", TagKind::Annotated);
    assert!(intent.signing);
    create(&mut owner, intent.clone(), "Retained annotation\n", false);
    assert_eq!(owner.tag_draft(&intent), Some("Retained annotation\n"));
    let record = owner.records().last().unwrap();
    assert!(
        String::from_utf8_lossy(&record.stderr).contains("Fixture signer rejected annotation"),
        "{record:?}"
    );
    assert!(record.argv.iter().any(|a| a == "tag"));
    assert!(repository.git(["tag", "--list"]).is_empty());
}

// Real SSH signing stays Git-owned and the stored annotation/signature verify
// as distinct content. No live key, credential agent or remote is used.
#[test]
fn signed_annotation() {
    let repository = Repository::new("sha256");
    repository.commit();
    let key = repository.fixture.0.join("fixture-key");
    let output = std::process::Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-f"])
        .arg(&key)
        .output()
        .unwrap();
    assert!(output.status.success());
    repository.git(["config", "gpg.format", "ssh"]);
    repository.git(["config", "user.signingkey", key.to_str().unwrap()]);
    repository.git(["config", "tag.gpgsign", "true"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let allowed = repository.fixture.0.join("allowed-signers");
    let public = fs::read_to_string(key.with_extension("pub")).unwrap();
    fs::write(&allowed, format!("author@example.invalid {public}")).unwrap();
    repository.git([
        "config",
        "gpg.ssh.allowedSignersFile",
        allowed.to_str().unwrap(),
    ]);
    for (name, text, force) in [
        ("signed", "Signed annotation\n", false),
        ("forced", "Annotation without final newline", true),
    ] {
        repository.git([
            "config",
            "tag.gpgsign",
            if force { "false" } else { "true" },
        ]);
        repository.git([
            "config",
            "tag.forcesignannotated",
            if force { "true" } else { "false" },
        ]);
        let refs = observe(&repository, &mut owner);
        let intent = prepare(&mut owner, refs, name, TagKind::Annotated);
        assert!(intent.signing);
        create(&mut owner, intent, text, true);
        let raw = repository.git(["cat-file", "tag", &format!("refs/tags/{name}")]);
        assert!(String::from_utf8_lossy(&raw).contains("-----BEGIN SSH SIGNATURE-----"));
        repository.git(["verify-tag", name]);
    }
    let refs = observe(&repository, &mut owner);
    let head = refs.head.unwrap();
    let intent = prepare(&mut owner, refs, "still-lightweight", TagKind::Lightweight);
    assert!(!intent.signing);
    create(&mut owner, intent, "", true);
    assert_eq!(
        repository.git(["rev-parse", "refs/tags/still-lightweight"]),
        format!("{}\n", head.hex()).as_bytes()
    );
}

// Existing names and changed signing policy reject before launch; a moving
// deletion target remains intact rather than being deleted by stale name.
#[test]
fn tag_guards() {
    let repository = Repository::new("sha1");
    repository.commit();
    repository.git(["tag", "exists"]);
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let refs = observe(&repository, &mut owner);
    let head = refs.head.unwrap();
    let id = owner
        .prepare_tag(refs, "exists", TagKind::Lightweight, head)
        .unwrap();
    assert!(!finish(&mut owner).published);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    let refs = observe(&repository, &mut owner);
    let intent = prepare(&mut owner, refs, "policy", TagKind::Annotated);
    repository.git(["config", "tag.gpgsign", "true"]);
    create(&mut owner, intent, "Do not sign silently\n", false);
    assert_eq!(owner.records().last().unwrap().effects, Effects::NotStarted);
    repository.git(["config", "tag.gpgsign", "false"]);
    let refs = observe(&repository, &mut owner);
    repository.commit();
    repository.git(["tag", "--force", "exists"]);
    let id = owner.delete_tag(refs, b"refs/tags/exists").unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert_eq!(
        repository.git(["rev-parse", "refs/tags/exists"]),
        repository.git(["rev-parse", "HEAD"])
    );
}

#[path = "refs_tags_presentation.rs"]
mod presentation;
