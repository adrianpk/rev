use crate::support::{Repository, finish};
use rev::git::{Effects, MAX_COMMIT_BYTES, Outcome};
use std::{fs, os::unix::fs::PermissionsExt};

#[path = "commits_acceptance.rs"]
mod acceptance;
#[path = "commits_effects.rs"]
mod effects;
#[path = "commits_modes.rs"]
mod modes;
#[path = "commits_presentation.rs"]
mod presentation;

// Real Git supplies the object, tree and metadata oracle. Committing uses only
// the captured index, including an unborn/SHA-256 repository and a message too
// large for an argv token; working bytes stay outside the commit request.
#[test]
fn normal_objects() {
    for (format, unborn) in [("sha1", false), ("sha256", true)] {
        let repository = Repository::new(format);
        fs::write(repository.root.join("source"), "base\n").unwrap();
        if !unborn {
            repository.commit();
        }
        let parent = if unborn {
            Vec::new()
        } else {
            repository.git(["rev-parse", "HEAD"])
        };
        fs::write(repository.root.join("source"), "staged\n").unwrap();
        repository.git(["add", "source"]);
        fs::write(repository.root.join("source"), "unstaged\n").unwrap();
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let mut owner = repository.owner("");
        let context = repository
            .open(&mut owner, &repository.root)
            .context
            .unwrap();
        let observation = owner.commit_observation(context).unwrap();
        let message = format!(
            "Literal c 'quoted' $()\n\nUnicode λ雪 {}",
            "body ".repeat(1024)
        );
        owner.commit(observation, &message).unwrap();
        finish(&mut owner);
        let record = owner.records().last().unwrap();
        assert_eq!(
            (record.outcome, record.effects),
            (Outcome::Verified, Effects::Verified),
            "{:?}",
            record
        );
        assert!(record.argv.iter().any(|a| a == "--file=-"));
        assert_eq!(repository.git(["show", "HEAD:source"]), b"staged\n");
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        assert_eq!(
            fs::read(repository.root.join("source")).unwrap(),
            b"unstaged\n"
        );
        let object = repository.git(["cat-file", "commit", "HEAD"]);
        let boundary = object.windows(2).position(|b| b == b"\n\n").unwrap();
        assert_eq!(&object[boundary + 2..], message.as_bytes());
        let parents = repository.git(["show", "--format=%P", "--no-patch", "HEAD"]);
        assert_eq!(parents, if unborn { b"\n".to_vec() } else { parent });
        assert_eq!(
            repository.git(["show", "--format=%an <%ae>", "--no-patch", "HEAD"]),
            b"Example Author <author@example.invalid>\n"
        );
        assert!(owner.commit_draft(context).is_none());
    }
}

// Neither stale checked index nor a moved HEAD can authorize a replacement
// commit. Retained drafts survive a rejected request and remain session-only.
#[test]
fn normal_preconditions() {
    for moved_head in [false, true] {
        let repository = Repository::new("sha1");
        fs::write(repository.root.join("source"), "base\n").unwrap();
        repository.commit();
        fs::write(repository.root.join("source"), "first\n").unwrap();
        repository.git(["add", "source"]);
        let mut owner = repository.owner("");
        let context = repository
            .open(&mut owner, &repository.root)
            .context
            .unwrap();
        let observation = owner.commit_observation(context).unwrap();
        if moved_head {
            repository.git(["commit", "-m", "External"]);
        } else {
            fs::write(repository.root.join("source"), "replacement\n").unwrap();
            repository.git(["add", "source"]);
        }
        let head = repository.git(["rev-parse", "HEAD"]);
        let index = repository.git(["ls-files", "--stage", "-z"]);
        owner.commit(observation, "Retained λ draft").unwrap();
        finish(&mut owner);
        let record = owner.records().last().unwrap();
        assert_eq!(
            (record.outcome, record.effects),
            (Outcome::Failed, Effects::NotStarted)
        );
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        assert_eq!(owner.commit_draft(context), Some("Retained λ draft"));
    }
}

// Failed hooks remain enabled, expose actual diagnostics and keep the accepted
// text. A child that creates a commit and then fails is never labeled verified
// or automatically replayed even when its resulting object looks correct.
#[test]
fn normal_failure_evidence() {
    for post_effect in [false, true] {
        let repository = Repository::new("sha1");
        fs::write(repository.root.join("source"), "base\n").unwrap();
        repository.commit();
        fs::write(repository.root.join("source"), "staged\n").unwrap();
        repository.git(["add", "source"]);
        let head = repository.git(["rev-parse", "HEAD"]);
        if post_effect {
            fs::write(&repository.program, "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nif [ \"$2\" = commit ]; then /usr/bin/git \"$@\"; exit 70; fi\nexec /usr/bin/git \"$@\"\n").unwrap();
        } else {
            let hooks = repository.fixture.0.join("hooks");
            fs::create_dir(&hooks).unwrap();
            let hook = hooks.join("pre-commit");
            fs::write(
                &hook,
                "#!/bin/sh\necho 'HOOK_REJECT_SENTINEL' >&2\nexit 1\n",
            )
            .unwrap();
            fs::set_permissions(hook, fs::Permissions::from_mode(0o700)).unwrap();
            repository.git(["config", "core.hooksPath", hooks.to_str().unwrap()]);
        }
        let mut owner = repository.owner("");
        let context = repository
            .open(&mut owner, &repository.root)
            .context
            .unwrap();
        let observation = owner.commit_observation(context).unwrap();
        owner.commit(observation, "Keep this body\n\nλ").unwrap();
        finish(&mut owner);
        let record = owner.records().last().unwrap();
        assert_eq!(
            record.outcome,
            if post_effect {
                Outcome::Indeterminate
            } else {
                Outcome::Failed
            },
            "{record:?}"
        );
        assert_eq!(
            record.exit.unwrap().code(),
            Some(if post_effect { 70 } else { 1 })
        );
        assert_eq!(record.effects, Effects::Possible);
        if !post_effect {
            assert!(String::from_utf8_lossy(&record.stderr).contains("HOOK_REJECT_SENTINEL"));
        }
        assert_eq!(repository.git(["rev-parse", "HEAD"]) == head, !post_effect);
        assert_eq!(owner.commit_draft(context), Some("Keep this body\n\nλ"));
    }
}

// Message admission rejects unsupported NUL, empty and over-limit input before
// child execution; the previous retained draft and shared budget survive.
#[test]
fn normal_draft_limits() {
    let repository = Repository::new("sha1");
    let mut owner = repository.owner("");
    let context = repository
        .open(&mut owner, &repository.root)
        .context
        .unwrap();
    owner.retain_commit_draft(context, "previous").unwrap();
    let observation = owner.commit_observation(context).unwrap();
    for text in [
        " ".to_owned(),
        "a\0b".to_owned(),
        "x".repeat(MAX_COMMIT_BYTES + 1),
    ] {
        assert!(owner.commit(observation.clone(), &text).is_err());
        assert_eq!(owner.commit_draft(context), Some("previous"));
        assert!(!owner.busy());
    }
}
