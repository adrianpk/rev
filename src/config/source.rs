//! Explicit data-only source program profiles. Loading never starts a program.
use serde::Deserialize;
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourcePrograms {
    pub confirm_execution: bool,
    pub programs: Vec<SourceProgram>,
    pub library_files: Vec<String>,
}
impl Default for SourcePrograms {
    fn default() -> Self {
        Self {
            confirm_execution: true,
            programs: Vec::new(),
            library_files: Vec::new(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialSourcePrograms {
    pub confirm_execution: Option<bool>,
    pub programs: Option<Vec<SourceProgram>>,
    pub library_files: Option<Vec<String>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceProgram {
    pub name: String,
    pub language: String,
    #[serde(default)]
    pub action: ProgramAction,
    pub executable: String,
    #[serde(default)]
    pub argv: Vec<String>,
    #[serde(default)]
    pub delivery: SourceDelivery,
    #[serde(default)]
    pub cwd: SourceCwd,
    #[serde(default)]
    pub steps: Vec<ProgramStep>,
    #[serde(default)]
    pub result: SourceResult,
    #[serde(default = "seconds")]
    pub seconds: i64,
    #[serde(default = "bytes")]
    pub output_bytes: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgramStep {
    pub executable: String,
    #[serde(default)]
    pub argv: Vec<String>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProgramAction {
    #[default]
    Execute,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceDelivery {
    #[default]
    Stdin,
    File,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceCwd {
    #[default]
    Workspace,
    Source,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceResult {
    #[default]
    Output,
    Value,
    Table,
}
fn seconds() -> i64 {
    30
}
fn bytes() -> i64 {
    1048576
}
pub fn validate(programs: &[SourceProgram]) -> Result<(), String> {
    if programs.len() > 32 {
        return Err("Source permits at most 32 program profiles".into());
    }
    let mut names = BTreeSet::new();
    for p in programs {
        identifier(&p.name)?;
        identifier(&p.language)?;
        if !names.insert(&p.name) {
            return Err("Source program names must be unique".into());
        }
        p.validate()?;
    }
    Ok(())
}
impl SourceProgram {
    pub fn validate(&self) -> Result<(), String> {
        identifier(&self.name)?;
        identifier(&self.language)?;
        if !(1..=600).contains(&self.seconds) || !(1..=16777216).contains(&self.output_bytes) {
            return Err(
                "Source programs require 1..600 seconds and 1..16777216 output bytes".into(),
            );
        }
        if self.steps.len() > 3 {
            return Err("Source programs permit at most four compile/run steps".into());
        }
        argv(&self.executable, &self.argv)?;
        for step in &self.steps {
            argv(&step.executable, &step.argv)?;
        }
        self.delivery_contract()?;
        Ok(())
    }
    fn delivery_contract(&self) -> Result<(), String> {
        if self.delivery == SourceDelivery::File
            && !self.argv.iter().any(|a| a == "{input}")
            && self.steps.is_empty()
        {
            return Err(
                "File delivery requires an explicit {input} argument or compile step".into(),
            );
        }
        if self.size() > 65536 {
            return Err("Source program profile exceeds 64 KiB".into());
        }
        Ok(())
    }
    fn size(&self) -> usize {
        let step_bytes: usize = self
            .steps
            .iter()
            .map(|s| s.executable.len() + s.argv.iter().map(String::len).sum::<usize>())
            .sum();
        self.executable.len() + self.argv.iter().map(String::len).sum::<usize>() + step_bytes
    }
}
fn identifier(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_+-".contains(&b))
    {
        return Err("Source identifiers require 1..128 ASCII letters, digits, _, + or -".into());
    }
    Ok(())
}
fn argv(executable: &str, args: &[String]) -> Result<(), String> {
    text(executable)?;
    if (executable.contains(['{', '}']) && executable != "{output}") || args.len() > 63 {
        return Err("Source argv permits 64 tokens and a literal executable".into());
    }
    for token in args {
        argument(token)?;
    }
    Ok(())
}
fn text(value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
        return Err("Source argv tokens require 1..4096 bytes without controls".into());
    }
    Ok(())
}
fn argument(value: &str) -> Result<(), String> {
    text(value)?;
    let variable = value
        .strip_prefix("{var:")
        .and_then(|s| s.strip_suffix('}'))
        .is_some_and(|s| {
            !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        });
    if value.contains(['{', '}'])
        && !["{input}", "{output}", "{workspace}"].contains(&value)
        && !variable
    {
        return Err(
            "Source placeholders must be whole {input}, {output} or {workspace} tokens".into(),
        );
    }
    Ok(())
}

pub fn libraries(files: &[String]) -> Result<(), String> {
    if files.len() > 32 {
        return Err("Source library permits at most 32 explicit files".into());
    }
    for file in files {
        super::capture::path(file)?;
        if file.is_empty() {
            return Err("Source library files require nonempty paths".into());
        }
    }
    Ok(())
}
