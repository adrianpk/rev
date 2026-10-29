//! Data-only project definitions. Admission does not inspect or create paths.
use super::ExportBackend;
use serde::Deserialize;
use std::collections::BTreeSet;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Publish {
    pub projects: Vec<PublishProject>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialPublish {
    pub projects: Option<Vec<PublishProject>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishProject {
    pub name: String,
    pub roots: Vec<String>,
    #[serde(default = "includes")]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    pub output: String,
    pub formats: Vec<ExportBackend>,
    #[serde(default)]
    pub assets: Vec<String>,
    /// Opt into a separate confirmation for the single org.tools.upload profile.
    #[serde(default)]
    pub upload: bool,
}
fn includes() -> Vec<String> {
    vec!["**/*.org".into()]
}
pub fn validate(projects: &[PublishProject]) -> Result<(), String> {
    if projects.len() > 64 {
        return Err("Publishing permits at most 64 projects".into());
    }
    let mut names = BTreeSet::new();
    for project in projects {
        text(&project.name, 128)?;
        if !names.insert(&project.name) {
            return Err("Publishing project names must be unique".into());
        }
        project.validate()?;
    }
    Ok(())
}
impl PublishProject {
    pub fn validate(&self) -> Result<(), String> {
        if self.roots.is_empty() || self.roots.len() > 64 {
            return Err("Publishing requires 1..64 explicit source roots".into());
        }
        for root in &self.roots {
            path(root)?;
        }
        path(&self.output)?;
        patterns(&self.include)?;
        patterns(&self.exclude)?;
        patterns(&self.assets)?;
        super::Export {
            default_format: *self
                .formats
                .first()
                .ok_or("Select native publishing formats")?,
            backends: self.formats.clone(),
        }
        .validate()
    }
}
fn text(value: &str, max: usize) -> Result<(), String> {
    if value.is_empty() || value.len() > max || value.chars().any(char::is_control) {
        return Err(format!(
            "Publishing values require 1..{max} bytes without controls"
        ));
    }
    Ok(())
}
fn path(value: &str) -> Result<(), String> {
    text(value, 4096)?;
    super::capture::path(value)?;
    if std::path::Path::new(value)
        .components()
        .any(|c| c == std::path::Component::ParentDir)
    {
        return Err("Publishing paths cannot contain parent traversal".into());
    }
    Ok(())
}
fn patterns(values: &[String]) -> Result<(), String> {
    if values.len() > 64 {
        return Err("Publishing permits at most 64 patterns per selection".into());
    }
    for value in values {
        text(value, 256)?;
        if value.starts_with('/')
            || value.split('/').any(|s| s == "..")
            || value.contains(['[', ']', '{', '}', '\\'])
        {
            return Err("Use relative publishing patterns with only *, ** and ? wildcards".into());
        }
    }
    Ok(())
}
/// Whole-token substitution retains argv boundaries and never invokes a shell.
pub fn upload(argv: &[String]) -> Result<(), String> {
    if argv.len() > 64 {
        return Err("Upload argv permits at most 64 tokens".into());
    }
    for (index, token) in argv.iter().enumerate() {
        text(token, 4096)?;
        if token.contains(['{', '}']) && (token != "{output_dir}" || index == 0) {
            return Err("Upload supports only whole {output_dir} argument tokens".into());
        }
    }
    Ok(())
}
