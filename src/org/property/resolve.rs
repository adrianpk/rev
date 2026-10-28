use super::*;

struct Layer {
    base: Option<usize>,
    additions: Vec<usize>,
}
impl Properties {
    pub(super) fn resolve(
        &self,
        buffer: &Buffer,
        entry: usize,
        key: &str,
        inherit: &[String],
    ) -> Result<PropertyValue, String> {
        let text = crate::org::Source::new(buffer)
            .snapshot()
            .map_err(|e| e.to_string())?;
        self.resolve_rope(&text, entry, key, inherit)
    }
    fn resolve_rope(
        &self,
        text: &ropey::Rope,
        entry: usize,
        key: &str,
        inherit: &[String],
    ) -> Result<PropertyValue, String> {
        let key = key.trim_end_matches('+');
        let selected = inherit.iter().any(|name| name.eq_ignore_ascii_case(key));
        let mut chain = Vec::new();
        let mut owner = Some(entry);
        while let Some(index) = owner {
            let layer = self.layer(index, key)?;
            let stop = layer.base.is_some() || !selected;
            chain.push(layer);
            if stop {
                break;
            }
            owner = self.entries[index].parent;
        }
        let mut value = PropertyValue {
            text: String::new(),
            records: Vec::new(),
            undefined: false,
        };
        for layer in chain.into_iter().rev() {
            self.combine(text, layer, &mut value)?;
        }
        Ok(value)
    }
    fn layer(&self, owner: usize, key: &str) -> Result<Layer, String> {
        let entry = &self.entries[owner];
        let mut layer = Layer {
            base: None,
            additions: Vec::new(),
        };
        // File defaults precede file-drawer ownership regardless of source order.
        for directive in [true, false] {
            for index in &entry.records {
                let record = &self.records[*index];
                if record.key == key && record.directive == directive {
                    self.admit(&mut layer, *index)?;
                }
            }
        }
        Ok(layer)
    }
    fn admit(&self, layer: &mut Layer, index: usize) -> Result<(), String> {
        let record = &self.records[index];
        if record.additive {
            layer.additions.push(index);
            return Ok(());
        }
        if let Some(old) = layer.base
            && !self.records[old].directive
            && !record.directive
        {
            return Err("Duplicate local Org property; inspect source before resolving".into());
        }
        // A drawer base overrides file declarations. Repeated file base
        // declarations reset their previous additions, as ordered data defaults.
        layer.base = Some(index);
        if record.directive {
            layer.additions.clear();
        } else {
            layer.additions.retain(|i| !self.records[*i].directive);
        }
        Ok(())
    }
    fn combine(
        &self,
        rope: &ropey::Rope,
        layer: Layer,
        value: &mut PropertyValue,
    ) -> Result<(), String> {
        if let Some(base) = layer.base {
            value.text = rope_text(rope, self.records[base].value.clone())?;
            value.undefined = value.text == "nil";
            value.records = vec![base];
        }
        for index in layer.additions {
            let addition = rope_text(rope, self.records[index].value.clone())?;
            if !value.text.is_empty() && !addition.is_empty() {
                value.text.push(' ');
            }
            value.text.push_str(&addition);
            value.records.push(index);
        }
        value.undefined = value.text == "nil";
        Ok(())
    }
}

fn rope_text(text: &ropey::Rope, range: Range<usize>) -> Result<String, String> {
    text.get_byte_slice(range)
        .map(|v| v.to_string())
        .ok_or_else(|| "Invalid property source range".into())
}
impl Properties {
    pub(in crate::org) fn query_value(
        &self,
        text: &ropey::Rope,
        byte: usize,
        key: &str,
        inherit: &[String],
    ) -> Result<Option<String>, String> {
        let index = self
            .entries
            .partition_point(|e| e.start <= byte)
            .saturating_sub(1);
        if !self.complete || self.entries[index].invalid {
            return Err("Incomplete or malformed Org property metadata".into());
        }
        let value = self.resolve_rope(text, index, key, inherit)?;
        Ok((!value.undefined && !value.records.is_empty()).then_some(value.text))
    }
}
