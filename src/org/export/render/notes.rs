use super::*;
#[derive(Default)]
pub(super) struct Notes {
    defined: BTreeSet<String>,
    used: BTreeSet<String>,
    inline: Vec<(String, String)>,
    bytes: usize,
}
impl Notes {
    pub fn reference(
        &mut self,
        label: String,
        definition: bool,
        markdown: bool,
    ) -> Result<String, String> {
        self.admit(&label)?;
        if definition {
            self.define(&label)?;
        } else {
            self.used.insert(label.clone());
        }
        let marker = if markdown { "^" } else { "" };
        let tail = if definition && markdown { ":" } else { "" };
        Ok(format!("[{marker}{label}]{tail}"))
    }
    pub fn label(&self, label: &str) -> String {
        if label.is_empty() {
            format!("inline-{}", self.inline.len() + 1)
        } else {
            label.into()
        }
    }
    pub fn add(&mut self, label: String, body: String, limit: usize) -> Result<(), String> {
        self.define(&label)?;
        self.bytes += body.len();
        if self.bytes > limit {
            return Err("Export footnote body byte limit reached".into());
        }
        self.inline.push((label, body));
        Ok(())
    }
    fn define(&mut self, label: &str) -> Result<(), String> {
        self.admit(label)?;
        if !self.defined.insert(label.into()) {
            return Err("Duplicate export footnote definition".into());
        }
        Ok(())
    }
    fn admit(&self, label: &str) -> Result<(), String> {
        if label.is_empty() || label.len() > 1024 || self.defined.len() + self.used.len() > 4096 {
            return Err("Export footnotes exceed native labels or count limits".into());
        }
        Ok(())
    }
    pub fn finish(&self) -> Result<String, String> {
        if self.used.iter().any(|l| !self.defined.contains(l)) {
            return Err(
                "Unresolved export footnote; include its definition in the selected scope".into(),
            );
        }
        Ok(String::new())
    }
    pub fn bodies(&self, markdown: bool) -> String {
        let marker = if markdown { "^" } else { "" };
        self.inline
            .iter()
            .map(|(label, body)| format!("\n[{marker}{label}]: {}\n", body.replace('\n', "\n    ")))
            .collect()
    }
}
