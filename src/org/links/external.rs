use super::*;
use crate::{
    config,
    org::io::process::{Outcome, Program},
};
use std::path::Path;
#[derive(Clone)]
pub(crate) struct Viewer {
    argv: Vec<String>,
    seconds: usize,
    output_bytes: usize,
}
pub(crate) enum Selection {
    Link,
    Attachment {
        attachment: crate::org::attachment::Selected,
        native: bool,
    },
}
pub(crate) struct External {
    source: Snapshot,
    selection: Selection,
    expected: Follow,
    viewer: Viewer,
}
pub(crate) fn viewer_argv(argv: &[String]) -> Result<(), String> {
    if argv.len() > 64
        || argv
            .iter()
            .any(|s| s.is_empty() || s.len() > 4096 || s.chars().any(char::is_control))
    {
        return Err("Viewer argv requires at most 64 nonempty tokens, 4096 bytes each".into());
    }
    if argv.is_empty() {
        return Ok(());
    }
    substitutions(argv)
}
fn substitutions(argv: &[String]) -> Result<(), String> {
    let substitutions = argv.iter().filter(|s| s.as_str() == "{target}").count();
    if substitutions > 1
        || argv[0].contains("{target}")
        || argv
            .iter()
            .any(|s| s.contains("{target}") && s != "{target}")
        || substitutions == 0 && argv.len() == 64
    {
        return Err("Viewer target substitution must be one whole non-executable argv token, or appended once".into());
    }
    Ok(())
}
impl Viewer {
    pub(crate) fn configured(snapshot: &config::Snapshot) -> Result<Self, String> {
        let policy = &snapshot.settings().org;
        let mut argv = policy.tools.viewer.clone();
        viewer_argv(&argv)?;
        if argv.is_empty() {
            return Err(
                "No external viewer configured; set org.tools.viewer to explicit argv".into(),
            );
        }
        let executable = PathBuf::from(&argv[0]);
        if executable.is_relative()
            && argv[0].contains('/')
            && let Some(root) = snapshot
                .origin(&["org", "tools", "viewer"])
                .and_then(|p| p.parent())
        {
            argv[0] = root
                .join(executable)
                .to_str()
                .ok_or("Viewer executable path is not UTF-8")?
                .into();
        }
        Ok(Self {
            argv,
            seconds: policy.limits.job_seconds,
            output_bytes: policy.limits.job_output_bytes,
        })
    }
    pub(crate) fn label(&self) -> &str {
        &self.argv[0]
    }
}
impl Follow {
    pub(crate) fn external_resource(&self) -> bool {
        match self {
            Self::External(_) => true,
            Self::Local { path, mode, .. } => {
                *mode == FileMode::External || *mode == FileMode::Automatic && media(path)
            }
            _ => false,
        }
    }
    pub(crate) fn label(&self) -> String {
        match self {
            Self::External(uri) => uri.clone(),
            Self::Local { path, .. } => path.display().to_string(),
            _ => String::new(),
        }
    }
    fn resource(&self) -> Result<Resource, String> {
        match self {
            Self::External(uri) => Ok(Resource::Uri(uri.clone())),
            Self::Local {
                path, search, root, ..
            } => {
                if !search.is_empty() {
                    return Err("External viewers cannot apply native file search suffixes".into());
                }
                let location = crate::file::resolve(path).map_err(|e| e.to_string())?;
                if !location.is_present() {
                    return Err("External resource does not exist".into());
                }
                super::resolve::contained(location.path(), root.as_deref())?;
                Ok(Resource::File(location))
            }
            _ => Err("This link is not an external resource".into()),
        }
    }
}
enum Resource {
    Uri(String),
    File(crate::file::Location),
}
impl Resource {
    fn value(&self) -> Result<String, String> {
        match self {
            Self::Uri(uri) => Ok(uri.clone()),
            Self::File(location) => {
                let current = crate::file::resolve(location.path()).map_err(|e| e.to_string())?;
                if current.observation() != location.observation() {
                    return Err("External resource changed before launch".into());
                }
                Ok(location
                    .path()
                    .to_str()
                    .ok_or("External resource path is not UTF-8")?
                    .into())
            }
        }
    }
}
impl External {
    pub(crate) fn new(source: Snapshot, expected: Follow, viewer: Viewer) -> Self {
        Self {
            source,
            selection: Selection::Link,
            expected,
            viewer,
        }
    }
    pub(crate) fn select(mut self, selection: Selection) -> Self {
        self.selection = selection;
        self
    }
    pub(crate) fn stamp(&self) -> Stamp {
        self.source.stamp
    }
    pub(crate) fn run(self, cancel: &Cancellation) -> Result<Outcome, String> {
        let directory = self.source.directory.clone();
        let observed = self.selection.observe(self.source, cancel)?;
        if observed.target != self.expected {
            return Err("External link source changed before launch".into());
        }
        let target = observed.resource_value()?;
        let mut argv = self.viewer.argv;
        if let Some(at) = argv.iter().position(|a| a == "{target}") {
            argv[at] = target;
        } else {
            argv.push(target);
        }
        Program {
            argv,
            directory,
            seconds: self.viewer.seconds,
            output_bytes: self.viewer.output_bytes,
        }
        .run(cancel)
    }
}
fn media(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| {
            [
                "pdf", "png", "jpg", "jpeg", "gif", "webp", "svg", "avif", "mp4", "webm", "mp3",
                "ogg", "zip", "odt",
            ]
            .contains(&ext.to_ascii_lowercase().as_str())
        })
}

struct Observed {
    target: Follow,
    attachment: Option<crate::org::attachment::Selected>,
}
impl Selection {
    fn observe(self, source: Snapshot, cancel: &Cancellation) -> Result<Observed, String> {
        match self {
            Self::Link => {
                let Output::Follow(target) =
                    source.observe(crate::command::OrgCommand::OpenLink, false, cancel)?
                else {
                    return Err("External link no longer resolves".into());
                };
                Ok(Observed {
                    target,
                    attachment: None,
                })
            }
            Self::Attachment { attachment, native } => {
                let target = source.attachment_follow(attachment.name.clone(), native, cancel)?;
                Ok(Observed {
                    target,
                    attachment: Some(attachment),
                })
            }
        }
    }
}
impl Observed {
    fn check(&self) -> Result<(), String> {
        if let Some(attachment) = &self.attachment {
            if !matches!(&self.target,Follow::Local{path,..} if path==attachment.path()) {
                return Err("Attachment location changed since selection".into());
            }
            attachment.check()?;
        }
        Ok(())
    }
    fn resource_value(&self) -> Result<String, String> {
        self.check()?;
        let resource = self.target.resource()?;
        let value = resource.value()?;
        self.check()?;
        Ok(value)
    }
}
