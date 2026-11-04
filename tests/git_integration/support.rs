use super::fixture::Fixture;
use rev::{
    config,
    git::{Repositories, Update},
};
use std::{
    ffi::OsStr,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::{Duration, Instant},
};

pub struct Repository {
    pub fixture: Fixture,
    pub root: PathBuf,
    pub program: PathBuf,
}

impl Repository {
    pub fn new(format: &str) -> Self {
        let fixture = Fixture::new();
        let root = fixture.0.join("repository");
        std::fs::create_dir(&root).unwrap();
        // This wrapper runs installed Git with isolated configuration. It does
        // not substitute Git output or model repository behavior.
        let program = fixture.write("git", "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nexec /usr/bin/git \"$@\"\n");
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
        let repository = Self {
            fixture,
            root,
            program,
        };
        repository.git([
            "init",
            "--initial-branch=main",
            &format!("--object-format={format}"),
        ]);
        for (key, value) in [
            ("user.name", "Example Author"),
            ("user.email", "author@example.invalid"),
            ("commit.gpgsign", "false"),
            ("tag.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            repository.git(["config", key, value]);
        }
        repository
    }

    pub fn git(&self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Vec<u8> {
        let output = Command::new(&self.program)
            .args(args)
            .current_dir(&self.root)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "Git fixture: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    }

    pub fn commit(&self) {
        self.git(["add", "--all"]);
        self.git(["commit", "--allow-empty", "--message=Fixture"]);
    }

    pub fn snapshot(&self, extra: &str) -> config::Snapshot {
        let text = format!(
            "[git]\nprogram={}\n{extra}",
            serde_json::to_string(&self.program).unwrap()
        );
        let root = self.fixture.write("config.toml", &text);
        let startup = config::prepare(&root);
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        startup.snapshot
    }

    pub fn owner(&self, extra: &str) -> Repositories {
        Repositories::new(&self.snapshot(extra), Arc::new(|| {}))
    }

    pub fn open(&self, owner: &mut Repositories, directory: &Path) -> Update {
        let operation = owner.open(directory).unwrap();
        let update = finish(owner);
        assert_eq!(update.operation, operation);
        update
    }
}

pub fn finish(owner: &mut Repositories) -> Update {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let mut updates = owner.poll();
        if !updates.is_empty() {
            return updates.remove(0);
        }
        assert!(
            Instant::now() < deadline,
            "Git work did not finish: {:?}",
            owner.progress().collect::<Vec<_>>()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}
