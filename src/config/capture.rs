//! Data-only native capture definitions. Admission performs no target IO.
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureKind {
    #[default]
    Entry,
    Item,
    Checkitem,
    TableLine,
    Plain,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum CaptureTarget {
    #[serde(rename = "file")]
    File {
        #[serde(default)]
        file: String,
    },
    #[serde(rename = "file+headline")]
    Headline {
        #[serde(default)]
        file: String,
        headline: String,
    },
    #[serde(rename = "file+olp")]
    Outline {
        #[serde(default)]
        file: String,
        outline: Vec<String>,
    },
    #[serde(rename = "file+olp+datetree")]
    DateTree {
        #[serde(default)]
        file: String,
        #[serde(default)]
        outline: Vec<String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureTemplate {
    pub key: String,
    pub description: String,
    #[serde(default)]
    pub kind: CaptureKind,
    pub target: CaptureTarget,
    #[serde(default)]
    pub template: String,
    #[serde(default)]
    pub prepend: bool,
}
impl CaptureKind {
    pub fn default_template(self) -> &'static str {
        match self {
            Self::Entry => "* %?\n  %a",
            Self::Item => "- %?",
            Self::Checkitem => "- [ ] %?",
            Self::TableLine => "| %? |",
            Self::Plain => "%?",
        }
    }
}
impl CaptureTemplate {
    /// Reference Task fallback; an empty file still requires a real destination.
    pub fn task() -> Self {
        Self {
            key: "t".into(),
            description: "Task".into(),
            kind: CaptureKind::Entry,
            target: CaptureTarget::Headline {
                file: String::new(),
                headline: "Tasks".into(),
            },
            template: "* TODO %?\n  %u\n  %a".into(),
            prepend: false,
        }
    }
    pub fn text(&self) -> &str {
        if self.template.is_empty() {
            self.kind.default_template()
        } else {
            &self.template
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        label(&self.key, 16)?;
        label(&self.description, 256)?;
        self.target.validate()?;
        if self.template.len() > 65536 || self.template.contains('\0') {
            return Err("Capture template exceeds 65536 bytes or contains NUL".into());
        }
        Ok(())
    }
}
impl CaptureTarget {
    pub fn file(&self) -> &str {
        match self {
            Self::File { file }
            | Self::Headline { file, .. }
            | Self::Outline { file, .. }
            | Self::DateTree { file, .. } => file,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        path(self.file())?;
        match self {
            Self::File { .. } => Ok(()),
            Self::Headline { headline, .. } => label(headline, 1024),
            Self::Outline { outline, .. } if outline.is_empty() => {
                Err("Capture outline requires 1..64 headings".into())
            }
            Self::Outline { outline, .. } | Self::DateTree { outline, .. } => outline_path(outline),
        }
    }
}
pub(super) fn templates(values: &[CaptureTemplate]) -> Result<(), String> {
    if values.len() > 64 {
        return Err("Capture admits at most 64 templates".into());
    }
    let mut keys = std::collections::BTreeSet::new();
    for value in values {
        value.validate()?;
        if !keys.insert(&value.key) {
            return Err("Capture template keys must be unique".into());
        }
    }
    Ok(())
}
pub(crate) fn path(value: &str) -> Result<(), String> {
    if value.len() > 4096 || value.contains("://") || value.chars().any(char::is_control) {
        return Err("Capture destination requires a local path of at most 4096 bytes".into());
    }
    Ok(())
}
fn outline_path(values: &[String]) -> Result<(), String> {
    if values.len() > 64 {
        return Err("Capture outline exceeds 64 headings".into());
    }
    for value in values {
        label(value, 1024)?;
    }
    Ok(())
}
fn label(value: &str, maximum: usize) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > maximum || value.chars().any(char::is_control) {
        return Err(format!(
            "Capture label requires 1..{maximum} bytes without control characters"
        ));
    }
    Ok(())
}
