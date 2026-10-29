use super::*;
pub(super) struct Expansion<'a> {
    blocks: &'a [Definition],
    names: BTreeMap<String, usize>,
    groups: BTreeMap<String, Vec<usize>>,
    stack: Vec<usize>,
    steps: usize,
    cancel: &'a Cancellation,
}
impl<'a> Expansion<'a> {
    pub fn new(blocks: &'a [Definition], cancel: &'a Cancellation) -> Result<Self, String> {
        let mut this = Self {
            blocks,
            names: BTreeMap::new(),
            groups: BTreeMap::new(),
            stack: Vec::new(),
            steps: 0,
            cancel,
        };
        for (i, block) in blocks.iter().enumerate() {
            this.index(i, block)?;
        }
        Ok(this)
    }
    fn index(&mut self, index: usize, block: &Definition) -> Result<(), String> {
        self.step()?;
        if let Some(name) = &block.name
            && self.names.insert(name.clone(), index).is_some()
        {
            return Err("Ambiguous named tangle reference".into());
        }
        if let Some(value) = block.headers.get("noweb-ref") {
            let name = options::literal(value)?;
            identifier(&name)?;
            self.groups.entry(name).or_default().push(index);
        }
        Ok(())
    }
    pub fn block(&mut self, index: usize) -> Result<String, String> {
        let mut text = if options::expands(&self.blocks[index])? {
            self.expanded(index)?
        } else {
            self.blocks[index].draft.clone()
        };
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        Ok(text)
    }
    fn expanded(&mut self, index: usize) -> Result<String, String> {
        self.step()?;
        if self.stack.contains(&index) {
            return Err("Cyclic noweb reference; no files written".into());
        }
        if self.stack.len() >= 64 {
            return Err("Noweb expansion exceeds 64 levels".into());
        }
        self.stack.push(index);
        let body = self.blocks[index].draft.clone();
        let mut result = String::new();
        for line in body.split_inclusive('\n') {
            let line = self.line(line, index)?;
            push(&mut result, &line)?;
        }
        self.stack.pop();
        Ok(result)
    }
    fn line(&mut self, line: &str, owner: usize) -> Result<String, String> {
        self.step()?;
        let mut rest = line;
        let mut result = String::new();
        while let Some(start) = rest.find("<<") {
            self.step()?;
            push(&mut result, &rest[..start])?;
            let tail = &rest[start + 2..];
            let end = tail.find(">>").ok_or("Unclosed noweb reference")?;
            let text = self.reference(&tail[..end], owner)?;
            let prefix = self.prefix(owner, &result)?;
            self.insert(&mut result, &text, &prefix)?;
            rest = &tail[end + 2..];
        }
        push(&mut result, rest)?;
        Ok(result)
    }
    fn reference(&mut self, name: &str, owner: usize) -> Result<String, String> {
        identifier(name)?;
        let targets = self.targets(name)?;
        let separator = options::literal(
            self.blocks[owner]
                .headers
                .get("noweb-sep")
                .unwrap_or("\"\\n\""),
        )?;
        let mut result = String::new();
        for (n, index) in targets.into_iter().enumerate() {
            self.step()?;
            if n > 0 {
                push(&mut result, &separator)?;
            }
            let body = self.fragment(index)?;
            push(&mut result, body.strip_suffix('\n').unwrap_or(&body))?;
        }
        Ok(result)
    }
    fn prefix(&self, owner: usize, result: &str) -> Result<String, String> {
        Ok(
            if options::yes(&self.blocks[owner], "noweb-prefix", true)? {
                result.rsplit('\n').next().unwrap_or("").to_owned()
            } else {
                String::new()
            },
        )
    }
    fn targets(&self, name: &str) -> Result<Vec<usize>, String> {
        if let Some(index) = self.names.get(name) {
            return Ok(vec![*index]);
        }
        self.groups
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Unresolved noweb reference: {name}"))
    }
    fn fragment(&mut self, index: usize) -> Result<String, String> {
        if self.blocks[index].headers.get("no-expand").is_some() {
            Ok(self.blocks[index].draft.clone())
        } else {
            self.expanded(index)
        }
    }
    fn insert(&mut self, result: &mut String, text: &str, prefix: &str) -> Result<(), String> {
        for (i, part) in text.split('\n').enumerate() {
            self.step()?;
            if i > 0 {
                push(result, "\n")?;
                push(result, prefix)?;
            }
            push(result, part)?;
        }
        Ok(())
    }
    fn step(&mut self) -> Result<(), String> {
        options::check(self.cancel)?;
        self.steps += 1;
        if self.steps > 100000 {
            return Err("Noweb expansion exceeds 100000 steps".into());
        }
        Ok(())
    }
}
fn identifier(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 128
        || name
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "()<>".contains(c))
    {
        return Err("Noweb requires a literal name; evaluated references are unsupported".into());
    }
    Ok(())
}
fn push(output: &mut String, value: &str) -> Result<(), String> {
    if output.len().saturating_add(value.len()) > BYTES {
        return Err("Noweb expansion exceeds 8 MiB".into());
    }
    output.push_str(value);
    Ok(())
}
