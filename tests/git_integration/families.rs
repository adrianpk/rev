use super::support::{Repository, finish};
use rev::git::{Effects, Family, FamilyPlan, Outcome, Repositories};
use std::{ffi::OsString, fs, os::unix::ffi::OsStringExt, path::Path, sync::Arc};

#[path = "families_metadata.rs"]
mod metadata;
#[path = "families_presentation.rs"]
mod presentation;
#[path = "families_submodule.rs"]
mod submodule;
#[path = "families_subtree.rs"]
mod subtree;

fn args(family: Family, source: &Path, destination: &Path) -> Vec<OsString> {
    let mut args: Vec<OsString> = family.seed().into_iter().map(OsString::from).collect();
    if family != Family::Init {
        let index = args.len() - 2;
        args[index] = format!("file://{}", source.display()).into();
    }
    *args.last_mut().unwrap() = destination.as_os_str().into();
    args
}
fn prepare(
    owner: &mut Repositories,
    directory: &Path,
    context: Option<rev::git::ContextId>,
    family: Family,
    args: Vec<OsString>,
) -> Arc<FamilyPlan> {
    let id = owner
        .prepare_family(directory, context, family, args)
        .unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(id));
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    owner.family_plan().unwrap().clone()
}
fn execute(owner: &mut Repositories, plan: Arc<FamilyPlan>) {
    let id = owner.execute_family(plan).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(id));
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome, Outcome::Verified, "{record:?}");
    assert_eq!(record.effects, Effects::Verified);
}

// Real clones preserve full objects and raw destinations in both formats;
// shallow/bare/mirror each prove their own Git state rather than their labels.
#[test]
fn clone_kinds() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        fs::write(repository.root.join("file"), "first\n").unwrap();
        repository.commit();
        fs::write(repository.root.join("file"), "second\n").unwrap();
        repository.commit();
        repository.git(["branch", "other"]);
        repository.git(["tag", "example"]);
        let head = repository.git(["rev-parse", "HEAD"]);
        let mut owner = repository.owner("");
        for family in [
            Family::Clone,
            Family::CloneShallow,
            Family::CloneBare,
            Family::CloneMirror,
        ] {
            let raw = OsString::from_vec(
                format!("-{}\n", family.label())
                    .into_bytes()
                    .into_iter()
                    .chain([0xff])
                    .collect(),
            );
            let destination = repository.fixture.0.join(raw);
            let plan = prepare(
                &mut owner,
                &repository.fixture.0,
                None,
                family,
                args(family, &repository.root, &destination),
            );
            assert!(!destination.exists());
            execute(&mut owner, plan);
            let actual = owner.family_result().unwrap().created.as_ref().unwrap();
            assert_eq!(
                actual.format,
                if format == "sha256" {
                    rev::git::ObjectFormat::Sha256
                } else {
                    rev::git::ObjectFormat::Sha1
                }
            );
            let git = |values: &[&str]| {
                let mut args: Vec<OsString> = vec!["-C".into(), destination.as_os_str().into()];
                args.extend(values.iter().map(OsString::from));
                repository.git(args)
            };
            assert_eq!(git(&["rev-parse", "HEAD"]), head);
            assert_eq!(
                git(&["rev-parse", "--is-bare-repository"]),
                if matches!(family, Family::CloneBare | Family::CloneMirror) {
                    b"true\n".as_slice()
                } else {
                    b"false\n".as_slice()
                }
            );
            if family == Family::CloneShallow {
                assert_eq!(git(&["rev-parse", "--is-shallow-repository"]), b"true\n");
                assert_eq!(git(&["rev-list", "--count", "HEAD"]), b"1\n");
            }
            if family == Family::CloneMirror {
                assert_eq!(
                    git(&["config", "--bool", "remote.origin.mirror"]),
                    b"true\n"
                );
                assert_eq!(git(&["show-ref"]), repository.git(["show-ref"]));
            }
            if actual.worktree.is_some() {
                assert_eq!(fs::read(destination.join("file")).unwrap(), b"second\n");
            }
        }
    }
}

// Init works without a repository context and requires an absent explicit
// destination; preparation never creates it or overwrites existing content.
#[test]
fn explicit_init() {
    let repository = Repository::new("sha1");
    let mut owner = repository.owner("");
    let destination = repository.fixture.0.join("new-repository");
    let plan = prepare(
        &mut owner,
        &repository.fixture.0,
        None,
        Family::Init,
        args(Family::Init, &repository.root, &destination),
    );
    assert!(!destination.exists());
    execute(&mut owner, plan);
    assert_eq!(
        owner
            .family_result()
            .unwrap()
            .created
            .as_ref()
            .unwrap()
            .worktree
            .as_ref(),
        Some(&destination)
    );
    assert!(
        fs::read(destination.join(".git/HEAD"))
            .unwrap()
            .starts_with(b"ref: refs/heads/")
    );
    let id = owner
        .prepare_family(
            &repository.fixture.0,
            None,
            Family::Init,
            args(Family::Init, &repository.root, &destination),
        )
        .unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert!(
        owner
            .record(id)
            .unwrap()
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("destination already exists")
    );
}

// Destination/parent replacement and saved source/index/HEAD races invalidate
// accepted plans before a changing child, preserving the external state.
#[test]
fn stale_preparation() {
    for variant in ["destination", "parent", "source", "index", "head"] {
        let repository = Repository::new("sha1");
        fs::write(repository.root.join("file"), "base\n").unwrap();
        repository.commit();
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let context = owner.contexts()[0].id;
        let destination = repository.fixture.0.join("created");
        let plan = prepare(
            &mut owner,
            &repository.root,
            Some(context),
            Family::Init,
            args(Family::Init, &repository.root, &destination),
        );
        match variant {
            "destination" => fs::create_dir(&destination).unwrap(),
            "parent" => fs::write(repository.fixture.0.join("sibling"), "external\n").unwrap(),
            "source" => fs::write(repository.root.join("file"), "external\n").unwrap(),
            "index" => {
                fs::write(repository.root.join("file"), "external\n").unwrap();
                repository.git(["add", "file"]);
            }
            "head" => {
                repository.commit();
            }
            _ => unreachable!(),
        }
        let id = owner.execute_family(plan).unwrap();
        finish(&mut owner);
        assert_eq!(
            owner.record(id).unwrap().effects,
            Effects::NotStarted,
            "{variant}: {:?}",
            owner.record(id)
        );
        assert!(!destination.join(".git").exists());
    }
}

// Invalid structures and literal input caps reject before worker admission;
// shell-looking strings remain one argument and do not expand into execution.
#[test]
fn literal_admission() {
    let repository = Repository::new("sha1");
    let mut owner = repository.owner("");
    let path = repository.fixture.0.join("literal-$(touch-not-run); space");
    let plan = prepare(
        &mut owner,
        &repository.fixture.0,
        None,
        Family::Init,
        args(Family::Init, &repository.root, &path),
    );
    execute(&mut owner, plan);
    assert!(path.join(".git").exists());
    for invalid in [
        vec!["init".into()],
        vec!["init".into(), "--".into(), "".into()],
        vec!["init".into(), "--".into(), "x".repeat(4097).into()],
        vec!["init".into(), "--".into(), "x\0y".into()],
        vec!["clone".into(), "--".into(), "x".into()],
    ] {
        assert!(
            owner
                .prepare_family(&repository.fixture.0, None, Family::Init, invalid)
                .is_err()
        );
    }
    assert!(!repository.fixture.0.join("touch-not-run").exists());
}

// An unborn source still has saved index, working and untracked state. Init
// elsewhere must check that evidence without inventing a nonexistent HEAD.
#[test]
fn unborn_source() {
    let repository = Repository::new("sha256");
    fs::write(repository.root.join("file"), "staged\n").unwrap();
    repository.git(["add", "file"]);
    fs::write(repository.root.join("file"), "working\n").unwrap();
    fs::write(repository.root.join("untracked"), "saved\n").unwrap();
    let mut owner = repository.owner("");
    repository.open(&mut owner, &repository.root);
    let context = owner.contexts()[0].id;
    let destination = repository.fixture.0.join("new-repository");
    let plan = prepare(
        &mut owner,
        &repository.root,
        Some(context),
        Family::Init,
        args(Family::Init, &repository.root, &destination),
    );
    fs::write(repository.root.join("untracked"), "changed\n").unwrap();
    let id = owner.execute_family(plan).unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
    assert!(!destination.exists());
}

// Empty remotes have no commit to invent; regular/bare/mirror clones still
// verify the actual unborn identity and requested repository configuration.
#[test]
fn empty_clone() {
    let repository = Repository::new("sha1");
    let mut owner = repository.owner("");
    for family in [Family::Clone, Family::CloneBare, Family::CloneMirror] {
        let destination = repository.fixture.0.join(family.label());
        let plan = prepare(
            &mut owner,
            &repository.fixture.0,
            None,
            family,
            args(family, &repository.root, &destination),
        );
        execute(&mut owner, plan);
        assert_eq!(
            owner
                .family_result()
                .unwrap()
                .created
                .as_ref()
                .unwrap()
                .worktree
                .is_none(),
            family != Family::Clone
        );
    }
}

// A real Git wrapper delays only execution. Local init must obey the shorter
// operation deadline while clone uses the configured transport deadline.
#[test]
fn configured_deadlines() {
    let repository = Repository::new("sha1");
    repository.commit();
    fs::write(&repository.program, "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\ncase \"$2\" in init|clone) sleep 1.2;; esac\nexec /usr/bin/git \"$@\"\n").unwrap();
    let mut owner = repository.owner("operation_timeout_seconds=1\ntransport_timeout_seconds=3");
    let destination = repository.fixture.0.join("timed-init");
    let plan = prepare(
        &mut owner,
        &repository.fixture.0,
        None,
        Family::Init,
        args(Family::Init, &repository.root, &destination),
    );
    let id = owner.execute_family(plan).unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(id).unwrap().outcome, Outcome::TimedOut);
    assert_eq!(owner.record(id).unwrap().effects, Effects::Possible);
    assert!(!destination.exists());
    let destination = repository.fixture.0.join("clone-after-delay");
    let plan = prepare(
        &mut owner,
        &repository.fixture.0,
        None,
        Family::Clone,
        args(Family::Clone, &repository.root, &destination),
    );
    execute(&mut owner, plan);
    assert!(destination.join(".git/HEAD").exists());
}
