use super::*;
use std::{
    os::unix::fs::PermissionsExt,
    time::{Duration, Instant},
};

fn prepared(
    repository: &Repository,
    owner: &mut Repositories,
    family: Family,
    args: Vec<OsString>,
) -> Arc<FamilyPlan> {
    repository.open(owner, &repository.root);
    let context = owner
        .contexts()
        .iter()
        .find(|c| c.observation.identity.worktree.as_ref() == Some(&repository.root))
        .unwrap()
        .id;
    prepare(owner, &repository.root, Some(context), family, args)
}
fn unknown(owner: &mut Repositories, plan: Arc<FamilyPlan>, code: i32) {
    let id = owner.execute_family(plan).unwrap();
    let update = finish(owner);
    assert!(update.published, "{:?}", owner.record(id));
    let record = owner.record(id).unwrap();
    assert_eq!(record.outcome, Outcome::Indeterminate, "{record:?}");
    assert_eq!(record.effects, Effects::Possible);
    assert_eq!(record.exit.and_then(|s| s.code()), Some(code));
    assert!(
        record
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("postconditions remain unknown")
    );
}

// Real literal Git argv preserves raw paths, empty operands and shell-looking
// bytes. Read and changing commands refresh facts without guessing postconditions.
#[test]
fn literal_git() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        fs::write(repository.root.join("base"), "base\n").unwrap();
        repository.commit();
        let head = repository.git(["rev-parse", "HEAD"]);
        let raw = OsString::from_vec(b"-$(touch injected)*[]\n\xff".to_vec());
        fs::write(repository.root.join(&raw), "literal\n").unwrap();
        let mut owner = repository.owner("");
        let args = vec!["add".into(), "--".into(), raw.clone()];
        let plan = prepared(&repository, &mut owner, Family::RunCommand, args.clone());
        let generation = plan.observation.as_ref().unwrap().generation;
        assert_eq!(plan.args, args);
        assert!(
            repository
                .git(["diff", "--cached", "--name-only"])
                .is_empty()
        );
        unknown(&mut owner, plan, 0);
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        assert!(!repository.root.join("injected").exists());
        let observation = &owner.contexts()[0].observation;
        assert!(observation.generation > generation);
        assert!(
            observation
                .status
                .as_ref()
                .unwrap()
                .group(rev::git::Group::Staged)
                .any(|p| p.path.as_os_str() == raw)
        );
        let record = owner.records().last().unwrap();
        assert_eq!(
            record.argv,
            [
                vec![
                    repository.program.as_os_str().into(),
                    "--no-optional-locks".into()
                ],
                args
            ]
            .concat()
        );
        for args in [
            vec!["rev-parse", "--sq-quote", ""],
            vec!["branch", "direct-created", "HEAD"],
            vec!["rev-parse", "HEAD"],
        ] {
            let args = args.into_iter().map(OsString::from).collect();
            let plan = prepared(&repository, &mut owner, Family::RunCommand, args);
            unknown(&mut owner, plan, 0);
        }
        assert_eq!(
            repository.git(["rev-parse", "refs/heads/direct-created"]),
            head
        );
        assert_eq!(owner.records().last().unwrap().stdout, head);
        let plan = prepared(
            &repository,
            &mut owner,
            Family::RunCommand,
            vec![
                "rev-parse".into(),
                "--verify".into(),
                "missing-object".into(),
            ],
        );
        let id = owner.execute_family(plan).unwrap();
        assert!(finish(&mut owner).published);
        let record = owner.record(id).unwrap();
        assert_eq!(record.outcome, Outcome::Indeterminate);
        assert!(record.exit.is_some_and(|s| !s.success()));
        assert!(!record.stderr.is_empty());
    }
}

// Explicit shell mode reuses configured executable/args, performs actual pipes
// and writes, and retains actual output while Git saved-state refresh stays honest.
#[test]
fn explicit_shell() {
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        fs::write(repository.root.join("text"), "base\n").unwrap();
        repository.commit();
        let program = repository.fixture.write(
            "selected-shell",
            "#!/bin/sh\nprintf selected > shell-selected\nexec /bin/sh \"$@\"\n",
        );
        fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).unwrap();
        let preferences = format!(
            "[shell]\nprogram={}\nargs=['-c']\ntimeout_seconds=2\nmax_output_bytes=65536",
            serde_json::to_string(&program).unwrap()
        );
        let mut owner = repository.owner(&preferences);
        let script = "printf 'shell-output\\n' | tr a-z A-Z; printf diagnostic >&2; printf 'changed by shell\\n' > text";
        let plan = prepared(
            &repository,
            &mut owner,
            Family::RunShellCommand,
            vec![script.into()],
        );
        assert_eq!(plan.program, program.to_str().unwrap());
        assert_eq!(plan.args, vec![OsString::from("-c"), script.into()]);
        assert_eq!(plan.direct_limits(), Some((2, 65536)));
        assert!(!repository.root.join("shell-selected").exists());
        unknown(&mut owner, plan, 0);
        assert_eq!(
            fs::read(repository.root.join("text")).unwrap(),
            b"changed by shell\n"
        );
        assert_eq!(
            fs::read(repository.root.join("shell-selected")).unwrap(),
            b"selected"
        );
        let record = owner.records().last().unwrap();
        assert_eq!(record.stdout, b"SHELL-OUTPUT\n");
        assert_eq!(record.stderr, b"diagnostic");
        assert_eq!(
            record.argv,
            vec![program.into_os_string(), "-c".into(), script.into()]
        );
        assert!(
            owner.contexts()[0]
                .observation
                .status
                .as_ref()
                .unwrap()
                .group(rev::git::Group::Unstaged)
                .any(|p| p.path == Path::new("text"))
        );
    }
}

// No repository context is invented outside Git. Both explicit creation and
// ordinary shell files still retain unknown results and inspectable child facts.
#[test]
fn outside_context() {
    let repository = Repository::new("sha1");
    let outside = repository.fixture.0.join("ordinary");
    fs::create_dir(&outside).unwrap();
    let mut owner = repository.owner("[shell]\nprogram='/bin/sh'");
    for (family, args) in [
        (Family::RunCommand, vec!["init", "--", "created"]),
        (Family::RunShellCommand, vec!["printf outside > marker"]),
    ] {
        let plan = prepare(
            &mut owner,
            &outside,
            None,
            family,
            args.into_iter().map(OsString::from).collect(),
        );
        unknown(&mut owner, plan, 0);
        assert!(owner.contexts().is_empty());
        assert!(owner.family_result().unwrap().created.is_none());
    }
    assert!(outside.join("created/.git/HEAD").is_file());
    assert_eq!(fs::read(outside.join("marker")).unwrap(), b"outside");
}

// Loss of source identity preserves the last confirmed observation with failed
// freshness. The actual successful child facts survive a failed Git refresh.
#[test]
fn incompatible_refresh() {
    let repository = Repository::new("sha1");
    repository.commit();
    let mut owner = repository.owner("[shell]\nprogram='/bin/sh'");
    let plan = prepared(
        &repository,
        &mut owner,
        Family::RunShellCommand,
        vec!["mv .git moved-git; printf actual-child".into()],
    );
    let prior = plan.observation.as_ref().unwrap().clone();
    unknown(&mut owner, plan, 0);
    let context = &owner.contexts()[0];
    assert!(Arc::ptr_eq(&context.observation, &prior));
    assert!(context.failure.is_some());
    let record = owner.records().last().unwrap();
    assert_eq!(record.stdout, b"actual-child");
    assert!(
        record
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("refresh unavailable")
    );
    assert!(!repository.root.join(".git").exists());
    assert!(repository.root.join("moved-git/HEAD").is_file());
}

// Input/cwd/source/config changes reject before a child. Command-size limits
// cover the configured executable and prefixes, not just the editable operands.
#[test]
fn direct_protection() {
    for change in ["source", "head", "index", "config", "directory"] {
        let repository = Repository::new("sha1");
        fs::write(repository.root.join("text"), "base\n").unwrap();
        repository.commit();
        let mut owner = repository.owner("");
        let plan = prepared(
            &repository,
            &mut owner,
            Family::RunCommand,
            vec!["branch".into(), "must-not-exist".into(), "HEAD".into()],
        );
        match change {
            "source" => fs::write(repository.root.join("text"), "external\n").unwrap(),
            "head" => {
                repository.git(["commit", "--allow-empty", "--message=External"]);
            }
            "index" => {
                fs::write(repository.root.join("text"), "staged\n").unwrap();
                repository.git(["add", "text"]);
            }
            "config" => {
                repository.git(["config", "user.name", "Changed"]);
            }
            "directory" => {
                fs::write(repository.root.join("external"), "new\n").unwrap();
            }
            _ => unreachable!(),
        }
        let id = owner.execute_family(plan).unwrap();
        finish(&mut owner);
        assert_eq!(
            owner.record(id).unwrap().effects,
            Effects::NotStarted,
            "{change}"
        );
        assert!(
            repository
                .git([
                    "for-each-ref",
                    "--format=%(refname)",
                    "refs/heads/must-not-exist"
                ])
                .is_empty()
        );
    }
    let repository = Repository::new("sha1");
    let mut owner = repository.owner("[shell]\nprogram='/bin/sh'");
    for (family, args) in [
        (Family::RunCommand, vec![]),
        (Family::RunCommand, vec![OsString::from("\0")]),
        (
            Family::RunShellCommand,
            vec![OsString::from("x".repeat(4097))],
        ),
        (
            Family::RunShellCommand,
            vec![OsString::from_vec(vec![0xff])],
        ),
    ] {
        assert!(
            owner
                .prepare_family(&repository.root, None, family, args)
                .is_err()
        );
    }
    let id = owner
        .prepare_family(
            &repository.root,
            None,
            Family::RunCommand,
            vec![OsString::from("status"); 63],
        )
        .unwrap();
    assert!(!finish(&mut owner).published);
    assert_eq!(owner.record(id).unwrap().effects, Effects::NotStarted);
}

// Direct work retains exclusive admission across known repositories. Real Git
// aliases and explicit shells keep their descendants owned through cancellation;
// shell timeout and output caps cannot become success during later refresh.
#[test]
fn direct_lifetime() {
    for family in [Family::RunCommand, Family::RunShellCommand] {
        let repository = Repository::new("sha1");
        let other = Repository::new("sha1");
        repository.commit();
        other.commit();
        let mut owner = repository.owner("[shell]\nprogram='/bin/sh'");
        repository.open(&mut owner, &repository.root);
        other.open(&mut owner, &other.root);
        let other_context = owner
            .contexts()
            .iter()
            .find(|c| c.observation.identity.worktree.as_ref() == Some(&other.root))
            .unwrap()
            .id;
        let script = "echo $$ > leader; sleep 30 & echo $! > descendant; printf before; wait";
        let args = if family == Family::RunShellCommand {
            vec![script.into()]
        } else {
            vec![
                "-c".into(),
                format!("alias.direct-wait=!{script}").into(),
                "direct-wait".into(),
            ]
        };
        let plan = prepared(&repository, &mut owner, family, args);
        let id = owner.execute_family(plan).unwrap();
        let deadline = Instant::now() + Duration::from_secs(4);
        while !fs::read_to_string(repository.root.join("descendant"))
            .is_ok_and(|s| s.trim().parse::<u32>().is_ok())
        {
            assert!(Instant::now() < deadline, "direct child did not start");
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(owner.refresh(other_context).is_err());
        owner.cancel(id).unwrap();
        finish(&mut owner);
        assert_eq!(owner.record(id).unwrap().outcome, Outcome::Cancelled);
        assert!(!owner.busy());
        assert!(Instant::now() < deadline);
        for name in ["leader", "descendant"] {
            let pid = fs::read_to_string(repository.root.join(name)).unwrap();
            let stat = fs::read_to_string(format!("/proc/{}/stat", pid.trim()));
            assert!(stat.is_err() || stat.unwrap().split_once(") ").unwrap().1.starts_with('Z'));
        }
    }
    for (preferences, script, outcome) in [
        (
            "timeout_seconds=1",
            "printf before; sleep 30",
            Outcome::TimedOut,
        ),
        (
            "max_output_bytes=8",
            "printf 0123456789",
            Outcome::OutputLimit,
        ),
    ] {
        let repository = Repository::new("sha1");
        let mut owner = repository.owner(&format!("[shell]\nprogram='/bin/sh'\n{preferences}"));
        let plan = prepare(
            &mut owner,
            &repository.fixture.0,
            None,
            Family::RunShellCommand,
            vec![script.into()],
        );
        let id = owner.execute_family(plan).unwrap();
        let started = Instant::now();
        finish(&mut owner);
        let record = owner.record(id).unwrap();
        assert_eq!(record.outcome, outcome, "{record:?}");
        assert_eq!(record.effects, Effects::Possible);
        assert!(started.elapsed() < Duration::from_secs(3));
        if outcome == Outcome::OutputLimit {
            assert!(record.output_truncated);
            assert_eq!(record.stdout, b"01234567");
        }
    }
}
