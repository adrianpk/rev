use super::*;

fn arguments(family: Family, values: &[&std::ffi::OsStr]) -> Vec<OsString> {
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
    values: &[&std::ffi::OsStr],
) -> Arc<FamilyPlan> {
    repository.open(owner, &repository.root);
    let context = owner.contexts()[0].id;
    prepare(
        owner,
        &repository.root,
        Some(context),
        family,
        arguments(family, values),
    )
}
pub(super) fn added(format: &str) -> (Repository, Repository, Repositories) {
    let repository = Repository::new(format);
    fs::write(repository.root.join("parent"), "preserved\n").unwrap();
    repository.commit();
    let child = Repository::new(format);
    fs::write(child.root.join("foreign"), "first\n").unwrap();
    child.commit();
    let url = OsString::from(format!("file://{}", child.root.display()));
    let mut owner = repository.owner("");
    let add = plan(
        &repository,
        &mut owner,
        Family::SubtreeAdd,
        &[
            std::ffi::OsStr::new("vendor module"),
            &url,
            std::ffi::OsStr::new("refs/heads/main"),
        ],
    );
    assert!(!repository.root.join("vendor module").exists());
    assert_eq!(
        add.args.last().unwrap().as_encoded_bytes(),
        child
            .git(["rev-parse", "HEAD"])
            .strip_suffix(b"\n")
            .unwrap()
    );
    execute(&mut owner, add);
    (repository, child, owner)
}

// All five subtree routes run installed Git in both formats; object/tree and
// saved-file assertions prove real import, merge, pull, split and publication.
#[test]
fn roundtrip() {
    for format in ["sha1", "sha256"] {
        let (repository, child, mut owner) = added(format);
        let prefix = std::ffi::OsStr::new("vendor module");
        let url = OsString::from(format!("file://{}", child.root.display()));
        assert_eq!(
            fs::read(repository.root.join(prefix).join("foreign")).unwrap(),
            b"first\n"
        );
        assert_eq!(
            repository.git(["rev-parse", "HEAD:vendor module"]),
            child.git(["rev-parse", "HEAD^{tree}"])
        );

        fs::write(child.root.join("foreign"), "merged\n").unwrap();
        child.commit();
        repository.git([
            std::ffi::OsStr::new("fetch"),
            std::ffi::OsStr::new("--no-tags"),
            std::ffi::OsStr::new("--"),
            url.as_os_str(),
            std::ffi::OsStr::new("refs/heads/main"),
        ]);
        let merge = plan(
            &repository,
            &mut owner,
            Family::SubtreeMerge,
            &[prefix, std::ffi::OsStr::new("FETCH_HEAD")],
        );
        execute(&mut owner, merge);
        assert_eq!(
            fs::read(repository.root.join(prefix).join("foreign")).unwrap(),
            b"merged\n"
        );
        assert_eq!(
            repository.git(["rev-parse", "HEAD:vendor module"]),
            child.git(["rev-parse", "HEAD^{tree}"])
        );

        fs::write(child.root.join("foreign"), "pulled\n").unwrap();
        child.commit();
        let pull = plan(
            &repository,
            &mut owner,
            Family::SubtreePull,
            &[prefix, &url, std::ffi::OsStr::new("refs/heads/main")],
        );
        assert_eq!(
            fs::read(repository.root.join(prefix).join("foreign")).unwrap(),
            b"merged\n"
        );
        execute(&mut owner, pull);
        assert_eq!(
            fs::read(repository.root.join(prefix).join("foreign")).unwrap(),
            b"pulled\n"
        );
        assert_eq!(
            repository.git(["rev-parse", "HEAD:vendor module"]),
            child.git(["rev-parse", "HEAD^{tree}"])
        );

        fs::write(
            repository.root.join(prefix).join("foreign"),
            "local subtree\n",
        )
        .unwrap();
        repository.commit();
        let before = repository.git(["rev-parse", "HEAD"]);
        let tree = repository.git(["rev-parse", "HEAD:vendor module"]);
        let split = plan(
            &repository,
            &mut owner,
            Family::SubtreeSplit,
            &[prefix, std::ffi::OsStr::new("HEAD")],
        );
        execute(&mut owner, split);
        let object = owner.family_result().unwrap().object.unwrap();
        assert_eq!(
            repository.git(["show", &format!("{}:foreign", object.hex())]),
            b"local subtree\n"
        );
        assert_eq!(
            repository.git(["rev-parse", &format!("{}^{{tree}}", object.hex())]),
            tree
        );
        assert_eq!(repository.git(["rev-parse", "HEAD"]), before);

        let remote = repository.fixture.0.join("bare-destination");
        repository.git([
            std::ffi::OsStr::new("init"),
            std::ffi::OsStr::new("--bare"),
            std::ffi::OsStr::new(&format!("--object-format={format}")),
            std::ffi::OsStr::new("--"),
            remote.as_os_str(),
        ]);
        let push = plan(
            &repository,
            &mut owner,
            Family::SubtreePush,
            &[
                prefix,
                remote.as_os_str(),
                std::ffi::OsStr::new("refs/heads/delivery"),
            ],
        );
        execute(&mut owner, push);
        assert_eq!(
            repository.git([
                std::ffi::OsStr::new("-C"),
                remote.as_os_str(),
                std::ffi::OsStr::new("rev-parse"),
                std::ffi::OsStr::new("refs/heads/delivery^{tree}")
            ]),
            tree
        );
        assert_eq!(
            repository.git([
                std::ffi::OsStr::new("-C"),
                remote.as_os_str(),
                std::ffi::OsStr::new("show"),
                std::ffi::OsStr::new("refs/heads/delivery:foreign")
            ]),
            b"local subtree\n"
        );
        assert_eq!(repository.git(["rev-parse", "HEAD"]), before);
        assert_eq!(
            fs::read(repository.root.join("parent")).unwrap(),
            b"preserved\n"
        );
    }
}

// A mutable remote branch cannot replace the pinned reviewed source, and
// expanded push policy is rejected before publication or a changing child.
#[test]
fn stale_and_policy() {
    let repository = Repository::new("sha1");
    repository.commit();
    let child = Repository::new("sha1");
    child.commit();
    let url = OsString::from(format!("file://{}", child.root.display()));
    let mut owner = repository.owner("");
    let add = plan(
        &repository,
        &mut owner,
        Family::SubtreeAdd,
        &[
            std::ffi::OsStr::new("prefix"),
            &url,
            std::ffi::OsStr::new("refs/heads/main"),
        ],
    );
    child.commit();
    let id = owner.execute_family(add).unwrap();
    assert!(!finish(&mut owner).published);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert!(!repository.root.join("prefix").exists());

    for key in ["push.followTags", "push.recurseSubmodules"] {
        let (repository, _child, mut owner) = added("sha1");
        repository.git([
            "config",
            key,
            if key == "push.followTags" {
                "true"
            } else {
                "on-demand"
            },
        ]);
        repository.open(&mut owner, &repository.root);
        let context = owner.contexts()[0].id;
        let target = repository.fixture.0.join("uncreated-remote");
        let id = owner
            .prepare_family(
                &repository.root,
                Some(context),
                Family::SubtreePush,
                arguments(
                    Family::SubtreePush,
                    &[
                        std::ffi::OsStr::new("vendor module"),
                        target.as_os_str(),
                        std::ffi::OsStr::new("refs/heads/delivery"),
                    ],
                ),
            )
            .unwrap();
        assert!(!finish(&mut owner).published);
        assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
        assert!(!target.exists());
    }
}

// A real conflicting subtree merge retains Git's actual sequence and original
// HEAD; an exit failure cannot turn a published observation into verification.
#[test]
fn conflict_state() {
    let (repository, child, mut owner) = added("sha1");
    fs::write(repository.root.join("vendor module/foreign"), "local\n").unwrap();
    repository.commit();
    let before = repository.git(["rev-parse", "HEAD"]);
    fs::write(child.root.join("foreign"), "remote\n").unwrap();
    child.commit();
    let url = format!("file://{}", child.root.display());
    repository.git(["fetch", "--no-tags", "--", &url, "refs/heads/main"]);
    let merge = plan(
        &repository,
        &mut owner,
        Family::SubtreeMerge,
        &[
            std::ffi::OsStr::new("vendor module"),
            std::ffi::OsStr::new("FETCH_HEAD"),
        ],
    );
    let id = owner.execute_family(merge).unwrap();
    let update = finish(&mut owner);
    assert!(update.published, "{:?}", owner.record(id));
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome, Outcome::InProgress, "{record:?}");
    assert_eq!(record.effects, Effects::Possible);
    let sequence = owner.contexts()[0].sequence.as_ref().unwrap();
    assert_eq!(sequence.family, Some(rev::git::SequenceFamily::Merge));
    assert_eq!(
        sequence.targets[0].hex().as_bytes(),
        child
            .git(["rev-parse", "HEAD"])
            .strip_suffix(b"\n")
            .unwrap()
    );
    assert_eq!(repository.git(["rev-parse", "HEAD"]), before);
    assert!(
        fs::read(repository.root.join("vendor module/foreign"))
            .unwrap()
            .windows(7)
            .any(|w| w == b"<<<<<<<")
    );
}

// Missing optional helpers are detected through installed Git capability
// discovery. This fixture changes exec-path, without fabricating Git output.
#[test]
fn unavailable() {
    let repository = Repository::new("sha1");
    repository.commit();
    fs::create_dir(repository.fixture.0.join("empty-exec")).unwrap();
    fs::write(&repository.program, "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nexport GIT_EXEC_PATH=\"${0%/*}/empty-exec\"\nexec /usr/bin/git \"$@\"\n").unwrap();
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let context = owner.contexts()[0].id;
    let id = owner
        .prepare_family(
            &repository.root,
            Some(context),
            Family::SubtreeAdd,
            arguments(
                Family::SubtreeAdd,
                &[
                    std::ffi::OsStr::new("prefix"),
                    std::ffi::OsStr::new("file:///explicit-fixture-source"),
                    std::ffi::OsStr::new("refs/heads/main"),
                ],
            ),
        )
        .unwrap();
    assert!(!finish(&mut owner).published);
    let record = owner.record(id).unwrap();
    assert_eq!(record.effects, Effects::NotStarted);
    assert!(
        record
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("Installed Git does not provide"),
        "{record:?}"
    );
    assert!(!repository.root.join("prefix").exists());
}
