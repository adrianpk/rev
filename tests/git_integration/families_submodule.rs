use super::*;

fn protocol(repository: &Repository) {
    // Only this isolated fixture admits local file transport. All repository
    // effects and output still come from installed Git, without substitution.
    fs::write(&repository.program, "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_ALLOW_PROTOCOL=file\nexec /usr/bin/git \"$@\"\n").unwrap();
}
fn parameters(family: Family, values: &[&std::ffi::OsStr]) -> Vec<OsString> {
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
        parameters(family, values),
    )
}
fn source(format: &str) -> Repository {
    let child = Repository::new(format);
    fs::write(child.root.join("child"), "first\n").unwrap();
    child.commit();
    child
}
pub(super) fn added(format: &str) -> (Repository, Repository, Repositories) {
    let repository = Repository::new(format);
    protocol(&repository);
    fs::write(repository.root.join("parent"), "parent\n").unwrap();
    repository.commit();
    let child = source(format);
    let url = OsString::from(format!("file://{}", child.root.display()));
    let mut owner = repository.owner("");
    let add = plan(
        &repository,
        &mut owner,
        Family::SubmoduleAdd,
        &[
            std::ffi::OsStr::new("module space"),
            &url,
            std::ffi::OsStr::new("module space"),
        ],
    );
    assert!(!repository.root.join("module space").exists());
    execute(&mut owner, add);
    (repository, child, owner)
}

// Every native submodule variant performs actual Git work in both object
// formats: gitlink creation, read-only listing, deinit, checkout and URL sync.
#[test]
fn roundtrip() {
    for format in ["sha1", "sha256"] {
        let (repository, child, mut owner) = added(format);
        let path = std::ffi::OsStr::new("module space");
        let head = child.git(["rev-parse", "HEAD"]);
        assert_eq!(
            fs::read(repository.root.join(path).join("child")).unwrap(),
            b"first\n"
        );
        let index = repository.git(["ls-files", "--stage", "--", "module space"]);
        assert!(index.starts_with(b"160000 "));
        assert!(
            index
                .windows(head.len() - 1)
                .any(|w| w == &head[..head.len() - 1])
        );
        repository.commit();

        let listing = plan(&repository, &mut owner, Family::SubmoduleList, &[]);
        let id = owner.execute_family(listing).unwrap();
        assert!(finish(&mut owner).published, "{:?}", owner.record(id));
        let record = owner.record(id).unwrap();
        assert_eq!(record.effects, Effects::NotStarted);
        assert_eq!(record.outcome, Outcome::Verified);
        assert!(
            record
                .stdout
                .windows(head.len() - 1)
                .any(|w| w == &head[..head.len() - 1])
        );

        let deinit = plan(&repository, &mut owner, Family::SubmoduleDeinit, &[path]);
        assert!(repository.root.join(path).join("child").exists());
        execute(&mut owner, deinit);
        assert!(!repository.root.join(path).join(".git").exists());
        assert!(
            fs::read_dir(repository.root.join(path))
                .unwrap()
                .next()
                .is_none()
        );
        assert_eq!(
            repository.git(["ls-files", "--stage", "--", "module space"]),
            index
        );

        let update = plan(&repository, &mut owner, Family::SubmoduleUpdate, &[path]);
        execute(&mut owner, update);
        assert_eq!(
            fs::read(repository.root.join(path).join("child")).unwrap(),
            b"first\n"
        );
        assert_eq!(
            repository.git(["-C", "module space", "rev-parse", "HEAD"]),
            head
        );
        let url = format!("file://{}", child.root.display());
        repository.git([
            "config",
            "submodule.module space.url",
            "file:///obsolete-local-fixture",
        ]);
        let sync = plan(&repository, &mut owner, Family::SubmoduleSync, &[path]);
        execute(&mut owner, sync);
        assert_eq!(
            repository.git(["config", "--get", "submodule.module space.url"]),
            format!("{url}\n").as_bytes()
        );
        assert!(owner.contexts()[0].failure.is_none());
    }
}

// Review currency includes configuration and actual child bytes, even where
// porcelain labels stay unchanged. Dirty updates/deinit never force or stash.
#[test]
fn stale_and_dirty() {
    for variant in ["configuration", "modules", "child", "symlink", "index"] {
        let (repository, _child, mut owner) = added("sha1");
        repository.commit();
        let target = repository.root.join("module space");
        let prepared = plan(
            &repository,
            &mut owner,
            Family::SubmoduleUpdate,
            &[std::ffi::OsStr::new("module space")],
        );
        match variant {
            "configuration" => {
                repository.git(["config", "submodule.module space.url", "file:///changed"]);
            }
            "modules" => {
                repository.git([
                    "config",
                    "--file=.gitmodules",
                    "submodule.module space.url",
                    "file:///changed",
                ]);
            }
            "child" => fs::write(target.join("child"), "external\n").unwrap(),
            "symlink" => {
                fs::rename(&target, repository.root.join("retained-child")).unwrap();
                std::os::unix::fs::symlink("retained-child", &target).unwrap();
            }
            "index" => {
                repository.git(["rm", "--cached", "module space"]);
            }
            _ => unreachable!(),
        }
        let id = owner.execute_family(prepared).unwrap();
        assert!(!finish(&mut owner).published, "{variant}");
        assert_eq!(
            owner.record(id).unwrap().effects,
            Effects::NotStarted,
            "{variant}: {:?}",
            owner.record(id)
        );
    }
    for family in [Family::SubmoduleUpdate, Family::SubmoduleDeinit] {
        let (repository, _child, mut owner) = added("sha1");
        fs::write(
            repository.root.join("module space/child"),
            "unsaved-on-disk\n",
        )
        .unwrap();
        repository.open(&mut owner, &repository.root);
        let context = owner.contexts()[0].id;
        let id = owner
            .prepare_family(
                &repository.root,
                Some(context),
                family,
                parameters(family, &[std::ffi::OsStr::new("module space")]),
            )
            .unwrap();
        assert!(!finish(&mut owner).published);
        assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
        assert_eq!(
            fs::read(repository.root.join("module space/child")).unwrap(),
            b"unsaved-on-disk\n"
        );
    }
}

// Literal pathspec mode keeps a metacharacter target separate from a matching
// sibling; raw newline/non-UTF8 names also preserve their actual gitlink bytes.
#[test]
fn literal_target() {
    for path in [
        OsString::from("module[1]"),
        OsString::from_vec(b"module\n\xff".to_vec()),
    ] {
        let repository = Repository::new("sha1");
        protocol(&repository);
        repository.commit();
        let child = source("sha1");
        let url = OsString::from(format!("file://{}", child.root.display()));
        let mut owner = repository.owner("");
        for (name, target) in [
            (std::ffi::OsStr::new("selected"), path.as_os_str()),
            (
                std::ffi::OsStr::new("sibling"),
                std::ffi::OsStr::new("module1"),
            ),
        ] {
            let add = plan(
                &repository,
                &mut owner,
                Family::SubmoduleAdd,
                &[name, &url, target],
            );
            execute(&mut owner, add);
        }
        repository.commit();
        let deinit = plan(&repository, &mut owner, Family::SubmoduleDeinit, &[&path]);
        execute(&mut owner, deinit);
        assert!(!repository.root.join(&path).join("child").exists());
        assert_eq!(
            fs::read(repository.root.join("module1/child")).unwrap(),
            b"first\n"
        );
        assert!(repository.root.join("module1/.git").exists());
        let update = plan(&repository, &mut owner, Family::SubmoduleUpdate, &[&path]);
        execute(&mut owner, update);
        assert_eq!(
            fs::read(repository.root.join(&path).join("child")).unwrap(),
            b"first\n"
        );
    }
}
