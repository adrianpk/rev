use super::*;
use std::path::{Component, Path};
#[derive(Clone)]
pub(crate) struct Selection {
    pub project: PublishProject,
    pub roots: Vec<PathBuf>,
    pub output: PathBuf,
}
impl Selection {
    pub fn configured(snapshot: &config::Snapshot, name: &str) -> Result<Self, String> {
        let projects = &snapshot.settings().org.publish.projects;
        config::publishing::validate(projects)?;
        let project = projects
            .iter()
            .find(|p| p.name == name)
            .ok_or("Choose a configured org.publish.projects name")?
            .clone();
        let base = snapshot
            .origin(&["org", "publish", "projects"])
            .and_then(Path::parent)
            .unwrap_or(Path::new("."));
        let base = if base.is_absolute() {
            base.to_owned()
        } else {
            std::env::current_dir()
                .map_err(|e| e.to_string())?
                .join(base)
        };
        let roots = project
            .roots
            .iter()
            .map(|r| normalized(base.join(r)))
            .collect();
        let output = normalized(base.join(&project.output));
        Ok(Self {
            project,
            roots,
            output,
        })
    }
}
fn normalized(path: PathBuf) -> PathBuf {
    path.components()
        .filter(|c| *c != Component::CurDir)
        .collect()
}
pub(super) fn extension(backend: ExportBackend) -> &'static str {
    const EXTENSIONS: [&str; 11] = [
        "txt",
        "utf8.txt",
        "org",
        "md",
        "html",
        "tex",
        "beamer.tex",
        "odt",
        "texi",
        "1",
        "ics",
    ];
    ExportBackend::ALL
        .iter()
        .zip(EXTENSIONS)
        .find(|(value, _)| **value == backend)
        .expect("all fixed native backends")
        .1
}
