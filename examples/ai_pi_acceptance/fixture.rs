use super::support::Result;
use std::fs;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) const SOURCE: &str = "const needle: u8 = 7;\n";
pub(crate) const GENERATED: &str = "// Controlled offline proposal.\n";

// The executable is an explicit native installation, never a PATH updater.
// This validation-only proxy adds a local provider inside Pi's own runtime.
pub(crate) fn executable() -> Result<PathBuf> {
    let path = PathBuf::from(
        std::env::var_os("REV_PI_EXECUTABLE")
            .ok_or("pin REV_PI_EXECUTABLE to installed Pi 1.3.0")?,
    )
    .canonicalize()?;
    let mut magic = [0; 4];
    fs::File::open(&path)?.read_exact(&mut magic)?;
    if magic != *b"\x7fELF" {
        return Err("REV_PI_EXECUTABLE must be the native binary, not an update wrapper".into());
    }
    Ok(path)
}

pub(crate) fn proxy(args: &[PathBuf]) -> Result<()> {
    let mut command = Command::new(executable()?);
    command.args(args);
    if args.first().is_some_and(|a| a == "--mode") {
        let resource = if std::env::var_os("REV_PI_LIVE").is_some() {
            "live-guard.js"
        } else {
            "provider.js"
        };
        command.arg("--extension").arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples/ai_pi_acceptance")
                .join(resource),
        );
    }
    Err(command.exec().into())
}

pub(crate) fn prepare(root: &Path, terminal: bool) -> Result<PathBuf> {
    let prefix = directories(root, terminal)?;
    let project = project(root)?;
    configuration(&prefix)?;
    Ok(project)
}

pub(crate) fn session(root: &Path, source: &Path) -> Result<rev::session::Session> {
    let startup = rev::config::prepare(&root.join("config/rev/config.toml"));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = rev::buffer::Buffers::with_settings(startup.snapshot)?;
    let id = buffers.visit(source)?;
    let view = rev::view::View::new(buffers.get(id)?);
    Ok(rev::session::Session::new(
        buffers,
        view,
        rev::view::Size {
            width: 100,
            height: 28,
        },
    )?)
}

pub(crate) fn pid(project: &Path) -> Result<String> {
    Ok(fs::read_to_string(project.join("fixture-pid"))?)
}
pub(crate) fn context(project: &Path) -> Result<serde_json::Value> {
    Ok(serde_json::from_slice(&fs::read(
        project.join("fixture-context"),
    )?)?)
}
pub(crate) fn history(root: &Path) -> Result<PathBuf> {
    let index: toml::Value = toml::from_str(&fs::read_to_string(
        root.join("state/rev/ai/pi/sessions.toml"),
    )?)?;
    assert_eq!(index["version"].as_integer(), Some(1));
    assert_eq!(
        index["bindings"]
            .as_array()
            .ok_or("missing bindings")?
            .len(),
        1
    );
    Ok(root.join("state/rev/ai/pi/sessions").join(
        index["bindings"][0]["session"]
            .as_str()
            .ok_or("missing history name")?,
    ))
}

fn directories(root: &Path, terminal: bool) -> Result<PathBuf> {
    let prefix = if terminal {
        root.join("pty")
    } else {
        root.to_owned()
    };
    for path in ["home", "config/rev", "data", "state", "cache"] {
        fs::create_dir_all(prefix.join(path))?;
    }
    Ok(prefix)
}

fn project(root: &Path) -> Result<PathBuf> {
    let project = root.join("workspace");
    fs::create_dir_all(project.join(".pi/extensions"))?;
    for name in ["main.zig", "terminal.rs"] {
        fs::write(project.join(name), SOURCE)?;
    }
    fs::write(project.join("expected-main"), SOURCE)?;
    restricted_resources(root, &project)?;
    Ok(project)
}

fn restricted_resources(root: &Path, project: &Path) -> Result<()> {
    fs::write(root.join("outside"), "OUTSIDE-MUST-NOT-LOAD")?;
    std::os::unix::fs::symlink(root.join("outside"), project.join("escape"))?;
    fs::write(project.join(".env"), "PRIVATE-MUST-NOT-LOAD")?;
    fs::write(project.join("binary"), [0, 1])?;
    fs::write(project.join("large"), vec![b'x'; 1_048_577])?;
    fs::write(
        project.join("AGENTS.md"),
        "RESOURCE-INSTRUCTION-MUST-NOT-LOAD",
    )?;
    fs::write(
        project.join(".pi/extensions/automatic.js"),
        "throw new Error('automatic extension loaded');",
    )?;
    Ok(())
}

fn configuration(prefix: &Path) -> Result<()> {
    let profile = std::env::var("REV_ACCEPTANCE_PROFILE")?;
    let command = std::env::current_exe()?;
    fs::write(
        prefix.join("config/rev/config.toml"),
        format!(
            "editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false\nfiles.external_change_debounce_ms=0\n[ai.pi]\ncommand={}\nmodel='rev-acceptance/local'\npersist_sessions=true\nidle_timeout_ms=2000\nrequest_timeout_ms=5000\n",
            serde_json::to_string(command.to_str().ok_or("non UTF-8 executable")?)?
        ),
    )?;
    Ok(())
}
