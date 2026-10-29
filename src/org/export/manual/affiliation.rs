use super::*;
impl Document<'_> {
    fn affiliations(&self, n: &structure::Raw) -> Vec<(String, String)> {
        let Some(range) = &n.affiliated else {
            return Vec::new();
        };
        self.text
            .byte_slice(range.clone())
            .to_string()
            .lines()
            .filter_map(|line| {
                let (key, value) = line.trim_start().strip_prefix("#+")?.split_once(':')?;
                Some((key.to_ascii_uppercase(), value.trim().to_string()))
            })
            .collect()
    }
    pub(super) fn index_affiliation(&mut self, n: &structure::Raw) -> Result<(), String> {
        for (key, value) in self.affiliations(n) {
            if key == "NAME" {
                self.admit_target(&value)?;
            }
        }
        Ok(())
    }
    pub(super) fn affiliation(
        &self,
        out: &mut Manual<'_>,
        n: &structure::Raw,
    ) -> Result<(), String> {
        for (key, value) in self.affiliations(n) {
            self.affiliated_value(out, &key, &value)?;
        }
        Ok(())
    }
    fn affiliated_value(&self, out: &mut Manual<'_>, key: &str, value: &str) -> Result<(), String> {
        if key == "NAME" {
            return self.named_anchor(out, value);
        }
        if key != "CAPTION" {
            out.diagnostic("Unsupported manual affiliation retained literally")?;
        }
        if !out.texinfo() {
            out.emit(".PP\n")?;
        }
        if key != "CAPTION" {
            out.text(&format!("#+{key}: "))?;
        }
        out.line(value)?;
        out.emit("\n\n")
    }
    fn named_anchor(&self, out: &mut Manual<'_>, value: &str) -> Result<(), String> {
        self.anchor(out, value)?;
        if out.texinfo() {
            out.emit("\n")?;
        }
        Ok(())
    }
}
