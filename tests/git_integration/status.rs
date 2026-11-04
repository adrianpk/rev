use super::support::{Repository, finish};
#[path = "presentation.rs"]
mod presentation;
use rev::{
    config,
    git::{Completeness, Group, Head, ObjectFormat, Outcome, Repositories},
};
use std::{ffi::OsString, fs, os::unix::ffi::OsStringExt, path::PathBuf, sync::Arc};

// Identity comes from real Git for both object formats and all supported HEAD
// forms. Merely finding a .git entry cannot establish these facts.
#[test]
fn repository_context() {
    for (format, expected) in [
        ("sha1", ObjectFormat::Sha1),
        ("sha256", ObjectFormat::Sha256),
    ] {
        let repository = Repository::new(format);
        let mut owner = repository.owner("");
        assert!(owner.contexts().is_empty());
        assert!(owner.remaining().is_none());
        let update = repository.open(&mut owner, &repository.root);
        assert!(update.published, "{:?}", owner.records());
        let context = owner.context(update.context.unwrap()).unwrap();
        assert_eq!(context.observation.identity.format, expected);
        assert_eq!(
            context.observation.identity.worktree.as_ref(),
            Some(&repository.root)
        );
        assert!(
            matches!(context.observation.status.as_ref().unwrap().head, Head::Unborn { ref branch } if branch == b"main")
        );
        fs::write(repository.root.join("source"), "first\n").unwrap();
        repository.commit();
        for detached in [false, true] {
            if detached {
                repository.git(["checkout", "--detach"]);
            }
            owner.refresh(context_id(&owner)).unwrap();
            assert!(finish(&mut owner).published, "{:?}", owner.records());
            let status = owner.contexts()[0].observation.status.as_ref().unwrap();
            let object = match &status.head {
                Head::Branch { name, object } => {
                    assert!(!detached);
                    assert_eq!(name, b"main");
                    object
                }
                Head::Detached { object } => {
                    assert!(detached);
                    object
                }
                _ => panic!("expected a real commit"),
            };
            assert_eq!(
                format!("{}\n", object.hex()).as_bytes(),
                repository.git(["rev-parse", "HEAD"])
            );
            assert!(status.paths.is_empty());
        }
        assert_eq!(owner.contexts().len(), 1);
        assert!(owner.admitted_bytes() < rev::git::SESSION_BYTES);
        assert!(owner.remaining().is_none());
    }
}

// Linked worktrees share common storage but have distinct per-worktree Git
// directories; nested repositories must resolve to their own root.
#[test]
fn worktree_identity() {
    let repository = Repository::new("sha1");
    repository.commit();
    let linked = repository.fixture.0.join("linked");
    repository.git([
        OsString::from("worktree"),
        OsString::from("add"),
        linked.clone().into_os_string(),
        OsString::from("-b"),
        OsString::from("linked"),
    ]);
    let mut owner = repository.owner("");
    assert!(repository.open(&mut owner, &repository.root).published);
    assert!(repository.open(&mut owner, &linked).published);
    assert_eq!(owner.contexts().len(), 2);
    let first = &owner.contexts()[0].observation.identity;
    let second = &owner.contexts()[1].observation.identity;
    assert_eq!(first.common_directory, second.common_directory);
    assert_ne!(first.git_directory, second.git_directory);
    assert_eq!(second.worktree.as_ref(), Some(&linked));
    let nested = repository.root.join("nested");
    fs::create_dir(&nested).unwrap();
    repository.git(["-C", nested.to_str().unwrap(), "init"]);
    assert!(repository.open(&mut owner, &nested).published);
    assert_eq!(owner.contexts().len(), 3);
    assert_eq!(
        owner.contexts()[2].observation.identity.worktree.as_ref(),
        Some(&nested)
    );
    let subdirectory = repository.root.join("child");
    fs::create_dir(&subdirectory).unwrap();
    assert!(repository.open(&mut owner, &subdirectory).published);
    assert_eq!(
        owner.contexts().len(),
        3,
        "a second entry spelling must reuse identity"
    );
}

// Git's raw pathname output, not lossy UI labels, owns identity. Repository
// roots and status records both retain newline and non-UTF-8 path bytes.
#[test]
fn exact_path_bytes() {
    let mut repository = Repository::new("sha1");
    let renamed = repository
        .fixture
        .0
        .join(OsString::from_vec(b"root\n\xff".to_vec()));
    fs::rename(&repository.root, &renamed).unwrap();
    repository.root = renamed;
    let paths: Vec<_> = [
        b"-leading".as_slice(),
        b"space name",
        b"line\nname",
        b"invalid-\xff",
    ]
    .iter()
    .map(|bytes| PathBuf::from(OsString::from_vec(bytes.to_vec())))
    .collect();
    for path in &paths {
        fs::write(repository.root.join(path), "untracked").unwrap();
    }
    let mut owner = repository.owner("");
    assert!(
        repository.open(&mut owner, &repository.root).published,
        "{:?}",
        owner.records()
    );
    let observation = &owner.contexts()[0].observation;
    assert_eq!(
        observation.identity.worktree.as_ref(),
        Some(&repository.root)
    );
    let actual: Vec<_> = observation
        .status
        .as_ref()
        .unwrap()
        .group(Group::Untracked)
        .map(|p| p.path.clone())
        .collect();
    assert_eq!(actual.len(), paths.len());
    for path in &paths {
        assert!(actual.contains(path));
    }
}

// A staged path can also have unstaged changes. Porcelain rename records carry
// a separate exact original name, and clean observations remain explicit.
#[test]
fn status_layers() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("original name"), "first\n").unwrap();
    repository.commit();
    repository.git(["mv", "original name", "renamed\nname"]);
    fs::write(repository.root.join("renamed\nname"), "second\n").unwrap();
    let mut owner = repository.owner("");
    assert!(
        repository.open(&mut owner, &repository.root).published,
        "{:?}",
        owner.records()
    );
    let status = owner.contexts()[0].observation.status.as_ref().unwrap();
    for group in [Group::Staged, Group::Unstaged] {
        let path = status.group(group).next().unwrap();
        assert_eq!(path.path, PathBuf::from("renamed\nname"));
        assert_eq!(path.old_path, Some(PathBuf::from("original name")));
        assert_eq!(path.objects.len(), 2);
    }
    assert_eq!(status.completeness, Completeness::Complete);
}

// Path-count admission is explicit, and a failed later refresh preserves the
// last immutable observation instead of publishing empty/clean placeholder data.
#[test]
fn bounded_replacement() {
    let repository = Repository::new("sha1");
    for name in ["a", "b", "c"] {
        fs::write(repository.root.join(name), "data").unwrap();
    }
    let mut owner = repository.owner("max_status_paths=2\nmax_process_records=2");
    assert!(repository.open(&mut owner, &repository.root).published);
    let id = context_id(&owner);
    let retained = owner.context(id).unwrap().observation.clone();
    let status = retained.status.as_ref().unwrap();
    assert_eq!(status.paths.len(), 2);
    assert_eq!(
        status.completeness,
        Completeness::Truncated {
            reason: "max_status_paths"
        }
    );
    fs::rename(
        repository.root.join(".git"),
        repository.root.join("unavailable"),
    )
    .unwrap();
    owner.refresh(id).unwrap();
    assert!(!finish(&mut owner).published);
    assert!(Arc::ptr_eq(
        &retained,
        &owner.context(id).unwrap().observation
    ));
    assert!(owner.context(id).unwrap().failure.is_some());
    assert_eq!(owner.records().last().unwrap().outcome, Outcome::Failed);
    assert!(owner.records().len() <= 2);
}

// Missing Git and non-repositories produce retained failure facts. A bare
// repository has a validated identity and explicitly lacks working-tree Status.
#[test]
fn unavailable_and_bare() {
    let repository = Repository::new("sha1");
    let mut owner = repository.owner("");
    assert!(!repository.open(&mut owner, &repository.fixture.0).published);
    assert!(owner.contexts().is_empty());
    let bare = repository.fixture.0.join("bare");
    repository.git(["init", "--bare", bare.to_str().unwrap()]);
    assert!(repository.open(&mut owner, &bare).published);
    let observation = &owner.contexts()[0].observation;
    assert!(observation.identity.worktree.is_none());
    assert!(observation.status.is_none());
    fs::remove_file(&repository.program).unwrap();
    assert!(!repository.open(&mut owner, &repository.root).published);
    assert!(
        owner
            .records()
            .last()
            .unwrap()
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("Start Git")
    );
    assert_eq!(owner.contexts().len(), 1);
}

// The compiled sample, include provenance and complete-candidate cross-field
// validation share the ordinary startup loader. Loading never needs Git.
#[test]
fn configuration_catalog() {
    let repository = Repository::new("sha1");
    let sample = repository
        .fixture
        .write("sample.toml", config::default_template());
    let startup = config::prepare(&sample);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    assert_eq!(startup.snapshot.settings().git, config::Git::default());
    let base = repository.fixture.write(
        "base.toml",
        "[git]\nmax_detail_bytes=4096\nmax_diagnostic_bytes=1024\nmax_output_bytes=8192",
    );
    let root = repository.fixture.write(
        "root.toml",
        "include=['base.toml']\n[git]\nmax_output_bytes=4096",
    );
    let startup = config::prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    assert_eq!(
        startup.snapshot.origin(&["git", "max_detail_bytes"]),
        Some(base.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["git", "max_output_bytes"]),
        Some(root.as_path())
    );
    for (name, body) in [
        ("empty program", "program=''"),
        ("negative timeout", "observation_timeout_seconds=-1"),
        ("excess paths", "max_status_paths=65537"),
        ("detail beyond output", "max_output_bytes=4096"),
        ("record below output", "max_process_record_bytes=65536"),
        ("unknown", "automatic_fetch=true"),
    ] {
        let root = repository.fixture.write(
            "invalid.toml",
            &format!("editor.tab_width=3\n[git]\n{body}"),
        );
        let startup = config::prepare(&root);
        assert!(startup.diagnostic.is_some(), "{name}");
        assert_eq!(
            startup.snapshot.settings(),
            &config::Settings::default(),
            "{name}"
        );
    }
    let owner = Repositories::new(&config::Snapshot::default(), Arc::new(|| {}));
    assert!(!owner.busy());
    assert_eq!(owner.admitted_bytes(), 65536);
}

// Resource ceilings apply to actual candidates. A high configured path ceiling
// does not reserve an enormous empty list; path/output exhaustion is explicit.
#[test]
fn byte_admission() {
    let repository = Repository::new("sha1");
    let mut owner = repository.owner("max_status_paths=65536\nmax_path_bytes=4194304");
    assert!(repository.open(&mut owner, &repository.root).published);
    for index in 0..120 {
        fs::write(
            repository
                .root
                .join(format!("{index:03}-{}", "name".repeat(15))),
            "data",
        )
        .unwrap();
    }
    let mut paths = repository.owner("max_path_bytes=4096");
    assert!(repository.open(&mut paths, &repository.root).published);
    let status = paths.contexts()[0].observation.status.as_ref().unwrap();
    assert_eq!(
        status.completeness,
        Completeness::Truncated {
            reason: "max_path_bytes"
        }
    );
    assert!(
        status
            .paths
            .iter()
            .map(|p| p.path.as_os_str().as_encoded_bytes().len())
            .sum::<usize>()
            <= 4096
    );
    let mut output =
        repository.owner("max_output_bytes=4096\nmax_detail_bytes=4096\nmax_diagnostic_bytes=1024");
    assert!(!repository.open(&mut output, &repository.root).published);
    assert!(output.contexts().is_empty());
    assert_eq!(
        output.records().last().unwrap().outcome,
        Outcome::OutputLimit
    );
    assert!(output.records().last().unwrap().output_truncated);
    assert!(output.admitted_bytes() < rev::git::SESSION_BYTES);
}

// Retained Arc observations remain charged across replacement. Context-count
// exhaustion preserves every existing identity rather than evicting live state.
#[test]
fn retained_contexts() {
    let repository = Repository::new("sha1");
    let mut owner = repository.owner("");
    assert!(repository.open(&mut owner, &repository.root).published);
    let retained = owner.contexts()[0].observation.clone();
    let id = retained.context;
    owner.refresh(id).unwrap();
    assert!(
        owner.refresh(id).is_err(),
        "one active operation per context"
    );
    assert!(finish(&mut owner).published);
    let before = owner.admitted_bytes();
    drop(retained);
    assert!(
        owner.admitted_bytes() < before,
        "external observation capacity must remain charged until released"
    );
    for index in 1..=4 {
        let directory = repository.fixture.0.join(format!("repository-{index}"));
        fs::create_dir(&directory).unwrap();
        repository.git(["-C", directory.to_str().unwrap(), "init"]);
        let update = repository.open(&mut owner, &directory);
        assert_eq!(update.published, index < 4);
    }
    assert_eq!(owner.contexts().len(), rev::git::MAX_CONTEXTS);
    assert!(owner.context(id).is_some());
    assert!(
        owner
            .records()
            .last()
            .unwrap()
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("context limit")
    );
}

// A cancellation token belongs to one operation, not a context or future job.
// After cleanup the same owner can obtain a fresh verified observation.
#[test]
fn operation_identity() {
    let repository = Repository::new("sha1");
    let mut owner = repository.owner("");
    let operation = owner.open(&repository.root).unwrap();
    owner.cancel(operation).unwrap();
    assert!(!finish(&mut owner).published);
    assert_eq!(owner.records().last().unwrap().outcome, Outcome::Cancelled);
    let fresh = owner.open(&repository.root).unwrap();
    assert_ne!(fresh, operation);
    assert!(
        owner.cancel(operation).is_err(),
        "a stale cancellation must not retarget fresh work"
    );
    assert!(finish(&mut owner).published, "{:?}", owner.records());
    assert!(!owner.busy());
}

fn context_id(owner: &Repositories) -> rev::git::ContextId {
    owner.contexts()[0].id
}

// Controlled output/deadline faults and a real concurrent HEAD change reject
// publication. Exit zero alone cannot replace confirmed repository facts.
#[test]
fn rejected_publication() {
    for case in ["malformed", "changed HEAD", "deadline"] {
        let repository = Repository::new("sha1");
        repository.commit();
        let mut owner = repository.owner(if case == "deadline" {
            "observation_timeout_seconds=2"
        } else {
            ""
        });
        repository.open(&mut owner, &repository.root);
        let retained = owner.contexts()[0].observation.clone();
        let injected = if case == "malformed" {
            "printf '# branch.oid (initial)\\000# branch.head main\\000invalid record\\000'\nexit 0"
        } else if case == "changed HEAD" {
            "/usr/bin/git \"$@\"\n/usr/bin/git symbolic-ref HEAD refs/heads/other\nexit 0"
        } else {
            "trap '' TERM\nexec sleep 60"
        };
        fs::write(&repository.program, format!("#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nif [ \"$2\" = status ]; then\n{injected}\nfi\nexec /usr/bin/git \"$@\"\n")).unwrap();
        owner.refresh(retained.context).unwrap();
        assert!(!finish(&mut owner).published, "{case}");
        let context = owner.context(retained.context).unwrap();
        assert!(Arc::ptr_eq(&retained, &context.observation), "{case}");
        assert!(context.failure.is_some());
        let record = owner.records().last().unwrap();
        assert_eq!(
            record.outcome,
            if case == "deadline" {
                Outcome::TimedOut
            } else {
                Outcome::Failed
            }
        );
        if case == "changed HEAD" {
            assert!(record.diagnostic.as_ref().unwrap().contains("HEAD changed"));
        }
        assert!(!owner.busy());
    }
}

// A real local remote and divergent commits establish tracking counts. A real
// merge conflict supplies porcelain-v2 unmerged layers without remote access.
#[test]
fn tracking_conflict() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("shared"), "base\n").unwrap();
    repository.commit();
    let remote = repository.fixture.0.join("remote.git");
    repository.git(["init", "--bare", remote.to_str().unwrap()]);
    repository.git(["remote", "add", "origin", remote.to_str().unwrap()]);
    repository.git(["push", "--set-upstream", "origin", "main"]);
    let peer = repository.fixture.0.join("peer");
    repository.git([
        "clone",
        "--local",
        "--branch=main",
        remote.to_str().unwrap(),
        peer.to_str().unwrap(),
    ]);
    repository.git([
        "-C",
        peer.to_str().unwrap(),
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "-c",
        "commit.gpgsign=false",
        "commit",
        "--allow-empty",
        "--message=Peer",
    ]);
    repository.git(["-C", peer.to_str().unwrap(), "push", "origin", "main"]);
    repository.git(["fetch", "origin"]);
    repository.git(["branch", "side"]);
    fs::write(repository.root.join("shared"), "main\n").unwrap();
    repository.commit();
    repository.git(["switch", "side"]);
    fs::write(repository.root.join("shared"), "side\n").unwrap();
    repository.commit();
    repository.git(["switch", "main"]);
    let conflict = std::process::Command::new(&repository.program)
        .args(["merge", "--no-edit", "side"])
        .current_dir(&repository.root)
        .output()
        .unwrap();
    assert_eq!(conflict.status.code(), Some(1));
    let mut owner = repository.owner("");
    assert!(repository.open(&mut owner, &repository.root).published);
    let status = owner.contexts()[0].observation.status.as_ref().unwrap();
    assert_eq!(status.upstream.as_deref(), Some(b"origin/main".as_slice()));
    assert_eq!(status.ahead, Some(1));
    assert_eq!(status.behind, Some(1));
    let paths: Vec<_> = status.group(Group::Unmerged).collect();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].path, PathBuf::from("shared"));
    assert_eq!(status.completeness, Completeness::Complete);
}
