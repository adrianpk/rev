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

// Notes operate on full object/ref identities, retain literal message bytes
// after Git cleanup, and leave HEAD/index/saved sources and other notes intact.
#[test]
fn notes_roundtrip() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        fs::write(repository.root.join("source"), "preserved\n").unwrap();
        repository.commit();
        let head = repository.git(["rev-parse", "HEAD"]);
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let name = OsStr::new("refs/notes/review");
        let message = "Literal $(touch sentinel); note\n\nbody\n";
        let mut owner = repository.owner("");
        let add = plan(
            &repository,
            &mut owner,
            Family::NotesAdd,
            &[name, OsStr::new(message), OsStr::new("HEAD")],
        );
        assert_eq!(
            add.target_object().unwrap().hex().as_bytes(),
            head.strip_suffix(b"\n").unwrap()
        );
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        execute(&mut owner, add);
        let note = owner.family_result().unwrap().object.unwrap();
        assert_eq!(
            repository.git(["cat-file", "blob", &note.hex()]),
            message.as_bytes()
        );
        assert_eq!(
            repository.git(["notes", "--ref", "refs/notes/review", "show", "HEAD"]),
            message.as_bytes()
        );
        repository.git([
            "notes",
            "--ref",
            "refs/notes/other",
            "add",
            "-m",
            "retained",
            "HEAD",
        ]);
        let remove = plan(
            &repository,
            &mut owner,
            Family::NotesRemove,
            &[name, OsStr::new("HEAD")],
        );
        execute(&mut owner, remove);
        assert!(
            repository
                .git(["notes", "--ref", "refs/notes/review", "list"])
                .is_empty()
        );
        assert_eq!(
            repository.git(["notes", "--ref", "refs/notes/other", "show", "HEAD"]),
            b"retained\n"
        );

        let orphan_file = repository
            .fixture
            .write("orphan-input", "unreachable annotation target\n");
        let orphan = String::from_utf8(repository.git([
            OsStr::new("hash-object"),
            OsStr::new("-w"),
            OsStr::new("--"),
            orphan_file.as_os_str(),
        ]))
        .unwrap();
        let orphan = orphan.trim();
        repository.git([
            "notes",
            "--ref",
            "refs/notes/review",
            "add",
            "-m",
            "orphan",
            orphan,
        ]);
        repository.git([
            "notes",
            "--ref",
            "refs/notes/review",
            "add",
            "-m",
            "keep",
            "HEAD",
        ]);
        // This loose blob is fixture-owned and unreachable from the commit;
        // removing it creates a real missing-object prune candidate.
        fs::remove_file(
            repository
                .root
                .join(".git/objects")
                .join(&orphan[..2])
                .join(&orphan[2..]),
        )
        .unwrap();
        let prune = plan(&repository, &mut owner, Family::NotesPrune, &[name]);
        assert_eq!(
            prune
                .prune_objects()
                .iter()
                .map(|o| o.hex())
                .collect::<Vec<_>>(),
            vec![orphan.to_owned()]
        );
        execute(&mut owner, prune);
        assert_eq!(
            repository.git(["notes", "--ref", "refs/notes/review", "show", "HEAD"]),
            b"keep\n"
        );
        let listed = repository.git(["notes", "--ref", "refs/notes/review", "list"]);
        assert_eq!(
            listed
                .split(|b| *b == b'\n')
                .filter(|l| !l.is_empty())
                .count(),
            1
        );
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        assert_eq!(
            fs::read(repository.root.join("source")).unwrap(),
            b"preserved\n"
        );
        assert!(!repository.root.join("sentinel").exists());
    }
}

// Review is invalidated by a note/ref/config race before effects, while bare
// repositories support the same exact notes operation without a fake index.
#[test]
fn notes_checked_state() {
    for changed in ["note", "other note", "config", "head"] {
        let repository = Repository::new("sha1");
        repository.commit();
        let mut owner = repository.owner("");
        let add = plan(
            &repository,
            &mut owner,
            Family::NotesAdd,
            &[
                OsStr::new("refs/notes/review"),
                OsStr::new("accepted"),
                OsStr::new("HEAD"),
            ],
        );
        match changed {
            "note" => {
                repository.git([
                    "notes",
                    "--ref",
                    "refs/notes/review",
                    "add",
                    "-m",
                    "external",
                    "HEAD",
                ]);
            }
            "other note" => {
                repository.git([
                    "notes",
                    "--ref",
                    "refs/notes/other",
                    "add",
                    "-m",
                    "external",
                    "HEAD",
                ]);
            }
            "config" => {
                repository.git(["config", "notes.rewriteMode", "concatenate"]);
            }
            "head" => {
                repository.commit();
            }
            _ => unreachable!(),
        }
        let id = owner.execute_family(add).unwrap();
        finish(&mut owner);
        assert_eq!(
            owner.record(id).unwrap().effects,
            Effects::NotStarted,
            "{changed}: {:?}",
            owner.record(id)
        );
    }
    for target_exists in [false, true] {
        let repository = Repository::new("sha1");
        repository.commit();
        if target_exists {
            repository.git([
                "notes",
                "--ref",
                "refs/notes/other",
                "add",
                "-m",
                "external",
                "HEAD",
            ]);
        }
        repository.git(["symbolic-ref", "refs/notes/review", "refs/notes/other"]);
        let mut owner = repository.owner("");
        repository.open(&mut owner, &repository.root);
        let context = owner.contexts()[0].id;
        let rejected = owner
            .prepare_family(
                &repository.root,
                Some(context),
                Family::NotesAdd,
                arguments(
                    Family::NotesAdd,
                    &[
                        OsStr::new("refs/notes/review"),
                        OsStr::new("refused"),
                        OsStr::new("HEAD"),
                    ],
                ),
            )
            .unwrap();
        finish(&mut owner);
        assert_eq!(owner.record(rejected).unwrap().effects, Effects::NotStarted);
        assert!(
            owner
                .record(rejected)
                .unwrap()
                .diagnostic
                .as_ref()
                .unwrap()
                .contains("direct full ref")
        );
        if target_exists {
            assert_eq!(
                repository.git(["notes", "--ref", "refs/notes/other", "show", "HEAD"]),
                b"external\n"
            );
        }
    }
    let mut repository = Repository::new("sha256");
    repository.commit();
    let bare = repository.fixture.0.join("bare");
    repository.git([
        OsStr::new("clone"),
        OsStr::new("--bare"),
        OsStr::new("--"),
        repository.root.as_os_str(),
        bare.as_os_str(),
    ]);
    repository.root = bare;
    repository.git(["config", "user.name", "Example Author"]);
    repository.git(["config", "user.email", "author@example.invalid"]);
    let mut owner = repository.owner("");
    let add = plan(
        &repository,
        &mut owner,
        Family::NotesAdd,
        &[
            OsStr::new("refs/notes/review"),
            OsStr::new("bare note"),
            OsStr::new("HEAD"),
        ],
    );
    execute(&mut owner, add);
    assert_eq!(
        repository.git(["notes", "--ref", "refs/notes/review", "show", "HEAD"]),
        b"bare note\n"
    );
    assert!(owner.contexts()[0].observation.status.is_none());
}

pub(super) fn sparse_repository(format: &str) -> Repository {
    let repository = Repository::new(format);
    for directory in ["alpha", "beta", "nested/deep"] {
        fs::create_dir_all(repository.root.join(directory)).unwrap();
        fs::write(
            repository.root.join(directory).join("file"),
            format!("{directory}\n"),
        )
        .unwrap();
    }
    fs::write(repository.root.join("root"), "root stays\n").unwrap();
    repository.commit();
    repository
}

// All five sparse variants prove actual materialization and index skip flags
// without changing HEAD, index objects, tracked bytes or unrelated config.
#[test]
fn sparse_roundtrip() {
    for format in ["sha1", "sha256"] {
        let repository = sparse_repository(format);
        let head = repository.git(["rev-parse", "HEAD"]);
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let mut owner = repository.owner("");
        for (family, values, alpha, beta) in [
            (Family::SparseEnable, vec![], false, false),
            (Family::SparseSet, vec![OsStr::new("alpha")], true, false),
            (Family::SparseAdd, vec![OsStr::new("beta")], true, true),
            (Family::SparseReapply, vec![], true, true),
            (Family::SparseDisable, vec![], true, true),
        ] {
            let prepared = plan(&repository, &mut owner, family, &values);
            execute(&mut owner, prepared);
            assert_eq!(
                repository.root.join("alpha/file").exists(),
                alpha,
                "{format} {family:?}"
            );
            assert_eq!(
                repository.root.join("beta/file").exists(),
                beta,
                "{format} {family:?}"
            );
            assert_eq!(
                repository.root.join("nested/deep/file").exists(),
                family == Family::SparseDisable
            );
            assert_eq!(
                fs::read(repository.root.join("root")).unwrap(),
                b"root stays\n"
            );
            assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
            assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
            let flags = repository.git(["ls-files", "-t", "-z"]);
            assert_eq!(
                flags.split(|b| *b == 0).any(|f| f.starts_with(b"S ")),
                family != Family::SparseDisable
            );
            assert_eq!(repository.git(["config", "user.name"]), b"Example Author\n");
        }
        assert_eq!(
            fs::read(repository.root.join("alpha/file")).unwrap(),
            b"alpha\n"
        );
        assert_eq!(
            fs::read(repository.root.join("beta/file")).unwrap(),
            b"beta\n"
        );
        assert_eq!(
            fs::read(repository.root.join("nested/deep/file")).unwrap(),
            b"nested/deep\n"
        );
    }
}

// Literal metacharacters and non-UTF8 bytes survive Git's quoted listing; cone
// requests must preserve their observable Git selection without pathspec magic.
#[test]
fn sparse_literal_paths() {
    let repository = sparse_repository("sha256");
    let raw = OsString::from_vec(b"-literal[1]\xff".to_vec());
    fs::create_dir(repository.root.join(&raw)).unwrap();
    fs::write(repository.root.join(&raw).join("file"), "raw\n").unwrap();
    repository.commit();
    let mut owner = repository.owner("");
    let newline = OsString::from_vec(b"line\nbreak".to_vec());
    fs::create_dir(repository.root.join(&newline)).unwrap();
    fs::write(repository.root.join(&newline).join("file"), "newline\n").unwrap();
    repository.commit();
    repository.open(&mut owner, &repository.root);
    let context = owner.contexts()[0].id;
    let rejected = owner
        .prepare_family(
            &repository.root,
            Some(context),
            Family::SparseSet,
            arguments(Family::SparseSet, &[&newline]),
        )
        .unwrap();
    finish(&mut owner);
    assert_eq!(owner.record(rejected).unwrap().effects, Effects::NotStarted);
    assert!(
        owner
            .record(rejected)
            .unwrap()
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("cannot represent a newline")
    );
    assert_eq!(
        fs::read(repository.root.join(&newline).join("file")).unwrap(),
        b"newline\n"
    );
    assert!(repository.root.join("alpha/file").is_file());
    let set = plan(&repository, &mut owner, Family::SparseSet, &[&raw]);
    execute(&mut owner, set);
    assert_eq!(
        fs::read(repository.root.join(&raw).join("file")).unwrap(),
        b"raw\n"
    );
    assert!(!repository.root.join("alpha/file").exists());
    let mut values = arguments(Family::SparseSet, &[OsStr::new("nested")]);
    values.push("nested/deep".into());
    repository.open(&mut owner, &repository.root);
    let context = owner.contexts()[0].id;
    let set = prepare(
        &mut owner,
        &repository.root,
        Some(context),
        Family::SparseSet,
        values,
    );
    execute(&mut owner, set);
    assert_eq!(
        fs::read(repository.root.join("nested/deep/file")).unwrap(),
        b"nested/deep\n"
    );
    assert!(!repository.root.join(&raw).join("file").exists());
}

// Saved dirty state, manually materialized excluded files and changed review
// evidence refuse before execution; no forced cleanup hides external changes.
#[test]
fn sparse_protected_state() {
    for variant in ["saved", "patterns", "flags", "config", "materialized"] {
        let repository = sparse_repository("sha1");
        let mut owner = repository.owner("");
        let enable = plan(&repository, &mut owner, Family::SparseEnable, &[]);
        execute(&mut owner, enable);
        let set = plan(
            &repository,
            &mut owner,
            Family::SparseSet,
            &[OsStr::new("alpha")],
        );
        match variant {
            "saved" => {
                fs::write(repository.root.join("root"), "external\n").unwrap();
            }
            "patterns" => {
                fs::write(repository.root.join(".git/info/sparse-checkout"), "/*\n").unwrap();
            }
            "flags" => {
                repository.git(["update-index", "--no-skip-worktree", "--", "beta/file"]);
            }
            "config" => {
                repository.git(["config", "user.name", "External Author"]);
            }
            "materialized" => {
                fs::create_dir(repository.root.join("beta")).unwrap();
                fs::write(repository.root.join("beta/file"), "external\n").unwrap();
            }
            _ => unreachable!(),
        }
        let id = owner.execute_family(set).unwrap();
        finish(&mut owner);
        assert_eq!(
            owner.record(id).unwrap().effects,
            Effects::NotStarted,
            "{variant}: {:?}",
            owner.record(id)
        );
        assert!(!repository.root.join("alpha/file").exists());
        if variant == "materialized" {
            assert_eq!(
                fs::read(repository.root.join("beta/file")).unwrap(),
                b"external\n"
            );
        }
    }
}

// Sparse state belongs to the selected linked worktree. Its private patterns,
// materialization and worktree config must not reshape the primary checkout.
#[test]
fn sparse_linked_worktree() {
    let mut repository = sparse_repository("sha1");
    let primary = repository.root.clone();
    let linked = repository.fixture.0.join("linked");
    repository.git([
        OsStr::new("worktree"),
        OsStr::new("add"),
        OsStr::new("-b"),
        OsStr::new("linked"),
        OsStr::new("--"),
        linked.as_os_str(),
        OsStr::new("HEAD"),
    ]);
    repository.root = linked;
    let mut owner = repository.owner("");
    let set = plan(
        &repository,
        &mut owner,
        Family::SparseSet,
        &[OsStr::new("alpha")],
    );
    execute(&mut owner, set);
    assert!(repository.root.join("alpha/file").is_file());
    assert!(!repository.root.join("beta/file").exists());
    assert!(primary.join("beta/file").is_file());
    let git_directory = owner.contexts()[0]
        .observation
        .identity
        .git_directory
        .clone();
    assert!(git_directory.join("info/sparse-checkout").is_file());
    assert!(!primary.join(".git/info/sparse-checkout").exists());
    let disable = plan(&repository, &mut owner, Family::SparseDisable, &[]);
    execute(&mut owner, disable);
    assert_eq!(
        fs::read(repository.root.join("beta/file")).unwrap(),
        b"beta\n"
    );
}
