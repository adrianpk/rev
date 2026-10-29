//! Fixed native backend choices; selection never loads or executes an exporter.
use serde::Deserialize;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportBackend {
    #[serde(alias = "ascii")]
    Text,
    Utf8,
    Org,
    Markdown,
    Html,
    Latex,
    Beamer,
    Odt,
    Texinfo,
    Man,
    Icalendar,
}
impl ExportBackend {
    pub const ALL: [Self; 11] = [
        Self::Text,
        Self::Utf8,
        Self::Org,
        Self::Markdown,
        Self::Html,
        Self::Latex,
        Self::Beamer,
        Self::Odt,
        Self::Texinfo,
        Self::Man,
        Self::Icalendar,
    ];
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Export {
    pub default_format: ExportBackend,
    pub backends: Vec<ExportBackend>,
}
impl Default for Export {
    fn default() -> Self {
        Self {
            default_format: ExportBackend::Html,
            backends: ExportBackend::ALL.to_vec(),
        }
    }
}
impl Export {
    pub fn validate(&self) -> Result<(), String> {
        if self.backends.is_empty() || self.backends.len() > ExportBackend::ALL.len() {
            return Err("Select 1..11 native export backends".into());
        }
        for (index, backend) in self.backends.iter().enumerate() {
            if self.backends[..index].contains(backend) {
                return Err("Duplicate native export backend".into());
            }
        }
        if !self.backends.contains(&self.default_format) {
            return Err("Default export format must be an enabled backend".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialExport {
    pub default_format: Option<ExportBackend>,
    pub backends: Option<Vec<ExportBackend>>,
}
