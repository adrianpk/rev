use super::*;
use std::ffi::OsStr;

fn arguments(family: Family, values: &[&OsStr]) -> Vec<OsString> {
    let mut values = values.iter();
    family
        .seed()
        .into_iter()
        .map(|a| {
            if a.is_empty() {
                (*values.next().unwrap()).into()
            } else {
                a.into()
            }
        })
        .collect()
}
fn plan(
    repository: &Repository,
    owner: &mut Repositories,
    family: Family,
    values: &[&OsStr],
    destination: &Path,
) -> Arc<FamilyPlan> {
    repository.open(owner, &repository.root);
    let context = owner.contexts()[0].id;
    let id = owner
        .prepare_family_output(
            &repository.root,
            Some(context),
            family,
            arguments(family, values),
            destination,
        )
        .unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(id));
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert!(!destination.exists());
    owner.family_plan().unwrap().clone()
}
fn history(format: &str) -> (Repository, String) {
    let repository = Repository::new(format);
    fs::write(repository.root.join("text"), "base\n").unwrap();
    fs::write(repository.root.join("binary"), [0, 1, 2, 3]).unwrap();
    repository.commit();
    let base = String::from_utf8(repository.git(["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    fs::write(repository.root.join("text"), "delivered\n").unwrap();
    fs::write(repository.root.join("binary"), [0, 8, 9, 255]).unwrap();
    repository.commit();
    (repository, base)
}
fn detached(repository: &Repository, name: &str, base: &str) -> std::path::PathBuf {
    let path = repository.fixture.0.join(name);
    repository.git([
        OsStr::new("worktree"),
        OsStr::new("add"),
        OsStr::new("--detach"),
        OsStr::new("--"),
        path.as_os_str(),
        OsStr::new(base),
    ]);
    path
}

// Exported mailboxes and full-index binary diffs are consumed by real Git in
// separate roots. Tree IDs and known bytes prove usable output in both formats.
#[test]
fn patch_roundtrip() {
    for format in ["sha1", "sha256"] {
        let (repository, base) = history(format);
        let head = repository.git(["rev-parse", "HEAD"]);
        let tree = repository.git(["rev-parse", "HEAD^{tree}"]);
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let range = OsString::from(format!("{base}..HEAD"));
        let raw = OsString::from_vec(b"mail-$(literal)\n\xff.patch".to_vec());
        let mailbox = repository.fixture.0.join(raw);
        let mut owner = repository.owner("");
        let prepared = plan(
            &repository,
            &mut owner,
            Family::FormatPatch,
            &[&range],
            &mailbox,
        );
        let (size, hash) = prepared.output_preview().unwrap();
        assert!(size > 0);
        assert_eq!(hash.hex().len(), if format == "sha256" { 64 } else { 40 });
        execute(&mut owner, prepared);
        let bytes = fs::read(&mailbox).unwrap();
        assert_eq!(bytes.len(), size);
        assert!(
            bytes
                .windows(b"GIT binary patch".len())
                .any(|w| w == b"GIT binary patch")
        );
        assert_eq!(
            repository.git([
                OsStr::new("hash-object"),
                OsStr::new("--no-filters"),
                OsStr::new("--"),
                mailbox.as_os_str()
            ]),
            format!("{}\n", hash.hex()).as_bytes()
        );
        let applied = detached(&repository, "mailbox-consumer", &base);
        repository.git([
            OsStr::new("-C"),
            applied.as_os_str(),
            OsStr::new("am"),
            OsStr::new("--"),
            mailbox.as_os_str(),
        ]);
        assert_eq!(
            repository.git([
                OsStr::new("-C"),
                applied.as_os_str(),
                OsStr::new("rev-parse"),
                OsStr::new("HEAD^{tree}")
            ]),
            tree
        );
        assert_eq!(fs::read(applied.join("text")).unwrap(), b"delivered\n");
        assert_eq!(fs::read(applied.join("binary")).unwrap(), [0, 8, 9, 255]);

        let saved = repository.root.join("reviewed.diff");
        let diff = plan(
            &repository,
            &mut owner,
            Family::SaveDiff,
            &[OsStr::new(&base), OsStr::new("HEAD")],
            &saved,
        );
        execute(&mut owner, diff);
        let applied = detached(&repository, "diff-consumer", &base);
        repository.git([
            OsStr::new("-C"),
            applied.as_os_str(),
            OsStr::new("apply"),
            OsStr::new("--index"),
            OsStr::new("--binary"),
            OsStr::new("--"),
            saved.as_os_str(),
        ]);
        assert_eq!(
            repository.git([
                OsStr::new("-C"),
                applied.as_os_str(),
                OsStr::new("write-tree")
            ]),
            tree
        );
        assert_eq!(fs::read(applied.join("text")).unwrap(), b"delivered\n");
        assert_eq!(fs::read(applied.join("binary")).unwrap(), [0, 8, 9, 255]);
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        assert!(fs::read_dir(&repository.root).unwrap().all(|e| {
            !e.unwrap()
                .file_name()
                .as_encoded_bytes()
                .starts_with(b".rev-save-")
        }));
        assert!(fs::read_dir(&repository.fixture.0).unwrap().all(|e| {
            !e.unwrap()
                .file_name()
                .as_encoded_bytes()
                .starts_with(b".rev-save-")
        }));
    }
}

// Request-pull names an explicit endpoint and branch already at the reviewed
// commit. Its saved text contains full anchors without changing either repo.
#[test]
fn request_pull() {
    for format in ["sha1", "sha256"] {
        let (mut repository, base) = history(format);
        let remote = repository.fixture.0.join("published");
        repository.git([
            OsStr::new("clone"),
            OsStr::new("--bare"),
            OsStr::new("--"),
            repository.root.as_os_str(),
            remote.as_os_str(),
        ]);
        let before = repository.git([OsStr::new("-C"), remote.as_os_str(), OsStr::new("show-ref")]);
        let head = repository.git(["rev-parse", "HEAD"]);
        let path = repository.fixture.0.join("request.txt");
        let mut owner = repository.owner("");
        let prepared = plan(
            &repository,
            &mut owner,
            Family::RequestPull,
            &[
                OsStr::new(&base),
                remote.as_os_str(),
                OsStr::new("refs/heads/main"),
            ],
            &path,
        );
        assert!(
            prepared
                .args
                .last()
                .unwrap()
                .as_encoded_bytes()
                .ends_with(b":refs/heads/main")
        );
        execute(&mut owner, prepared);
        let output = String::from_utf8(fs::read(&path).unwrap()).unwrap();
        assert!(output.contains(&base));
        assert!(output.contains(String::from_utf8(head.clone()).unwrap().trim()));
        assert!(output.contains("for you to fetch changes up to"));
        assert!(output.contains(&format!("{} main", remote.display())));
        assert_eq!(
            repository.git([OsStr::new("-C"), remote.as_os_str(), OsStr::new("show-ref")]),
            before
        );
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        repository.root = remote;
        let mut bare_owner = repository.owner("");
        let empty = repository.fixture.0.join("empty.diff");
        let prepared = plan(
            &repository,
            &mut bare_owner,
            Family::SaveDiff,
            &[OsStr::new("HEAD"), OsStr::new("HEAD")],
            &empty,
        );
        assert!(bare_owner.contexts()[0].observation.status.is_none());
        assert_eq!(prepared.output_preview().unwrap().0, 0);
        execute(&mut bare_owner, prepared);
        assert!(fs::read(&empty).unwrap().is_empty());
    }
}

// Destination collisions, parent/ref/config changes and incompatible output
// bytes invalidate review before any file publication or existing-file overwrite.
#[test]
fn checked_output() {
    for changed in ["destination", "parent", "config", "head", "source"] {
        let (repository, base) = history("sha1");
        let destination = repository.fixture.0.join("saved.diff");
        let mut owner = repository.owner("");
        let prepared = plan(
            &repository,
            &mut owner,
            Family::SaveDiff,
            &[OsStr::new(&base), OsStr::new("HEAD")],
            &destination,
        );
        match changed {
            "destination" => {
                fs::write(&destination, "external\n").unwrap();
            }
            "parent" => {
                fs::write(repository.fixture.0.join("new-sibling"), "external\n").unwrap();
            }
            "config" => {
                repository.git(["config", "diff.algorithm", "patience"]);
            }
            "head" => {
                repository.commit();
            }
            "source" => {
                fs::write(repository.root.join("text"), "external\n").unwrap();
            }
            _ => unreachable!(),
        }
        let id = owner.execute_family(prepared).unwrap();
        finish(&mut owner);
        assert_eq!(
            owner.record(id).unwrap().effects,
            Effects::NotStarted,
            "{changed}: {:?}",
            owner.record(id)
        );
        if changed == "destination" {
            assert_eq!(fs::read(&destination).unwrap(), b"external\n");
        } else {
            assert!(!destination.exists());
        }
    }
}

// Complete-output limits and metadata/path admission reject as a whole, and
// no existing output or Git control file becomes an implicit write target.
#[test]
fn output_admission() {
    let (repository, base) = history("sha1");
    fs::write(repository.root.join("large"), "large line\n".repeat(1024)).unwrap();
    repository.commit();
    let mut owner = repository.owner("max_detail_bytes=4096");
    repository.open(&mut owner, &repository.root);
    let context = owner.contexts()[0].id;
    let path = repository.fixture.0.join("partial.patch");
    let id = owner
        .prepare_family_output(
            &repository.root,
            Some(context),
            Family::SaveDiff,
            arguments(Family::SaveDiff, &[OsStr::new(&base), OsStr::new("HEAD")]),
            &path,
        )
        .unwrap();
    finish(&mut owner);
    assert!(!path.exists());
    assert_eq!(owner.record(id).unwrap().outcome, Outcome::OutputLimit);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    for path in [
        repository.root.join(".git/control"),
        repository.fixture.0.join("existing"),
    ] {
        if !path.starts_with(repository.root.join(".git")) {
            fs::write(&path, "preserved\n").unwrap();
        }
        repository.open(&mut owner, &repository.root);
        let id = owner
            .prepare_family_output(
                &repository.root,
                Some(context),
                Family::SaveDiff,
                arguments(Family::SaveDiff, &[OsStr::new(&base), OsStr::new("HEAD")]),
                &path,
            )
            .unwrap();
        finish(&mut owner);
        assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
        assert!(!owner.family_plan().is_some_and(|p| p.destination == path));
    }
    assert_eq!(
        fs::read(repository.fixture.0.join("existing")).unwrap(),
        b"preserved\n"
    );
}

// Attribute changes can alter real diff bytes without changing refs/config or
// clean saved state. Exit zero still cannot publish different reviewed output.
#[test]
fn changed_producer() {
    let (repository, base) = history("sha1");
    let destination = repository.fixture.0.join("reviewed.diff");
    let mut owner = repository.owner("");
    let prepared = plan(
        &repository,
        &mut owner,
        Family::SaveDiff,
        &[OsStr::new(&base), OsStr::new("HEAD")],
        &destination,
    );
    fs::write(
        repository.root.join(".git/info/attributes"),
        "text binary\n",
    )
    .unwrap();
    let id = owner.execute_family(prepared).unwrap();
    finish(&mut owner);
    let record = owner.record(id).unwrap();
    assert_eq!(record.exit.unwrap().code(), Some(0));
    assert_eq!(record.outcome, Outcome::Indeterminate);
    assert_eq!(record.effects, Effects::Possible);
    assert!(
        record
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("differs from the reviewed")
    );
    assert!(!destination.exists());
    assert_eq!(
        fs::read(repository.root.join("text")).unwrap(),
        b"delivered\n"
    );
}

// Read-only remote preparation and accepted generation use the configured
// transport deadline; a moved remote branch refuses before any output file.
#[test]
fn remote_review() {
    let (repository, base) = history("sha1");
    let remote = repository.fixture.0.join("published");
    repository.git([
        OsStr::new("clone"),
        OsStr::new("--bare"),
        OsStr::new("--"),
        repository.root.as_os_str(),
        remote.as_os_str(),
    ]);
    fs::write(&repository.program, "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\ncase \"$*\" in *request-pull*) sleep 1.2;; esac\nexec /usr/bin/git \"$@\"\n").unwrap();
    let path = repository.fixture.0.join("request.txt");
    let mut owner = repository.owner(
        "observation_timeout_seconds=1\noperation_timeout_seconds=1\ntransport_timeout_seconds=3",
    );
    let prepared = plan(
        &repository,
        &mut owner,
        Family::RequestPull,
        &[
            OsStr::new(&base),
            remote.as_os_str(),
            OsStr::new("refs/heads/main"),
        ],
        &path,
    );
    execute(&mut owner, prepared);
    assert!(path.is_file());
    let path = repository.fixture.0.join("stale-request.txt");
    let prepared = plan(
        &repository,
        &mut owner,
        Family::RequestPull,
        &[
            OsStr::new(&base),
            remote.as_os_str(),
            OsStr::new("refs/heads/main"),
        ],
        &path,
    );
    repository.git([
        OsStr::new("-C"),
        remote.as_os_str(),
        OsStr::new("update-ref"),
        OsStr::new("refs/heads/main"),
        OsStr::new(&base),
    ]);
    let id = owner.execute_family(prepared).unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert!(!path.exists());
}
