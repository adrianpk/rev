use super::*;
#[derive(Default)]
pub(super) struct Properties {
    pub values: BTreeMap<String, String>,
    pub caption: Option<Range<usize>>,
    anchor: Option<String>,
}
impl Properties {
    fn declaration(&mut self, value: &str) -> Result<(), String> {
        if value.len() > 4096 {
            return Err("HTML attributes exceed 4096 bytes".into());
        }
        let mut remaining = value;
        while !remaining.trim().is_empty() {
            let key = word(&mut remaining)?;
            let key = key
                .strip_prefix(':')
                .ok_or("Use :name value HTML attributes")?;
            let value = word(&mut remaining)?;
            if ![
                "id",
                "class",
                "title",
                "width",
                "height",
                "alt",
                "border",
                "cellpadding",
                "cellspacing",
            ]
            .contains(&key)
            {
                return Err(format!("Unsupported HTML attribute: {key}"));
            }
            if value.len() > 1024 || self.values.len() >= 16 {
                return Err("HTML attribute value/count limit reached".into());
            }
            self.values.insert(key.into(), value);
        }
        Ok(())
    }
    pub fn figure(&self, html: &mut Html<'_>) -> Result<(), String> {
        if self.caption.is_some() {
            html.emit("<figure>")?;
        }
        Ok(())
    }
    pub fn end_figure(&self, html: &mut Html<'_>) -> Result<(), String> {
        if self.caption.is_some() {
            html.emit("</figure>\n")?;
        }
        Ok(())
    }
    pub fn admit(&self, tag: &str) -> Result<(), String> {
        for (key, value) in &self.values {
            if !admitted(tag, key) {
                return Err(format!("HTML attribute {key} is unsupported on {tag}"));
            }
            validate(key, value)?;
        }
        Ok(())
    }
    pub fn anchor(&self, html: &mut Html<'_>) -> Result<(), String> {
        if let Some(anchor) = &self.anchor {
            html.identity(anchor)?;
            html.emit(&format!("<span id=\"{anchor}\"></span>"))?;
        }
        Ok(())
    }
    pub fn open(&self, html: &mut Html<'_>, tag: &str) -> Result<(), String> {
        self.admit(tag)?;
        self.anchor(html)?;
        html.open(tag, &self.values)
    }
}
impl Html<'_> {
    pub(super) fn open(
        &mut self,
        tag: &str,
        attributes: &BTreeMap<String, String>,
    ) -> Result<(), String> {
        self.emit(&format!("<{tag}"))?;
        for (key, value) in attributes {
            if key == "id" {
                self.identity(value)?;
            }
            self.emit(&format!(" {key}=\""))?;
            self.text(value)?;
            self.emit("\"")?;
        }
        self.emit(">")
    }
}
impl Document<'_> {
    pub(super) fn properties(&self, html: &mut Html<'_>, id: usize) -> Result<Properties, String> {
        let mut result = Properties::default();
        for child in &self.children[id] {
            document::check(html.cancel)?;
            let n = &self.nodes[*child];
            if n.kind != StructureKind::AffiliatedKeyword {
                continue;
            }
            let name = n
                .name
                .as_ref()
                .ok_or("HTML affiliation has no keyword name")?;
            let key = self
                .text
                .byte_slice(name.clone())
                .to_string()
                .to_ascii_uppercase();
            let value = self.text.byte_slice(n.content.clone()).to_string();
            match key.as_str() {
                "ATTR_HTML" => result.declaration(&value)?,
                "CAPTION" => result.caption = Some(n.content.clone()),
                "NAME" => {
                    if value.len() > 1024 {
                        return Err("HTML name exceeds 1024 bytes".into());
                    }
                    result.anchor = Some(format!("org-target-{}", encoded(&value)));
                }
                _ => {}
            }
        }
        Ok(result)
    }
    pub(super) fn caption(
        &self,
        html: &mut Html<'_>,
        range: Option<&Range<usize>>,
        tag: &str,
    ) -> Result<(), String> {
        if let Some(range) = range {
            html.emit(&format!("<{tag}>"))?;
            self.inline(html, range.clone(), 0)?;
            html.emit(&format!("</{tag}>\n"))?;
        }
        Ok(())
    }
}
fn word(value: &mut &str) -> Result<String, String> {
    *value = value.trim_start();
    if let Some(rest) = value.strip_prefix('"') {
        let (word, tail) = rest
            .split_once('"')
            .ok_or("Unclosed HTML attribute quote")?;
        if !tail.is_empty() && !tail.starts_with(char::is_whitespace) {
            return Err("HTML attributes require separated values".into());
        }
        *value = tail;
        return Ok(word.into());
    }
    let end = value.find(char::is_whitespace).unwrap_or(value.len());
    if end == 0 {
        return Err("Missing HTML attribute value".into());
    }
    let word = value[..end].to_owned();
    *value = &value[end..];
    Ok(word)
}
fn admitted(tag: &str, key: &str) -> bool {
    if ["id", "class", "title"].contains(&key) {
        return true;
    }
    match tag {
        "img" => ["width", "height", "alt"].contains(&key),
        "table" => ["border", "cellpadding", "cellspacing"].contains(&key),
        _ => false,
    }
}
fn validate(key: &str, value: &str) -> Result<(), String> {
    match key {
        "id" => identifier(value),
        "width" | "height" => dimension(value),
        "border" | "cellpadding" | "cellspacing" => integer(value),
        _ => Ok(()),
    }
}
fn identifier(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 256
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_:.".contains(&b))
    {
        return Err("HTML id requires 1..256 ASCII identifier bytes".into());
    }
    Ok(())
}
fn integer(value: &str) -> Result<(), String> {
    let n = value
        .parse::<u32>()
        .map_err(|_| "HTML table attribute requires an integer")?;
    if n > 4096 {
        return Err("HTML table attribute exceeds 4096".into());
    }
    Ok(())
}
fn dimension(value: &str) -> Result<(), String> {
    let (number, limit) = value.strip_suffix('%').map_or((value, 16384), |v| (v, 100));
    let n = number
        .parse::<u32>()
        .map_err(|_| "HTML image dimension requires pixels or percent")?;
    if n == 0 || n > limit {
        return Err("HTML image dimension exceeds admitted range".into());
    }
    Ok(())
}
