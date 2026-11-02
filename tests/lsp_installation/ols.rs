use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use rev::lsp::install::{self, Registry, State, Storage};
use rev::org::Cancellation;

use super::optional::{executable, fixtures};
use super::storage::roots;
use crate::fixture::Fixture;

pub(super) fn tools(directory: &Path) {
    fixtures(directory);
    for name in ["git", "odin"] {
        executable(
            &directory.join("bin").join(name),
            include_str!("ols-tools.sh"),
        );
    }
    let upstream = directory.join("upstream");
    fs::create_dir(&upstream).unwrap();
    fs::create_dir(upstream.join("builtin")).unwrap();
    fs::write(upstream.join("builtin/fixture.odin"), "package builtin\n").unwrap();
    // The upstream build wrapper can exit zero after the compiler fails. The
    // installer must check its output and transport rather than trust that code.
    fs::write(
        upstream.join("build.sh"),
        "odin build src/ -out:ols\nexit 0\n",
    )
    .unwrap();
    for args in [
        vec!["init", "--template="],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Owned Fixture",
            "-c",
            "user.email=fixture@example.test",
            "commit",
            "-m",
            "owned source",
        ],
    ] {
        let output = Command::new("/usr/bin/git")
            .args(args)
            .current_dir(&upstream)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let head = Command::new("/usr/bin/git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&upstream)
        .output()
        .unwrap();
    assert!(head.status.success());
    fs::write(directory.join("revision"), head.stdout).unwrap();
}

// Local Git really checks out a pinned commit; a controlled installed Odin
// boundary builds the protocol peer. This does not establish upstream semantics.
#[test]
fn pinned_build() {
    let Some(directory) = std::env::var_os("REV_OLS_CHILD") else {
        let f = Fixture::new();
        tools(&f.0);
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "ols::pinned_build", "--nocapture"])
            .env("REV_OLS_CHILD", &f.0)
            .env("REV_OPTIONAL_FIXTURE", &f.0)
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", f.0.join("bin").display()),
            )
            .env("GIT_DIR", f.0.join("untouched-git"))
            .env("GIT_OBJECT_DIRECTORY", f.0.join("untouched-objects"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let acquire = |storage: &mut Storage| {
        install::install(
            storage,
            "ols",
            "odin",
            BTreeMap::new(),
            &Cancellation::default(),
        )
    };
    let outcome = acquire(&mut storage).unwrap();
    assert_eq!(outcome.installation.status, State::Ready);
    assert_eq!(
        outcome.installation.version,
        fs::read_to_string(directory.join("revision"))
            .unwrap()
            .trim()
    );
    assert_eq!(
        outcome.installation.evidence,
        format!("git-revision:{}", outcome.installation.version)
    );
    let payload = roots.data.join(&outcome.installation.payload);
    assert_eq!(
        outcome.installation.launches[0].env["OLS_BUILTIN_FOLDER"],
        payload.join("source/builtin").to_str().unwrap()
    );
    let frames = fs::read_to_string(directory.join("probe.frames")).unwrap();
    for field in ["enable_snippets", "enable_fake_methods"] {
        assert!(frames.contains(&format!("\"{field}\":false")), "{frames}");
    }
    let original = fs::read(payload.join("source/ols")).unwrap();
    for (failure, expected) in [
        ("bad-revision", "immutable"),
        ("wrong-head", "differs"),
        ("reject-build", "produced no executable"),
        ("no-builtin", "No such file"),
        ("reject-probe", "probe failed"),
    ] {
        fs::write(directory.join(failure), "").unwrap();
        let error = acquire(&mut storage).err().expect(failure);
        assert!(error.contains(expected), "{failure}: {error}");
        fs::remove_file(directory.join(failure)).unwrap();
        let registry = Registry::load(&roots.data).unwrap();
        assert_eq!(registry.installations[0], outcome.installation);
        assert_eq!(
            registry
                .installations
                .iter()
                .filter(|i| i.status == State::Ready)
                .count(),
            1
        );
        assert_eq!(fs::read(payload.join("source/ols")).unwrap(), original);
        assert!(!roots.config.exists());
        assert!(
            fs::read_dir(roots.cache.join("lsp"))
                .unwrap()
                .next()
                .is_none()
        );
    }
    assert!(!directory.join("untouched-git").exists());
    assert!(!directory.join("untouched-objects").exists());
}
