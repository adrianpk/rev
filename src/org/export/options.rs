use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub toc: bool,
    pub numbered: bool,
    pub author: bool,
    pub title: bool,
    pub todo: bool,
    pub tags: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            toc: true,
            numbered: true,
            author: true,
            title: true,
            todo: true,
            tags: true,
        }
    }
}
impl Options {
    pub fn parse(value: &str) -> Result<Self, String> {
        let mut options = Self::default();
        if value.len() > 4096 {
            return Err("Export OPTIONS exceeds 4096 bytes".into());
        }
        for token in value.split_ascii_whitespace() {
            let (key, value) = token
                .split_once(':')
                .ok_or("Use key:t or key:nil export options")?;
            let target = options.target(key)?;
            *target = match value {
                "t" | "1" => true,
                "nil" | "0" => false,
                _ => return Err("Export options require t, nil, 1 or 0".into()),
            };
        }
        Ok(options)
    }
    fn target(&mut self, key: &str) -> Result<&mut bool, String> {
        match key {
            "toc" => Ok(&mut self.toc),
            "num" => Ok(&mut self.numbered),
            "author" => Ok(&mut self.author),
            "title" => Ok(&mut self.title),
            "todo" => Ok(&mut self.todo),
            "tags" => Ok(&mut self.tags),
            _ => Err(format!("Unsupported native export option: {key}")),
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub email: Option<String>,
    pub date: Option<String>,
    pub language: Option<String>,
    pub options: Options,
}
impl Metadata {
    pub(super) fn observe(text: &Rope, nodes: &[structure::Raw]) -> Result<Self, String> {
        let mut result = Self::default();
        for node in nodes.iter().filter(|n| preamble(nodes, n)) {
            if let Some((key, value)) = document::keyword(text, node) {
                result.assign(&key, &value)?;
            }
        }
        Ok(result)
    }
    fn assign(&mut self, key: &str, value: &str) -> Result<(), String> {
        let target = match key {
            "TITLE" => &mut self.title,
            "AUTHOR" => &mut self.author,
            "EMAIL" => &mut self.email,
            "DATE" => &mut self.date,
            "LANGUAGE" => &mut self.language,
            "OPTIONS" => {
                self.options = Options::parse(value)?;
                return Ok(());
            }
            _ => return Ok(()),
        };
        if value.len() > 4096 {
            return Err("Export metadata exceeds 4096 bytes".into());
        }
        *target = Some(value.into());
        Ok(())
    }
}
fn preamble(nodes: &[structure::Raw], n: &structure::Raw) -> bool {
    n.parent
        .is_none_or(|p| nodes[p].kind == StructureKind::Section && nodes[p].parent.is_none())
}
